use std::collections::HashMap;

use crate::persistence::{
    PersistenceError, ResourceRef, ResourceWrite, ResourceWriteMode, UnitOfWorkContext,
};
use crate::{EvidenceOriginal, EvidenceTransformation, EvidenceTrustMetadata, Id, ResourceType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidencePersistenceError {
    ValidationFailure,
    NotFound,
    Duplicate,
    IntegrityFailure,
}

impl From<EvidencePersistenceError> for PersistenceError {
    fn from(error: EvidencePersistenceError) -> Self {
        match error {
            EvidencePersistenceError::ValidationFailure => PersistenceError::ValidationFailure,
            EvidencePersistenceError::NotFound => PersistenceError::NotFound,
            EvidencePersistenceError::Duplicate => PersistenceError::Duplicate,
            EvidencePersistenceError::IntegrityFailure => PersistenceError::IntegrityFailure,
        }
    }
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

/// Evidence Trust persistence adapter over the canonical Unit-of-Work context.
///
/// The adapter deliberately does not begin, commit, or roll back a transaction.
/// The caller owns the surrounding Unit-of-Work so evidence can be committed
/// atomically with Case, Incident, or other resources.
pub struct EvidenceTrustUnitOfWorkRepository<'a, C: UnitOfWorkContext> {
    context: &'a mut C,
}

impl<'a, C: UnitOfWorkContext> EvidenceTrustUnitOfWorkRepository<'a, C> {
    pub fn new(context: &'a mut C) -> Self {
        Self { context }
    }

    pub fn create(&mut self, evidence: PersistedEvidence) -> Result<(), PersistenceError> {
        let resource_ref = ResourceRef::new(
            ResourceType::Evidence,
            evidence.original.evidence_id.clone(),
        )
        .map_err(|_| PersistenceError::IntegrityFailure)?;
        let payload = serde_json::to_value(&evidence)
            .map_err(|_| PersistenceError::SerializationFailure)?;
        let write = ResourceWrite::new(
            resource_ref,
            evidence.schema_version,
            payload,
            ResourceWriteMode::Insert,
        )
        .map_err(|_| PersistenceError::ValidationFailure)?;
        self.context
            .write_resource(write)
            .map_err(|error| match error {
                crate::persistence::UnitOfWorkError::Persistence(error) => error,
                crate::persistence::UnitOfWorkError::InvalidOperation => {
                    PersistenceError::ValidationFailure
                }
            })
    }

    pub fn get(&mut self, evidence_id: &Id) -> Result<Option<PersistedEvidence>, PersistenceError> {
        let resource_ref = ResourceRef::new(ResourceType::Evidence, evidence_id.clone())
            .map_err(|_| PersistenceError::IntegrityFailure)?;
        let record = self
            .context
            .read_resource(&resource_ref)
            .map_err(|error| match error {
                crate::persistence::UnitOfWorkError::Persistence(error) => error,
                crate::persistence::UnitOfWorkError::InvalidOperation => {
                    PersistenceError::ValidationFailure
                }
            })?;
        record
            .map(|record| {
                serde_json::from_value::<PersistedEvidence>(record.payload)
                    .map_err(|_| PersistenceError::SerializationFailure)
            })
            .transpose()
    }

    pub fn list_transformations(
        &mut self,
        evidence_id: &Id,
    ) -> Result<Vec<EvidenceTransformation>, PersistenceError> {
        self.get(evidence_id)?
            .map(|evidence| evidence.transformations)
            .ok_or(PersistenceError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{ResourceRecord, Revision, UnitOfWorkError};
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
        assert_eq!(
            repository.get(&"evidence-1".into()).unwrap(),
            Some(evidence)
        );
    }

    #[test]
    fn original_is_immutable_by_rejecting_duplicate_create() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        repository.create(evidence.clone()).unwrap();
        assert_eq!(
            repository.create(evidence),
            Err(EvidencePersistenceError::Duplicate)
        );
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
        let evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
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
            repository
                .list_transformations(&"evidence-1".into())
                .unwrap(),
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

    #[derive(Default)]
    struct FakeContext {
        record: Option<ResourceRecord>,
    }

    impl UnitOfWorkContext for FakeContext {
        fn read_resource(
            &mut self,
            resource_ref: &ResourceRef,
        ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
            Ok(self
                .record
                .as_ref()
                .filter(|record| record.resource_ref == *resource_ref)
                .cloned())
        }

        fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
            if self.record.is_some() && write.mode == ResourceWriteMode::Insert {
                return Err(UnitOfWorkError::Persistence(PersistenceError::Duplicate));
            }
            self.record = Some(ResourceRecord {
                resource_ref: write.resource_ref,
                schema_version: write.schema_version,
                revision: Revision::initial(),
                payload: write.payload,
            });
            Ok(())
        }

        fn link_resources(
            &mut self,
            _link: crate::persistence::ResourceLink,
        ) -> Result<(), UnitOfWorkError> {
            Ok(())
        }
    }

    #[test]
    fn uow_adapter_round_trips_without_owning_transaction() {
        let evidence = PersistedEvidence::new(
            1,
            original(),
            EvidenceTrustMetadata {
                media_origin: MediaOrigin::LiveCapture,
                ..Default::default()
            },
        )
        .unwrap();
        let mut context = FakeContext::default();
        {
            let mut repository = EvidenceTrustUnitOfWorkRepository::new(&mut context);
            repository.create(evidence.clone()).unwrap();
            assert_eq!(
                repository.get(&"evidence-1".into()).unwrap(),
                Some(evidence)
            );
        }
    }

    #[test]
    fn uow_adapter_preserves_duplicate_error_from_provider() {
        let evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        let mut context = FakeContext::default();
        {
            let mut repository = EvidenceTrustUnitOfWorkRepository::new(&mut context);
            repository.create(evidence.clone()).unwrap();
            assert_eq!(
                repository.create(evidence),
                Err(PersistenceError::Duplicate)
            );
        }
    }

    #[test]
    fn uow_adapter_returns_not_found_for_missing_transformations() {
        let mut context = FakeContext::default();
        let mut repository = EvidenceTrustUnitOfWorkRepository::new(&mut context);
        assert_eq!(
            repository.list_transformations(&"missing".into()),
            Err(PersistenceError::NotFound)
        );
    }
}
