use serde::{Deserialize, Serialize};

use super::error::PersistenceError;
use crate::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyState {
    Claimed,
    InProgress,
    Completed,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyClaim {
    Claimed,
    AlreadyClaimed,
}

pub trait IdempotencyStore {
    fn lookup(&self, operation_id: &Id) -> Result<bool, PersistenceError>;
    fn claim(&mut self, operation_id: Id) -> Result<IdempotencyClaim, PersistenceError>;
}

/// Durable execution-state contract layered on top of idempotency claim semantics.
pub trait IdempotencyLifecycleStore: IdempotencyStore {
    fn state(&self, operation_id: &Id) -> Result<Option<IdempotencyState>, PersistenceError>;
    fn mark_in_progress(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
    fn mark_completed(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
    fn mark_failed(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
    fn mark_unknown(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
}
