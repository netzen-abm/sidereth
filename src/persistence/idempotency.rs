use serde::{Deserialize, Serialize};

use crate::Id;
use super::error::PersistenceError;

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

/// Durable execution-state contract layered on top of idempotency claim semantics.
pub trait IdempotencyLifecycleStore: IdempotencyStore {
    fn state(&self, operation_id: &Id) -> Result<Option<IdempotencyState>, PersistenceError>;
    fn mark_in_progress(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
    fn mark_completed(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
    fn mark_failed(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
    fn mark_unknown(&mut self, operation_id: &Id) -> Result<(), PersistenceError>;
}

