use serde::{Deserialize, Serialize};

use crate::{Case, EventEnvelope, Id, Incident};

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
}
