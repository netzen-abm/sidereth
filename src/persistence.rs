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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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
    pub expected_revision: Option<Revision>,
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
            expected_revision: None,
        })
    }

    pub fn with_expected_revision(mut self, revision: Revision) -> Self {
        self.expected_revision = Some(revision);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceRecord {
    pub resource_ref: ResourceRef,
    pub schema_version: u16,
    pub revision: Revision,
    pub payload: Value,
}

/// Semantic class for a cross-resource reference.
///
/// Strong references require an internally resolvable target when the
/// authoritative policy requires it. Forward references permit deferred target
/// creation. External references identify relationships whose target is outside
/// the authoritative SIDERETH resource graph.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLinkClass {
    #[default]
    Strong,
    Forward,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceLink {
    pub source_ref: ResourceRef,
    pub relation: String,
    pub target_ref: ResourceRef,
    #[serde(default)]
    pub class: ResourceLinkClass,
}

impl ResourceLink {
    /// Compatibility constructor. Existing callers retain Strong semantics.
    pub fn new(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
    ) -> Result<Self, UnitOfWorkError> {
        Self::new_with_class(source_ref, relation, target_ref, ResourceLinkClass::Strong)
    }

    pub fn new_with_class(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
        class: ResourceLinkClass,
    ) -> Result<Self, UnitOfWorkError> {
        let relation = relation.into();
        if relation.trim().is_empty() {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        Ok(Self {
            source_ref,
            relation,
            target_ref,
            class,
        })
    }

    pub fn strong(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
    ) -> Result<Self, UnitOfWorkError> {
        Self::new_with_class(source_ref, relation, target_ref, ResourceLinkClass::Strong)
    }

    pub fn forward(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
    ) -> Result<Self, UnitOfWorkError> {
        Self::new_with_class(source_ref, relation, target_ref, ResourceLinkClass::Forward)
    }

    pub fn external(
        source_ref: ResourceRef,
        relation: impl Into<String>,
        target_ref: ResourceRef,
    ) -> Result<Self, UnitOfWorkError> {
        Self::new_with_class(source_ref, relation, target_ref, ResourceLinkClass::External)
    }
}

/// Provider-neutral read/write context used inside a unit of work.
/// Reads occur in the same transaction as subsequent CAS writes.
pub trait UnitOfWorkContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError>;
    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError>;
    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError>;
}

/// Provider-neutral atomic boundary for a multi-resource workflow.
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
