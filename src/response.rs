use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseState {
    Draft,
    ReviewRequired,
    Approved,
    Submitted,
    Withdrawn,
    Resolved,
}

impl ResponseState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::ReviewRequired)
                | (Self::Draft, Self::Approved)
                | (Self::Draft, Self::Withdrawn)
                | (Self::ReviewRequired, Self::Approved)
                | (Self::ReviewRequired, Self::Draft)
                | (Self::ReviewRequired, Self::Withdrawn)
                | (Self::Approved, Self::Submitted)
                | (Self::Approved, Self::Withdrawn)
                | (Self::Submitted, Self::Resolved)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub response_id: Id,
    pub case_id: Id,
    pub obligation_id: Option<Id>,
    pub title: String,
    pub content_ref: Id,
    pub state: ResponseState,
    pub evidence_refs: Vec<Id>,
    pub source_refs: Vec<Id>,
}

impl Response {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.response_id.is_empty() {
            return Err("response id is required");
        }
        if self.case_id.is_empty() {
            return Err("response case is required");
        }
        if self.title.is_empty() {
            return Err("response title is required");
        }
        if self.content_ref.is_empty() {
            return Err("response content reference is required");
        }
        if self.source_refs.is_empty() {
            return Err("response source references are required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscalationState {
    Draft,
    Ready,
    Submitted,
    Resolved,
    Withdrawn,
}

impl EscalationState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Ready)
                | (Self::Draft, Self::Withdrawn)
                | (Self::Ready, Self::Submitted)
                | (Self::Ready, Self::Withdrawn)
                | (Self::Submitted, Self::Resolved)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Escalation {
    pub escalation_id: Id,
    pub case_id: Id,
    pub reason: String,
    pub target_ref: Id,
    pub response_id: Option<Id>,
    pub state: EscalationState,
    pub evidence_refs: Vec<Id>,
    pub source_refs: Vec<Id>,
}

impl Escalation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.escalation_id.is_empty() {
            return Err("escalation id is required");
        }
        if self.case_id.is_empty() {
            return Err("escalation case is required");
        }
        if self.reason.is_empty() {
            return Err("escalation reason is required");
        }
        if self.target_ref.is_empty() {
            return Err("escalation target is required");
        }
        if self.source_refs.is_empty() {
            return Err("escalation source references are required");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ResponseRegistry {
    responses: HashMap<Id, Response>,
    escalations: HashMap<Id, Escalation>,
}

impl ResponseRegistry {
    pub fn insert_response(&mut self, response: Response) -> Result<(), &'static str> {
        response.validate()?;
        if self.responses.contains_key(&response.response_id) {
            return Err("duplicate response id");
        }
        self.responses
            .insert(response.response_id.clone(), response);
        Ok(())
    }

    pub fn transition_response(
        &mut self,
        response_id: &Id,
        next: ResponseState,
    ) -> Result<(), &'static str> {
        let response = self
            .responses
            .get_mut(response_id)
            .ok_or("response not found")?;
        if !response.state.can_transition_to(&next) {
            return Err("invalid response state transition");
        }
        response.state = next;
        Ok(())
    }

    pub fn insert_escalation(
        &mut self,
        escalation: Escalation,
    ) -> Result<(), &'static str> {
        escalation.validate()?;
        if self.escalations.contains_key(&escalation.escalation_id) {
            return Err("duplicate escalation id");
        }
        if let Some(response_id) = &escalation.response_id {
            if !self.responses.contains_key(response_id) {
                return Err("escalation response not found");
            }
        }
        self.escalations
            .insert(escalation.escalation_id.clone(), escalation);
        Ok(())
    }

    pub fn transition_escalation(
        &mut self,
        escalation_id: &Id,
        next: EscalationState,
    ) -> Result<(), &'static str> {
        let escalation = self
            .escalations
            .get_mut(escalation_id)
            .ok_or("escalation not found")?;
        if !escalation.state.can_transition_to(&next) {
            return Err("invalid escalation state transition");
        }
        escalation.state = next;
        Ok(())
    }

    pub fn get_response(&self, id: &Id) -> Option<&Response> {
        self.responses.get(id)
    }

    pub fn get_escalation(&self, id: &Id) -> Option<&Escalation> {
        self.escalations.get(id)
    }

    pub fn response_ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.responses.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn escalation_ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.escalations.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response() -> Response {
        Response {
            response_id: "resp-1".into(),
            case_id: "case-1".into(),
            obligation_id: Some("obl-1".into()),
            title: "Response to notice".into(),
            content_ref: "doc-1".into(),
            state: ResponseState::Draft,
            evidence_refs: vec!["ev-1".into()],
            source_refs: vec!["src-1".into()],
        }
    }

    fn escalation() -> Escalation {
        Escalation {
            escalation_id: "esc-1".into(),
            case_id: "case-1".into(),
            reason: "Response remains unresolved".into(),
            target_ref: "authority-2".into(),
            response_id: Some("resp-1".into()),
            state: EscalationState::Draft,
            evidence_refs: vec!["ev-1".into()],
            source_refs: vec!["src-1".into()],
        }
    }

    #[test]
    fn response_lifecycle_is_deterministic() {
        let mut registry = ResponseRegistry::default();
        registry.insert_response(response()).unwrap();
        registry
            .transition_response(&"resp-1".into(), ResponseState::ReviewRequired)
            .unwrap();
        registry
            .transition_response(&"resp-1".into(), ResponseState::Approved)
            .unwrap();
        registry
            .transition_response(&"resp-1".into(), ResponseState::Submitted)
            .unwrap();
        assert_eq!(
            registry.get_response(&"resp-1".into()).unwrap().state,
            ResponseState::Submitted
        );
    }

    #[test]
    fn invalid_response_transition_is_rejected() {
        let mut registry = ResponseRegistry::default();
        registry.insert_response(response()).unwrap();
        assert_eq!(
            registry.transition_response(&"resp-1".into(), ResponseState::Submitted),
            Err("invalid response state transition")
        );
    }

    #[test]
    fn escalation_requires_existing_response_when_linked() {
        let mut registry = ResponseRegistry::default();
        assert_eq!(
            registry.insert_escalation(escalation()),
            Err("escalation response not found")
        );
    }

    #[test]
    fn escalation_lifecycle_is_deterministic() {
        let mut registry = ResponseRegistry::default();
        registry.insert_response(response()).unwrap();
        registry.insert_escalation(escalation()).unwrap();
        registry
            .transition_escalation(&"esc-1".into(), EscalationState::Ready)
            .unwrap();
        registry
            .transition_escalation(&"esc-1".into(), EscalationState::Submitted)
            .unwrap();
        assert_eq!(
            registry.get_escalation(&"esc-1".into()).unwrap().state,
            EscalationState::Submitted
        );
    }

    #[test]
    fn duplicate_response_ids_are_rejected() {
        let mut registry = ResponseRegistry::default();
        registry.insert_response(response()).unwrap();
        assert_eq!(
            registry.insert_response(response()),
            Err("duplicate response id")
        );
    }
}
