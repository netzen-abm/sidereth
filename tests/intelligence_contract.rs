use sidereth_core::{
    EpistemicStatus, IntelligenceClaim, IntelligenceDataClass, IntelligenceError,
    IntelligenceProvider, IntelligenceRequest, IntelligenceResponse, IntelligenceRiskClass,
    IntelligenceToolProposal, ResourceRef, ResourceType,
};

fn reference(resource_type: ResourceType, id: &str) -> ResourceRef {
    ResourceRef::new(resource_type, id).unwrap()
}

fn request() -> IntelligenceRequest {
    IntelligenceRequest {
        request_id: "req-1".into(),
        capability_id: "legal.analysis".into(),
        contract_version: "1.0".into(),
        provider_id: "fake-success".into(),
        model_id: Some("fake-model".into()),
        model_version: Some("1".into()),
        task_type: "analysis".into(),
        context_refs: vec![reference(ResourceType::Document, "doc-1")],
        jurisdiction_ref: Some(reference(ResourceType::Jurisdiction, "india")),
        data_class: IntelligenceDataClass::Confidential,
        required_output_schema: "intelligence.response.v1".into(),
        tool_permissions: Vec::new(),
        retrieval_required: true,
        provenance_required: true,
        risk_class: IntelligenceRiskClass::Medium,
        human_approval_required: false,
    }
}

fn success_response(request: &IntelligenceRequest) -> IntelligenceResponse {
    IntelligenceResponse {
        request_id: request.request_id.clone(),
        provider_id: request.provider_id.clone(),
        model_id: request.model_id.clone(),
        model_version: request.model_version.clone(),
        output: "candidate analysis".into(),
        claims: vec![IntelligenceClaim {
            claim_id: "claim-1".into(),
            content: "A candidate proposition".into(),
            status: EpistemicStatus::Inferred,
            source_refs: vec![reference(ResourceType::Document, "source-1")],
            evidence_refs: vec![reference(ResourceType::Evidence, "evidence-1")],
            provenance_ref: Some(reference(ResourceType::Provenance, "prov-1")),
        }],
        source_refs: vec![reference(ResourceType::Document, "source-1")],
        evidence_refs: vec![reference(ResourceType::Evidence, "evidence-1")],
        tool_proposals: Vec::new(),
        warnings: Vec::new(),
        provenance_refs: vec![reference(ResourceType::Provenance, "prov-1")],
    }
}

struct SuccessProvider;

impl IntelligenceProvider for SuccessProvider {
    fn execute(
        &self,
        request: &IntelligenceRequest,
    ) -> Result<IntelligenceResponse, IntelligenceError> {
        request
            .validate()
            .map_err(IntelligenceError::InvalidRequest)?;
        let response = success_response(request);
        response
            .validate_against(request)
            .map_err(IntelligenceError::MalformedOutput)?;
        Ok(response)
    }
}

#[derive(Clone, Copy)]
enum FailureMode {
    ProviderRejected,
    ProviderUnavailable,
    Timeout,
    MalformedOutput,
    PolicyDenied,
}

struct AdversarialProvider {
    mode: FailureMode,
}

impl IntelligenceProvider for AdversarialProvider {
    fn execute(
        &self,
        request: &IntelligenceRequest,
    ) -> Result<IntelligenceResponse, IntelligenceError> {
        request
            .validate()
            .map_err(IntelligenceError::InvalidRequest)?;
        match self.mode {
            FailureMode::ProviderRejected => Err(IntelligenceError::ProviderRejected),
            FailureMode::ProviderUnavailable => Err(IntelligenceError::ProviderUnavailable),
            FailureMode::Timeout => Err(IntelligenceError::Timeout),
            FailureMode::MalformedOutput => {
                let mut response = success_response(request);
                response.request_id = "wrong-request".into();
                Err(IntelligenceError::MalformedOutput(
                    response.validate_against(request).unwrap_err(),
                ))
            }
            FailureMode::PolicyDenied => Err(IntelligenceError::PolicyDenied("data boundary")),
        }
    }
}

#[test]
fn deterministic_provider_produces_contract_valid_result() {
    let request = request();
    let response = SuccessProvider.execute(&request).unwrap();
    assert_eq!(response.request_id, request.request_id);
    assert_eq!(response.provider_id, request.provider_id);
    assert_eq!(response.claims[0].status, EpistemicStatus::Inferred);
    assert_eq!(response.provenance_refs.len(), 1);
}

#[test]
fn request_identity_is_preserved_end_to_end() {
    let request = request();
    let response = SuccessProvider.execute(&request).unwrap();
    assert_eq!(response.request_id, "req-1");
}

#[test]
fn capability_and_contract_identity_are_request_owned() {
    let request = request();
    assert_eq!(request.capability_id, "legal.analysis");
    assert_eq!(request.contract_version, "1.0");
}

#[test]
fn provider_and_model_identity_are_auditable() {
    let request = request();
    let response = SuccessProvider.execute(&request).unwrap();
    assert_eq!(response.provider_id, "fake-success");
    assert_eq!(response.model_id.as_deref(), Some("fake-model"));
    assert_eq!(response.model_version.as_deref(), Some("1"));
}

