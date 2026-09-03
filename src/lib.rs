//! SIDERETH Core
//! Legal and regulatory workflow primitives.
//!
//! The core is deterministic and independent of AI, network transport,
//! government submission, and autonomous legal decisions.

use serde::{Deserialize, Serialize};

pub type Id = String;

pub mod audit;
pub mod authority;
pub mod authorization;
pub mod compliance;
pub mod deadline;
pub mod event;
pub mod evidence;
pub mod evidence_store;
pub mod jurisdiction;
pub mod legal_source;
pub mod legal_source_registry;
pub mod procedure;
pub mod repository;
pub mod response;
pub mod security;
pub mod timeline;

pub use audit::{AuditRecord, AuditSink, InMemoryAudit};
pub use authority::{Authority, AuthorityPower, AuthorityRegistry, AuthorityStatus, AuthorityType};
pub use authorization::{AccessAction, AccessRequest, AuthorizationPolicy, CaseAccessPolicy};
pub use compliance::{ComplianceRegistry, ComplianceRequirement, ComplianceState};
pub use deadline::{
    ApplicabilityStatus, CivilDate, Deadline, DeadlineRegistry, DeadlineStatus, DeadlineType,
    Obligation,
};
pub use event::EventEnvelope;
pub use evidence::{sha256_hex, DerivedArtifact, EvidenceCapture, EvidenceOriginal};
pub use evidence_store::{EvidenceObjectStore, EvidenceRepository, InMemoryEvidenceVault};
pub use jurisdiction::{Jurisdiction, JurisdictionRegistry, JurisdictionStatus, JurisdictionType};
pub use legal_source::{
    LegalProposition, LegalSource, PropositionType, SourceType, VerificationStatus,
};
pub use legal_source_registry::LegalSourceRegistry;
pub use procedure::{Procedure, ProcedureRegistry, ProcedureStatus, ProcedureStep};
pub use repository::{CaseRepository, EventRepository, InMemoryRepositories, IncidentRepository};
pub use response::{Escalation, EscalationState, Response, ResponseRegistry, ResponseState};
pub use security::{
    AuthorizedAudit, EvidenceError, EvidenceExport, EvidenceExporter, KeyProvider, RetentionPolicy,
};
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
