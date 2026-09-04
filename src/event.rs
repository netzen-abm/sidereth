use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Id, ResourceRef};

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
    pub payload: Value,
    pub source_refs: Vec<Id>,
    pub correlation_id: Id,
    pub causation_id: Option<Id>,
}

impl EventEnvelope {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.event_id.is_empty() {
            return Err("event id is required");
        }
        if self.event_type.is_empty() {
            return Err("event type is required");
        }
        if self.aggregate_type.is_empty() {
            return Err("aggregate type is required");
        }
        if self.aggregate_id.is_empty() {
            return Err("aggregate id is required");
        }
        if self.occurred_at.is_empty() {
            return Err("event time is required");
        }
        if self.actor_type.is_empty() {
            return Err("actor type is required");
        }
        if self.actor_id.is_empty() {
            return Err("actor id is required");
        }
        if self.schema_version == 0 {
            return Err("schema version must be positive");
        }
        if self.correlation_id.is_empty() {
            return Err("correlation id is required");
        }
        Ok(())
    }

    /// Returns the event's aggregate as an explicit ecosystem boundary reference.
    pub fn aggregate_ref(&self) -> Result<ResourceRef, &'static str> {
        let resource_type = match self.aggregate_type.as_str() {
            "case" => crate::ResourceType::Case,
            "incident" => crate::ResourceType::Incident,
            "event" => crate::ResourceType::Event,
            "evidence" => crate::ResourceType::Evidence,
            "authority" => crate::ResourceType::Authority,
            "jurisdiction" => crate::ResourceType::Jurisdiction,
            "party" => crate::ResourceType::Party,
            "document" => crate::ResourceType::Document,
            "action" => crate::ResourceType::Action,
            "deadline" => crate::ResourceType::Deadline,
            "response" => crate::ResourceType::Response,
            "escalation" => crate::ResourceType::Escalation,
            "remedy" => crate::ResourceType::Remedy,
            "resolution" => crate::ResourceType::Resolution,
            "procedure" => crate::ResourceType::Procedure,
            "compliance_requirement" => crate::ResourceType::ComplianceRequirement,
            "legal_source" => crate::ResourceType::LegalSource,
            "timeline" => crate::ResourceType::Timeline,
            _ => crate::ResourceType::Other,
        };
        ResourceRef::new(resource_type, self.aggregate_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> EventEnvelope {
        EventEnvelope {
            event_id: "event-1".into(),
            event_type: "case.activated".into(),
            aggregate_type: "case".into(),
            aggregate_id: "case-1".into(),
            occurred_at: "2026-09-02T10:00:00Z".into(),
            actor_type: "user".into(),
            actor_id: "user-1".into(),
            schema_version: 1,
            payload: Value::Null,
            source_refs: vec![],
            correlation_id: "corr-1".into(),
            causation_id: None,
        }
    }

    #[test]
    fn valid_event_passes_validation() {
        assert!(event().validate().is_ok());
    }

    #[test]
    fn missing_event_id_is_rejected() {
        let mut value = event();
        value.event_id.clear();
        assert_eq!(value.validate(), Err("event id is required"));
    }

    #[test]
    fn zero_schema_version_is_rejected() {
        let mut value = event();
        value.schema_version = 0;
        assert_eq!(value.validate(), Err("schema version must be positive"));
    }

    #[test]
    fn aggregate_ref_is_explicit() {
        let reference = event().aggregate_ref().unwrap();
        assert_eq!(reference.resource_type, crate::ResourceType::Case);
        assert_eq!(reference.id, "case-1");
    }
}