#[test]
fn epistemic_uncertainty_is_preserved() {
    let request = request();
    let response = SuccessProvider.execute(&request).unwrap();
    assert!(response.claims[0].status.is_uncertain());
}

#[test]
fn source_and_evidence_references_are_traceable() {
    let request = request();
    let response = SuccessProvider.execute(&request).unwrap();
    assert_eq!(response.source_refs[0].id, "source-1");
    assert_eq!(response.evidence_refs[0].id, "evidence-1");
    assert_eq!(response.claims[0].evidence_refs[0].id, "evidence-1");
}

#[test]
fn provenance_is_required_and_present() {
    let request = request();
    let response = SuccessProvider.execute(&request).unwrap();
    assert!(!response.provenance_refs.is_empty());
}

#[test]
fn missing_provenance_is_rejected() {
    let request = request();
    let mut response = success_response(&request);
    response.provenance_refs.clear();
    assert_eq!(
        response.validate_against(&request),
        Err("intelligence response provenance is required")
    );
}

#[test]
fn response_request_mismatch_is_rejected() {
    let request = request();
    let mut response = success_response(&request);
    response.request_id = "other-request".into();
    assert_eq!(
        response.validate_against(&request),
        Err("intelligence response request id mismatch")
    );
}

#[test]
fn response_provider_mismatch_is_rejected() {
    let request = request();
    let mut response = success_response(&request);
    response.provider_id = "other-provider".into();
    assert_eq!(
        response.validate_against(&request),
        Err("intelligence response provider id mismatch")
    );
}

#[test]
fn high_impact_request_cannot_omit_approval_requirement() {
    let mut request = request();
    request.risk_class = IntelligenceRiskClass::HighImpact;
    request.human_approval_required = false;
    assert_eq!(
        request.validate(),
        Err("high-impact intelligence requires human approval")
    );
}

#[test]
fn high_impact_request_can_be_declared_as_requiring_approval() {
    let mut request = request();
    request.risk_class = IntelligenceRiskClass::HighImpact;
    request.human_approval_required = true;
    assert!(request.validate().is_ok());
}

#[test]
fn provider_rejection_is_explicit() {
    let result = AdversarialProvider {
        mode: FailureMode::ProviderRejected,
    }
    .execute(&request());
    assert_eq!(result, Err(IntelligenceError::ProviderRejected));
}

#[test]
fn provider_unavailability_is_explicit() {
    let result = AdversarialProvider {
        mode: FailureMode::ProviderUnavailable,
    }
    .execute(&request());
    assert_eq!(result, Err(IntelligenceError::ProviderUnavailable));
}

#[test]
fn provider_timeout_is_explicit() {
    let result = AdversarialProvider {
        mode: FailureMode::Timeout,
    }
    .execute(&request());
    assert_eq!(result, Err(IntelligenceError::Timeout));
}

#[test]
fn malformed_provider_output_is_rejected() {
    let result = AdversarialProvider {
        mode: FailureMode::MalformedOutput,
    }
    .execute(&request());
    assert_eq!(
        result,
        Err(IntelligenceError::MalformedOutput(
            "intelligence response request id mismatch"
        ))
    );
}

#[test]
fn policy_denial_is_explicit() {
    let result = AdversarialProvider {
        mode: FailureMode::PolicyDenied,
    }
    .execute(&request());
    assert_eq!(
        result,
        Err(IntelligenceError::PolicyDenied("data boundary"))
    );
}

#[test]
fn tool_proposals_are_data_not_execution_authority() {
    let request = request();
    let mut response = success_response(&request);
    response.tool_proposals.push(IntelligenceToolProposal {
        tool_ref: reference(ResourceType::Action, "submit-tool"),
        arguments: serde_json::json!({"draft": true}),
    });
    assert!(response.validate_against(&request).is_ok());
    assert_eq!(response.tool_proposals.len(), 1);
}

#[test]
fn canonical_request_does_not_depend_on_provider_output_for_risk() {
    let mut request = request();
    request.risk_class = IntelligenceRiskClass::HighImpact;
    request.human_approval_required = true;
    let mut response = success_response(&request);
    response.tool_proposals.push(IntelligenceToolProposal {
        tool_ref: reference(ResourceType::Action, "submission"),
        arguments: serde_json::json!({"approved": false}),
    });
    assert!(response.validate_against(&request).is_ok());
    assert_eq!(request.risk_class, IntelligenceRiskClass::HighImpact);
    assert!(request.human_approval_required);
}

#[test]
fn json_round_trip_preserves_contract_fields() {
    let request = request();
    let json = serde_json::to_string(&request).unwrap();
    let decoded: IntelligenceRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, request);
}

#[test]
fn restricted_data_classification_remains_explicit() {
    let mut request = request();
    request.data_class = IntelligenceDataClass::Restricted;
    assert_eq!(request.data_class, IntelligenceDataClass::Restricted);
}

#[test]
fn retrieval_requirement_remains_explicit() {
    let request = request();
    assert!(request.retrieval_required);
}
