//! Bounded, provider-neutral Tool Gateway execution boundary.
//!
//! The gateway composes canonical authorization, Tool Registry resolution,
//! capability leases and canonical idempotency. It does not grant authority.

use crate::action::{
    Action, ApprovalRecord, ExecutionGate, ExecutionGateError, ExecutionGateInput,
};
use crate::audit::{AuditProvenanceSink, AuditRecord};
use crate::authorization::{AuthorizationRequest, AuthorizationResult};
use crate::authorization_enforcement::{validate_authorization, AuthorizationValidationError};
use crate::capability_lease::{CapabilityLease, CapabilityLeaseError};
use crate::persistence::{IdempotencyClaim, IdempotencyLifecycleStore, PersistenceError};
use crate::tool_data_access::ToolDataAccessGrant;
use crate::tool_registry::{
    InMemoryToolRegistry, ToolDataClass, ToolExecutionMode, ToolRegistryEntry, ToolRegistryError,
    ToolVersionRequirement,
};
use crate::{sha256_hex, Id, ResourceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolGatewayInvocation {
    pub request_id: Id,
    /// Timestamp captured when the invocation was created.
    pub occurred_at: String,
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
    Audit(&'static str),
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
        data_access: &ToolDataAccessGrant,
    ) -> Result<Self::Output, ToolGatewayError>;
}

pub struct ToolGatewayExecutionContext<'a> {
    pub request: &'a AuthorizationRequest,
    pub authorization: &'a AuthorizationResult,
    pub action: Option<&'a Action>,
    pub approval: Option<&'a ApprovalRecord>,
    pub now_epoch_seconds: u64,
    /// Canonical audit/provenance sink. Tool execution cannot bypass invocation audit.
    pub audit: &'a mut dyn AuditProvenanceSink,
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
    fn claim_validated(
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
        context: ToolGatewayExecutionContext<'_>,
        provider: &mut P,
    ) -> Result<P::Output, ToolGatewayError> {
        if invocation.occurred_at.trim().is_empty() {
            return Err(ToolGatewayError::Audit("invocation timestamp is required"));
        }
        self.validate(
            invocation,
            context.request,
            context.authorization,
            context.now_epoch_seconds,
        )?;
        let tool = self
            .registry
            .resolve(&invocation.tool_id, &invocation.tool_version)
            .map_err(ToolGatewayError::Registry)?;
        validate_provider_binding(invocation, tool, provider)?;
        validate_execution_gate(
            invocation,
            tool,
            context.authorization,
            context.action,
            context.approval,
        )?;
        let operation_id = operation_key(invocation)?;
        let data_access = ToolDataAccessGrant::from_invocation(
            operation_id.clone(),
            invocation.authorization_ref.clone(),
            invocation.resource_ref.clone(),
            invocation.purpose.clone(),
            invocation
                .data_class
                .ok_or(ToolGatewayError::DataClassNotSupported)?,
            invocation.requested_scope.clone(),
        )
        .map_err(|_| {
            ToolGatewayError::Authorization(AuthorizationValidationError::ConstraintViolation)
        })?;
        self.claim_validated(invocation)?;
        self.idempotency
            .mark_in_progress(&operation_id)
            .map_err(ToolGatewayError::Idempotency)?;
        match provider.execute(invocation, tool, &data_access) {
            Ok(output) => {
                if record_invocation_audit(
                    context.audit,
                    invocation,
                    ToolGatewayAuditContext {
                        request: context.request,
                        approval: context.approval,
                        occurred_at: invocation.occurred_at.as_str(),
                        outcome: "completed",
                        failure: None,
                    },
                )
                .is_err()
                {
                    let _ = self.idempotency.mark_unknown(&operation_id);
                    return Err(ToolGatewayError::Unknown);
                }
                if self.idempotency.mark_completed(&operation_id).is_err() {
                    let _ = self.idempotency.mark_unknown(&operation_id);
                    return Err(ToolGatewayError::Unknown);
                }
                Ok(output)
            }
            Err(error) => {
                if record_invocation_audit(
                    context.audit,
                    invocation,
                    ToolGatewayAuditContext {
                        request: context.request,
                        approval: context.approval,
                        occurred_at: invocation.occurred_at.as_str(),
                        outcome: "failed",
                        failure: Some("provider_failed"),
                    },
                )
                .is_err()
                {
                    let _ = self.idempotency.mark_unknown(&operation_id);
                    return Err(ToolGatewayError::Unknown);
                }
                if self.idempotency.mark_failed(&operation_id).is_err() {
                    let _ = self.idempotency.mark_unknown(&operation_id);
                    return Err(ToolGatewayError::Unknown);
                }
                Err(error)
            }
        }
    }
}

