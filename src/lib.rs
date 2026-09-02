//! SIDERETH Core
//! Legal and regulatory workflow primitives.
//!
//! This crate intentionally contains no AI, network transport, government
//! submission, or autonomous legal-decision capability. Those concerns belong
//! to bounded shared infrastructure layered above the deterministic core.

use serde::{Deserialize, Serialize};

pub type Id = String;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentState {
    Prepared,
    Active,
    Paused,
    Concluded,
    EvidenceReview,
    LinkedToCase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Case {
    pub id: Id,
    pub title: String,
    pub jurisdiction_id: Option<Id>,
    pub authority_id: Option<Id>,
    pub state: CaseState,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    pub event_id: Id,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: Id,
    pub occurred_at: String,
    pub actor_type: String,
    pub actor_id: Id,
    pub schema_version: u32,
    pub payload: serde_json::Value,
    pub source_refs: Vec<Id>,
    pub correlation_id: Id,
    pub causation_id: Option<Id>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_starts_in_draft() {
        let case = Case {
            id: "case-1".into(),
            title: "Test matter".into(),
            jurisdiction_id: None,
            authority_id: None,
            state: CaseState::Draft,
        };
        assert_eq!(case.state, CaseState::Draft);
    }

    #[test]
    fn incident_can_exist_before_linking_to_case() {
        let incident = Incident {
            id: "incident-1".into(),
            case_id: None,
            incident_type: "inspection".into(),
            state: IncidentState::Prepared,
            started_at: None,
            ended_at: None,
        };
        assert!(incident.case_id.is_none());
    }
}
