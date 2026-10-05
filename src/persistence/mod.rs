mod error;
mod idempotency;
mod resource;
mod stores;
mod transaction;
mod unit_of_work;

pub use error::{PersistenceError, UnitOfWorkError};
pub use idempotency::{
    IdempotencyClaim, IdempotencyLifecycleStore, IdempotencyState, IdempotencyStore,
};
pub use resource::{
    Persisted, ResourceLink, ResourceLinkClass, ResourceRecord, ResourceWrite, ResourceWriteMode,
    Revision,
};
pub use stores::{CaseStore, EventStore, IncidentStore};
pub use transaction::{Transaction, TransactionFactory};
pub use unit_of_work::{UnitOfWork, UnitOfWorkContext, UnitOfWorkFactory};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn revision_is_deterministic() {
        assert_eq!(Revision::initial().next().unwrap().value, 1);
    }

    #[test]
    fn zero_schema_version_is_rejected() {
        let case = crate::Case::new("case-1".into()).unwrap();
        assert_eq!(
            Persisted::new(0, case),
            Err(PersistenceError::ValidationFailure)
        );
    }

    #[test]
    fn revision_overflow_is_a_conflict() {
        let revision = Revision { value: u64::MAX };
        assert_eq!(revision.next(), Err(PersistenceError::Conflict));
    }

    #[test]
    fn resource_write_requires_schema_version() {
        let resource = crate::ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        assert_eq!(
            ResourceWrite::new(resource, 0, Value::Null, ResourceWriteMode::Insert),
            Err(UnitOfWorkError::InvalidOperation)
        );
    }

    #[test]
    fn resource_write_mode_uses_canonical_snake_case_wire_values() {
        assert_eq!(
            serde_json::to_string(&ResourceWriteMode::Insert).unwrap(),
            "\"insert\""
        );
        assert_eq!(
            serde_json::to_string(&ResourceWriteMode::Upsert).unwrap(),
            "\"upsert\""
        );
    }

    #[test]
    fn resource_link_requires_relation() {
        let source = crate::ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        let target = crate::ResourceRef::new(crate::ResourceType::Document, "doc-1").unwrap();
        assert_eq!(
            ResourceLink::new(source, "   ", target),
            Err(UnitOfWorkError::InvalidOperation)
        );
    }

    #[test]
    fn resource_write_can_carry_expected_revision() {
        let resource = crate::ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        let write = ResourceWrite::new(resource, 1, Value::Null, ResourceWriteMode::Upsert)
            .unwrap()
            .with_expected_revision(Revision { value: 7 });
        assert_eq!(write.expected_revision, Some(Revision { value: 7 }));
    }

    #[test]
    fn resource_link_classes_are_explicit_and_legacy_is_not_strong() {
        let source = crate::ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        let target = crate::ResourceRef::new(crate::ResourceType::Event, "event-1").unwrap();
        let strong = ResourceLink::new_with_class(
            source.clone(),
            "has_event",
            target.clone(),
            ResourceLinkClass::Strong,
        )
        .unwrap();
        assert_eq!(strong.semantic_class(), Some(ResourceLinkClass::Strong));
        let legacy = ResourceLink::new(source, "has_event", target).unwrap();
        assert_eq!(legacy.semantic_class(), None);
    }

    #[test]
    fn resource_link_classes_round_trip() {
        let source = crate::ResourceRef::new(crate::ResourceType::Case, "case-1").unwrap();
        let target = crate::ResourceRef::new(crate::ResourceType::Event, "event-1").unwrap();
        for class in [
            ResourceLinkClass::Strong,
            ResourceLinkClass::Forward,
            ResourceLinkClass::External,
        ] {
            let link =
                ResourceLink::new_with_class(source.clone(), "rel", target.clone(), class).unwrap();
            let encoded = serde_json::to_string(&link).unwrap();
            let decoded: ResourceLink = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, link);
        }
    }

    #[test]
    fn legacy_resource_link_decodes_without_class() {
        let json = r#"{"source_ref":{"resource_type":"case","id":"case-1"},"relation":"has_event","target_ref":{"resource_type":"event","id":"event-1"}}"#;
        let link: ResourceLink = serde_json::from_str(json).unwrap();
        assert_eq!(link.semantic_class(), None);
    }
}
