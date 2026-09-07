use crate::{Id, ResourceRef};
use serde::{Deserialize, Serialize};

/// Epistemic status carried by intelligence-produced claims.
///
/// These statuses describe the evidentiary position of a claim. They do not
/// grant legal authority and must not be silently strengthened by a provider.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EpistemicStatus {
    Observed,
    UserReported,
    EvidenceSupported,
    SourceSupported,
    SystemDerived,
    Inferred,
    Unverified,
    Contested,
    Unknown,
}

impl EpistemicStatus {
    pub fn is_uncertain(&self) -> bool {
        matches!(
            self,
            Self::Inferred | Self::Unverified | Self::Contested | Self::Unknown
        )
    }
}

/// Risk classification supplied by the caller/policy layer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntelligenceRiskClass {
    Low,
    Medium,
    HighImpact,
}

/// Data classification supplied by policy, not inferred by the model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntelligenceDataClass {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntelligenceRequest {
    pub request_id: Id,
    pub capability_id: Id,
    pub contract_version: String,
    pub provider_id: Id,
    pub model_id: Option<Id>,
    pub model_version: Option<String>,
    pub task_type: String,
    pub context_refs: Vec<ResourceRef>,
    pub jurisdiction_ref: Option<ResourceRef>,
    pub data_class: IntelligenceDataClass,
    pub required_output_schema: String,
    pub tool_permissions: Vec<ResourceRef>,
    pub retrieval_required: bool,
    pub provenance_required: bool,
    pub risk_class: IntelligenceRiskClass,
    pub human_approval_required: bool,
}

