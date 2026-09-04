//! SIDERETH Core
//! Legal and regulatory workflow primitives.
//!
//! The core is deterministic and independent of AI, network transport,
//! government submission, and autonomous legal decisions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// Canonical opaque identifier used across SIDERETH primitives.
///
/// The wire representation remains a JSON string for backward compatibility,
/// while the Rust type prevents accidental substitution with arbitrary
/// application strings at typed API boundaries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Id(String);

impl Id {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("id is required");
        }
        if value.len() > 512 {
            return Err("id exceeds maximum length");
        }
        Ok(Self(value))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub mod action;
pub mod audit;
pub mod authority;
pub mod authorization;
pub mod compliance;
pub mod deadline;
pub mod document;
pub mod event;
pub mod evidence;
pub mod evidence_store;
pub mod jurisdiction;
pub mod legal_source;
pub mod legal_source_registry;
pub mod local_store;
pub mod party;
pub mod persistence;
pub mod procedure;
pub mod remedy;
pub mod repository;
pub mod resolution;
pub mod response;
pub mod security;
pub mod service;
pub mod timeline;

pub use action::{Action, ActionKind, ActionStatus};
pub use audit::{AuditRecord, AuditSink, InMemoryAudit};
pub use authority::{Authority, AuthorityPower, AuthorityRegistry, AuthorityStatus, AuthorityType};
pub use authorization::{AccessAction, AccessRequest, AuthorizationPolicy, CaseAccessPolicy};
pub use compliance::{ComplianceRegistry, ComplianceRequirement, ComplianceState};
pub use deadline::{
    ApplicabilityStatus, CivilDate, Deadline, DeadlineRegistry, DeadlineStatus, DeadlineType,
    Obligation,
};
pub use document::{
    DerivedArtifact as DocumentDerivedArtifact, Document, DocumentRegistry, DocumentStatus,
    DocumentVersion, IntegrityStatus,
};
pub use event::EventEnvelope;
pub use evidence::{sha256_hex, DerivedArtifact, EvidenceCapture, EvidenceOriginal};
pub use evidence_store::{EvidenceObjectStore, EvidenceRepository, InMemoryEvidenceVault};
pub use jurisdiction::{Jurisdiction, JurisdictionRegistry, JurisdictionStatus, JurisdictionType};
pub use legal_source::{
    LegalProposition, LegalSource, PropositionType, SourceType, VerificationStatus,
};
pub use legal_source_registry::LegalSourceRegistry;
pub use local_store::LocalFileStore;
pub use party::{Party, PartyKind, PartyRegistry, PartyRelationship, PartyStatus};
pub use persistence::{
    CaseStore, EventStore, IdempotencyClaim, IdempotencyStore, IncidentStore, Persisted,
    PersistenceError, Revision, Transaction, TransactionFactory,
};
pub use procedure::{Procedure, ProcedureRegistry, ProcedureStatus, ProcedureStep};
pub use remedy::{Remedy, RemedyApplicabilityStatus, RemedyRegistry, RemedyState};
pub use repository::{CaseRepository, EventRepository, InMemoryRepositories, IncidentRepository};
pub use resolution::{Resolution, ResolutionRegistry, ResolutionState};
pub use response::{Escalation, EscalationState, Response, ResponseRegistry, ResponseState};
pub use security::{
    AuthorizedAudit, EvidenceError, EvidenceExport, EvidenceExporter, KeyProvider, RetentionPolicy,
};
pub use service::{CaseCommand, CaseService, CommandContext, CommandResult, ServiceError};
pub use timeline::Timeline;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaseState {
    Draft,
    Active,
    WaitingUser,
    WaitingAuthority,
    ResponseDue,
    Resolved,
    Closed,
}

impl CaseState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Active)
                | (Self::Active, Self::WaitingUser)
                | (Self::Active, Self::WaitingAuthority)
                | (Self::Active, Self::ResponseDue)
                | (Self::Active, Self::Resolved)
                | (Self::WaitingUser, Self::Active)
                | (Self::WaitingAuthority, Self::Active)
                | (Self::ResponseDue, Self::Active)
                | (Self::ResponseDue, Self::Resolved)
                | (Self::Resolved, Self::Closed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Case {
    pub case_id: Id,
    pub state: CaseState,
}

impl Case {
    pub fn new(case_id: Id) -> Result<Self, &'static str> {
        if case_id.is_empty() {
            return Err("case id is required");
        }
        Ok(Self {
            case_id,
            state: CaseState::Draft,
        })
    }

    pub fn transition(&mut self, next: CaseState) -> Result<(), &'static str> {
        if !self.state.can_transition_to(&next) {
            return Err("invalid case state transition");
        }
        self.state = next;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentState {
    Open,
    Recorded,
    UnderReview,
    Resolved,
    Closed,
}

impl IncidentState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::Recorded)
                | (Self::Recorded, Self::UnderReview)
                | (Self::UnderReview, Self::Resolved)
                | (Self::Resolved, Self::Closed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Incident {
    pub incident_id: Id,
    pub state: IncidentState,
}

impl Incident {
    pub fn new(incident_id: Id) -> Result<Self, &'static str> {
        if incident_id.is_empty() {
            return Err("incident id is required");
        }
        Ok(Self {
            incident_id,
            state: IncidentState::Open,
        })
    }

    pub fn transition(&mut self, next: IncidentState) -> Result<(), &'static str> {
        if !self.state.can_transition_to(&next) {
            return Err("invalid incident state transition");
        }
        self.state = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_transparent_on_wire_and_rejects_empty_constructor_input() {
        let id = Id::new("case-1").unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"case-1\"");
        assert_eq!(Id::new("   "), Err("id is required"));
    }

    #[test]
    fn id_rejects_excessive_length() {
        let value = "x".repeat(513);
        assert_eq!(Id::new(value), Err("id exceeds maximum length"));
    }
}
