//! Bounded, provider-neutral Tool Gateway execution boundary.
//!
//! The gateway composes canonical authorization, typed constraint validation,
//! Tool Registry resolution, capability leases, idempotency and provider
//! execution. It does not define policy or grant authority.

use crate::authorization::{AuthorizationRequest, AuthorizationResult};
use crate::authorization_enforcement::{validate_authorization, AuthorizationValidationError};
use crate::capability_lease::{CapabilityLease, CapabilityLeaseError};
use crate::persistence::{IdempotencyClaim, IdempotencyStore, PersistenceError};
use crate::tool_registry::{
    InMemoryToolRegistry, ToolDataClass, ToolExecutionMode, ToolRegistryError, ToolVersion,
    ToolVersionRequirement,
};
use crate::{Id, ResourceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolGatewayInvocation {
    pub request_id: Id,
    pub authorization_ref: ResourceRef,
    pub subject_ref: ResourceRef,
    pub actor_ref: Option<ResourceRef>,
    pub action: ResourceRef,
    pub resource_ref: ResourceRef,
    pub purpose: String,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<ToolDataClass>,
    pub tool_id: Id,
    pub tool_version: ToolVersionRequirement,
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
    CapabilityMismatch,
    ProviderFailed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGatewayPhase {
    Validated,
    Claimed,
}

/// Provider boundary. Providers receive an already validated invocation and
/// registry entry; they never receive authority to reinterpret policy.
pub trait ToolGatewayProvider {
    type Output;

    fn execute(
        &mut self,
        invocation: &ToolGatewayInvocation,
        tool: &crate::tool_registry::ToolRegistryEntry,
    ) -> Result<Self::Output, ToolGatewayError>;
}

/// Bounded gateway kernel. The caller supplies the canonical AuthorizationResult
/// and the registry; the gateway performs consumer-side enforcement before any
/// provider call.
pub struct ToolGateway<'a, I: IdempotencyStore> {
    registry: &'a InMemoryToolRegistry,
    idempotency: &'a mut I,
}

impl<'a, I: IdempotencyStore> ToolGateway<'a, I> {
    pub fn new(registry: &'a InMemoryToolRegistry, idempotency: &'a mut I) -> Self {
        Self {
            registry,
            idempotency,
        }
    }

    /// Validate all pre-execution control-plane requirements without claiming
    /// idempotency. This phase must remain side-effect free.
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
        if invocation.request_id != request.request_id
            || invocation.authorization_ref != request.authorization_ref
            || invocation.subject_ref != request.subject_ref
            || invocation.action != request.action
            || invocation.resource_ref != request.resource_ref
            || invocation.purpose != request.purpose
            || invocation.jurisdiction_ref != request.jurisdiction_ref
            || invocation.data_class.as_ref().map(|v| format!("{v:?}"))
                != request.data_class.as_deref().map(str::to_owned)
        {
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
        enforce_constraints(&authorization.constraints, &invocation.requested_scope)?;

        if let Some(lease) = invocation.capability_lease.as_ref() {
            if lease.authorization_ref != invocation.authorization_ref {
                return Err(ToolGatewayError::Lease(
                    CapabilityLeaseError::InvalidLease,
                ));
            }
            lease.validate_use(
                now_epoch_seconds,
                &tool.capability_ref,
                Some(&invocation.resource_ref),
                &invocation.subject_ref,
                invocation.actor_ref.as_ref(),
                &invocation.purpose,
                None,
                &invocation.requested_scope,
                invocation.incident_ref.as_ref(),
                invocation.session_ref.as_ref(),
            )
            .map_err(ToolGatewayError::Lease)?;
        }

        Ok(ToolGatewayPhase::Validated)
    }

    /// Claim the canonical operation identity after all authorization and scope
    /// checks have passed. Durable/concurrent safety is delegated to the
    /// deployment's IdempotencyStore implementation.
    pub fn claim(
        &mut self,
        invocation: &ToolGatewayInvocation,
    ) -> Result<ToolGatewayPhase, ToolGatewayError> {
        match self
            .idempotency
            .claim(invocation.idempotency_ref.clone())
            .map_err(ToolGatewayError::Idempotency)?
        {
            IdempotencyClaim::Claimed => Ok(ToolGatewayPhase::Claimed),
            IdempotencyClaim::AlreadyClaimed => Err(ToolGatewayError::IdempotencyAlreadyClaimed),
        }
    }

    /// Validate, then claim idempotency, then invoke the provider. Provider
    /// execution is the first point at which an external side effect may occur.
    pub fn execute<P: ToolGatewayProvider>(
        &mut self,
        invocation: &ToolGatewayInvocation,
        request: &AuthorizationRequest,
        authorization: &AuthorizationResult,
        now_epoch_seconds: u64,
        provider: &mut P,
    ) -> Result<P::Output, ToolGatewayError> {
        self.validate(invocation, request, authorization, now_epoch_seconds)?;
        self.claim(invocation)?;
        let tool = self
            .registry
            .resolve(&invocation.tool_id, &invocation.tool_version)
            .map_err(ToolGatewayError::Registry)?;
        provider.execute(invocation, tool)
    }
}

fn enforce_constraints(
    constraints: &[crate::authorization::AuthorizationConstraint],
    requested_scope: &str,
) -> Result<(), ToolGatewayError> {
    let mut scope = None;
    let mut read_only = false;

    for constraint in constraints {
        match (constraint.key.as_str(), constraint.value.as_str()) {
            ("scope", "exact_resource") => {
                if let Some(existing) = scope {
                    if existing != "exact_resource" {
                        return Err(ToolGatewayError::Authorization(
                            AuthorizationValidationError::ConstraintViolation,
                        ));
                    }
                }
                scope = Some("exact_resource");
                if requested_scope.trim().is_empty() {
                    return Err(ToolGatewayError::Authorization(
                        AuthorizationValidationError::ConstraintViolation,
                    ));
                }
            }
            ("scope", "exact_evidence") => {
                if let Some(existing) = scope {
                    if existing != "exact_evidence" {
                        return Err(ToolGatewayError::Authorization(
                            AuthorizationValidationError::ConstraintViolation,
                        ));
                    }
                }
                scope = Some("exact_evidence");
            }
            ("access_mode", "read_only") => read_only = true,
            _ => {
                return Err(ToolGatewayError::Authorization(
                    AuthorizationValidationError::ConstraintViolation,
                ))
            }
        }
    }

    let _ = read_only;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{AccessAction, AuthorizationConstraint, AuthorizationDecision};
    use crate::persistence::{IdempotencyClaim, PersistenceError};
    use crate::tool_registry::{
        ToolImplementation, ToolLifecycle, ToolRiskClass, ToolVersionRequirement,
    };
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
            if self.claimed.insert(operation_id) {
                Ok(IdempotencyClaim::Claimed)
            } else {
                Ok(IdempotencyClaim::AlreadyClaimed)
            }
        }
    }

    #[derive(Default)]
    struct Provider {
        calls: usize,
    }

    impl ToolGatewayProvider for Provider {
        type Output = &'static str;

        fn execute(
            &mut self,
            _invocation: &ToolGatewayInvocation,
            _tool: &crate::tool_registry::ToolRegistryEntry,
        ) -> Result<Self::Output, ToolGatewayError> {
            self.calls += 1;
            Ok("ok")
        }
    }

    fn reference(resource_type: crate::ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn registry() -> InMemoryToolRegistry {
        let mut registry = InMemoryToolRegistry::new();
        registry
            .register(
                crate::tool_registry::ToolRegistryEntry {
                    tool_id: "tool-1".into(),
                    version: ToolVersion::new(1, 0, 0),
                    name: "test".into(),
                    purpose: "protected read".into(),
                    lifecycle: ToolLifecycle::Active,
                    risk_class: ToolRiskClass::ReadOnly,
                    data_classes: [ToolDataClass::Restricted].into_iter().collect(),
                    capability_ref: reference(crate::ResourceType::Other, "cap-1"),
                    function_ref: None,
                    jurisdiction_scope: vec!["IN".into()],
                    permission_requirements: vec![],
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
            authorization_ref: reference(crate::ResourceType::Other, "auth-1"),
            subject_ref: reference(crate::ResourceType::Party, "party-1"),
            action: reference(crate::ResourceType::Action, "read"),
            resource_ref: reference(crate::ResourceType::Case, "case-1"),
            purpose: "protected read".into(),
            policy_refs: vec![reference(crate::ResourceType::Other, "policy-1")],
            jurisdiction_ref: Some(reference(crate::ResourceType::Jurisdiction, "IN")),
            data_class: Some("restricted".into()),
            requested_at_epoch_seconds: 1_000,
            freshness_seconds: Some(100),
        }
    }

    fn authorization() -> AuthorizationResult {
        let request = request();
        AuthorizationResult {
            request_id: request.request_id.clone(),
            authorization_ref: request.authorization_ref.clone(),
            subject_ref: request.subject_ref.clone(),
            action: request.action.clone(),
            resource_ref: request.resource_ref.clone(),
            purpose: request.purpose.clone(),
            jurisdiction_ref: request.jurisdiction_ref.clone(),
            data_class: request.data_class.clone(),
            decision: AuthorizationDecision::Allow,
            constraints: vec![AuthorizationConstraint {
                key: "scope".into(),
                value: "exact_resource".into(),
            }],
            policy_refs: request.policy_refs.clone(),
            evaluated_at_epoch_seconds: 1_000,
            expires_at_epoch_seconds: Some(1_100),
        }
    }

    fn invocation() -> ToolGatewayInvocation {
        ToolGatewayInvocation {
            request_id: "req-1".into(),
            authorization_ref: reference(crate::ResourceType::Other, "auth-1"),
            subject_ref: reference(crate::ResourceType::Party, "party-1"),
            actor_ref: None,
            action: reference(crate::ResourceType::Action, "read"),
            resource_ref: reference(crate::ResourceType::Case, "case-1"),
            purpose: "protected read".into(),
            jurisdiction_ref: Some(reference(crate::ResourceType::Jurisdiction, "IN")),
            data_class: Some(ToolDataClass::Restricted),
            tool_id: "tool-1".into(),
            tool_version: ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
            idempotency_ref: "op-1".into(),
            requested_scope: "case-1".into(),
            execution_mode: ToolExecutionMode::Sync,
            capability_lease: None,
            incident_ref: None,
            session_ref: None,
        }
    }

    #[test]
    fn exact_context_and_registry_are_required() {
        let registry = registry();
        let mut idempotency = Idempotency::default();
        let gateway = ToolGateway::new(&registry, &mut idempotency);
        assert_eq!(
            gateway.validate(&invocation(), &request(), &authorization(), 1_050),
            Ok(ToolGatewayPhase::Validated)
        );
    }

    #[test]
    fn subject_mismatch_fails_before_provider() {
        let registry = registry();
        let mut idempotency = Idempotency::default();
        let gateway = ToolGateway::new(&registry, &mut idempotency);
        let mut invocation = invocation();
        invocation.subject_ref = reference(crate::ResourceType::Party, "party-2");
        assert!(matches!(
            gateway.validate(&invocation, &request(), &authorization(), 1_050),
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
            gateway.validate(&invocation(), &request(), &authorization(), 1_100),
            Err(ToolGatewayError::Authorization(
                AuthorizationValidationError::Expired
            ))
        ));
    }

    #[test]
    fn duplicate_idempotency_is_blocked_before_provider() {
        let registry = registry();
        let mut idempotency = Idempotency::default();
        let mut gateway = ToolGateway::new(&registry, &mut idempotency);
        let request = request();
        let authorization = authorization();
        let invocation = invocation();
        let mut provider = Provider::default();
        assert_eq!(
            gateway.execute(&invocation, &request, &authorization, 1_050, &mut provider),
            Ok("ok")
        );
        assert_eq!(provider.calls, 1);
        assert_eq!(
            gateway.execute(&invocation, &request, &authorization, 1_050, &mut provider),
            Err(ToolGatewayError::IdempotencyAlreadyClaimed)
        );
        assert_eq!(provider.calls, 1);
    }

    #[test]
    fn deny_never_claims_idempotency() {
        let registry = registry();
        let mut idempotency = Idempotency::default();
        let mut gateway = ToolGateway::new(&registry, &mut idempotency);
        let mut denied = authorization();
        denied.decision = AuthorizationDecision::Deny;
        let invocation = invocation();
        let request = request();
        let mut provider = Provider::default();
        assert_eq!(
            gateway.execute(&invocation, &request, &denied, 1_050, &mut provider),
            Err(ToolGatewayError::Authorization(
                AuthorizationValidationError::Denied
            ))
        );
        assert_eq!(provider.calls, 0);
        assert!(!idempotency.lookup(&"op-1".into()).unwrap());
    }

    #[test]
    fn unsupported_constraint_fails_closed() {
        let registry = registry();
        let mut idempotency = Idempotency::default();
        let gateway = ToolGateway::new(&registry, &mut idempotency);
        let mut authorization = authorization();
        authorization.constraints.push(AuthorizationConstraint {
            key: "future".into(),
            value: "x".into(),
        });
        assert!(matches!(
            gateway.validate(&invocation(), &request(), &authorization, 1_050),
            Err(ToolGatewayError::Authorization(
                AuthorizationValidationError::ConstraintViolation
            ))
        ));
    }
}
