use sidereth_core::{ResourceRef, ResourceType};

    #[test]
    fn resource_ref_is_explicit_and_stable_on_wire() {
        let reference = ResourceRef::new(ResourceType::Document, "doc-1").unwrap();
        let json = serde_json::to_string(&reference).unwrap();
        let expected = "{\"resource_type\":\"document\",\"id\":\"doc-1\"}".to_string();
        assert_eq!(json, expected);
        let decoded: ResourceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, reference);
    }

    #[test]
    fn empty_resource_ref_is_rejected() {
        assert_eq!(
            ResourceRef::new(ResourceType::Case, ""),
            Err("resource reference id is required")
        );
    }

    #[test]
    fn party_relationship_resource_type_has_canonical_wire_value() {
        assert_eq!(
            serde_json::to_string(&ResourceType::PartyRelationship).unwrap(),
            "\"party_relationship\""
        );
    }

    #[test]
    fn observation_resource_type_has_canonical_wire_value() {
        assert_eq!(
            serde_json::to_string(&ResourceType::Observation).unwrap(),
            "\"observation\""
        );
    }
