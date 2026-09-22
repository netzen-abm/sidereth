use super::*;
use crate::audit::AuditSink;
use crate::authorization::{AuthorizationConstraint, AuthorizationDecision};
use crate::persistence::{
    IdempotencyClaim, IdempotencyLifecycleStore, IdempotencyState, IdempotencyStore,
    PersistenceError,
};
use crate::tool_registry::{ToolImplementation, ToolLifecycle, ToolRiskClass, ToolVersion};
use crate::AuthorizationValidationError;
use crate::Id;
use std::collections::BTreeSet;

#[derive(Default)]
struct Idempotency {
    claimed: BTreeSet<Id>,
    fail_mark_completed: bool,
    fail_mark_failed: bool,
    states: std::collections::BTreeMap<Id, IdempotencyState>,
}

impl IdempotencyStore for Idempotency {
    fn lookup(&self, operation_id: &Id) -> Result<bool, PersistenceError> {
        Ok(self.claimed.contains(operation_id))
    }

    fn claim(&mut self, operation_id: Id) -> Result<IdempotencyClaim, PersistenceError> {
        Ok(if self.claimed.insert(operation_id) {
            IdempotencyClaim::Claimed
        } else {
            IdempotencyClaim::AlreadyClaimed
        })
    }
}

impl IdempotencyLifecycleStore for Idempotency {
    fn state(&self, operation_id: &Id) -> Result<Option<IdempotencyState>, PersistenceError> {
        Ok(self.states.get(operation_id).copied())
    }

    fn mark_in_progress(&mut self, operation_id: &Id) -> Result<(), PersistenceError> {
        self.states
            .insert(operation_id.clone(), IdempotencyState::InProgress);
        Ok(())
    }
    fn mark_completed(&mut self, operation_id: &Id) -> Result<(), PersistenceError> {
        if self.fail_mark_completed {
            return Err(PersistenceError::Unavailable);
        }
        self.states
            .insert(operation_id.clone(), IdempotencyState::Completed);
        Ok(())
    }
    fn mark_failed(&mut self, operation_id: &Id) -> Result<(), PersistenceError> {
        if self.fail_mark_failed {
            return Err(PersistenceError::Unavailable);
        }
        self.states
            .insert(operation_id.clone(), IdempotencyState::Failed);
        Ok(())
    }
    fn mark_unknown(&mut self, operation_id: &Id) -> Result<(), PersistenceError> {
        self.states
            .insert(operation_id.clone(), IdempotencyState::Unknown);
        Ok(())
    }
}

#[derive(Default)]
struct Provider {
    calls: usize,
    fail: bool,
}

impl ToolGatewayProvider for Provider {
    type Output = &'static str;

    fn provider_id(&self) -> &str {
        "provider-1"
    }
    fn implementation_id(&self) -> &str {
        "impl-1"
    }
    fn implementation_version(&self) -> &str {
        "1"
    }

    fn execute(
        &mut self,
        _: &ToolGatewayInvocation,
        _: &ToolRegistryEntry,
        _: &ToolDataAccessGrant,
    ) -> Result<Self::Output, ToolGatewayError> {
        self.calls += 1;
        if self.fail {
            return Err(ToolGatewayError::ProviderFailed);
        }
        Ok("ok")
    }
}

fn r(t: crate::ResourceType, id: &str) -> ResourceRef {
    ResourceRef::new(t, id).unwrap()
}

fn registry() -> InMemoryToolRegistry {
    let mut registry = InMemoryToolRegistry::new();
    registry
        .register(
            ToolRegistryEntry {
                tool_id: "tool-1".into(),
                version: ToolVersion::new(1, 0, 0),
                name: "test".into(),
                purpose: "protected read".into(),
                lifecycle: ToolLifecycle::Active,
                risk_class: ToolRiskClass::ReadOnly,
                data_classes: [ToolDataClass::Restricted].into_iter().collect(),
                capability_ref: r(crate::ResourceType::Other, "cap-1"),
                function_ref: None,
                jurisdiction_scope: vec!["IN".into()],
                permission_requirements: vec![],
                capability_lease_required: false,
                approval_required: false,
                input_schema_ref: None,
                output_schema_ref: None,
                execution_modes: [ToolExecutionMode::Sync].into_iter().collect(),
                dependencies: vec![],
                implementations: vec![ToolImplementation {
                    implementation_id: "impl-1".into(),
                    provider_id: "provider-1".into(),
                    implementation_version: "1".into(),
                    adapter_refs: vec![],
                }],
                provenance_requirements: vec![],
                observability_refs: vec![],
                documentation_refs: vec![],
            },
            crate::tool_registry::ToolRegistryAuditRecord {
                change_id: "change-1".into(),
                tool_id: "tool-1".into(),
                version: ToolVersion::new(1, 0, 0),
                change_type: "register".into(),
                actor_ref: "actor-1".into(),
                authorization_ref: "auth-1".into(),
                timestamp: "1000".into(),
                previous_lifecycle: None,
                new_lifecycle: None,
            },
        )
        .unwrap();
    registry
}

fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: "req-1".into(),
        authorization_ref: r(crate::ResourceType::Other, "auth-1"),
        subject_ref: r(crate::ResourceType::Party, "party-1"),
        action: r(crate::ResourceType::Action, "read"),
        resource_ref: r(crate::ResourceType::Case, "case-1"),
        purpose: "protected read".into(),
        policy_refs: vec![r(crate::ResourceType::Other, "policy-1")],
        jurisdiction_ref: Some(r(crate::ResourceType::Jurisdiction, "IN")),
        data_class: Some("restricted".into()),
        requested_at_epoch_seconds: 1000,
        freshness_seconds: Some(100),
    }
}

fn authorization() -> AuthorizationResult {
    let q = request();
    AuthorizationResult {
        request_id: q.request_id.clone(),
        authorization_ref: q.authorization_ref.clone(),
        subject_ref: q.subject_ref.clone(),
        action: q.action.clone(),
        resource_ref: q.resource_ref.clone(),
        purpose: q.purpose.clone(),
        jurisdiction_ref: q.jurisdiction_ref.clone(),
        data_class: q.data_class.clone(),
        decision: AuthorizationDecision::Allow,
        constraints: vec![AuthorizationConstraint {
            key: "scope".into(),
            value: "exact_resource".into(),
        }],
        policy_refs: q.policy_refs.clone(),
        evaluated_at_epoch_seconds: 1000,
        expires_at_epoch_seconds: Some(1100),
    }
}

fn invocation() -> ToolGatewayInvocation {
    ToolGatewayInvocation {
        request_id: "req-1".into(),
        occurred_at: "2026-09-20T10:00:00Z".into(),
        authorization_ref: r(crate::ResourceType::Other, "auth-1"),
        subject_ref: r(crate::ResourceType::Party, "party-1"),
        actor_ref: Some(r(crate::ResourceType::Party, "party-1")),
        capability_ref: r(crate::ResourceType::Other, "cap-1"),
        function_ref: None,
        action: r(crate::ResourceType::Action, "read"),
        resource_ref: r(crate::ResourceType::Case, "case-1"),
        purpose: "protected read".into(),
        purpose_version: None,
        jurisdiction_ref: Some(r(crate::ResourceType::Jurisdiction, "IN")),
        data_class: Some(ToolDataClass::Restricted),
        tool_id: "tool-1".into(),
        tool_version: ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
        implementation_id: "impl-1".into(),
        provider_id: "provider-1".into(),
        implementation_version: "1".into(),
        idempotency_ref: "op-1".into(),
        requested_scope: "case-1".into(),
        execution_mode: ToolExecutionMode::Sync,
        capability_lease: None,
        incident_ref: None,
        session_ref: None,
    }
}

