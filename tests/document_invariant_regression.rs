use sidereth_core::{Document, DocumentRegistry, DocumentStatus, DocumentVersion, IntegrityStatus};

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
fn failed_version_transition_does_not_mutate_registry() {
    let mut registry = DocumentRegistry::default();
    registry
        .insert_document_with_initial_version(document(), version(1, "dv-1", None))
        .unwrap();
    registry
        .insert_version(version(2, "dv-2", Some("dv-1")))
        .unwrap();

    assert_eq!(
        registry.insert_version(version(3, "dv-3", Some("dv-1"))),
        Err("new version must supersede the document current version")
    );
    assert_eq!(
        registry.get_document(&"doc-1".into()).unwrap().current_version_id,
        "dv-2"
    );
    assert!(registry.get_version(&"dv-3".into()).is_none());
    assert!(registry.validate_integrity().is_ok());
}
