//! SIDERETH Core
//! Legal and regulatory workflow primitives.
//!
//! The core is deterministic and independent of AI, network transport,
//! government submission, and autonomous legal decisions.

use serde::{Deserialize, Serialize};

pub type Id = String;

pub mod audit;
pub mod authorization;
pub mod event;
pub mod evidence;
pub mod evidence_store;
pub mod repository;
pub mod timeline;

pub use audit::{AuditRecord, AuditSink, InMemoryAudit};
pub use authorization::{AccessAction, AccessRequest, AuthorizationPolicy, CaseAccessPolicy};
pub use event::EventEnvelope;
pub use evidence::{sha256_hex, DerivedArtifact, EvidenceCapture, EvidenceOriginal};
pub use evidence_store::{
    EvidenceObjectStore, EvidenceRepository, InMemoryEvidenceVault,
};
pub use repository::{CaseRepository, EventRepository, InMemoryRepositories, IncidentRepository};
pub use timeline::Timeline;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaseState {
    Draft,
    Active,
    WaitingUser,
    WaitingAuthority,
    ResponseDue,
    Escalated,
    Resolved,
    Closed,
}

impl CaseState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        use CaseState::*;
        matches!(
            (self, next),
            (Draft, Active)
                | (Active, WaitingUser | WaitingAuthority | ResponseDue)
                | (Active, Escalated | Resolved)
                | (WaitingUser, Active | ResponseDue)
                | (WaitingUser, Escalated | Resolved)
                | (WaitingAuthority, Active | ResponseDue)
                | (WaitingAuthority, Escalated | Resolved)
                | (ResponseDue, Active | Escalated | Resolved)
                | (Escalated, Active | Resolved)
                | (Resolved, Closed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentState {
    Prepared,
    Active,
    Paused,
    Concluded,
    EvidenceReview,
    LinkedToCase,
}

impl IncidentState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        use IncidentState::*;
        matches!(
            (self, next),
            (Prepared, Active)
                | (Active, Paused | Concluded)
                | (Paused, Active | Concluded)
                | (Concluded, EvidenceReview)
                | (EvidenceReview, LinkedToCase)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Case {
    pub id: Id,
    pub title: String,
    pub jurisdiction_id: Option<Id>,
    pub authority_id: Option<Id>,
    pub state: CaseState,
}

impl Case {
    pub fn transition_to(&mut self, next: CaseState) -> Result<(), &'static str> {
        if self.state.can_transition_to(&next) {
            self.state = next;
            Ok(())
        } else {
            Err("invalid case state transition")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Incident {
    pub id: Id,
    pub case_id: Option<Id>,
    pub incident_type: String,
    pub state: IncidentState,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

impl Incident {
    pub fn transition_to(&mut self, next: IncidentState) -> Result<(), &'static str> {
        if self.state.can_transition_to(&next) {
            self.state = next;
            Ok(())
        } else {
            Err("invalid incident state transition")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_case() -> Case {
        Case {
            id: "case-1".into(),
            title: "Test matter".into(),
            jurisdiction_id: None,
            authority_id: None,
            state: CaseState::Draft,
        }
    }

    fn test_incident() -> Incident {
        Incident {
            id: "incident-1".into(),
            case_id: None,
            incident_type: "inspection".into(),
            state: IncidentState::Prepared,
            started_at: None,
            ended_at: None,
        }
    }

    #[test]
    fn case_starts_in_draft() {
        assert_eq!(test_case().state, CaseState::Draft);
    }

    #[test]
    fn valid_case_transition_is_allowed() {
        let mut case = test_case();
        assert!(case.transition_to(CaseState::Active).is_ok());
        assert_eq!(case.state, CaseState::Active);
    }

    #[test]
    fn invalid_case_transition_is_rejected() {
        let mut case = test_case();
        assert!(case.transition_to(CaseState::Closed).is_err());
        assert_eq!(case.state, CaseState::Draft);
    }

    #[test]
    fn incident_can_exist_before_linking_to_case() {
        assert!(test_incident().case_id.is_none());
    }

    #[test]
    fn incident_transition_requires_valid_sequence() {
        let mut incident = test_incident();
        assert!(incident.transition_to(IncidentState::Active).is_ok());
        assert!(incident.transition_to(IncidentState::LinkedToCase).is_err());
        assert_eq!(incident.state, IncidentState::Active);
    }
}
