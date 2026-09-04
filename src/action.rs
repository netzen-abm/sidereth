use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Proposed,
    Approved,
    Rejected,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Information,
    Communication,
    DocumentCreation,
    EvidenceOperation,
    Submission,
    Escalation,
    Decision,
    ExternalOperation,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Action {
    pub action_id: Id,
    pub schema_version: u32,
    pub kind: ActionKind,
    pub status: ActionStatus,
    pub actor_id: Id,
    pub context_refs: Vec<Id>,
    pub target_refs: Vec<Id>,
    pub intent: String,
    pub authorization_ref: Option<Id>,
    pub precondition_refs: Vec<Id>,
    pub input_refs: Vec<Id>,
    pub output_refs: Vec<Id>,
    pub evidence_refs: Vec<Id>,
    pub requires_explicit_approval: bool,
    pub provenance_ref: Id,
    pub created_at: String,
    pub updated_at: String,
}

impl Action {
    pub fn new(
        action_id: Id,
        kind: ActionKind,
        actor_id: Id,
        intent: String,
        provenance_ref: Id,
        created_at: String,
    ) -> Result<Self, &'static str> {
        let value = Self {
            action_id,
            schema_version: 1,
            kind,
            status: ActionStatus::Proposed,
            actor_id,
            context_refs: Vec::new(),
            target_refs: Vec::new(),
            intent,
            authorization_ref: None,
            precondition_refs: Vec::new(),
            input_refs: Vec::new(),
            output_refs: Vec::new(),
            evidence_refs: Vec::new(),
            requires_explicit_approval: false,
            provenance_ref,
            created_at: created_at.clone(),
            updated_at: created_at,
        };

        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.action_id.is_empty() {
            return Err("action id is required");
        }
        if self.schema_version == 0 {
            return Err("schema version must be positive");
        }
        if self.actor_id.is_empty() {
            return Err("actor id is required");
        }
        if self.intent.trim().is_empty() {
            return Err("action intent is required");
        }
        if self.provenance_ref.is_empty() {
            return Err("provenance reference is required");
        }
        if self.created_at.is_empty() || self.updated_at.is_empty() {
            return Err("action timestamps are required");
        }
        if self.requires_explicit_approval && self.authorization_ref.is_none() {
            return Err("authorization reference is required for explicitly approved actions");
        }
        Ok(())
    }

    pub fn can_transition_to(&self, next: &ActionStatus) -> bool {
        matches!(
            (&self.status, next),
            (ActionStatus::Proposed, ActionStatus::Approved)
                | (ActionStatus::Proposed, ActionStatus::Rejected)
                | (ActionStatus::Proposed, ActionStatus::Cancelled)
                | (ActionStatus::Approved, ActionStatus::Executing)
                | (ActionStatus::Approved, ActionStatus::Cancelled)
                | (ActionStatus::Executing, ActionStatus::Completed)
                | (ActionStatus::Executing, ActionStatus::Failed)
                | (ActionStatus::Executing, ActionStatus::Cancelled)
        )
    }

    pub fn transition(
        &mut self,
        next: ActionStatus,
        updated_at: String,
    ) -> Result<(), &'static str> {
        if !self.can_transition_to(&next) {
            return Err("invalid action state transition");
        }
        if updated_at.is_empty() {
            return Err("action update timestamp is required");
        }
        self.status = next;
        self.updated_at = updated_at;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action() -> Action {
        Action::new(
            "action-1".into(),
            ActionKind::Submission,
            "actor-1".into(),
            "Prepare a submission for review".into(),
            "prov-1".into(),
            "2026-09-04T10:00:00Z".into(),
        )
        .unwrap()
    }

    #[test]
    fn new_action_starts_proposed() {
        assert_eq!(action().status, ActionStatus::Proposed);
    }

    #[test]
    fn action_requires_intent() {
        let result = Action::new(
            "action-1".into(),
            ActionKind::Information,
            "actor-1".into(),
            " ".into(),
            "prov-1".into(),
            "2026-09-04T10:00:00Z".into(),
        );
        assert_eq!(result, Err("action intent is required"));
    }

    #[test]
    fn explicit_approval_requires_authorization_reference() {
        let mut value = action();
        value.requires_explicit_approval = true;
        assert_eq!(
            value.validate(),
            Err("authorization reference is required for explicitly approved actions")
        );
    }

    #[test]
    fn approved_action_requires_proposal_first() {
        let mut value = action();
        value
            .transition(ActionStatus::Approved, "2026-09-04T10:01:00Z".into())
            .unwrap();
        assert_eq!(value.status, ActionStatus::Approved);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut value = action();
        assert_eq!(
            value.transition(ActionStatus::Completed, "2026-09-04T10:01:00Z".into()),
            Err("invalid action state transition")
        );
    }

    #[test]
    fn public_enums_use_canonical_snake_case_wire_values() {
        assert_eq!(serde_json::to_string(&ActionKind::DocumentCreation).unwrap(), "\"document_creation\"");
        assert_eq!(serde_json::to_string(&ActionStatus::Proposed).unwrap(), "\"proposed\"");
    }
}