struct ToolGatewayAuditContext<'a> {
    request: &'a AuthorizationRequest,
    approval: Option<&'a ApprovalRecord>,
    occurred_at: &'a str,
    outcome: &'a str,
    failure: Option<&'a str>,
}

fn record_invocation_audit(
    audit: &mut dyn AuditProvenanceSink,
    invocation: &ToolGatewayInvocation,
    context: ToolGatewayAuditContext<'_>,
) -> Result<(), ToolGatewayError> {
    let actor = invocation
        .actor_ref
        .as_ref()
        .ok_or(ToolGatewayError::Audit(
            "invocation actor is required for audit",
        ))?;
    let operation_id = operation_key(invocation)?;
    let provenance_id = format!("provenance-tool-gateway-{}", operation_id);
    let audit_id = format!("audit-tool-gateway-{}", operation_id);
    let provenance_ref =
        ResourceRef::new(crate::ResourceType::Provenance, provenance_id.clone())
            .map_err(|_| ToolGatewayError::Audit("invalid invocation provenance reference"))?;

    let record = AuditRecord {
        audit_id,
        actor_id: actor.id.clone(),
        action: invocation.action.id.clone(),
        aggregate_type: format!("{:?}", invocation.resource_ref.resource_type),
        aggregate_id: invocation.resource_ref.id.clone(),
        occurred_at: context.occurred_at.to_owned(),
        correlation_id: Some(invocation.request_id.clone()),
        causation_id: None,
        provenance_ref: Some(provenance_ref),
        invocation_id: Some(operation_id.clone()),
        request_id: Some(context.request.request_id.clone()),
        authorization_ref: Some(context.request.authorization_ref.clone()),
        action_ref: Some(invocation.action.clone()),
        approval_ref: context.approval.map(|value| value.action_ref.clone()),
        tool_id: Some(invocation.tool_id.clone()),
        tool_version: Some(format_tool_version(&invocation.tool_version)),
        capability_ref: Some(invocation.capability_ref.clone()),
        function_ref: invocation.function_ref.clone(),
        provider_id: Some(invocation.provider_id.clone()),
        implementation_id: Some(invocation.implementation_id.clone()),
        implementation_version: Some(invocation.implementation_version.clone()),
        resource_ref: Some(invocation.resource_ref.clone()),
        purpose: Some(invocation.purpose.clone()),
        jurisdiction_ref: invocation.jurisdiction_ref.clone(),
        data_class: invocation
            .data_class
            .map(|value| format!("{value:?}").to_lowercase()),
        requested_scope: Some(invocation.requested_scope.clone()),
        execution_mode: Some(format!("{:?}", invocation.execution_mode).to_lowercase()),
        idempotency_ref: Some(invocation.idempotency_ref.clone()),
        outcome: Some(context.outcome.to_owned()),
        failure: context.failure.map(str::to_owned),
        input_hash: Some(sha256_hex(
            &serde_json::to_vec(invocation)
                .map_err(|_| ToolGatewayError::Audit("cannot hash invocation"))?,
        )),
        output_hash: None,
    };
    let provenance = crate::Provenance {
        provenance_id,
        actor_ref: Some(actor.clone()),
        source_refs: vec![
            context.request.authorization_ref.clone(),
            invocation.capability_ref.clone(),
        ],
        input_refs: vec![invocation.resource_ref.clone()],
        operation: format!("tool-gateway.{}", context.outcome),
        occurred_at: context.occurred_at.to_owned(),
    };
    audit
        .record_invocation(record, provenance)
        .map_err(ToolGatewayError::Audit)
}

fn format_tool_version(value: &ToolVersionRequirement) -> String {
    match value {
        ToolVersionRequirement::Exact(version) => {
            format!("{}.{}.{}", version.major, version.minor, version.patch)
        }
        ToolVersionRequirement::CompatibleMajor(major) => format!("^{}.x", major),
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
    _tool: &ToolRegistryEntry,
    authorization: &AuthorizationResult,
    action: Option<&Action>,
    approval: Option<&ApprovalRecord>,
) -> Result<(), ToolGatewayError> {
    // Every consequential provider execution must pass the canonical Execution
    // Gate. The registry's approval_required flag is metadata about whether
    // explicit human approval is expected; it is not permission to bypass the
    // canonical gate.
    let action = action.ok_or(ToolGatewayError::ExecutionGate(
        ExecutionGateError::AuthorizationRequired,
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
#[path = "tool_gateway_tests.rs"]
mod tests;
