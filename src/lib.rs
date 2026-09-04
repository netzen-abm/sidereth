//! SIDERETH Core
//! Legal and regulatory workflow primitives.
//!
//! The core is deterministic and independent of AI, network transport,
//! government submission, and autonomous legal decisions.

use serde::{Deserialize, Serialize};

pub type Id = String;

/// Explicit cross-primitive reference contract for ecosystem boundaries.
///
/// Existing domain structs retain `Id = String` for source compatibility.
/// New integrations should use this typed boundary instead of relying on an
/// implicit target type for an identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ResourceRef {
    pub resource_type: ResourceType,
    pub id: Id,
}

impl ResourceRef {
    pub fn new(resource_type: ResourceType, id: impl Into<Id>) -> Result<Self, &'static str> {
        let id = id.into();
        if id.is_empty() {
            return Err("resource reference id is required");
        }
        Ok(Self { resource_type, id })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Case,
    Incident,
    Event,
    Evidence,
    Authority,
    Jurisdiction,
    Party,
    Document,
    Action,
    Deadline,
    Response,
    Escalation,
    Remedy,
    Resolution,
    Procedure,
    ComplianceRequirement,
    LegalSource,
    Timeline,
    Other,
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
pub mod lifecycle;
pub mod local_store;
pub mod party;
pub mod persistence;
pub mod procedure;
pub mod provenance;
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
pub use authorization::{
    AccessAction, AccessRequest, AuthorizationDecision, AuthorizationEvaluator,
    AuthorizationPolicy, AuthorizationRequest, AuthorizationResult, CaseAccessPolicy,
};
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
pub use lifecycle::{LifecycleMeta, LifecycleTransition};
pub use local_store::LocalFileStore;
pub use party::{Party, PartyKind, PartyRegistry, PartyRelationship, PartyStatus};
pub use persistence::{
    CaseStore, EventStore, IdempotencyClaim, IdempotencyStore, IncidentStore, Persisted,
    PersistenceError, Revision, Transaction, TransactionFactory,
};
pub use procedure::{Procedure, ProcedureRegistry, ProcedureStatus, ProcedureStep};
pub use provenance::{Provenance, ProvenanceRef};
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
    fn resource_ref_is_explicit_and_stable_on_wire() {
        let reference = ResourceRef::new(ResourceType::Document, "doc-1").unwrap();
        let json = serde_json::to_string(&reference).unwrap();
        assert_eq!(json, r#"{"resource_type":"document","id":"doc-1"}"#);
        let decoded: ResourceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, reference);
    }

    #[test]
    fn empty_resource_ref_is_rejected() {
        assert_eq!(
            ResourceRef::new(ResourceType::Case, ""),
            Err("resource reference id is required")
        );
    }
}
