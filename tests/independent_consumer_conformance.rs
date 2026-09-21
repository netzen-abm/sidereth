use sidereth_core::{
    Action, ActionKind, ApprovalDecision, ApprovalOrigin, ApprovalRecord, AuthorizationDecision,
    AuthorizationResult, CapabilityLease, CapabilityLeaseState, ExecutionGate, ExecutionGateInput,
    ResourceRef, ResourceType,
};

fn r(kind: ResourceType, id: &str) -> ResourceRef {
    ResourceRef::new(kind, id).expect("valid resource reference")
}

fn authorization() -> AuthorizationResult {
    AuthorizationResult {
        request_id: "request-1".into(),
        authorization_ref: r(ResourceType::Other, "auth-1"),
        subject_ref: r(ResourceType::Party, "subject-1"),
        action: r(ResourceType::Action, "action-1"),
        resource_ref: r(ResourceType::Document, "document-1"),
        purpose: "independent consumer conformance".into(),
        jurisdiction_ref: Some(r(ResourceType::Jurisdiction, "jurisdiction-1")),
        data_class: Some("restricted".into()),
        decision: AuthorizationDecision::Allow,
        constraints: Vec::new(),
        policy_refs: vec![r(ResourceType::Other, "policy-1")],
        evaluated_at_epoch_seconds: 100,
        expires_at_epoch_seconds: Some(200),
    }
}

fn action() -> Action {
    let mut action = Action::new(
        "action-1".into(),
        ActionKind::ExternalOperation,
        "subject-1".into(),
        "Execute a bounded independent-consumer operation".into(),
        "provenance-1".into(),
        "2026-09-21T10:00:00Z".into(),
    )
    .expect("valid action");
    action.authorization_ref = Some("auth-1".into());
    action.requires_explicit_approval = true;
    action
}

fn approval() -> ApprovalRecord {
    ApprovalRecord {
        approval_id: "approval-1".into(),
        action_ref: r(ResourceType::Action, "action-1"),
        approver_ref: r(ResourceType::Party, "approver-1"),
        authorization_ref: r(ResourceType::Other, "auth-1"),
        decision: ApprovalDecision::Granted,
        origin: ApprovalOrigin::Human,
        rationale: "Explicitly reviewed by the human approver".into(),
        provenance_ref: r(ResourceType::Provenance, "approval-provenance-1"),
        decided_at: "2026-09-21T10:01:00Z".into(),
    }
}

fn active_lease() -> CapabilityLease {
    CapabilityLease {
        lease_id: r(ResourceType::Other, "lease-1"),
        schema_version: "0.1".into(),
        capability_ref: r(ResourceType::Other, "bounded-operation"),
        resource_ref: Some(r(ResourceType::Document, "document-1")),
        subject_ref: r(ResourceType::Party, "subject-1"),
        actor_ref: Some(r(ResourceType::Other, "independent-consumer")),
        purpose: "independent consumer conformance".into(),
        purpose_version: Some("1".into()),
        authorization_ref: r(ResourceType::Other, "auth-1"),
        incident_ref: None,
        session_ref: Some(r(ResourceType::Other, "session-1")),
        scope: "document-1:read".into(),
        issued_at_epoch_seconds: 100,
        expires_at_epoch_seconds: 200,
        state: CapabilityLeaseState::Active,
        revoked_at_epoch_seconds: None,
        cancelled_at_epoch_seconds: None,
        activated_at_epoch_seconds: Some(110),
        released_at_epoch_seconds: None,
        adapter_ref: None,
        audit_ref: None,
    }
}

#[test]
fn independent_consumer_can_use_public_control_plane_contracts() {
    let authorization = authorization();
    let mut action = action();
    let approval = approval();

    action
        .bind_approval(&approval, "2026-09-21T10:01:30Z".into())
        .expect("approval must bind to the action");
    action
        .transition(
            sidereth_core::ActionStatus::Approved,
            "2026-09-21T10:02:00Z".into(),
        )
        .expect("approved action must transition");

    ExecutionGate::permit(
        &action,
        ExecutionGateInput {
            authorization: Some(&authorization),
            approval: Some(&approval),
        },
    )
    .expect("canonical execution gate must permit the matching context");

    let lease = active_lease();
    lease
        .validate_use(
            150,
            &lease.capability_ref,
            lease.resource_ref.as_ref(),
            &lease.subject_ref,
            lease.actor_ref.as_ref(),
            &lease.purpose,
            lease.purpose_version.as_deref(),
            &lease.scope,
            lease.incident_ref.as_ref(),
            lease.session_ref.as_ref(),
        )
        .expect("lease must permit its exact bound context");
}

#[test]
fn independent_consumer_cannot_widen_execution_context() {
    let mut authorization = authorization();
    authorization.decision = AuthorizationDecision::Deny;
    let action = action();
    let approval = approval();

    let result = ExecutionGate::permit(
        &action,
        ExecutionGateInput {
            authorization: Some(&authorization),
            approval: Some(&approval),
        },
    );

    assert!(matches!(
        result,
        Err(sidereth_core::ExecutionGateError::AuthorizationDenied)
    ));
}
