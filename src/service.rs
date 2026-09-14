//! Case command service.
//!
//! Authorization semantics live in the canonical authorization contract and
//! evaluator. This module intentionally contains no legacy policy/request path.

pub use crate::service_canonical::{CaseCommand, CaseService, CommandContext, CommandResult, ServiceError};
