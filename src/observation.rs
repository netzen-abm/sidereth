use crate::{EpistemicStatus, Id, IntelligenceDataClass, ResourceRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The origin/modality of an observation. This is deliberately distinct from
/// `EpistemicStatus`: origin describes how an assertion entered SIDERETH;
/// epistemic status describes its evidentiary position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservationOrigin {
    DirectObservation,
    UserReport,
    Measurement,
    SourceAssertion,
    SystemDetection,
    AiDerivation,
}

/// The semantic class of an observation. This is intentionally generic and
/// does not encode a domain-specific ontology.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservationType {
    Condition,
    Measurement,
    Location,
    Statement,
    Status,
    SensorDetection,
    DocumentAssertion,
    SystemDetection,
}

/// A bounded assertion about a subject/resource at a stated time.
///
/// Observation is a semantic primitive, not an Event subtype, Evidence record,
/// legal conclusion, repository, or longitudinal aggregate. Evidence and
/// provenance are references to their existing canonical primitives.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Observation {
    pub observation_id: Id,
    pub schema_version: u32,
    pub subject_ref: ResourceRef,
    pub observation_type: ObservationType,
    pub observation_origin: ObservationOrigin,
    pub observed_at: String,
    pub recorded_at: String,
    pub assertion: Value,
    pub epistemic_status: EpistemicStatus,
    pub source_refs: Vec<ResourceRef>,
    pub evidence_refs: Vec<ResourceRef>,
    pub provenance_ref: Option<ResourceRef>,
    pub context_refs: Vec<ResourceRef>,
    pub data_class: IntelligenceDataClass,
}

impl Observation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.observation_id.is_empty() {
            return Err("observation id is required");
        }
        if self.schema_version == 0 {
            return Err("schema version must be positive");
        }
        if self.subject_ref.id.is_empty() {
            return Err("observation subject id is required");
        }
        if self.observed_at.is_empty() {
            return Err("observation time is required");
        }
        if self.recorded_at.is_empty() {
            return Err("observation recorded time is required");
        }
        if self.assertion.is_null() {
            return Err("observation assertion is required");
        }
        Ok(())
    }

    /// Observations retain their epistemic status exactly as supplied by the
    /// trusted caller/policy boundary. AI or other providers cannot silently
    /// promote the status through this type.
    pub fn epistemic_status(&self) -> EpistemicStatus {
        self.epistemic_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceType;

    fn subject() -> ResourceRef {
        ResourceRef::new(ResourceType::Incident, "incident-1").unwrap()
    }

    fn observation() -> Observation {
        Observation {
            observation_id: "obs-1".into(),
            schema_version: 1,
            subject_ref: subject(),
            observation_type: ObservationType::Condition,
            observation_origin: ObservationOrigin::DirectObservation,
            observed_at: "2026-09-12T10:00:00Z".into(),
            recorded_at: "2026-09-12T10:01:00Z".into(),
            assertion: serde_json::json!({"condition":"damaged"}),
            epistemic_status: EpistemicStatus::Observed,
            source_refs: Vec::new(),
            evidence_refs: Vec::new(),
            provenance_ref: None,
            context_refs: Vec::new(),
            data_class: IntelligenceDataClass::Confidential,
        }
    }

    #[test]
    fn valid_observation_passes_validation() {
        assert!(observation().validate().is_ok());
    }

    #[test]
    fn observation_origin_and_epistemic_status_are_independent() {
        let mut value = observation();
        value.observation_origin = ObservationOrigin::Measurement;
        value.epistemic_status = EpistemicStatus::Unverified;
        assert!(value.validate().is_ok());
        assert_eq!(value.observation_origin, ObservationOrigin::Measurement);
        assert_eq!(value.epistemic_status(), EpistemicStatus::Unverified);
    }

    #[test]
    fn ai_derivation_does_not_imply_epistemic_upgrade() {
        let mut value = observation();
        value.observation_origin = ObservationOrigin::AiDerivation;
        value.epistemic_status = EpistemicStatus::Inferred;
        assert!(value.validate().is_ok());
        assert_eq!(value.epistemic_status(), EpistemicStatus::Inferred);
    }

    #[test]
    fn empty_identity_is_rejected() {
        let mut value = observation();
        value.observation_id.clear();
        assert_eq!(value.validate(), Err("observation id is required"));
    }

    #[test]
    fn missing_observation_time_is_rejected() {
        let mut value = observation();
        value.observed_at.clear();
        assert_eq!(value.validate(), Err("observation time is required"));
    }

    #[test]
    fn missing_recorded_time_is_rejected() {
        let mut value = observation();
        value.recorded_at.clear();
        assert_eq!(
            value.validate(),
            Err("observation recorded time is required")
        );
    }

    #[test]
    fn null_assertion_is_rejected() {
        let mut value = observation();
        value.assertion = Value::Null;
        assert_eq!(value.validate(), Err("observation assertion is required"));
    }

    #[test]
    fn wire_format_is_stable() {
        let value = observation();
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["observation_id"], "obs-1");
        assert_eq!(json["subject_ref"]["resource_type"], "incident");
        assert_eq!(json["observation_origin"], "DIRECT_OBSERVATION");
        assert_eq!(json["epistemic_status"], "OBSERVED");
        assert_eq!(json["data_class"], "CONFIDENTIAL");
    }
}
