use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Id;

pub struct EvidenceCapture<'a> {
    pub evidence_id: Id,
    pub schema_version: u32,
    pub case_id: Option<Id>,
    pub incident_id: Option<Id>,
    pub captured_at: String,
    pub captured_by: Id,
    pub media_type: String,
    pub storage_ref: Id,
    pub content: &'a [u8],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceOriginal {
    pub evidence_id: Id,
    pub schema_version: u32,
    pub case_id: Option<Id>,
    pub incident_id: Option<Id>,
    pub captured_at: String,
    pub captured_by: Id,
    pub media_type: String,
    pub content_hash: String,
    pub storage_ref: Id,
}

impl EvidenceOriginal {
    pub fn from_capture(capture: EvidenceCapture<'_>) -> Result<Self, &'static str> {
        let value = Self {
            evidence_id: capture.evidence_id,
            schema_version: capture.schema_version,
            case_id: capture.case_id,
            incident_id: capture.incident_id,
            captured_at: capture.captured_at,
            captured_by: capture.captured_by,
            media_type: capture.media_type,
            content_hash: sha256_hex(capture.content),
            storage_ref: capture.storage_ref,
        };

        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.evidence_id.is_empty() {
            return Err("evidence id is required");
        }
        if self.schema_version == 0 {
            return Err("schema version must be positive");
        }
        if self.case_id.is_none() && self.incident_id.is_none() {
            return Err("evidence aggregate is required");
        }
        if self.case_id.is_some() && self.incident_id.is_some() {
            return Err("evidence aggregate must be singular");
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
        Ok(())
    }
}

pub fn sha256_hex(content: &[u8]) -> String {
    let digest = Sha256::digest(content);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivedArtifact {
    pub artifact_id: Id,
    pub schema_version: u32,
    pub source_evidence_id: Id,
    pub artifact_type: String,
    pub created_at: String,
    pub created_by: Id,
    pub storage_ref: Id,
}

impl DerivedArtifact {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.artifact_id.is_empty() {
            return Err("artifact id is required");
        }
        if self.schema_version == 0 {
            return Err("schema version must be positive");
        }
        if self.source_evidence_id.is_empty() {
            return Err("source evidence id is required");
        }
        if self.artifact_type.is_empty() {
            return Err("artifact type is required");
        }
        if self.created_at.is_empty() {
            return Err("artifact time is required");
        }
        if self.created_by.is_empty() {
            return Err("created by is required");
        }
        if self.storage_ref.is_empty() {
            return Err("storage reference is required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture<'a>(content: &'a [u8]) -> EvidenceCapture<'a> {
        EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-02T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            storage_ref: "storage-1".into(),
            content,
        }
    }

    fn original() -> EvidenceOriginal {
        EvidenceOriginal::from_capture(capture(b"original evidence")).unwrap()
    }

    #[test]
    fn content_hash_is_deterministic() {
        assert_eq!(sha256_hex(b"hello"), sha256_hex(b"hello"));
    }

    #[test]
    fn original_requires_case_or_incident() {
        let mut value = capture(b"evidence");
        value.case_id = None;

        let result = EvidenceOriginal::from_capture(value);

        assert_eq!(result, Err("evidence aggregate is required"));
    }

    #[test]
    fn original_cannot_target_two_aggregates() {
        let mut value = capture(b"evidence");
        value.incident_id = Some("incident-1".into());

        let result = EvidenceOriginal::from_capture(value);

        assert_eq!(result, Err("evidence aggregate must be singular"));
    }

    #[test]
    fn original_hash_matches_content() {
        assert_eq!(original().content_hash, sha256_hex(b"original evidence"));
    }

    #[test]
    fn derived_artifact_requires_source() {
        let artifact = DerivedArtifact {
            artifact_id: "artifact-1".into(),
            schema_version: 1,
            source_evidence_id: String::new(),
            artifact_type: "ocr".into(),
            created_at: "2026-09-02T10:01:00Z".into(),
            created_by: "system".into(),
            storage_ref: "storage-2".into(),
        };

        assert_eq!(artifact.validate(), Err("source evidence id is required"));
    }
}
