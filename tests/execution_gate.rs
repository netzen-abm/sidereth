use sidereth_core::{
    Action, ActionKind, ActionStatus, ApprovalDecision, ApprovalOrigin, ApprovalRecord,
    AuthorizationDecision, AuthorizationResult, ExecutionGate, ExecutionGateError,
    ExecutionGateInput, ResourceRef, ResourceType,
};

fn action() -> Action {
    let mut value = Action::new(
        "action-1".into(),
        ActionKind::Submission,
        "actor-1".into(),
        "Submit the reviewed filing".into(),
        "prov-action-1".into(),
        "2026-09-08T10:00:00Z".into(),
    )
    .unwrap();
    value.requires_explicit_approval = true;
    value.authorization_ref = Some("auth-1".into());
    value
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

fn approval(decision: ApprovalDecision, origin: ApprovalOrigin) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: "approval-1".into(),
        action_ref: ResourceRef::new(ResourceType::Action, "action-1").unwrap(),
        approver_ref: ResourceRef::new(ResourceType::Party, "human-1").unwrap(),
        authorization_ref: ResourceRef::new(ResourceType::Other, "auth-1").unwrap(),
        decision,
        origin,
        rationale: "Reviewed and explicitly approved for execution".into(),
        provenance_ref: ResourceRef::new(ResourceType::Provenance, "prov-approval-1").unwrap(),
        decided_at: "2026-09-08T10:01:00Z".into(),
    }
}

fn gate_input<'a>(
    authorization: Option<&'a AuthorizationResult>,
    approval: Option<&'a ApprovalRecord>,
) -> ExecutionGateInput<'a> {
    ExecutionGateInput {
        authorization,
        approval,
    }
}

#[test]
fn missing_authorization_is_rejected() {
    let value = action();
    let result = ExecutionGate::permit(&value, gate_input(None, None));
    assert_eq!(result, Err(ExecutionGateError::AuthorizationRequired));
}

#[test]
fn not_applicable_authorization_is_rejected() {
    let value = action();
    let result = ExecutionGate::permit(
        &value,
        gate_input(
            Some(&authorization(AuthorizationDecision::NotApplicable)),
            None,
        ),
    );
    assert_eq!(result, Err(ExecutionGateError::AuthorizationDenied));
}

#[test]
fn denied_authorization_is_rejected() {
    let value = action();
    let result = ExecutionGate::permit(
        &value,
        gate_input(Some(&authorization(AuthorizationDecision::Deny)), None),
    );
    assert_eq!(result, Err(ExecutionGateError::AuthorizationDenied));
}

#[test]
fn wrong_authorization_reference_is_rejected() {
    let value = action();
    let mut result = authorization(AuthorizationDecision::Allow);
    result.authorization_ref = ResourceRef::new(ResourceType::Other, "auth-other").unwrap();
    let result = ExecutionGate::permit(&value, gate_input(Some(&result), None));
    assert_eq!(result, Err(ExecutionGateError::ApprovalMismatch));
}

#[test]
fn missing_human_approval_is_rejected() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), None));
    assert_eq!(result, Err(ExecutionGateError::ApprovalRequired));
}

#[test]
fn mismatched_action_approval_is_rejected() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let mut record = approval(ApprovalDecision::Granted, ApprovalOrigin::Human);
    record.action_ref = ResourceRef::new(ResourceType::Action, "other-action").unwrap();
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ApprovalMismatch));
}

#[test]
fn mismatched_approval_authorization_is_rejected() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let mut record = approval(ApprovalDecision::Granted, ApprovalOrigin::Human);
    record.authorization_ref = ResourceRef::new(ResourceType::Other, "auth-other").unwrap();
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ApprovalMismatch));
}

#[test]
fn rejected_approval_is_rejected() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let record = approval(ApprovalDecision::Rejected, ApprovalOrigin::Human);
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ApprovalNotGranted));
}

#[test]
fn revoked_approval_is_rejected() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let record = approval(ApprovalDecision::Revoked, ApprovalOrigin::Human);
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ApprovalNotGranted));
}

#[test]
fn intelligence_produced_approval_is_rejected_even_when_marked_granted() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let record = approval(ApprovalDecision::Granted, ApprovalOrigin::Intelligence);
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ApprovalNotHuman));
}

#[test]
fn system_produced_approval_is_rejected_even_when_marked_granted() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let record = approval(ApprovalDecision::Granted, ApprovalOrigin::System);
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ApprovalNotHuman));
}

#[test]
fn human_approval_cannot_execute_an_unapproved_action() {
    let value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let record = approval(ApprovalDecision::Granted, ApprovalOrigin::Human);
    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::ActionNotApproved));
}

#[test]
fn valid_authorization_and_matching_human_grant_permit_execution_after_approval() {
    let mut value = action();
    let authorization = authorization(AuthorizationDecision::Allow);
    let record = approval(ApprovalDecision::Granted, ApprovalOrigin::Human);
    value
        .bind_approval(&record, "2026-09-08T10:02:00Z".into())
        .unwrap();
    value
        .transition(ActionStatus::Approved, "2026-09-08T10:03:00Z".into())
        .unwrap();

    let result = ExecutionGate::permit(&value, gate_input(Some(&authorization), Some(&record)));
    assert_eq!(result, Ok(()));
}

#[test]
fn approved_action_still_requires_canonical_authorization_result() {
    let mut value = action();
    let record = approval(ApprovalDecision::Granted, ApprovalOrigin::Human);
    value
        .bind_approval(&record, "2026-09-08T10:02:00Z".into())
        .unwrap();
    value
        .transition(ActionStatus::Approved, "2026-09-08T10:03:00Z".into())
        .unwrap();

    let result = ExecutionGate::permit(&value, gate_input(None, Some(&record)));
    assert_eq!(result, Err(ExecutionGateError::AuthorizationRequired));
}

#[test]
fn execution_gate_error_wire_free_contract_is_stable() {
    assert_eq!(
        ExecutionGateError::ApprovalNotHuman.as_str(),
        "only human approval can permit consequential execution"
    );
}
