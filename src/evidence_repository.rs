use std::collections::HashMap;

use crate::{EvidenceOriginal, EvidenceTransformation, EvidenceTrustMetadata, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidencePersistenceError {
    ValidationFailure,
    NotFound,
    Duplicate,
    IntegrityFailure,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersistedEvidence {
    pub schema_version: u16,
    pub original: EvidenceOriginal,
    pub trust: EvidenceTrustMetadata,
    pub transformations: Vec<EvidenceTransformation>,
}

impl PersistedEvidence {
    pub fn new(
        schema_version: u16,
        original: EvidenceOriginal,
        trust: EvidenceTrustMetadata,
    ) -> Result<Self, EvidencePersistenceError> {
        if schema_version == 0 {
            return Err(EvidencePersistenceError::ValidationFailure);
        }
        original
            .validate()
            .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
        trust
            .validate()
            .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
        Ok(Self {
            schema_version,
            original,
            trust,
            transformations: Vec::new(),
        })
    }

    pub fn with_transformation(
        mut self,
        transformation: EvidenceTransformation,
    ) -> Result<Self, EvidencePersistenceError> {
        transformation
            .validate()
            .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
        if transformation.source_evidence_id != self.original.evidence_id {
            return Err(EvidencePersistenceError::IntegrityFailure);
        }
        if self
            .transformations
            .iter()
            .any(|item| item.transformation_id == transformation.transformation_id)
        {
            return Err(EvidencePersistenceError::Duplicate);
        }
        self.transformations.push(transformation);
        Ok(self)
    }
}

pub trait EvidenceTrustRepository {
    fn create(&mut self, evidence: PersistedEvidence) -> Result<(), EvidencePersistenceError>;
    fn get(&self, evidence_id: &Id) -> Result<Option<PersistedEvidence>, EvidencePersistenceError>;
    fn list_transformations(
        &self,
        evidence_id: &Id,
    ) -> Result<Vec<EvidenceTransformation>, EvidencePersistenceError>;
}

#[derive(Debug, Default)]
pub struct InMemoryEvidenceTrustRepository {
    records: HashMap<Id, PersistedEvidence>,
}

impl EvidenceTrustRepository for InMemoryEvidenceTrustRepository {
    fn create(&mut self, evidence: PersistedEvidence) -> Result<(), EvidencePersistenceError> {
        if self.records.contains_key(&evidence.original.evidence_id) {
            return Err(EvidencePersistenceError::Duplicate);
        }
        self.records
            .insert(evidence.original.evidence_id.clone(), evidence);
        Ok(())
    }

    fn get(&self, evidence_id: &Id) -> Result<Option<PersistedEvidence>, EvidencePersistenceError> {
        Ok(self.records.get(evidence_id).cloned())
    }

    fn list_transformations(
        &self,
        evidence_id: &Id,
    ) -> Result<Vec<EvidenceTransformation>, EvidencePersistenceError> {
        let record = self
            .records
            .get(evidence_id)
            .ok_or(EvidencePersistenceError::NotFound)?;
        Ok(record.transformations.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvidenceCapture, MediaOrigin};

    fn original() -> EvidenceOriginal {
        EvidenceOriginal::from_capture(EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-07T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "image/jpeg".into(),
            storage_ref: "object-1".into(),
            content: b"original",
        })
        .unwrap()
    }

    #[test]
    fn create_and_round_trip_preserves_trust_state() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let trust = EvidenceTrustMetadata {
            media_origin: MediaOrigin::LiveCapture,
            ..Default::default()
        };
        let evidence = PersistedEvidence::new(1, original(), trust).unwrap();
        repository.create(evidence.clone()).unwrap();
        assert_eq!(repository.get(&"evidence-1".into()).unwrap(), Some(evidence));
    }

    #[test]
    fn original_is_immutable_by_rejecting_duplicate_create() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let evidence = PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        repository.create(evidence.clone()).unwrap();
        assert_eq!(repository.create(evidence), Err(EvidencePersistenceError::Duplicate));
    }

    #[test]
    fn transformation_must_reference_same_evidence() {
        let transformation = EvidenceTransformation {
            transformation_id: "transform-1".into(),
            source_evidence_id: "evidence-2".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "system".into(),
            tool_id: Some("ocr-engine".into()),
            tool_version: Some("1.0".into()),
            input_hash: Some("input".into()),
            output_hash: Some("output".into()),
        };
        let evidence = PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        assert_eq!(
            evidence.with_transformation(transformation),
            Err(EvidencePersistenceError::IntegrityFailure)
        );
    }

    #[test]
    fn transformations_round_trip() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let transformation = EvidenceTransformation {
            transformation_id: "transform-1".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "system".into(),
            tool_id: Some("ocr-engine".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        };
        let evidence = PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default())
            .unwrap()
            .with_transformation(transformation.clone())
            .unwrap();
        repository.create(evidence).unwrap();
        assert_eq!(
            repository.list_transformations(&"evidence-1".into()).unwrap(),
            vec![transformation]
        );
    }

    #[test]
    fn zero_schema_version_is_rejected_before_persistence() {
        assert_eq!(
            PersistedEvidence::new(0, original(), EvidenceTrustMetadata::default()),
            Err(EvidencePersistenceError::ValidationFailure)
        );
    }
}
