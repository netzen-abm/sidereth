use serde::{Deserialize, Serialize};

use crate::{Id, ResourceRef};

/// Resource-neutral lifecycle transition metadata. Domain resources retain
/// their own valid state machines; this contract standardizes the transition
/// record around them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleTransition {
    pub transition_id: Id,
    pub resource_ref: ResourceRef,
    pub from_state: Option<String>,
    pub to_state: String,
    pub actor_ref: ResourceRef,
    pub occurred_at: String,
    pub reason: Option<String>,
    pub authorization_ref: Option<ResourceRef>,
    pub provenance_ref: Option<ResourceRef>,
    pub correlation_id: Id,
    pub causation_id: Option<Id>,
}

impl LifecycleTransition {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.transition_id.is_empty() { return Err("transition id is required"); }
        if self.resource_ref.id.is_empty() { return Err("transition resource is required"); }
        if self.to_state.is_empty() { return Err("transition target state is required"); }
        if self.actor_ref.id.is_empty() { return Err("transition actor is required"); }
        if self.occurred_at.is_empty() { return Err("transition time is required"); }
        if self.correlation_id.is_empty() { return Err("transition correlation id is required"); }
        Ok(())
    }
}

/// Common lifecycle metadata without imposing a universal state vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleMeta {
    pub state: String,
    pub state_changed_at: Option<String>,
    pub state_changed_by: Option<ResourceRef>,
}

impl LifecycleMeta {
    pub fn new(state: impl Into<String>) -> Result<Self, &'static str> {
        let state = state.into();
        if state.is_empty() { return Err("lifecycle state is required"); }
        Ok(Self { state, state_changed_at: None, state_changed_by: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceType;

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    #[test]
    fn lifecycle_transition_validates() {
        let transition = LifecycleTransition {
            transition_id: "transition-1".into(),
            resource_ref: reference(ResourceType::Case, "case-1"),
            from_state: Some("draft".into()), to_state: "active".into(),
            actor_ref: reference(ResourceType::Party, "party-1"),
            occurred_at: "2026-09-04T10:00:00Z".into(), reason: Some("submitted".into()),
            authorization_ref: None, provenance_ref: None,
            correlation_id: "corr-1".into(), causation_id: None,
        };
        assert!(transition.validate().is_ok());
    }

    #[test]
    fn lifecycle_meta_does_not_impose_a_universal_state_vocabulary() {
        assert!(LifecycleMeta::new("waiting_authority").is_ok());
        assert!(LifecycleMeta::new("under_review").is_ok());
    }

    #[test]
    fn lifecycle_transition_wire_contract_is_stable() {
        let transition = LifecycleTransition {
            transition_id: "transition-1".into(), resource_ref: reference(ResourceType::Case, "case-1"),
            from_state: None, to_state: "draft".into(), actor_ref: reference(ResourceType::Party, "p-1"),
            occurred_at: "2026-09-04T10:00:00Z".into(), reason: None, authorization_ref: None,
            provenance_ref: None, correlation_id: "corr-1".into(), causation_id: None,
        };
        let json = serde_json::to_string(&transition).unwrap();
        let decoded: LifecycleTransition = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, transition);
    }
}
