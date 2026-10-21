use std::collections::HashMap;

use crate::persistence::{PersistenceError, ResourceWrite, ResourceWriteMode, UnitOfWorkContext};
use crate::{
    EvidenceOriginal, EvidenceTransformation, EvidenceTrustMetadata, Id, Provenance, ResourceRef,
    ResourceType,
};

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PersistedEvidence {
    pub schema_version: u16,
    pub original: EvidenceOriginal,
    pub trust: EvidenceTrustMetadata,
    pub transformations: Vec<EvidenceTransformation>,
    /// Provenance records bound one-to-one to transformation IDs.
    #[serde(default)]
    pub transformation_provenance: Vec<TransformationProvenance>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransformationProvenance {
    pub transformation_id: Id,
    pub provenance: Provenance,
}

/**
 * Validates the persisted evidence aggregate at the generic resource-write
 * boundary. This catches callers that bypass EvidenceTrustUnitOfWorkRepository
 * or mutate/deserialize a ResourceWrite after construction.
 */
pub(crate) fn validate_evidence_resource_payload(
    resource_ref: &ResourceRef,
    schema_version: u16,
    payload: &serde_json::Value,
) -> Result<(), PersistenceError> {
    if resource_ref.resource_type != ResourceType::Evidence {
        return Ok(());
    }
    if schema_version == 0 {
        return Err(PersistenceError::ValidationFailure);
    }

    let evidence: PersistedEvidence = serde_json::from_value(payload.clone())
        .map_err(|_| PersistenceError::SerializationFailure)?;
    if evidence.schema_version != schema_version || evidence.original.evidence_id != resource_ref.id
    {
        return Err(PersistenceError::IntegrityFailure);
    }
    evidence.validate().map_err(PersistenceError::from)
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
            transformation_provenance: Vec::new(),
        })
    }

    pub fn validate(&self) -> Result<(), EvidencePersistenceError> {
        if self.schema_version == 0 {
            return Err(EvidencePersistenceError::ValidationFailure);
        }
        self.original
            .validate()
            .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
        self.trust
            .validate()
            .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
        let mut transformation_ids = std::collections::HashSet::<Id>::new();
        for transformation in &self.transformations {
            transformation
                .validate()
                .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
            if transformation.source_evidence_id != self.original.evidence_id {
                return Err(EvidencePersistenceError::IntegrityFailure);
            }
            if !transformation_ids.insert(transformation.transformation_id.clone()) {
                return Err(EvidencePersistenceError::Duplicate);
            }
            let matching = self
                .transformation_provenance
                .iter()
                .filter(|binding| binding.transformation_id == transformation.transformation_id)
                .collect::<Vec<_>>();
            if matching.len() != 1 {
                return Err(EvidencePersistenceError::IntegrityFailure);
            }
            transformation
                .validate_provenance(&matching[0].provenance)
                .map_err(|_| EvidencePersistenceError::IntegrityFailure)?;
        }
        let mut bound_ids = std::collections::HashSet::new();
        for binding in &self.transformation_provenance {
            if !transformation_ids.contains(&binding.transformation_id)
                || !bound_ids.insert(&binding.transformation_id)
            {
                return Err(EvidencePersistenceError::IntegrityFailure);
            }
        }
        Ok(())
    }

    pub fn with_transformation(
        mut self,
        transformation: EvidenceTransformation,
        provenance: Provenance,
    ) -> Result<Self, EvidencePersistenceError> {
        transformation
            .validate()
            .map_err(|_| EvidencePersistenceError::ValidationFailure)?;
        if transformation.source_evidence_id != self.original.evidence_id {
            return Err(EvidencePersistenceError::IntegrityFailure);
        }
        transformation
            .validate_provenance(&provenance)
            .map_err(|_| EvidencePersistenceError::IntegrityFailure)?;
        if self
            .transformations
            .iter()
            .any(|item| item.transformation_id == transformation.transformation_id)
            || self
                .transformation_provenance
                .iter()
                .any(|binding| binding.transformation_id == transformation.transformation_id)
        {
            return Err(EvidencePersistenceError::Duplicate);
        }
        self.transformation_provenance
            .push(TransformationProvenance {
                transformation_id: transformation.transformation_id.clone(),
                provenance,
            });
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
        evidence.validate()?;
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
        evidence.validate().map_err(PersistenceError::from)?;
        let resource_ref = ResourceRef::new(
            ResourceType::Evidence,
            evidence.original.evidence_id.clone(),
        )
        .map_err(|_| PersistenceError::IntegrityFailure)?;
        let payload =
            serde_json::to_value(&evidence).map_err(|_| PersistenceError::SerializationFailure)?;
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
                let evidence = serde_json::from_value::<PersistedEvidence>(record.payload)
                    .map_err(|_| PersistenceError::SerializationFailure)?;
                evidence.validate().map_err(PersistenceError::from)?;
                Ok(evidence)
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
    fn repository_rejects_invalid_persisted_evidence() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let mut evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        evidence.schema_version = 0;
        assert_eq!(
            repository.create(evidence),
            Err(EvidencePersistenceError::ValidationFailure)
        );
    }

    #[test]
    fn repository_rejects_invalid_transformation_in_persisted_record() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let mut evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        evidence.transformations.push(EvidenceTransformation {
            transformation_id: "transform-invalid".into(),
            source_evidence_id: "other-evidence".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "system".into(),
            tool_id: None,
            tool_version: None,
            input_hash: None,
            output_hash: None,
        });
        assert_eq!(
            repository.create(evidence),
            Err(EvidencePersistenceError::IntegrityFailure)
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
            evidence.with_transformation(
                transformation,
                Provenance {
                    provenance_id: "prov-mismatch".into(),
                    actor_ref: Some(ResourceRef::new(ResourceType::Tool, "system").unwrap()),
                    source_refs: vec![],
                    input_refs: vec![
                        ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()
                    ],
                    operation: "ocr".into(),
                    occurred_at: "2026-09-07T10:01:00Z".into(),
                },
            ),
            Err(EvidencePersistenceError::IntegrityFailure)
        );
    }

    #[test]
    fn persisted_transformation_without_provenance_is_rejected() {
        let mut repository = InMemoryEvidenceTrustRepository::default();
        let mut evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        evidence.transformations.push(EvidenceTransformation {
            transformation_id: "transform-without-provenance".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "ocr-service".into(),
            tool_id: Some("ocr-service".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        });
        assert_eq!(
            repository.create(evidence),
            Err(EvidencePersistenceError::IntegrityFailure)
        );
    }

    #[test]
    fn legacy_serialized_transformation_without_provenance_fails_closed() {
        let transformation = EvidenceTransformation {
            transformation_id: "legacy-transform-unbound".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "legacy-ocr-service".into(),
            tool_id: Some("legacy-ocr-service".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        };
        let mut evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        evidence.transformations.push(transformation);

        // The field did not exist on legacy serialized records. Serde's
        // backward-compatible default decodes it as empty, but must not invent
        // provenance for historical transformations.
        let mut payload = serde_json::to_value(&evidence).unwrap();
        payload
            .as_object_mut()
            .unwrap()
            .remove("transformation_provenance")
            .unwrap();
        let decoded: PersistedEvidence = serde_json::from_value(payload.clone()).unwrap();

        assert_eq!(
            decoded.validate(),
            Err(EvidencePersistenceError::IntegrityFailure)
        );
        assert_eq!(
            ResourceWrite::new(
                ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap(),
                1,
                payload,
                ResourceWriteMode::Insert,
            ),
            Err(UnitOfWorkError::Persistence(
                PersistenceError::IntegrityFailure
            ))
        );
    }

    #[test]
    fn uow_read_rejects_legacy_transformation_without_provenance() {
        let transformation = EvidenceTransformation {
            transformation_id: "legacy-transform-read".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "legacy-ocr-service".into(),
            tool_id: Some("legacy-ocr-service".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        };
        let mut evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        evidence.transformations.push(transformation);
        let mut payload = serde_json::to_value(&evidence).unwrap();
        payload
            .as_object_mut()
            .unwrap()
            .remove("transformation_provenance")
            .unwrap();

        let resource_ref = ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap();
        let mut context = FakeContext {
            record: Some(ResourceRecord {
                resource_ref,
                schema_version: 1,
                revision: Revision::initial(),
                payload,
            }),
        };
        let mut repository = EvidenceTrustUnitOfWorkRepository::new(&mut context);

        assert_eq!(
            repository.get(&"evidence-1".into()),
            Err(PersistenceError::IntegrityFailure)
        );
    }

    #[test]
    fn generic_resource_write_rejects_evidence_without_provenance_binding() {
        let mut evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();
        evidence.transformations.push(EvidenceTransformation {
            transformation_id: "transform-unbound".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "ocr-service".into(),
            tool_id: Some("ocr-service".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        });

        let payload = serde_json::to_value(&evidence).unwrap();
        let result = ResourceWrite::new(
            ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap(),
            evidence.schema_version,
            payload,
            ResourceWriteMode::Insert,
        );

        assert_eq!(
            result,
            Err(UnitOfWorkError::Persistence(
                PersistenceError::IntegrityFailure
            ))
        );
    }

    #[test]
    fn generic_resource_write_rejects_evidence_id_binding_mismatch() {
        let evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();

        let result = ResourceWrite::new(
            ResourceRef::new(ResourceType::Evidence, "different-evidence").unwrap(),
            evidence.schema_version,
            serde_json::to_value(&evidence).unwrap(),
            ResourceWriteMode::Insert,
        );

        assert_eq!(
            result,
            Err(UnitOfWorkError::Persistence(
                PersistenceError::IntegrityFailure
            ))
        );
    }

    #[test]
    fn generic_resource_write_rejects_evidence_upsert() {
        let evidence =
            PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default()).unwrap();

        let result = ResourceWrite::new(
            ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap(),
            evidence.schema_version,
            serde_json::to_value(&evidence).unwrap(),
            ResourceWriteMode::Upsert,
        );

        assert_eq!(result, Err(UnitOfWorkError::InvalidOperation));
    }

    #[test]
    fn resource_write_revalidates_evidence_after_payload_mutation() {
        let transformation = EvidenceTransformation {
            transformation_id: "transform-mutate".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "ocr-service".into(),
            tool_id: Some("ocr-service".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        };
        let provenance = Provenance {
            provenance_id: "prov-mutate".into(),
            actor_ref: Some(ResourceRef::new(ResourceType::Service, "ocr-service").unwrap()),
            source_refs: Vec::new(),
            input_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
            operation: "ocr".into(),
            occurred_at: "2026-09-07T10:01:00Z".into(),
        };
        let evidence = PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default())
            .unwrap()
            .with_transformation(transformation, provenance)
            .unwrap();
        let mut write = ResourceWrite::new(
            ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap(),
            evidence.schema_version,
            serde_json::to_value(&evidence).unwrap(),
            ResourceWriteMode::Insert,
        )
        .unwrap();

        write.payload["transformation_provenance"][0]["provenance"]["actor_ref"]["id"] =
            serde_json::Value::String("forged-actor".into());

        assert_eq!(
            write.validate(),
            Err(UnitOfWorkError::Persistence(
                PersistenceError::IntegrityFailure
            ))
        );
    }

    #[test]
    fn service_actor_provenance_round_trips() {
        let transformation = EvidenceTransformation {
            transformation_id: "transform-service".into(),
            source_evidence_id: "evidence-1".into(),
            transformation_type: "ocr".into(),
            created_at: "2026-09-07T10:01:00Z".into(),
            created_by: "ocr-service".into(),
            tool_id: Some("ocr-engine".into()),
            tool_version: Some("1.0".into()),
            input_hash: None,
            output_hash: None,
        };
        let provenance = Provenance {
            provenance_id: "prov-service".into(),
            actor_ref: Some(ResourceRef::new(ResourceType::Service, "ocr-service").unwrap()),
            source_refs: Vec::new(),
            input_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
            operation: "ocr".into(),
            occurred_at: "2026-09-07T10:01:00Z".into(),
        };
        let evidence = PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default())
            .unwrap()
            .with_transformation(transformation, provenance)
            .unwrap();

        let mut repository = InMemoryEvidenceTrustRepository::default();
        repository.create(evidence.clone()).unwrap();
        assert_eq!(
            repository.get(&"evidence-1".into()).unwrap(),
            Some(evidence)
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
        let provenance = Provenance {
            provenance_id: "prov-transform-1".into(),
            actor_ref: Some(ResourceRef::new(ResourceType::Tool, "system").unwrap()),
            source_refs: vec![],
            input_refs: vec![ResourceRef::new(ResourceType::Evidence, "evidence-1").unwrap()],
            operation: "ocr".into(),
            occurred_at: "2026-09-07T10:01:00Z".into(),
        };
        let evidence = PersistedEvidence::new(1, original(), EvidenceTrustMetadata::default())
            .unwrap()
            .with_transformation(transformation.clone(), provenance)
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
