use crate::{Id, ResourceRef};
use serde::{Deserialize, Serialize};

/// First-class provenance for material facts, transformations, and decisions.
///
/// Provenance identifies who or what produced a resource, what source or
/// inputs support it, when it was produced, and what operation produced it.
/// It does not itself establish legal authority or authorization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub provenance_id: Id,
    pub actor_ref: Option<ResourceRef>,
    pub source_refs: Vec<ResourceRef>,
    pub input_refs: Vec<ResourceRef>,
    pub operation: String,
    pub occurred_at: String,
}

impl Provenance {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.provenance_id.is_empty() {
            return Err("provenance id is required");
        }
        if self.operation.is_empty() {
            return Err("provenance operation is required");
        }
        if self.occurred_at.is_empty() {
            return Err("provenance occurred_at is required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceRef {
    pub provenance_id: Id,
}

impl ProvenanceRef {
    pub fn new(provenance_id: impl Into<Id>) -> Result<Self, &'static str> {
        let provenance_id = provenance_id.into();
        if provenance_id.is_empty() {
            return Err("provenance reference id is required");
        }
        Ok(Self { provenance_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(resource_type: crate::ResourceType, id: &str) -> ResourceRef {
        ResourceRef::new(resource_type, id).unwrap()
    }

    #[test]
    fn provenance_captures_actor_sources_inputs_and_operation() {
        let provenance = Provenance {
            provenance_id: "prov-1".into(),
            actor_ref: Some(reference(crate::ResourceType::Party, "party-1")),
            source_refs: vec![reference(crate::ResourceType::LegalSource, "source-1")],
            input_refs: vec![reference(crate::ResourceType::Document, "doc-1")],
            operation: "document.extraction".into(),
            occurred_at: "2026-09-04T00:00:00Z".into(),
        };
        assert!(provenance.validate().is_ok());
        let json = serde_json::to_string(&provenance).unwrap();
        let decoded: Provenance = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, provenance);
    }

    #[test]
    fn provenance_validation_rejects_missing_identity() {
        let provenance = Provenance {
            provenance_id: String::new(),
            actor_ref: None,
            source_refs: Vec::new(),
            input_refs: Vec::new(),
            operation: "test".into(),
            occurred_at: "2026-09-04T00:00:00Z".into(),
        };
        assert_eq!(provenance.validate(), Err("provenance id is required"));
    }

    #[test]
    fn provenance_reference_rejects_empty_id() {
        assert_eq!(
            ProvenanceRef::new(""),
            Err("provenance reference id is required")
        );
    }
}
