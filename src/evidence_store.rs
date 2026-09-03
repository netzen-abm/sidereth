use std::collections::HashMap;

use crate::{DerivedArtifact, EvidenceOriginal, Id};

pub trait EvidenceObjectStore {
    fn put(&mut self, storage_ref: Id, content: Vec<u8>) -> Result<(), &'static str>;
    fn get(&self, storage_ref: &Id) -> Result<Option<Vec<u8>>, &'static str>;
}

pub trait EvidenceRepository {
    fn get_original(
        &self,
        evidence_id: &Id,
    ) -> Result<Option<EvidenceOriginal>, &'static str>;
    fn save_original(&mut self, evidence: EvidenceOriginal) -> Result<(), &'static str>;
    fn get_artifact(
        &self,
        artifact_id: &Id,
    ) -> Result<Option<DerivedArtifact>, &'static str>;
    fn save_artifact(&mut self, artifact: DerivedArtifact) -> Result<(), &'static str>;
}

#[derive(Debug, Default)]
pub struct InMemoryEvidenceVault {
    objects: HashMap<Id, Vec<u8>>,
    originals: HashMap<Id, EvidenceOriginal>,
    artifacts: HashMap<Id, DerivedArtifact>,
}

impl EvidenceObjectStore for InMemoryEvidenceVault {
    fn put(&mut self, storage_ref: Id, content: Vec<u8>) -> Result<(), &'static str> {
        if storage_ref.is_empty() {
            return Err("storage reference is required");
        }
        if self.objects.contains_key(&storage_ref) {
            return Err("storage object already exists");
        }
        self.objects.insert(storage_ref, content);
        Ok(())
    }

    fn get(&self, storage_ref: &Id) -> Result<Option<Vec<u8>>, &'static str> {
        Ok(self.objects.get(storage_ref).cloned())
    }
}

impl EvidenceRepository for InMemoryEvidenceVault {
    fn get_original(
        &self,
        evidence_id: &Id,
    ) -> Result<Option<EvidenceOriginal>, &'static str> {
        Ok(self.originals.get(evidence_id).cloned())
    }

    fn save_original(&mut self, evidence: EvidenceOriginal) -> Result<(), &'static str> {
        evidence.validate()?;
        if self.originals.contains_key(&evidence.evidence_id) {
            return Err("evidence original already exists");
        }
        self.originals
            .insert(evidence.evidence_id.clone(), evidence);
        Ok(())
    }

    fn get_artifact(
        &self,
        artifact_id: &Id,
    ) -> Result<Option<DerivedArtifact>, &'static str> {
        Ok(self.artifacts.get(artifact_id).cloned())
    }

    fn save_artifact(&mut self, artifact: DerivedArtifact) -> Result<(), &'static str> {
        artifact.validate()?;
        if self.artifacts.contains_key(&artifact.artifact_id) {
            return Err("derived artifact already exists");
        }
        self.artifacts.insert(artifact.artifact_id.clone(), artifact);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn original() -> EvidenceOriginal {
        EvidenceOriginal::from_capture(crate::EvidenceCapture {
            evidence_id: "evidence-1".into(),
            schema_version: 1,
            case_id: Some("case-1".into()),
            incident_id: None,
            captured_at: "2026-09-03T10:00:00Z".into(),
            captured_by: "user-1".into(),
            media_type: "text/plain".into(),
            storage_ref: "object-1".into(),
            content: b"original evidence",
        })
        .unwrap()
    }

    #[test]
    fn object_store_round_trip() {
        let mut vault = InMemoryEvidenceVault::default();
        EvidenceObjectStore::put(&mut vault, "object-1".into(), b"evidence".to_vec())
            .unwrap();
        let content = EvidenceObjectStore::get(&vault, &"object-1".into())
            .unwrap()
            .unwrap();
        assert_eq!(content, b"evidence");
    }

    #[test]
    fn object_store_rejects_duplicate_reference() {
        let mut vault = InMemoryEvidenceVault::default();
        EvidenceObjectStore::put(&mut vault, "object-1".into(), b"one".to_vec())
            .unwrap();
        assert_eq!(
            EvidenceObjectStore::put(&mut vault, "object-1".into(), b"two".to_vec()),
            Err("storage object already exists")
        );
    }

    #[test]
    fn original_can_be_saved_and_loaded() {
        let mut vault = InMemoryEvidenceVault::default();
        EvidenceRepository::save_original(&mut vault, original()).unwrap();
        let saved = EvidenceRepository::get_original(&vault, &"evidence-1".into())
            .unwrap()
            .unwrap();
        assert_eq!(saved.evidence_id, "evidence-1");
    }

    #[test]
    fn original_cannot_be_replaced() {
        let mut vault = InMemoryEvidenceVault::default();
        EvidenceRepository::save_original(&mut vault, original()).unwrap();
        assert_eq!(
            EvidenceRepository::save_original(&mut vault, original()),
            Err("evidence original already exists")
        );
    }
}