#[test]
fn valid_context_passes_without_claiming() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let gateway = ToolGateway::new(&registry, &mut idempotency);
    assert_eq!(
        gateway.validate(&invocation(), &request(), &authorization(), 1050),
        Ok(ToolGatewayPhase::Validated)
    );
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn capability_mismatch_fails_before_claim_and_provider_execution() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    let mut i = invocation();
    i.capability_ref = r(crate::ResourceType::Other, "cap-forged");
    assert!(matches!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
    ));
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn function_mismatch_fails_before_claim_and_provider_execution() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    let mut i = invocation();
    i.function_ref = Some(r(crate::ResourceType::Other, "fn-forged"));
    assert!(matches!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
    ));
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn implementation_mismatch_fails_before_claim_and_provider_execution() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    let mut i = invocation();
    i.implementation_id = "impl-forged".into();
    assert!(matches!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
    ));
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn provider_mismatch_fails_before_claim_and_provider_execution() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    let mut i = invocation();
    i.provider_id = "provider-forged".into();
    assert!(matches!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
    ));
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn implementation_version_mismatch_fails_before_claim_and_provider_execution() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    let mut i = invocation();
    i.implementation_version = "forged".into();
    assert!(matches!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
    ));
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn provider_object_mismatch_fails_before_claim_and_execution() {
    struct WrongProvider;
    impl ToolGatewayProvider for WrongProvider {
        type Output = &'static str;
        fn provider_id(&self) -> &str {
            "provider-forged"
        }
        fn implementation_id(&self) -> &str {
            "impl-1"
        }
        fn implementation_version(&self) -> &str {
            "1"
        }
        fn execute(
            &mut self,
            _: &ToolGatewayInvocation,
            _: &ToolRegistryEntry,
            _: &ToolDataAccessGrant,
        ) -> Result<Self::Output, ToolGatewayError> {
            panic!("wrong provider must never execute");
        }
    }

    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = WrongProvider;
    let mut audit = crate::InMemoryAudit::default();
    assert!(matches!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
    ));
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn subject_mismatch_fails_closed() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut i = invocation();
    i.subject_ref = r(crate::ResourceType::Party, "party-2");
    assert!(matches!(
        gateway.validate(&i, &request(), &authorization(), 1050),
        Err(ToolGatewayError::Authorization(
            AuthorizationValidationError::RequestResultMismatch
        ))
    ));
}

#[test]
fn exact_expiry_fails_closed() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let gateway = ToolGateway::new(&registry, &mut idempotency);
    assert!(matches!(
        gateway.validate(&invocation(), &request(), &authorization(), 1100),
        Err(ToolGatewayError::Authorization(
            AuthorizationValidationError::Expired
        ))
    ));
}

#[test]
fn exact_resource_scope_binds_to_resource() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut i = invocation();
    i.requested_scope = "case-2".into();
    assert!(matches!(
        gateway.validate(&i, &request(), &authorization(), 1050),
        Err(ToolGatewayError::Authorization(
            AuthorizationValidationError::ConstraintViolation
        ))
    ));
}

#[test]
fn duplicate_context_bound_operation_is_blocked() {
    let registry = registry();
    let mut action = Action::new(
        "read".into(),
        crate::action::ActionKind::Information,
        "party-1".into(),
        "Execute protected tool".into(),
        "prov-action-read".into(),
        "2026-09-20T10:00:00Z".into(),
    )
    .unwrap();
    action.authorization_ref = Some("auth-1".into());
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let q = request();
    let a = authorization();
    let i = invocation();
    let mut p = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    assert_eq!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &q,
                authorization: &a,
                action: Some(&action),
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut p
        ),
        Ok("ok")
    );
    assert_eq!(p.calls, 1);
    assert_eq!(
        gateway.execute(
            &i,
            ToolGatewayExecutionContext {
                request: &q,
                authorization: &a,
                action: Some(&action),
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut p
        ),
        Err(ToolGatewayError::IdempotencyAlreadyClaimed)
    );
    assert_eq!(p.calls, 1);
}

#[test]
fn required_capability_lease_cannot_be_omitted() {
    let mut registry = registry();
    let mut entry = registry
        .resolve(
            &String::from("tool-1"),
            &ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
        )
        .unwrap()
        .clone();
    entry.capability_lease_required = true;
    registry = InMemoryToolRegistry::new();
    registry
        .register(
            entry,
            crate::tool_registry::ToolRegistryAuditRecord {
                change_id: "change-lease".into(),
                tool_id: "tool-1".into(),
                version: ToolVersion::new(1, 0, 0),
                change_type: "register".into(),
                actor_ref: "actor-1".into(),
                authorization_ref: "auth-1".into(),
                timestamp: "1002".into(),
                previous_lifecycle: None,
                new_lifecycle: None,
            },
        )
        .unwrap();
    let mut idempotency = Idempotency::default();
    let gateway = ToolGateway::new(&registry, &mut idempotency);
    assert!(matches!(
        gateway.validate(&invocation(), &request(), &authorization(), 1050),
        Err(ToolGatewayError::Lease(CapabilityLeaseError::InvalidLease))
    ));
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn nonapproval_tool_cannot_bypass_canonical_execution_gate() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();

    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::ExecutionGate(
            ExecutionGateError::AuthorizationRequired
        ))
    );
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn approval_required_fails_before_claim_without_canonical_action() {
    let mut registry = registry();
    let mut entry = registry
        .resolve(
            &String::from("tool-1"),
            &ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
        )
        .unwrap()
        .clone();
    entry.approval_required = true;
    registry = InMemoryToolRegistry::new();
    registry
        .register(
            entry,
            crate::tool_registry::ToolRegistryAuditRecord {
                change_id: "change-approval".into(),
                tool_id: "tool-1".into(),
                version: ToolVersion::new(1, 0, 0),
                change_type: "register".into(),
                actor_ref: "actor-1".into(),
                authorization_ref: "auth-1".into(),
                timestamp: "1003".into(),
                previous_lifecycle: None,
                new_lifecycle: None,
            },
        )
        .unwrap();

    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: None,
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::ExecutionGate(
            ExecutionGateError::AuthorizationRequired
        ))
    );
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

