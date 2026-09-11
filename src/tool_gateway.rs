//! Provider-neutral Tool Gateway enforcement boundary.
//!
//! The gateway composes Tool Registry, canonical AuthorizationResult and the
//! existing Action/ExecutionGate contracts. It is not an authorization engine,
//! approval authority, trust authority, transport, provider SDK or MCP runtime.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::action::{
    Action, ApprovalRecord, ExecutionGate, ExecutionGateError, ExecutionGateInput,
};
use crate::authorization::{AuthorizationDecision, AuthorizationResult};
use crate::tool_registry::{
    InMemoryToolRegistry, ToolDataClass, ToolExecutionMode, ToolLifecycle, ToolRegistryError,
    ToolRiskClass, ToolVersion, ToolVersionRequirement,
};
use crate::{Id, ResourceRef};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolInvocation {
    pub invocation_id: Id,
    pub subject_ref: ResourceRef,
    pub tool_id: Id,
    pub version_requirement: ToolVersionRequirement,
    pub capability_ref: ResourceRef,
    pub function_ref: Option<ResourceRef>,
    pub action_ref: Option<ResourceRef>,
    pub purpose: String,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: Option<ToolDataClass>,
    pub execution_mode: ToolExecutionMode,
    pub input: Value,
    pub authorization: AuthorizationResult,
    pub approval: Option<ApprovalRecord>,
    pub action: Option<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolExecutionResult {
    pub invocation_id: Id,
    pub tool_id: Id,
    pub version: ToolVersion,
    pub output: Value,
}

pub trait ToolAdapter {
    fn invoke(
        &self,
        invocation: &ToolInvocation,
        tool_version: ToolVersion,
    ) -> Result<Value, ToolGatewayError>;
}

pub trait ToolGatewayAuditSink {
    fn record(&mut self, record: ToolGatewayAuditRecord);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolGatewayAuditRecord {
    pub invocation_id: Id,
    pub tool_id: Id,
    pub version: ToolVersion,
    pub authorization_ref: ResourceRef,
    pub subject_ref: ResourceRef,
    pub outcome: ToolGatewayAuditOutcome,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolGatewayAuditOutcome {
    Accepted,
    Rejected,
}

#[derive(Debug, Default)]
pub struct InMemoryToolGatewayAudit {
    records: Vec<ToolGatewayAuditRecord>,
}

impl InMemoryToolGatewayAudit {
    pub fn records(&self) -> &[ToolGatewayAuditRecord] {
        &self.records
    }
}

impl ToolGatewayAuditSink for InMemoryToolGatewayAudit {
    fn record(&mut self, record: ToolGatewayAuditRecord) {
        self.records.push(record);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolGatewayError {
    EmptyInvocationId,
    EmptyPurpose,
    SubjectMismatch,
    AuthorizationRequired,
    AuthorizationDenied,
    AuthorizationExpired,
    AuthorizationContextMismatch,
    ToolRegistry(ToolRegistryError),
    ToolNotActive,
    CapabilityMismatch,
    FunctionMismatch,
    ActionMismatch,
    JurisdictionMismatch,
    DataClassMismatch,
    ExecutionModeUnsupported,
    RiskDataClassMismatch,
    ApprovalRequired,
    ExecutionGate(ExecutionGateError),
    DuplicateInvocation,
    AdapterFailure(String),
}

impl std::fmt::Display for ToolGatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ToolGatewayError {}

impl From<ToolRegistryError> for ToolGatewayError {
    fn from(value: ToolRegistryError) -> Self {
        Self::ToolRegistry(value)
    }
}

impl From<ExecutionGateError> for ToolGatewayError {
    fn from(value: ExecutionGateError) -> Self {
        Self::ExecutionGate(value)
    }
}

#[derive(Debug, Default)]
pub struct InMemoryToolGateway {
    completed_invocations: BTreeSet<Id>,
}

impl InMemoryToolGateway {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invoke<A: ToolAdapter, S: ToolGatewayAuditSink>(
        &mut self,
        registry: &InMemoryToolRegistry,
        invocation: &ToolInvocation,
        adapter: &A,
        audit: &mut S,
    ) -> Result<ToolExecutionResult, ToolGatewayError> {
        self.validate_invocation(invocation)?;
        if self
            .completed_invocations
            .contains(&invocation.invocation_id)
        {
            return Err(ToolGatewayError::DuplicateInvocation);
        }

        let entry = registry.resolve(&invocation.tool_id, &invocation.version_requirement)?;
        if entry.lifecycle != ToolLifecycle::Active {
            return Err(ToolGatewayError::ToolNotActive);
        }
        if entry.capability_ref != invocation.capability_ref {
            return Err(ToolGatewayError::CapabilityMismatch);
        }
        if entry.function_ref != invocation.function_ref {
            return Err(ToolGatewayError::FunctionMismatch);
        }
        if let Some(action_ref) = &invocation.action_ref {
            let action = invocation
                .action
                .as_ref()
                .ok_or(ToolGatewayError::ActionMismatch)?;
            if action.action_id != action_ref.id {
                return Err(ToolGatewayError::ActionMismatch);
            }
        } else if invocation.action.is_some() {
            return Err(ToolGatewayError::ActionMismatch);
        }
        if let Some(jurisdiction) = &invocation.jurisdiction_ref {
            if !entry.jurisdiction_scope.is_empty()
                && !entry.jurisdiction_scope.contains(&jurisdiction.id)
            {
                return Err(ToolGatewayError::JurisdictionMismatch);
            }
        }
        if let Some(data_class) = invocation.data_class {
            if !entry.data_classes.contains(&data_class) {
                return Err(ToolGatewayError::DataClassMismatch);
            }
        }
        if !entry.execution_modes.contains(&invocation.execution_mode) {
            return Err(ToolGatewayError::ExecutionModeUnsupported);
        }
        if entry.risk_class == ToolRiskClass::HighImpact
            && invocation.data_class == Some(ToolDataClass::Public)
        {
            return Err(ToolGatewayError::RiskDataClassMismatch);
        }

        let authorization = &invocation.authorization;
        if authorization.decision != AuthorizationDecision::Allow {
            audit.record(Self::audit_record(
                invocation,
                entry.version,
                ToolGatewayAuditOutcome::Rejected,
            ));
            return Err(ToolGatewayError::AuthorizationDenied);
        }
        if authorization.authorization_ref.id.trim().is_empty() {
            audit.record(Self::audit_record(
                invocation,
                entry.version,
                ToolGatewayAuditOutcome::Rejected,
            ));
            return Err(ToolGatewayError::AuthorizationRequired);
        }
        if authorization.subject_ref != invocation.subject_ref
            || authorization.action != invocation.capability_ref
            || authorization.resource_ref != invocation.capability_ref
            || authorization.purpose != invocation.purpose
            || authorization.jurisdiction_ref != invocation.jurisdiction_ref
            || authorization.data_class
                != invocation.data_class.map(|v| format!("{v:?}"))
        {
            audit.record(Self::audit_record(
                invocation,
                entry.version,
                ToolGatewayAuditOutcome::Rejected,
            ));
            return Err(ToolGatewayError::AuthorizationContextMismatch);
        }
        if authorization.expires_at_epoch_seconds == Some(0) {
            audit.record(Self::audit_record(
                invocation,
                entry.version,
                ToolGatewayAuditOutcome::Rejected,
            ));
            return Err(ToolGatewayError::AuthorizationExpired);
        }

        if entry.approval_required {
            let action = invocation
                .action
                .as_ref()
                .ok_or(ToolGatewayError::ApprovalRequired)?;
            ExecutionGate::permit(
                action,
                ExecutionGateInput {
                    authorization: Some(authorization),
                    approval: invocation.approval.as_ref(),
                },
            )?;
        }

        let output = adapter.invoke(invocation, entry.version)?;
        self.completed_invocations
            .insert(invocation.invocation_id.clone());
        audit.record(Self::audit_record(
            invocation,
            entry.version,
            ToolGatewayAuditOutcome::Accepted,
        ));
        Ok(ToolExecutionResult {
            invocation_id: invocation.invocation_id.clone(),
            tool_id: entry.tool_id.clone(),
            version: entry.version,
            output,
        })
    }

    fn validate_invocation(&self, invocation: &ToolInvocation) -> Result<(), ToolGatewayError> {
        if invocation.invocation_id.trim().is_empty() {
            return Err(ToolGatewayError::EmptyInvocationId);
        }
        if invocation.subject_ref.id.trim().is_empty() {
            return Err(ToolGatewayError::SubjectMismatch);
        }
        if invocation.purpose.trim().is_empty() {
            return Err(ToolGatewayError::EmptyPurpose);
        }
        if invocation.authorization.subject_ref != invocation.subject_ref {
            return Err(ToolGatewayError::SubjectMismatch);
        }
        Ok(())
    }

    fn audit_record(
        invocation: &ToolInvocation,
        version: ToolVersion,
        outcome: ToolGatewayAuditOutcome,
    ) -> ToolGatewayAuditRecord {
        ToolGatewayAuditRecord {
            invocation_id: invocation.invocation_id.clone(),
            tool_id: invocation.tool_id.clone(),
            version,
            authorization_ref: invocation.authorization.authorization_ref.clone(),
            subject_ref: invocation.subject_ref.clone(),
            outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_registry::{ToolImplementation, ToolRegistryAuditRecord, ToolRiskClass};
    use crate::ResourceType;

    struct EchoAdapter;

    impl ToolAdapter for EchoAdapter {
        fn invoke(
            &self,
            invocation: &ToolInvocation,
            _version: ToolVersion,
        ) -> Result<Value, ToolGatewayError> {
            Ok(invocation.input.clone())
        }
    }

    fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn registry() -> InMemoryToolRegistry {
        let mut registry = InMemoryToolRegistry::new();
        let version = ToolVersion::new(1, 0, 0);
        registry
            .register(
                crate::ToolRegistryEntry {
                    tool_id: "echo".into(),
                    version,
                    name: "Echo".into(),
                    purpose: "return input".into(),
                    lifecycle: ToolLifecycle::Active,
                    risk_class: ToolRiskClass::ReadOnly,
                    data_classes: [ToolDataClass::Public].into_iter().collect(),
                    capability_ref: reference(ResourceType::Other, "cap.echo"),
                    function_ref: Some(reference(ResourceType::Other, "fn.echo")),
                    jurisdiction_scope: vec![],
                    permission_requirements: vec![],
                    approval_required: false,
                    input_schema_ref: None,
                    output_schema_ref: None,
                    execution_modes: [ToolExecutionMode::Sync].into_iter().collect(),
                    dependencies: vec![],
                    implementations: vec![ToolImplementation {
                        implementation_id: "impl.echo".into(),
                        provider_id: "provider.test".into(),
                        implementation_version: "1".into(),
                        adapter_refs: vec![],
                    }],
                    provenance_requirements: vec![],
                    observability_refs: vec![],
                    documentation_refs: vec![],
                },
                ToolRegistryAuditRecord {
                    change_id: "change-1".into(),
                    tool_id: "echo".into(),
                    version,
                    change_type: "register".into(),
                    actor_ref: "actor-1".into(),
                    authorization_ref: "auth-1".into(),
                    timestamp: "2026-01-01T00:00:00Z".into(),
                    previous_lifecycle: None,
                    new_lifecycle: None,
                },
            )
            .unwrap();
        registry
    }

    fn invocation() -> ToolInvocation {
        let subject = reference(ResourceType::Party, "party-1");
        let capability = reference(ResourceType::Other, "cap.echo");
        let auth = AuthorizationResult {
            request_id: "request-1".into(),
            authorization_ref: reference(ResourceType::Other, "auth-1"),
            subject_ref: subject.clone(),
            action: capability.clone(),
            resource_ref: capability.clone(),
            purpose: "return input".into(),
            jurisdiction_ref: None,
            data_class: Some("Public".into()),
            decision: AuthorizationDecision::Allow,
            constraints: vec![],
            policy_refs: vec![],
            evaluated_at_epoch_seconds: 1,
            expires_at_epoch_seconds: None,
        };
        ToolInvocation {
            invocation_id: "inv-1".into(),
            subject_ref: subject,
            tool_id: "echo".into(),
            version_requirement: ToolVersionRequirement::Exact(ToolVersion::new(1, 0, 0)),
            capability_ref: capability,
            function_ref: Some(reference(ResourceType::Other, "fn.echo")),
            action_ref: None,
            purpose: "return input".into(),
            jurisdiction_ref: None,
            data_class: Some(ToolDataClass::Public),
            execution_mode: ToolExecutionMode::Sync,
            input: serde_json::json!({"value": 1}),
            authorization: auth,
            approval: None,
            action: None,
        }
    }

    #[test]
    fn gateway_executes_active_tool_with_matching_canonical_authorization() {
        let mut gateway = InMemoryToolGateway::new();
        let mut audit = InMemoryToolGatewayAudit::default();
        let result = gateway
            .invoke(&registry(), &invocation(), &EchoAdapter, &mut audit)
            .unwrap();
        assert_eq!(result.output, serde_json::json!({"value": 1}));
        assert_eq!(audit.records().len(), 1);
        assert_eq!(audit.records()[0].outcome, ToolGatewayAuditOutcome::Accepted);
    }

    #[test]
    fn gateway_rejects_authorization_context_mismatch() {
        let mut gateway = InMemoryToolGateway::new();
        let mut audit = InMemoryToolGatewayAudit::default();
        let mut request = invocation();
        request.authorization.purpose = "different purpose".into();
        let error = gateway
            .invoke(&registry(), &request, &EchoAdapter, &mut audit)
            .unwrap_err();
        assert_eq!(error, ToolGatewayError::AuthorizationContextMismatch);
        assert_eq!(audit.records()[0].outcome, ToolGatewayAuditOutcome::Rejected);
    }

    #[test]
    fn gateway_rejects_duplicate_invocation_after_success() {
        let mut gateway = InMemoryToolGateway::new();
        let mut audit = InMemoryToolGatewayAudit::default();
        let request = invocation();
        gateway
            .invoke(&registry(), &request, &EchoAdapter, &mut audit)
            .unwrap();
        let error = gateway
            .invoke(&registry(), &request, &EchoAdapter, &mut audit)
            .unwrap_err();
        assert_eq!(error, ToolGatewayError::DuplicateInvocation);
    }
}
