pub type Id = String;

/// Explicit cross-primitive reference contract for ecosystem boundaries.
///
/// Existing domain structs retain `Id = String` for source compatibility.
/// New integrations should use this typed boundary instead of relying on an
/// implicit target type for an identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
