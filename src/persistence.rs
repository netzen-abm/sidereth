use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Case, EventEnvelope, Id, Incident, ResourceRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceError {
    Unavailable,
    Timeout,
    Conflict,
    IntegrityFailure,
    AuthorizationFailure,
    ValidationFailure,
    SerializationFailure,
    UnsupportedSchemaVersion,
    NotFound,
    Duplicate,
    IdempotencyAlreadyClaimed,
    RetentionBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitOfWorkError {
    Persistence(PersistenceError),
    InvalidOperation,
}

impl From<PersistenceError> for UnitOfWorkError {
    fn from(value: PersistenceError) -> Self {
        Self::Persistence(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceWriteMode {
    Insert,
    Upsert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceWrite {
    pub resource_ref: ResourceRef,
    pub schema_version: u16,
    pub payload: Value,
    pub mode: ResourceWriteMode,
}

impl ResourceWrite {
    pub fn new(
        resource_ref: ResourceRef,
        schema_version: u16,
        payload: Value,
        mode: ResourceWriteMode,
    ) -> Result<Self, UnitOfWorkError> {
        if schema_version == 0 {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        Ok(Self {
            resource_ref,
            schema_version,
            payload,
            mode,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceLink {
    pub source_ref: ResourceRef,
    pub relation: String,
    pub target_ref: ResourceRef,
}

impl ResourceLink {
    pub fn new(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
    ) -> Result<Self, UnitOfWorkError> {
        let relation = relation.into();
        if relation.trim().is_empty() {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        Ok(Self {
            source_ref,
            relation,
            target_ref,
        })
    }
}

/// Provider-neutral write context used inside a unit of work.
///
/// Implementations may persist immediately into a provider transaction, but the
/// caller receives commit/rollback semantics only from the surrounding UoW.
pub trait UnitOfWorkContext {
    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError>;
    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError>;
}

/// Provider-neutral atomic boundary for a multi-resource workflow.
///
/// The contract deliberately knows nothing about PostgreSQL, HTTP, files, or
/// a specific ORM. A single UoW can therefore contain a Case write plus links
/// to Document/Evidence resources and commit them as one logical operation.
pub trait UnitOfWork {
    type Context: UnitOfWorkContext;

    fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
    where
        F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>;

    fn commit(self) -> Result<(), PersistenceError>;
    fn rollback(self) -> Result<(), PersistenceError>;
}

pub trait UnitOfWorkFactory {
    type Uow: UnitOfWork;

    fn begin(&mut self) -> Result<Self::Uow, PersistenceError>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Revision {
    pub value: u64,
}

impl Revision {
    pub fn initial() -> Self {
        Self { value: 0 }
    }

    pub fn next(self) -> Result<Self, PersistenceError> {
        self.value
            .checked_add(1)
            .map(|value| Self { value })
            .ok_or(PersistenceError::Conflict)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Persisted<T> {
    pub schema_version: u16,
    pub revision: Revision,
    pub value: T,
}

impl<T> Persisted<T> {
    pub fn new(schema_version: u16, value: T) -> Result<Self, PersistenceError> {
        if schema_version == 0 {
            return Err(PersistenceError::ValidationFailure);
        }
        Ok(Self {
            schema_version,
            revision: Revision::initial(),
            value,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyClaim {
    Claimed,
    AlreadyClaimed,
}

pub trait CaseStore {
    fn get_case(&self, id: &Id) -> Result<Option<Persisted<Case>>, PersistenceError>;
    fn create_case(&mut self, value: Persisted<Case>) -> Result<(), PersistenceError>;
    fn update_case(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Case,
    ) -> Result<Revision, PersistenceError>;
}

pub trait IncidentStore {
    fn get_incident(&self, id: &Id) -> Result<Option<Persisted<Incident>>, PersistenceError>;
    fn create_incident(&mut self, value: Persisted<Incident>) -> Result<(), PersistenceError>;
    fn update_incident(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Incident,
    ) -> Result<Revision, PersistenceError>;
}

pub trait EventStore {
    fn get_event(&self, id: &Id) -> Result<Option<Persisted<EventEnvelope>>, PersistenceError>;
    fn append_event(&mut self, value: Persisted<EventEnvelope>) -> Result<(), PersistenceError>;
}

pub trait Transaction {
    fn commit(self) -> Result<(), PersistenceError>;
    fn rollback(self) -> Result<(), PersistenceError>;
}

pub trait TransactionFactory {
    type Tx: Transaction;

    fn begin(&mut self) -> Result<Self::Tx, PersistenceError>;
}

pub trait IdempotencyStore {
    fn lookup(&self, operation_id: &Id) -> Result<bool, PersistenceError>;
    fn claim(&mut self, operation_id: Id) -> Result<IdempotencyClaim, PersistenceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_is_deterministic() {
        assert_eq!(Revision::initial().next().unwrap().value, 1);
    }

    #[test]
    fn zero_schema_version_is_rejected() {
        let case = Case::new("case-1".into()).unwrap();
        assert_eq!(
            Persisted::new(0, case),
            Err(PersistenceError::ValidationFailure)
        );
    }

    #[test]
    fn revision_overflow_is_a_conflict() {
        let revision = Revision { value: u64::MAX };
        assert_eq!(revision.next(), Err(PersistenceError::Conflict));
    }

    #[test]
    fn resource_write_requires_schema_version() {
        let resource = ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        assert_eq!(
            ResourceWrite::new(resource, 0, Value::Null, ResourceWriteMode::Insert),
            Err(UnitOfWorkError::InvalidOperation)
        );
    }

    #[test]
    fn resource_link_requires_relation() {
        let source = ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        let target = ResourceRef::new(crate::ResourceType::Document, "doc-1").unwrap();
        assert_eq!(
            ResourceLink::new(source, "   ", target),
            Err(UnitOfWorkError::InvalidOperation)
        );
    }
}
