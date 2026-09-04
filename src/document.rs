use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentStatus {
    Draft,
    Active,
    Superseded,
    Revoked,
    Archived,
    Deleted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrityStatus {
    Verified,
    Unverified,
    Modified,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub document_id: Id,
    pub schema_version: u32,
    pub document_type: String,
    pub status: DocumentStatus,
    pub title: String,
    pub issuer_party_id: Option<Id>,
    pub recipient_party_refs: Vec<Id>,
    pub case_refs: Vec<Id>,
    pub incident_refs: Vec<Id>,
    pub jurisdiction_refs: Vec<Id>,
    pub authority_ref: Option<Id>,
    pub current_version_id: Id,
    pub provenance_ref: Option<Id>,
    pub privacy_classification: String,
    pub retention_policy_ref: Option<Id>,
    pub created_at: String,
    pub updated_at: String,
}

impl Document {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.document_id.is_empty() {
            return Err("document id is required");
        }
        if self.schema_version == 0 {
            return Err("document schema version is required");
        }
        if self.document_type.is_empty() {
            return Err("document type is required");
        }
        if self.title.is_empty() {
            return Err("document title is required");
        }
        if self.current_version_id.is_empty() {
            return Err("current document version is required");
        }
        if self.privacy_classification.is_empty() {
            return Err("document privacy classification is required");
        }
        if self.created_at.is_empty() || self.updated_at.is_empty() {
            return Err("document timestamps are required");
        }
        if self.recipient_party_refs.iter().any(Id::is_empty)
            || self.case_refs.iter().any(Id::is_empty)
            || self.incident_refs.iter().any(Id::is_empty)
            || self.jurisdiction_refs.iter().any(Id::is_empty)
        {
            return Err("document references cannot be empty");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentVersion {
    pub document_version_id: Id,
    pub document_id: Id,
    pub schema_version: u32,
    pub version_number: u32,
    pub media_type: String,
    pub content_ref: Id,
    pub content_hash: String,
    pub byte_length: Option<u64>,
    pub captured_at: Option<String>,
    pub created_by: Id,
    pub source_ref: Option<Id>,
    pub provenance_ref: Option<Id>,
    pub integrity_status: IntegrityStatus,
    pub supersedes_version_id: Option<Id>,
    pub language: Option<String>,
    pub created_at: String,
}

impl DocumentVersion {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.document_version_id.is_empty() || self.document_id.is_empty() {
            return Err("document version identity is required");
        }
        if self.schema_version == 0 || self.version_number == 0 {
            return Err("document version number and schema version are required");
        }
        if self.media_type.is_empty() || self.content_ref.is_empty() || self.content_hash.is_empty()
        {
            return Err("document version content metadata is required");
        }
        if self.created_by.is_empty() || self.created_at.is_empty() {
            return Err("document version creator and timestamp are required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivedArtifact {
    pub artifact_id: Id,
    pub source_document_version_id: Id,
    pub artifact_type: String,
    pub content_ref: Id,
    pub content_hash: Option<String>,
    pub created_at: String,
    pub created_by: Id,
    pub processing_provenance_ref: Option<Id>,
    pub model_ref: Option<Id>,
    pub confidence: Option<u8>,
}

impl DerivedArtifact {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.artifact_id.is_empty() || self.source_document_version_id.is_empty() {
            return Err("artifact identity and source are required");
        }
        if self.artifact_type.is_empty()
            || self.content_ref.is_empty()
            || self.created_by.is_empty()
            || self.created_at.is_empty()
        {
            return Err("artifact metadata is required");
        }
        if self.confidence.is_some_and(|v| v > 100) {
            return Err("artifact confidence must be between 0 and 100");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DocumentRegistry {
    documents: HashMap<Id, Document>,
    versions: HashMap<Id, DocumentVersion>,
    artifacts: HashMap<Id, DerivedArtifact>,
}

impl DocumentRegistry {
    pub fn insert_document(&mut self, document: Document) -> Result<(), &'static str> {
        document.validate()?;
        if self.documents.contains_key(&document.document_id) {
            return Err("duplicate document id");
        }
        self.documents
            .insert(document.document_id.clone(), document);
        Ok(())
    }

    pub fn insert_version(&mut self, version: DocumentVersion) -> Result<(), &'static str> {
        version.validate()?;
        if !self.documents.contains_key(&version.document_id) {
            return Err("document version document not found");
        }
        if self.versions.contains_key(&version.document_version_id) {
            return Err("duplicate document version id");
        }
        let doc = self
            .documents
            .get(&version.document_id)
            .expect("validated document exists");
        if version.version_number > 1 && version.supersedes_version_id.is_none() {
            return Err("non-initial document version must identify superseded version");
        }
        if version.version_number == 1 && version.supersedes_version_id.is_some() {
            return Err("initial document version cannot supersede another version");
        }
        if version.version_number > 1
            && !self
                .versions
                .contains_key(version.supersedes_version_id.as_ref().unwrap())
        {
            return Err("superseded document version not found");
        }
        if version.version_number == 1
            && doc.current_version_id != version.document_version_id
            && self
                .versions
                .values()
                .any(|v| v.document_id == version.document_id && v.version_number == 1)
        {
            return Err("document already has initial version");
        }
        self.versions
            .insert(version.document_version_id.clone(), version);
        Ok(())
    }

    pub fn insert_artifact(&mut self, artifact: DerivedArtifact) -> Result<(), &'static str> {
        artifact.validate()?;
        if !self
            .versions
            .contains_key(&artifact.source_document_version_id)
        {
            return Err("artifact source document version not found");
        }
        if self.artifacts.contains_key(&artifact.artifact_id) {
            return Err("duplicate artifact id");
        }
        self.artifacts.insert(artifact.artifact_id.clone(), artifact);
        Ok(())
    }

    pub fn get_document(&self, id: &Id) -> Option<&Document> {
        self.documents.get(id)
    }

    pub fn get_version(&self, id: &Id) -> Option<&DocumentVersion> {
        self.versions.get(id)
    }

    pub fn get_artifact(&self, id: &Id) -> Option<&DerivedArtifact> {
        self.artifacts.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Document {
        Document {
            document_id: "doc-1".into(),
            schema_version: 1,
            document_type: "notice".into(),
            status: DocumentStatus::Active,
            title: "Notice".into(),
            issuer_party_id: Some("p-1".into()),
            recipient_party_refs: vec!["p-2".into()],
            case_refs: vec!["case-1".into()],
            incident_refs: vec![],
            jurisdiction_refs: vec!["j-1".into()],
            authority_ref: Some("a-1".into()),
            current_version_id: "dv-1".into(),
            provenance_ref: None,
            privacy_classification: "sensitive".into(),
            retention_policy_ref: None,
            created_at: "2026-09-04T00:00:00Z".into(),
            updated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    fn version(n: u32, id: &str, supersedes: Option<&str>) -> DocumentVersion {
        DocumentVersion {
            document_version_id: id.into(),
            document_id: "doc-1".into(),
            schema_version: 1,
            version_number: n,
            media_type: "application/pdf".into(),
            content_ref: format!("blob-{id}"),
            content_hash: "sha256:abc".into(),
            byte_length: Some(10),
            captured_at: None,
            created_by: "p-2".into(),
            source_ref: None,
            provenance_ref: None,
            integrity_status: IntegrityStatus::Unverified,
            supersedes_version_id: supersedes.map(str::to_string),
            language: Some("en".into()),
            created_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    #[test]
    fn valid_document_passes() {
        assert!(document().validate().is_ok());
    }

    #[test]
    fn missing_document_id_rejected() {
        let mut d = document();
        d.document_id.clear();
        assert_eq!(d.validate(), Err("document id is required"));
    }

    #[test]
    fn version_requires_parent_document() {
        let mut r = DocumentRegistry::default();
        assert_eq!(
            r.insert_version(version(1, "dv-1", None)),
            Err("document version document not found")
        );
    }

    #[test]
    fn initial_version_can_be_registered() {
        let mut r = DocumentRegistry::default();
        r.insert_document(document()).unwrap();
        assert!(r.insert_version(version(1, "dv-1", None)).is_ok());
    }

    #[test]
    fn later_version_requires_superseded_version() {
        let mut r = DocumentRegistry::default();
        r.insert_document(document()).unwrap();
        r.insert_version(version(1, "dv-1", None)).unwrap();
        assert_eq!(
            r.insert_version(version(2, "dv-2", None)),
            Err("non-initial document version must identify superseded version")
        );
    }

    #[test]
    fn derived_artifact_requires_source_version() {
        let mut r = DocumentRegistry::default();
        r.insert_document(document()).unwrap();
        let a = DerivedArtifact {
            artifact_id: "art-1".into(),
            source_document_version_id: "missing".into(),
            artifact_type: "ocr".into(),
            content_ref: "blob-art".into(),
            content_hash: None,
            created_at: "2026-09-04T00:00:00Z".into(),
            created_by: "system".into(),
            processing_provenance_ref: None,
            model_ref: None,
            confidence: Some(95),
        };
        assert_eq!(
            r.insert_artifact(a),
            Err("artifact source document version not found")
        );
    }
}
