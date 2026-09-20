//! Bounded, provider-neutral Tool Gateway execution boundary.
//!
//! The gateway composes canonical authorization, Tool Registry resolution,
//! capability leases and canonical idempotency. It does not grant authority.

use crate::action::{
    Action, ApprovalRecord, ExecutionGate, ExecutionGateError, ExecutionGateInput,
};
use crate::authorization::{AuthorizationRequest, AuthorizationResult};
use crate::authorization_enforcement::{validate_authorization, AuthorizationValidationError};
use crate::capability_lease::{CapabilityLease, CapabilityLeaseError};
use crate::persistence::{IdempotencyClaim, IdempotencyLifecycleStore, PersistenceError};
use crate::tool_registry::{
    InMemoryToolRegistry, ToolDataClass, ToolExecutionMode, ToolRegistryEntry, ToolRegistryError,
    ToolVersionRequirement,
};
use crate::{sha256_hex, Id, ResourceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolGatewayInvocation {
    pub request_id: Id,
    pub authorization_ref: ResourceRef,
    pub subject_ref: ResourceRef,
    pub actor_ref: Option<ResourceRef>,
    pub action: ResourceRef,
    pub resource_ref: ResourceRef,
    /// Canonical capability bound by the registered tool contract.
    pub capability_ref: ResourceRef,
    /// Optional canonical function binding declared by the registered tool.
    pub function_ref: Option<ResourceRef>,
    pub purpose: String,
    pub purpose_version: Option<String>,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<ToolDataClass>,
    pub tool_id: Id,
    pub tool_version: ToolVersionRequirement,
    /// Exact implementation selected from the Tool Registry.
    pub implementation_id: Id,
    /// Exact provider identity for the selected implementation.
    pub provider_id: Id,
    /// Provider implementation version bound by the registry contract.
    pub implementation_version: String,
    pub idempotency_ref: Id,
    pub requested_scope: String,
    pub execution_mode: ToolExecutionMode,
    pub capability_lease: Option<CapabilityLease>,
    pub incident_ref: Option<ResourceRef>,
    pub session_ref: Option<ResourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolGatewayError {
    Authorization(AuthorizationValidationError),
    Registry(ToolRegistryError),
    Lease(CapabilityLeaseError),
    MissingIdempotency,
    IdempotencyAlreadyClaimed,
    Idempotency(PersistenceError),
    ToolLifecycleUnavailable,
    DataClassNotSupported,
    JurisdictionNotSupported,
    ExecutionModeNotSupported,
    ApprovalRequired,
    ExecutionGate(ExecutionGateError),
    ProviderFailed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGatewayPhase {
    Validated,
    Claimed,
}

pub trait ToolGatewayProvider {
    type Output;

    /// Stable provider identity. This must match the registry-selected implementation.
    fn provider_id(&self) -> &str;

    /// Stable implementation identity. This must match the registry-selected implementation.
    fn implementation_id(&self) -> &str;

    /// Provider implementation version. This must match the registry contract.
    fn implementation_version(&self) -> &str;

    fn execute(
        &mut self,
        invocation: &ToolGatewayInvocation,
        tool: &ToolRegistryEntry,
    ) -> Result<Self::Output, ToolGatewayError>;
}

pub struct ToolGateway<'a, I: IdempotencyLifecycleStore> {
    registry: &'a InMemoryToolRegistry,
    idempotency: &'a mut I,
}

impl<'a, I: IdempotencyLifecycleStore> ToolGateway<'a, I> {
    pub fn new(registry: &'a InMemoryToolRegistry, idempotency: &'a mut I) -> Self {
        Self {
            registry,
            idempotency,
        }
    }

    /// Side-effect-free control-plane validation. Idempotency is deliberately
    /// claimed only after every authority and execution-context check passes.
    pub fn validate(
        &self,
        invocation: &ToolGatewayInvocation,
        request: &AuthorizationRequest,
        authorization: &AuthorizationResult,
        now_epoch_seconds: u64,
    ) -> Result<ToolGatewayPhase, ToolGatewayError> {
        if invocation.idempotency_ref.trim().is_empty() {
            return Err(ToolGatewayError::MissingIdempotency);
        }
        if !invocation_context_matches_request(invocation, request) {
            return Err(ToolGatewayError::Authorization(
                AuthorizationValidationError::RequestResultMismatch,
            ));
        }
        validate_authorization(request, authorization, now_epoch_seconds)
            .map_err(ToolGatewayError::Authorization)?;

        let tool = self
            .registry
            .resolve(&invocation.tool_id, &invocation.tool_version)
            .map_err(ToolGatewayError::Registry)?;
        validate_registry_context(invocation, tool)?;
        validate_registry_implementation_binding(invocation, tool)?;
        if invocation.capability_ref != tool.capability_ref
            || invocation.function_ref != tool.function_ref
        {
            return Err(ToolGatewayError::Registry(
                ToolRegistryError::UnsupportedVersion,
            ));
        }
        validate_authorized_constraints(&authorization.constraints, invocation)?;

        if tool.capability_lease_required && invocation.capability_lease.is_none() {
            return Err(ToolGatewayError::Lease(CapabilityLeaseError::InvalidLease));
        }

        if let Some(lease) = invocation.capability_lease.as_ref() {
            if lease.authorization_ref != invocation.authorization_ref {
                return Err(ToolGatewayError::Lease(CapabilityLeaseError::InvalidLease));
            }
            lease
                .validate_use(
                    now_epoch_seconds,
                    &tool.capability_ref,
                    Some(&invocation.resource_ref),
                    &invocation.subject_ref,
                    invocation.actor_ref.as_ref(),
                    &invocation.purpose,
                    invocation.purpose_version.as_deref(),
                    &invocation.requested_scope,
                    invocation.incident_ref.as_ref(),
                    invocation.session_ref.as_ref(),
                )
                .map_err(ToolGatewayError::Lease)?;
        }

        Ok(ToolGatewayPhase::Validated)
    }

    /// Claim is intentionally a separate phase from validation. Callers should
    /// normally use execute(), which enforces validation before this claim.
    pub fn claim(
        &mut self,
        invocation: &ToolGatewayInvocation,
    ) -> Result<ToolGatewayPhase, ToolGatewayError> {
        let key = operation_key(invocation)?;
        match self
            .idempotency
            .claim(key)
            .map_err(ToolGatewayError::Idempotency)?
        {
            IdempotencyClaim::Claimed => Ok(ToolGatewayPhase::Claimed),
            IdempotencyClaim::AlreadyClaimed => Err(ToolGatewayError::IdempotencyAlreadyClaimed),
        }
    }

    pub fn execute<P: ToolGatewayProvider>(
        &mut self,
        invocation: &ToolGatewayInvocation,
        request: &AuthorizationRequest,
        authorization: &AuthorizationResult,
        action: Option<&Action>,
        approval: Option<&ApprovalRecord>,
        now_epoch_seconds: u64,
        provider: &mut P,
    ) -> Result<P::Output, ToolGatewayError> {
        self.validate(invocation, request, authorization, now_epoch_seconds)?;
        let tool = self
            .registry
            .resolve(&invocation.tool_id, &invocation.tool_version)
            .map_err(ToolGatewayError::Registry)?;
        validate_provider_binding(invocation, tool, provider)?;
        validate_execution_gate(invocation, tool, authorization, action, approval)?;
        self.claim(invocation)?;
        let operation_id = operation_key(invocation)?;
        self.idempotency
            .mark_in_progress(&operation_id)
            .map_err(ToolGatewayError::Idempotency)?;
        match provider.execute(invocation, tool) {
            Ok(output) => {
                self.idempotency
                    .mark_completed(&operation_id)
                    .map_err(ToolGatewayError::Idempotency)?;
                Ok(output)
            }
            Err(error) => {
                self.idempotency
                    .mark_failed(&operation_id)
                    .map_err(ToolGatewayError::Idempotency)?;
                Err(error)
            }
        }
    }
}

fn invocation_context_matches_request(
    invocation: &ToolGatewayInvocation,
    request: &AuthorizationRequest,
) -> bool {
    invocation.request_id == request.request_id
        && invocation.authorization_ref == request.authorization_ref
        && invocation.subject_ref == request.subject_ref
        && invocation.action == request.action
        && invocation.resource_ref == request.resource_ref
        && invocation.purpose == request.purpose
        && invocation.jurisdiction_ref == request.jurisdiction_ref
        && invocation.data_class.map(data_class_wire).as_deref() == request.data_class.as_deref()
}

fn data_class_wire(value: ToolDataClass) -> String {
    match value {
        ToolDataClass::Public => "public",
        ToolDataClass::Internal => "internal",
        ToolDataClass::Confidential => "confidential",
        ToolDataClass::Restricted => "restricted",
    }
    .to_owned()
}

fn validate_registry_context(
    invocation: &ToolGatewayInvocation,
    tool: &ToolRegistryEntry,
) -> Result<(), ToolGatewayError> {
    if !tool.lifecycle.selectable_for_execution() {
        return Err(ToolGatewayError::ToolLifecycleUnavailable);
    }
    if !tool.execution_modes.contains(&invocation.execution_mode) {
        return Err(ToolGatewayError::ExecutionModeNotSupported);
    }
    if let Some(data_class) = invocation.data_class {
        if !tool.data_classes.contains(&data_class) {
            return Err(ToolGatewayError::DataClassNotSupported);
        }
    }
    if let Some(jurisdiction) = invocation.jurisdiction_ref.as_ref() {
        if !tool.jurisdiction_scope.contains(&jurisdiction.id) {
            return Err(ToolGatewayError::JurisdictionNotSupported);
        }
    }
    if invocation.requested_scope.trim().is_empty() {
        return Err(ToolGatewayError::Authorization(
            AuthorizationValidationError::ConstraintViolation,
        ));
    }
    Ok(())
}

fn validate_execution_gate(
    invocation: &ToolGatewayInvocation,
    tool: &ToolRegistryEntry,
    authorization: &AuthorizationResult,
    action: Option<&Action>,
    approval: Option<&ApprovalRecord>,
) -> Result<(), ToolGatewayError> {
    if !tool.approval_required {
        return Ok(());
    }

    let action = action.ok_or(ToolGatewayError::ExecutionGate(
        ExecutionGateError::ApprovalRequired,
    ))?;

    if action.action_id != invocation.action.id {
        return Err(ToolGatewayError::ExecutionGate(
            ExecutionGateError::ApprovalMismatch,
        ));
    }

    ExecutionGate::permit(
        action,
        ExecutionGateInput {
            authorization: Some(authorization),
            approval,
        },
    )
    .map_err(ToolGatewayError::ExecutionGate)
}

fn validate_registry_implementation_binding(
    invocation: &ToolGatewayInvocation,
    tool: &ToolRegistryEntry,
) -> Result<(), ToolGatewayError> {
    tool.implementations
        .iter()
        .find(|implementation| {
            implementation.implementation_id == invocation.implementation_id
                && implementation.provider_id == invocation.provider_id
                && implementation.implementation_version == invocation.implementation_version
        })
        .map(|_| ())
        .ok_or(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ))
}

fn validate_provider_binding<P: ToolGatewayProvider>(
    invocation: &ToolGatewayInvocation,
    tool: &ToolRegistryEntry,
    provider: &P,
) -> Result<(), ToolGatewayError> {
    validate_registry_implementation_binding(invocation, tool)?;
    if provider.provider_id() != invocation.provider_id
        || provider.implementation_id() != invocation.implementation_id
        || provider.implementation_version() != invocation.implementation_version
    {
        return Err(ToolGatewayError::Registry(
            ToolRegistryError::UnsupportedVersion,
        ));
    }
    Ok(())
}

fn validate_authorized_constraints(
    constraints: &[crate::authorization::AuthorizationConstraint],
    invocation: &ToolGatewayInvocation,
) -> Result<(), ToolGatewayError> {
    let mut scope: Option<&str> = None;
    let mut access_mode: Option<&str> = None;

    for constraint in constraints {
        match (constraint.key.as_str(), constraint.value.as_str()) {
            ("scope", "exact_resource") | ("scope", "exact_evidence") => {
                if let Some(existing) = scope {
                    if existing != constraint.value {
                        return Err(ToolGatewayError::Authorization(
                            AuthorizationValidationError::ConstraintViolation,
                        ));
                    }
                }
                scope = Some(constraint.value.as_str());
            }
            ("access_mode", "read_only") => {
                if let Some(existing) = access_mode {
                    if existing != constraint.value {
                        return Err(ToolGatewayError::Authorization(
                            AuthorizationValidationError::ConstraintViolation,
                        ));
                    }
                }
                access_mode = Some(constraint.value.as_str());
            }
            _ => {
                return Err(ToolGatewayError::Authorization(
                    AuthorizationValidationError::ConstraintViolation,
                ));
            }
        }
    }

    if let Some(scope_constraint) = scope {
        if (scope_constraint == "exact_resource" || scope_constraint == "exact_evidence")
            && invocation.requested_scope != invocation.resource_ref.id
        {
            return Err(ToolGatewayError::Authorization(
                AuthorizationValidationError::ConstraintViolation,
            ));
        }
    }

    Ok(())
}

fn operation_key(invocation: &ToolGatewayInvocation) -> Result<Id, ToolGatewayError> {
    let canonical = serde_json::to_vec(invocation)
        .map_err(|_| ToolGatewayError::Idempotency(PersistenceError::SerializationFailure))?;
    Ok(format!("tool-gateway:{}", sha256_hex(&canonical)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{AuthorizationConstraint, AuthorizationDecision};
    use crate::persistence::{
        IdempotencyClaim, IdempotencyLifecycleStore, IdempotencyState, IdempotencyStore,
        PersistenceError,
    };
    use crate::tool_registry::{ToolImplementation, ToolLifecycle, ToolRiskClass, ToolVersion};
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct Idempotency {
        claimed: BTreeSet<Id>,
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
            Ok(self
                .claimed
                .contains(operation_id)
                .then_some(IdempotencyState::Completed))
        }

        fn mark_in_progress(&mut self, _: &Id) -> Result<(), PersistenceError> {
            Ok(())
        }
        fn mark_completed(&mut self, _: &Id) -> Result<(), PersistenceError> {
            Ok(())
        }
        fn mark_failed(&mut self, _: &Id) -> Result<(), PersistenceError> {
            Ok(())
        }
        fn mark_unknown(&mut self, _: &Id) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct Provider {
        calls: usize,
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
        ) -> Result<Self::Output, ToolGatewayError> {
            self.calls += 1;
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
            authorization_ref: r(crate::ResourceType::Other, "auth-1"),
            subject_ref: r(crate::ResourceType::Party, "party-1"),
            actor_ref: None,
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
        let mut i = invocation();
        i.capability_ref = r(crate::ResourceType::Other, "cap-forged");
        assert!(matches!(
            gateway.execute(
                &i,
                &request(),
                &authorization(),
                None,
                None,
                1050,
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
        let mut i = invocation();
        i.function_ref = Some(r(crate::ResourceType::Other, "fn-forged"));
        assert!(matches!(
            gateway.execute(
                &i,
                &request(),
                &authorization(),
                None,
                None,
                1050,
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
        let mut i = invocation();
        i.implementation_id = "impl-forged".into();
        assert!(matches!(
            gateway.execute(
                &i,
                &request(),
                &authorization(),
                None,
                None,
                1050,
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
        let mut i = invocation();
        i.provider_id = "provider-forged".into();
        assert!(matches!(
            gateway.execute(
                &i,
                &request(),
                &authorization(),
                None,
                None,
                1050,
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
        let mut i = invocation();
        i.implementation_version = "forged".into();
        assert!(matches!(
            gateway.execute(
                &i,
                &request(),
                &authorization(),
                None,
                None,
                1050,
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
            ) -> Result<Self::Output, ToolGatewayError> {
                panic!("wrong provider must never execute");
            }
        }

        let registry = registry();
        let mut idempotency = Idempotency::default();
        let mut gateway = ToolGateway::new(&registry, &mut idempotency);
        let mut provider = WrongProvider;
        assert!(matches!(
            gateway.execute(
                &invocation(),
                &request(),
                &authorization(),
                None,
                None,
                1050,
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
        let mut idempotency = Idempotency::default();
        let mut gateway = ToolGateway::new(&registry, &mut idempotency);
        let q = request();
        let a = authorization();
        let i = invocation();
        let mut p = Provider::default();
        assert_eq!(
            gateway.execute(&i, &q, &a, None, None, 1050, &mut p),
            Ok("ok")
        );
        assert_eq!(p.calls, 1);
        assert_eq!(
            gateway.execute(&i, &q, &a, None, None, 1050, &mut p),
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
        assert_eq!(
            gateway.execute(
                &invocation(),
                &request(),
                &authorization(),
                None,
                None,
                1050,
                &mut provider
            ),
            Err(ToolGatewayError::ExecutionGate(
                ExecutionGateError::ApprovalRequired
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
            "action-1".into(),
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
            action_ref: r(crate::ResourceType::Action, "action-1"),
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
        assert_eq!(
            gateway.execute(
                &invocation(),
                &request(),
                &authorization(),
                Some(&action),
                Some(&approval),
                1050,
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
            "action-1".into(),
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
            action_ref: r(crate::ResourceType::Action, "action-1"),
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
        assert_eq!(
            gateway.execute(
                &invocation(),
                &request(),
                &authorization(),
                Some(&action),
                Some(&approval),
                1050,
                &mut provider
            ),
            Err(ToolGatewayError::ExecutionGate(
                ExecutionGateError::ApprovalNotHuman
            ))
        );
        assert_eq!(provider.calls, 0);
        assert!(idempotency.claimed.is_empty());
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
}