#[test]
fn human_approval_allows_consequential_tool_execution() {
    let mut registry = registry();
    let mut entry = registry
        .resolve(
            &String::from("tool-1"),
            &ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
        )
        .unwrap()
        .clone();
    entry.approval_required = true;
    registry = InMemoryToolRegistry::new();
    registry
        .register(
            entry,
            crate::tool_registry::ToolRegistryAuditRecord {
                change_id: "change-approval-valid".into(),
                tool_id: "tool-1".into(),
                version: ToolVersion::new(1, 0, 0),
                change_type: "register".into(),
                actor_ref: "actor-1".into(),
                authorization_ref: "auth-1".into(),
                timestamp: "1004".into(),
                previous_lifecycle: None,
                new_lifecycle: None,
            },
        )
        .unwrap();

    let mut action = Action::new(
        "read".into(),
        crate::action::ActionKind::ExternalOperation,
        "party-1".into(),
        "Execute protected tool".into(),
        "prov-action-1".into(),
        "2026-09-20T10:00:00Z".into(),
    )
    .unwrap();
    action.requires_explicit_approval = true;
    action.authorization_ref = Some("auth-1".into());

    let approval = ApprovalRecord {
        approval_id: "approval-1".into(),
        action_ref: r(crate::ResourceType::Action, "read"),
        approver_ref: r(crate::ResourceType::Party, "approver-1"),
        authorization_ref: r(crate::ResourceType::Other, "auth-1"),
        decision: crate::action::ApprovalDecision::Granted,
        origin: crate::action::ApprovalOrigin::Human,
        rationale: "Reviewed and approved".into(),
        provenance_ref: r(crate::ResourceType::Provenance, "prov-approval-1"),
        decided_at: "2026-09-20T10:01:00Z".into(),
    };
    action
        .bind_approval(&approval, "2026-09-20T10:01:00Z".into())
        .unwrap();
    action
        .transition(
            crate::action::ActionStatus::Approved,
            "2026-09-20T10:02:00Z".into(),
        )
        .unwrap();

    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: Some(&action),
                approval: Some(&approval),
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Ok("ok")
    );
    assert_eq!(provider.calls, 1);
}

#[test]
fn non_human_approval_cannot_satisfy_consequential_tool_requirement() {
    let mut registry = registry();
    let mut entry = registry
        .resolve(
            &String::from("tool-1"),
            &ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
        )
        .unwrap()
        .clone();
    entry.approval_required = true;
    registry = InMemoryToolRegistry::new();
    registry
        .register(
            entry,
            crate::tool_registry::ToolRegistryAuditRecord {
                change_id: "change-approval-system".into(),
                tool_id: "tool-1".into(),
                version: ToolVersion::new(1, 0, 0),
                change_type: "register".into(),
                actor_ref: "actor-1".into(),
                authorization_ref: "auth-1".into(),
                timestamp: "1005".into(),
                previous_lifecycle: None,
                new_lifecycle: None,
            },
        )
        .unwrap();

    let mut action = Action::new(
        "read".into(),
        crate::action::ActionKind::ExternalOperation,
        "party-1".into(),
        "Execute protected tool".into(),
        "prov-action-1".into(),
        "2026-09-20T10:00:00Z".into(),
    )
    .unwrap();
    action.requires_explicit_approval = true;
    action.authorization_ref = Some("auth-1".into());
    action.approval_ref = Some("approval-1".into());
    action.status = crate::action::ActionStatus::Approved;

    let approval = ApprovalRecord {
        approval_id: "approval-1".into(),
        action_ref: r(crate::ResourceType::Action, "read"),
        approver_ref: r(crate::ResourceType::Party, "approver-1"),
        authorization_ref: r(crate::ResourceType::Other, "auth-1"),
        decision: crate::action::ApprovalDecision::Granted,
        origin: crate::action::ApprovalOrigin::System,
        rationale: "System approval must not satisfy human approval".into(),
        provenance_ref: r(crate::ResourceType::Provenance, "prov-approval-system"),
        decided_at: "2026-09-20T10:01:00Z".into(),
    };

    let mut idempotency = Idempotency::default();
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: Some(&action),
                approval: Some(&approval),
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::ExecutionGate(
            ExecutionGateError::ApprovalNotHuman
        ))
    );
    assert_eq!(provider.calls, 0);
    assert!(idempotency.claimed.is_empty());
}

