use serde::{Deserialize, Serialize};

use crate::{AuthorizationDecision, AuthorizationResult, Id, ResourceRef, ResourceType};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalOrigin {
    Human,
    System,
    Intelligence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Granted,
    Rejected,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalRecord {
    pub approval_id: Id,
    pub action_ref: ResourceRef,
    pub approver_ref: ResourceRef,
    pub authorization_ref: ResourceRef,
    pub decision: ApprovalDecision,
    pub origin: ApprovalOrigin,
    pub rationale: String,
    pub provenance_ref: ResourceRef,
    pub decided_at: String,
}

impl ApprovalRecord {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.approval_id.is_empty() {
            return Err("approval id is required");
        }
        if self.action_ref.resource_type != ResourceType::Action {
            return Err("approval action reference must target an action");
        }
        if self.approver_ref.id.is_empty() {
            return Err("approval approver reference is required");
        }
        if self.authorization_ref.id.is_empty() {
            return Err("approval authorization reference is required");
        }
        if self.rationale.trim().is_empty() {
            return Err("approval rationale is required");
        }
        if self.provenance_ref.id.is_empty() {
            return Err("approval provenance reference is required");
        }
        if self.decided_at.is_empty() {
            return Err("approval decision timestamp is required");
        }
        Ok(())
    }

    pub fn grants_execution(&self) -> bool {
        self.decision == ApprovalDecision::Granted && self.origin == ApprovalOrigin::Human
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionGateError {
    AuthorizationRequired,
    AuthorizationDenied,
    ApprovalRequired,
    ApprovalMismatch,
    ApprovalNotGranted,
    ApprovalNotHuman,
    ActionNotApproved,
}

impl ExecutionGateError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthorizationRequired => "authorization is required for execution",
            Self::AuthorizationDenied => "authorization does not permit execution",
            Self::ApprovalRequired => "human approval is required for execution",
            Self::ApprovalMismatch => "approval does not match the action authorization",
            Self::ApprovalNotGranted => "approval decision does not grant execution",
            Self::ApprovalNotHuman => "only human approval can permit consequential execution",
            Self::ActionNotApproved => "action must be approved before execution",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionGateInput<'a> {
    pub authorization: Option<&'a AuthorizationResult>,
    pub approval: Option<&'a ApprovalRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionGate;

impl ExecutionGate {
    pub fn permit(
        action: &Action,
        input: ExecutionGateInput<'_>,
    ) -> Result<(), ExecutionGateError> {
        let authorization = input
            .authorization
            .ok_or(ExecutionGateError::AuthorizationRequired)?;
        if authorization.decision != AuthorizationDecision::Allow {
            return Err(ExecutionGateError::AuthorizationDenied);
        }

        let action_authorization = action
            .authorization_ref
            .as_ref()
            .ok_or(ExecutionGateError::AuthorizationRequired)?;
        if authorization.authorization_ref.id.as_str() != action_authorization.as_str() {
            return Err(ExecutionGateError::ApprovalMismatch);
        }

        if !action.requires_explicit_approval {
            return Ok(());
        }

        let approval = input.approval.ok_or(ExecutionGateError::ApprovalRequired)?;
        if approval.action_ref.id != action.action_id
            || approval.authorization_ref.id.as_str() != action_authorization.as_str()
        {
            return Err(ExecutionGateError::ApprovalMismatch);
        }
        if approval.origin != ApprovalOrigin::Human {
            return Err(ExecutionGateError::ApprovalNotHuman);
        }
        if approval.decision != ApprovalDecision::Granted {
            return Err(ExecutionGateError::ApprovalNotGranted);
        }
        if action.status != ActionStatus::Approved {
            return Err(ExecutionGateError::ActionNotApproved);
        }
        Ok(())
    }
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
    pub approval_ref: Option<Id>,
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
            approval_ref: None,
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
        if self.requires_explicit_approval {
            if self.authorization_ref.is_none() {
                return Err("authorization reference is required for explicitly approved actions");
            }
            if self.approval_ref.is_none() {
                return Err("approval reference is required for explicitly approved actions");
            }
        }
        Ok(())
    }

    pub fn bind_approval(
        &mut self,
        approval: &ApprovalRecord,
        updated_at: String,
    ) -> Result<(), &'static str> {
        approval.validate()?;
        if approval.action_ref.id != self.action_id {
            return Err("approval action reference does not match action");
        }
        if self.authorization_ref.as_deref() != Some(approval.authorization_ref.id.as_str()) {
            return Err("approval authorization reference does not match action");
        }
        if !approval.grants_execution() {
            return Err("approval decision does not grant execution");
        }
        if updated_at.is_empty() {
            return Err("action update timestamp is required");
        }
        self.approval_ref = Some(approval.approval_id.clone());
        self.updated_at = updated_at;
        Ok(())
    }

    pub fn can_transition_to(&self, next: &ActionStatus) -> bool {
        if *next == ActionStatus::Approved && self.requires_explicit_approval {
            return self.authorization_ref.is_some() && self.approval_ref.is_some();
        }
        if *next == ActionStatus::Executing && self.requires_explicit_approval {
            return self.status == ActionStatus::Approved
                && self.authorization_ref.is_some()
                && self.approval_ref.is_some();
        }
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
    use crate::ResourceType;

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

    fn authorization(decision: AuthorizationDecision) -> AuthorizationResult {
        AuthorizationResult {
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            decision,
            constraints: Vec::new(),
            policy_refs: vec![ResourceRef::new(ResourceType::Other, "policy-1").unwrap()],
            evaluated_at_epoch_seconds: 100,
            expires_at_epoch_seconds: Some(160),
        }
    }

    fn approval(decision: ApprovalDecision) -> ApprovalRecord {
        ApprovalRecord {
            approval_id: "approval-1".into(),
            action_ref: ResourceRef::new(ResourceType::Action, "action-1").unwrap(),
            approver_ref: ResourceRef::new(ResourceType::Party, "approver-1").unwrap(),
            authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
            decision,
            origin: ApprovalOrigin::Human,
            rationale: "Reviewed and approved within delegated authority".into(),
            provenance_ref: ResourceRef::new(ResourceType::Provenance, "prov-approval-1").unwrap(),
            decided_at: "2026-09-04T10:01:00Z".into(),
        }
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
    fn explicit_approval_requires_authorization_and_approval_references() {
        let mut value = action();
        value.requires_explicit_approval = true;
        assert_eq!(
            value.validate(),
            Err("authorization reference is required for explicitly approved actions")
        );
        value.authorization_ref = Some("auth-1".into());
        assert_eq!(
            value.validate(),
            Err("approval reference is required for explicitly approved actions")
        );
    }

    #[test]
    fn explicit_approval_must_be_bound_to_matching_authorization_and_action() {
        let mut value = action();
        value.requires_explicit_approval = true;
        value.authorization_ref = Some("auth-1".into());
        let record = approval(ApprovalDecision::Granted);
        value
            .bind_approval(&record, "2026-09-04T10:02:00Z".into())
            .unwrap();
        value
            .transition(ActionStatus::Approved, "2026-09-04T10:03:00Z".into())
            .unwrap();
        assert_eq!(value.status, ActionStatus::Approved);
        assert_eq!(value.approval_ref.as_deref(), Some("approval-1"));
    }

    #[test]
    fn rejected_or_revoked_approval_cannot_bind_execution_authority() {
        for decision in [ApprovalDecision::Rejected, ApprovalDecision::Revoked] {
            let mut value = action();
            value.requires_explicit_approval = true;
            value.authorization_ref = Some("auth-1".into());
            assert_eq!(
                value.bind_approval(&approval(decision), "2026-09-04T10:02:00Z".into()),
                Err("approval decision does not grant execution")
            );
        }
    }

    #[test]
    fn mismatched_approval_is_rejected() {
        let mut value = action();
        value.requires_explicit_approval = true;
        value.authorization_ref = Some("auth-1".into());
        let mut record = approval(ApprovalDecision::Granted);
        record.action_ref = ResourceRef::new(ResourceType::Action, "other-action").unwrap();
        assert_eq!(
            value.bind_approval(&record, "2026-09-04T10:02:00Z".into()),
            Err("approval action reference does not match action")
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
    fn explicitly_approved_action_cannot_skip_approval_binding() {
        let mut value = action();
        value.requires_explicit_approval = true;
        value.authorization_ref = Some("auth-1".into());
        assert_eq!(
            value.transition(ActionStatus::Approved, "2026-09-04T10:01:00Z".into()),
            Err("invalid action state transition")
        );
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
    fn approval_decision_wire_values_are_stable() {
        assert_eq!(
            serde_json::to_string(&ApprovalDecision::Granted).unwrap(),
            "\"granted\""
        );
    }
}