impl IntelligenceRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.request_id.is_empty() {
            return Err("intelligence request id is required");
        }
        if self.capability_id.is_empty() {
            return Err("intelligence capability id is required");
        }
        if self.contract_version.is_empty() {
            return Err("intelligence contract version is required");
        }
        if self.provider_id.is_empty() {
            return Err("intelligence provider id is required");
        }
        if self.task_type.is_empty() {
            return Err("intelligence task type is required");
        }
        if self.required_output_schema.is_empty() {
            return Err("intelligence output schema is required");
        }
        if self.risk_class == IntelligenceRiskClass::HighImpact && !self.human_approval_required {
            return Err("high-impact intelligence requires human approval");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntelligenceClaim {
    pub claim_id: Id,
    pub content: String,
    pub status: EpistemicStatus,
    pub source_refs: Vec<ResourceRef>,
    pub evidence_refs: Vec<ResourceRef>,
    pub provenance_ref: Option<ResourceRef>,
}

impl IntelligenceClaim {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.claim_id.is_empty() {
            return Err("intelligence claim id is required");
        }
        if self.content.is_empty() {
            return Err("intelligence claim content is required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntelligenceToolProposal {
    pub tool_ref: ResourceRef,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntelligenceResponse {
    pub request_id: Id,
    pub provider_id: Id,
    pub model_id: Option<Id>,
    pub model_version: Option<String>,
    pub output: String,
    pub claims: Vec<IntelligenceClaim>,
    pub source_refs: Vec<ResourceRef>,
    pub evidence_refs: Vec<ResourceRef>,
    pub tool_proposals: Vec<IntelligenceToolProposal>,
    pub warnings: Vec<String>,
    pub provenance_refs: Vec<ResourceRef>,
}

impl IntelligenceResponse {
    pub fn validate_against(&self, request: &IntelligenceRequest) -> Result<(), &'static str> {
        request.validate()?;
        if self.request_id != request.request_id {
            return Err("intelligence response request id mismatch");
        }
        if self.provider_id != request.provider_id {
            return Err("intelligence response provider id mismatch");
        }
        if self.output.is_empty() && self.claims.is_empty() {
            return Err("intelligence response output is empty");
        }
        for claim in &self.claims {
            claim.validate()?;
        }
        if request.provenance_required && self.provenance_refs.is_empty() {
            return Err("intelligence response provenance is required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntelligenceError {
    InvalidRequest(&'static str),
    ProviderRejected,
    ProviderUnavailable,
    Timeout,
    MalformedOutput(&'static str),
    PolicyDenied(&'static str),
}

/// Provider-neutral intelligence boundary. Implementations are untrusted
/// computation and must not bypass SIDERETH policy, authorization, tool,
/// provenance or approval boundaries.
pub trait IntelligenceProvider {
    fn execute(
        &self,
        request: &IntelligenceRequest,
    ) -> Result<IntelligenceResponse, IntelligenceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(resource_type: crate::ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    fn request() -> IntelligenceRequest {
        IntelligenceRequest {
            request_id: "req-1".into(),
            capability_id: "legal.analysis".into(),
            contract_version: "1.0".into(),
            provider_id: "fake-provider".into(),
            model_id: Some("fake-model".into()),
            model_version: Some("1".into()),
            task_type: "analysis".into(),
            context_refs: vec![reference(crate::ResourceType::Document, "doc-1")],
            jurisdiction_ref: Some(reference(crate::ResourceType::Jurisdiction, "india")),
            data_class: IntelligenceDataClass::Confidential,
            required_output_schema: "intelligence.response.v1".into(),
            tool_permissions: Vec::new(),
            retrieval_required: true,
            provenance_required: true,
            risk_class: IntelligenceRiskClass::Medium,
            human_approval_required: false,
        }
    }

    #[test]
    fn request_validation_rejects_missing_identity() {
        let mut value = request();
        value.request_id.clear();
        assert_eq!(value.validate(), Err("intelligence request id is required"));
    }

    #[test]
    fn high_impact_request_requires_approval() {
        let mut value = request();
        value.risk_class = IntelligenceRiskClass::HighImpact;
        assert_eq!(
            value.validate(),
            Err("high-impact intelligence requires human approval")
        );
        value.human_approval_required = true;
        assert!(value.validate().is_ok());
    }

    #[test]
    fn response_must_bind_to_request() {
        let req = request();
        let response = IntelligenceResponse {
            request_id: "other".into(),
            provider_id: "fake-provider".into(),
            model_id: Some("fake-model".into()),
            model_version: Some("1".into()),
            output: "candidate".into(),
            claims: Vec::new(),
            source_refs: Vec::new(),
            evidence_refs: Vec::new(),
            tool_proposals: Vec::new(),
            warnings: Vec::new(),
            provenance_refs: vec![reference(crate::ResourceType::Provenance, "prov-1")],
        };
        assert_eq!(
            response.validate_against(&req),
            Err("intelligence response request id mismatch")
        );
    }

    #[test]
    fn provenance_is_required_when_requested() {
        let req = request();
        let response = IntelligenceResponse {
            request_id: req.request_id.clone(),
            provider_id: req.provider_id.clone(),
            model_id: req.model_id.clone(),
            model_version: req.model_version.clone(),
            output: "candidate".into(),
            claims: Vec::new(),
            source_refs: Vec::new(),
            evidence_refs: Vec::new(),
            tool_proposals: Vec::new(),
            warnings: Vec::new(),
            provenance_refs: Vec::new(),
        };
        assert_eq!(
            response.validate_against(&req),
            Err("intelligence response provenance is required")
        );
    }

    #[test]
    fn epistemic_status_round_trips_stably() {
        let statuses = [
            EpistemicStatus::Observed,
            EpistemicStatus::UserReported,
            EpistemicStatus::EvidenceSupported,
            EpistemicStatus::SourceSupported,
            EpistemicStatus::SystemDerived,
            EpistemicStatus::Inferred,
            EpistemicStatus::Unverified,
            EpistemicStatus::Contested,
            EpistemicStatus::Unknown,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let decoded: EpistemicStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, status);
        }
    }

    #[test]
    fn uncertain_epistemic_status_remains_uncertain() {
        assert!(EpistemicStatus::Inferred.is_uncertain());
        assert!(EpistemicStatus::Unverified.is_uncertain());
        assert!(EpistemicStatus::Contested.is_uncertain());
        assert!(EpistemicStatus::Unknown.is_uncertain());
        assert!(!EpistemicStatus::Observed.is_uncertain());
    }
}