struct FailingAudit;

impl AuditSink for FailingAudit {
    fn record(&mut self, _: AuditRecord) -> Result<(), &'static str> {
        Err("injected audit failure")
    }
}

impl AuditProvenanceSink for FailingAudit {
    fn record_invocation(
        &mut self,
        _: AuditRecord,
        _: crate::Provenance,
    ) -> Result<(), &'static str> {
        Err("injected audit failure")
    }
}

#[test]
fn provider_success_with_lifecycle_persistence_failure_becomes_unknown() {
    let registry = registry();
    let idempotency = Idempotency {
        fail_mark_completed: true,
        ..Default::default()
    };
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider::default();
    let mut audit = crate::InMemoryAudit::default();
    let mut action = Action::new(
        "read".into(),
        crate::action::ActionKind::Information,
        "party-1".into(),
        "Execute protected tool".into(),
        "prov-action-lifecycle".into(),
        "2026-09-20T10:00:00Z".into(),
    )
    .unwrap();
    action.authorization_ref = Some("auth-1".into());

    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: Some(&action),
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Unknown)
    );
    assert_eq!(provider.calls, 1);
    assert_eq!(
        idempotency
            .state(&operation_key(&invocation()).unwrap())
            .unwrap(),
        Some(IdempotencyState::Unknown)
    );
}

#[test]
fn provider_failure_with_audit_failure_becomes_unknown() {
    let registry = registry();
    let idempotency = Idempotency {
        fail_mark_failed: true,
        ..Default::default()
    };
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider {
        fail: true,
        ..Default::default()
    };
    let mut audit = FailingAudit;
    let mut action = Action::new(
        "read".into(),
        crate::action::ActionKind::Information,
        "party-1".into(),
        "Execute protected tool".into(),
        "prov-action-provider-failure".into(),
        "2026-09-20T10:00:00Z".into(),
    )
    .unwrap();
    action.authorization_ref = Some("auth-1".into());

    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: Some(&action),
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Unknown)
    );
    assert_eq!(provider.calls, 1);
    assert_eq!(
        idempotency
            .state(&operation_key(&invocation()).unwrap())
            .unwrap(),
        Some(IdempotencyState::Unknown)
    );
}

#[test]
fn provider_failure_with_durable_lifecycle_failure_is_unknown_not_failed() {
    let registry = registry();
    let mut idempotency = Idempotency {
        fail_mark_failed: true,
        ..Default::default()
    };
    let mut gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut provider = Provider {
        fail: true,
        ..Default::default()
    };
    let mut audit = crate::InMemoryAudit::default();
    let mut action = Action::new(
        "read".into(),
        crate::action::ActionKind::Information,
        "party-1".into(),
        "Execute protected tool".into(),
        "prov-action-provider-lifecycle".into(),
        "2026-09-20T10:00:00Z".into(),
    )
    .unwrap();
    action.authorization_ref = Some("auth-1".into());

    assert_eq!(
        gateway.execute(
            &invocation(),
            ToolGatewayExecutionContext {
                request: &request(),
                authorization: &authorization(),
                action: Some(&action),
                approval: None,
                now_epoch_seconds: 1050,
                audit: &mut audit,
            },
            &mut provider
        ),
        Err(ToolGatewayError::Unknown)
    );
    assert_eq!(
        idempotency
            .state(&operation_key(&invocation()).unwrap())
            .unwrap(),
        Some(IdempotencyState::Unknown)
    );
}

#[test]
fn unsupported_constraint_fails_closed() {
    let registry = registry();
    let mut idempotency = Idempotency::default();
    let gateway = ToolGateway::new(&registry, &mut idempotency);
    let mut a = authorization();
    a.constraints.push(AuthorizationConstraint {
        key: "scope".into(),
        value: "broader_resource_set".into(),
    });
    assert!(matches!(
        gateway.validate(&invocation(), &request(), &a, 1050),
        Err(ToolGatewayError::Authorization(
            AuthorizationValidationError::ConstraintViolation
        ))
    ));
}
