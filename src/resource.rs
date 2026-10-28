use serde::{Deserialize, Serialize};

pub type Id = String;

/// Explicit cross-primitive reference contract for ecosystem boundaries.
///
/// Existing domain structs retain `Id = String` for source compatibility.
/// New integrations should use this typed boundary instead of relying on an
/// implicit target type for an identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceRef {
    pub resource_type: ResourceType,
    pub id: Id,
}

impl ResourceRef {
    pub fn new(resource_type: ResourceType, id: impl Into<Id>) -> Result<Self, &'static str> {
        let id = id.into();
        if id.is_empty() {
            return Err("resource reference id is required");
        }
        Ok(Self { resource_type, id })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Case,
    Incident,
    Event,
    Observation,
    Evidence,
    Authority,
    Jurisdiction,
    Party,
    Tool,
    Service,
    PartyRelationship,
    Document,
    Action,
    Deadline,
    Response,
    Escalation,
    Remedy,
    Resolution,
    Procedure,
    ComplianceRequirement,
    LegalSource,
    Timeline,
    Audit,
    Provenance,
    Idempotency,
    Other,
}

#[cfg(test)]
mod schema_contract_tests {
    use super::{ResourceRef, ResourceType};

    const RESOURCE_REF_SCHEMA: &str = include_str!("../schemas/resource-ref.schema.json");

    // Keep Other as the final enum variant. Deriving the array size from its
    // discriminant makes adding a variant before Other require this list to
    // be updated before the code can compile.
    fn canonical_resource_types() -> [ResourceType; ResourceType::Other as usize + 1] {
        [
            ResourceType::Case,
            ResourceType::Incident,
            ResourceType::Event,
            ResourceType::Observation,
            ResourceType::Evidence,
            ResourceType::Authority,
            ResourceType::Jurisdiction,
            ResourceType::Party,
            ResourceType::Tool,
            ResourceType::Service,
            ResourceType::PartyRelationship,
            ResourceType::Document,
            ResourceType::Action,
            ResourceType::Deadline,
            ResourceType::Response,
            ResourceType::Escalation,
            ResourceType::Remedy,
            ResourceType::Resolution,
            ResourceType::Procedure,
            ResourceType::ComplianceRequirement,
            ResourceType::LegalSource,
            ResourceType::Timeline,
            ResourceType::Audit,
            ResourceType::Provenance,
            ResourceType::Idempotency,
            ResourceType::Other,
        ]
    }

    #[test]
    fn every_rust_resource_type_matches_the_resource_ref_schema_enum() {
        let schema: serde_json::Value = serde_json::from_str(RESOURCE_REF_SCHEMA)
            .expect("ResourceRef schema must be valid JSON");
        let schema_values = schema["properties"]["resource_type"]["enum"]
            .as_array()
            .expect("resource_type must declare an enum")
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .expect("resource_type enum values must be strings")
                    .to_owned()
            })
            .collect::<Vec<_>>();

        let mut rust_values = canonical_resource_types()
            .into_iter()
            .map(|resource_type| {
                serde_json::to_value(resource_type)
                    .expect("ResourceType serialization must succeed")
                    .as_str()
                    .expect("ResourceType must serialize as a string")
                    .to_owned()
            })
            .collect::<Vec<_>>();

        let mut schema_values = schema_values;
        rust_values.sort();
        schema_values.sort();

        assert_eq!(
            rust_values, schema_values,
            "ResourceType Rust wire values and ResourceRef JSON Schema enum must match exactly"
        );
    }

    #[test]
    fn resource_ref_schema_declares_closed_enum_and_nonempty_id_constraint() {
        let schema: serde_json::Value = serde_json::from_str(RESOURCE_REF_SCHEMA)
            .expect("ResourceRef schema must be valid JSON");

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["properties"]["resource_type"]["type"], "string");
        assert_eq!(schema["properties"]["id"]["type"], "string");
        assert_eq!(schema["properties"]["id"]["minLength"], 1);

        let required = schema["required"]
            .as_array()
            .expect("ResourceRef schema must declare required fields");
        assert!(required.iter().any(|value| value == "resource_type"));
        assert!(required.iter().any(|value| value == "id"));

        let allowed = schema["properties"]["resource_type"]["enum"]
            .as_array()
            .expect("resource_type must declare a closed enum");
        assert!(
            !allowed
                .iter()
                .any(|value| value == "unsupported_future_type"),
            "closed ResourceType contract must not list unknown wire values"
        );
    }

    #[test]
    fn resource_ref_serialization_uses_canonical_wire_names() {
        let reference = ResourceRef::new(ResourceType::PartyRelationship, "relationship-1")
            .expect("nonempty ResourceRef should be valid");
        let serialized =
            serde_json::to_value(reference).expect("ResourceRef serialization must succeed");

        assert_eq!(serialized["resource_type"], "party_relationship");
        assert_eq!(serialized["id"], "relationship-1");
    }
}
