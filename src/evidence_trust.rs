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
        assert_eq!(transformation.validate(), Err("source evidence id is required"));
    }
}
