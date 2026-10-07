use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaOrigin {
    LiveCapture,
    ImportedFile,
    DerivedArtifact,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityStatus {
    Unverified,
    HashVerified,
    SignatureVerified,
    HardwareAttested,
    IntegrityFailed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAttestationStatus {
    NotAvailable,
    NotRequested,
    SoftwareFallback,
    HardwareAttested,
    VerificationFailed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocationDisclosure {
    Exact,
    ReducedPrecision,
    Redacted,
    NotShareable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureLocation {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub horizontal_accuracy_m: Option<f64>,
    pub altitude_m: Option<f64>,
    pub altitude_accuracy_m: Option<f64>,
    pub disclosure: LocationDisclosure,
}

impl CaptureLocation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(-90.0..=90.0).contains(&self.latitude_deg) {
            return Err("latitude is out of range");
        }
        if !(-180.0..=180.0).contains(&self.longitude_deg) {
            return Err("longitude is out of range");
        }
        if self.horizontal_accuracy_m.is_some_and(|v| v < 0.0) {
            return Err("horizontal accuracy must not be negative");
        }
        if self.altitude_accuracy_m.is_some_and(|v| v < 0.0) {
            return Err("altitude accuracy must not be negative");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceTransformation {
    pub transformation_id: Id,
    pub source_evidence_id: Id,
    pub transformation_type: String,
    pub created_at: String,
    pub created_by: Id,
    pub tool_id: Option<Id>,
    pub tool_version: Option<String>,
    pub input_hash: Option<String>,
    pub output_hash: Option<String>,
}

impl EvidenceTransformation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.transformation_id.is_empty() {
            return Err("transformation id is required");
        }
        if self.source_evidence_id.is_empty() {
            return Err("source evidence id is required");
        }
        if self.transformation_type.is_empty() {
            return Err("transformation type is required");
        }
        if self.created_at.is_empty() {
            return Err("transformation time is required");
        }
        if self.created_by.is_empty() {
            return Err("transformation creator is required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceTrustMetadata {
    pub media_origin: MediaOrigin,
    pub integrity_status: IntegrityStatus,
    pub hardware_attestation: HardwareAttestationStatus,
    pub device_ref: Option<Id>,
    pub app_version: Option<String>,
    pub capture_method: Option<String>,
    pub location: Option<CaptureLocation>,
}

impl Default for EvidenceTrustMetadata {
    fn default() -> Self {
        Self {
            media_origin: MediaOrigin::Unknown,
            integrity_status: IntegrityStatus::Unverified,
            hardware_attestation: HardwareAttestationStatus::NotRequested,
            device_ref: None,
            app_version: None,
            capture_method: None,
            location: None,
        }
    }
}

impl EvidenceTrustMetadata {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(location) = &self.location {
            location.validate()?;
        }
        if self.integrity_status == IntegrityStatus::HardwareAttested
            && self.hardware_attestation != HardwareAttestationStatus::HardwareAttested
        {
            return Err("hardware-attested integrity requires verified hardware attestation");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidencePassport {
    pub evidence_id: Id,
    pub media_origin: MediaOrigin,
    pub captured_at: String,
    pub captured_by: Id,
    pub media_type: String,
    pub content_hash: String,
    pub storage_ref: Id,
    pub integrity_status: IntegrityStatus,
    pub hardware_attestation: HardwareAttestationStatus,
    pub device_ref: Option<Id>,
    pub app_version: Option<String>,
    pub capture_method: Option<String>,
    pub location: Option<CaptureLocation>,
}

impl EvidencePassport {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.evidence_id.is_empty() {
            return Err("evidence id is required");
        }
        if self.captured_at.is_empty() {
            return Err("capture time is required");
        }
        if self.captured_by.is_empty() {
            return Err("captured by is required");
        }
        if self.media_type.is_empty() {
            return Err("media type is required");
        }
        if self.content_hash.is_empty() {
            return Err("content hash is required");
        }
        if self.storage_ref.is_empty() {
            return Err("storage reference is required");
        }
        if let Some(location) = &self.location {
            location.validate()?;
        }
        if self.integrity_status == IntegrityStatus::HardwareAttested
            && self.hardware_attestation != HardwareAttestationStatus::HardwareAttested
        {
            return Err("hardware-attested passport requires verified hardware attestation");
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceIntegrityChain {
    evidence_id: Id,
    original_hash: String,
    transformations: Vec<EvidenceTransformation>,
}

impl EvidenceIntegrityChain {
    pub fn new(original: &crate::EvidenceOriginal) -> Result<Self, &'static str> {
        original.validate()?;
        if original.content_hash.is_empty() {
            return Err("original content hash is required");
        }
        Ok(Self {
            evidence_id: original.evidence_id.clone(),
            original_hash: original.content_hash.clone(),
            transformations: Vec::new(),
        })
    }

    pub fn append(
        mut self,
        transformation: EvidenceTransformation,
    ) -> Result<Self, &'static str> {
        transformation.validate()?;
        if transformation.source_evidence_id != self.evidence_id {
            return Err("transformation source does not match evidence chain");
        }

        let input_hash = transformation
            .input_hash
            .as_deref()
            .ok_or("transformation input hash is required")?;
        let output_hash = transformation
            .output_hash
            .as_deref()
            .ok_or("transformation output hash is required")?;

        if input_hash != self.current_hash() {
            return Err("transformation input hash does not match chain head");
        }
        if output_hash.is_empty() {
            return Err("transformation output hash is required");
        }

        self.transformations.push(transformation);
        Ok(self)
    }

    pub fn evidence_id(&self) -> &Id {
        &self.evidence_id
    }

    pub fn original_hash(&self) -> &str {
        &self.original_hash
    }

    pub fn current_hash(&self) -> &str {
        self.transformations
            .last()
            .and_then(|step| step.output_hash.as_deref())
            .unwrap_or(&self.original_hash)
    }

    pub fn transformations(&self) -> &[EvidenceTransformation] {
        &self.transformations
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.evidence_id.is_empty() {
            return Err("evidence chain id is required");
        }
        if self.original_hash.is_empty() {
            return Err("evidence chain original hash is required");
        }

        let mut current = self.original_hash.as_str();
        let mut ids = std::collections::HashSet::new();
        for transformation in &self.transformations {
            transformation.validate()?;
            if transformation.source_evidence_id != self.evidence_id {
                return Err("transformation source does not match evidence chain");
            }
            if !ids.insert(&transformation.transformation_id) {
                return Err("duplicate transformation in evidence chain");
            }
            let input_hash = transformation
                .input_hash
                .as_deref()
                .ok_or("transformation input hash is required")?;
            let output_hash = transformation
                .output_hash
                .as_deref()
                .ok_or("transformation output hash is required")?;
            if input_hash != current {
                return Err("transformation input hash does not match chain head");
            }
            if output_hash.is_empty() {
                return Err("transformation output hash is required");
            }
            current = output_hash;
        }
        Ok(())
    }
}

    #[test]
    fn integrity_chain_starts_at_original_hash() {
        let original = crate::EvidenceOriginal::from_capture(crate::EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-07T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            storage_ref: "object-1".into(),
            content: b"original",
        })
        .unwrap();
        let chain = EvidenceIntegrityChain::new(&original).unwrap();
        assert_eq!(chain.current_hash(), original.content_hash);
    }

    #[test]
    fn integrity_chain_requires_contiguous_hashes() {
        let original = crate::EvidenceOriginal::from_capture(crate::EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-07T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            storage_ref: "object-1".into(),
            content: b"original",
        })
        .unwrap();
        let chain = EvidenceIntegrityChain::new(&original).unwrap();
        let broken = EvidenceTransformation {
            transformation_id: "transform-1".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "system".into(),
            tool_id: Some("ocr".into()),
            tool_version: Some("1.0".into()),
            input_hash: Some("wrong-input".into()),
            output_hash: Some("output-hash".into()),
        };
        assert_eq!(
            chain.append(broken),
            Err("transformation input hash does not match chain head")
        );
    }

    #[test]
    fn integrity_chain_accepts_contiguous_transformation() {
        let original = crate::EvidenceOriginal::from_capture(crate::EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-07T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            storage_ref: "object-1".into(),
            content: b"original",
        })
        .unwrap();
        let chain = EvidenceIntegrityChain::new(&original).unwrap();
        let output_hash = crate::sha256_hex(b"ocr output");
        let transformation = EvidenceTransformation {
            transformation_id: "transform-1".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "system".into(),
            tool_id: Some("ocr".into()),
            tool_version: Some("1.0".into()),
            input_hash: Some(original.content_hash.clone()),
            output_hash: Some(output_hash.clone()),
        };
        let chain = chain.append(transformation).unwrap();
        assert_eq!(chain.current_hash(), output_hash);
        assert!(chain.validate().is_ok());
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_capture_location_is_accepted() {
        let location = CaptureLocation {
            latitude_deg: 12.9716,
            longitude_deg: 77.5946,
            horizontal_accuracy_m: Some(8.0),
            altitude_m: None,
            altitude_accuracy_m: None,
            disclosure: LocationDisclosure::Exact,
        };
        assert!(location.validate().is_ok());
    }

    #[test]
    fn invalid_coordinates_are_rejected() {
        let location = CaptureLocation {
            latitude_deg: 91.0,
            longitude_deg: 77.5946,
            horizontal_accuracy_m: None,
            altitude_m: None,
            altitude_accuracy_m: None,
            disclosure: LocationDisclosure::Exact,
        };
        assert_eq!(location.validate(), Err("latitude is out of range"));
    }

    #[test]
    fn hardware_claim_requires_hardware_attestation() {
        let metadata = EvidenceTrustMetadata {
            integrity_status: IntegrityStatus::HardwareAttested,
            hardware_attestation: HardwareAttestationStatus::SoftwareFallback,
            ..Default::default()
        };
        assert_eq!(
            metadata.validate(),
            Err("hardware-attested integrity requires verified hardware attestation")
        );
    }

    #[test]
    fn imported_media_is_distinct_from_live_capture() {
        assert_ne!(MediaOrigin::ImportedFile, MediaOrigin::LiveCapture);
    }

    #[test]
    fn transformation_requires_source() {
        let transformation = EvidenceTransformation {
            transformation_id: "transform-1".into(),
            source_evidence_id: String::new(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:00:00Z".into(),
            created_by: "system".into(),
            tool_id: None,
            tool_version: None,
            input_hash: None,
            output_hash: None,
        };
        assert_eq!(
            transformation.validate(),
            Err("source evidence id is required")
        );
    }
}
