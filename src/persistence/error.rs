use serde::{Deserialize, Serialize};

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

