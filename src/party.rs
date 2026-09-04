use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PartyKind {
    Person,
    Organization,
    GovernmentEntity,
    RoleActor,
    SystemActor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PartyStatus {
    Active,
    Inactive,
    Suspended,
    Unknown,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Party {
    pub party_id: Id,
    pub schema_version: u32,
    pub party_kind: PartyKind,
    pub status: PartyStatus,
    pub display_name: String,
    pub identity_refs: Vec<Id>,
    pub jurisdiction_refs: Vec<Id>,
    pub organization_ref: Option<Id>,
    pub provenance_ref: Option<Id>,
    pub privacy_classification: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Party {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.party_id.is_empty() {
            return Err("party id is required");
        }
        if self.schema_version == 0 {
            return Err("party schema version is required");
        }
        if self.display_name.is_empty() {
            return Err("party display name is required");
        }
        if self.privacy_classification.is_empty() {
            return Err("party privacy classification is required");
        }
        if self.created_at.is_empty() || self.updated_at.is_empty() {
            return Err("party timestamps are required");
        }
        if self.identity_refs.iter().any(Id::is_empty)
            || self.jurisdiction_refs.iter().any(Id::is_empty)
        {
            return Err("party references cannot be empty");
        }
        if self.organization_ref.as_ref().is_some_and(String::is_empty)
            || self.provenance_ref.as_ref().is_some_and(String::is_empty)
        {
            return Err("party reference cannot be empty");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyRelationship {
    pub relationship_id: Id,
    pub schema_version: u32,
    pub from_party_id: Id,
    pub to_party_id: Id,
    pub relationship_type: String,
    pub context_ref: Option<Id>,
    pub role: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub provenance_ref: Option<Id>,
    pub authorization_ref: Option<Id>,
    pub created_at: String,
}

impl PartyRelationship {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.relationship_id.is_empty() {
            return Err("relationship id is required");
        }
        if self.schema_version == 0 {
            return Err("relationship schema version is required");
        }
        if self.from_party_id.is_empty() || self.to_party_id.is_empty() {
            return Err("relationship parties are required");
        }
        if self.from_party_id == self.to_party_id {
            return Err("party relationship cannot self-reference");
        }
        if self.relationship_type.is_empty() || self.role.is_empty() {
            return Err("relationship type and role are required");
        }
        if self.valid_from.is_empty() || self.created_at.is_empty() {
            return Err("relationship timestamps are required");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PartyRegistry {
    parties: HashMap<Id, Party>,
    relationships: HashMap<Id, PartyRelationship>,
}

impl PartyRegistry {
    pub fn insert_party(&mut self, party: Party) -> Result<(), &'static str> {
        party.validate()?;
        if self.parties.contains_key(&party.party_id) {
            return Err("duplicate party id");
        }
        self.parties.insert(party.party_id.clone(), party);
        Ok(())
    }

    pub fn insert_relationship(
        &mut self,
        relationship: PartyRelationship,
    ) -> Result<(), &'static str> {
        relationship.validate()?;
        if !self.parties.contains_key(&relationship.from_party_id)
            || !self.parties.contains_key(&relationship.to_party_id)
        {
            return Err("relationship party not found");
        }
        if self
            .relationships
            .contains_key(&relationship.relationship_id)
        {
            return Err("duplicate relationship id");
        }
        self.relationships
            .insert(relationship.relationship_id.clone(), relationship);
        Ok(())
    }

    pub fn get_party(&self, id: &Id) -> Option<&Party> {
        self.parties.get(id)
    }

    pub fn get_relationship(&self, id: &Id) -> Option<&PartyRelationship> {
        self.relationships.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn party(id: &str) -> Party {
        Party {
            party_id: id.into(),
            schema_version: 1,
            party_kind: PartyKind::Person,
            status: PartyStatus::Active,
            display_name: "Test Person".into(),
            identity_refs: vec![],
            jurisdiction_refs: vec!["j-1".into()],
            organization_ref: None,
            provenance_ref: None,
            privacy_classification: "private".into(),
            created_at: "2026-09-04T00:00:00Z".into(),
            updated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    fn relationship() -> PartyRelationship {
        PartyRelationship {
            relationship_id: "rel-1".into(),
            schema_version: 1,
            from_party_id: "p-1".into(),
            to_party_id: "p-2".into(),
            relationship_type: "representation".into(),
            context_ref: Some("case-1".into()),
            role: "legal_representative".into(),
            valid_from: "2026-09-04T00:00:00Z".into(),
            valid_to: None,
            provenance_ref: None,
            authorization_ref: Some("authz-1".into()),
            created_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    #[test]
    fn valid_party_passes() {
        assert!(party("p-1").validate().is_ok());
    }

    #[test]
    fn missing_party_id_rejected() {
        let mut p = party("p-1");
        p.party_id = "".into();
        assert_eq!(p.validate(), Err("party id is required"));
    }

    #[test]
    fn missing_display_name_rejected() {
        let mut p = party("p-1");
        p.display_name.clear();
        assert_eq!(p.validate(), Err("party display name is required"));
    }

    #[test]
    fn relationship_requires_existing_parties() {
        let mut r = PartyRegistry::default();
        r.insert_party(party("p-1")).unwrap();
        assert_eq!(
            r.insert_relationship(relationship()),
            Err("relationship party not found")
        );
    }

    #[test]
    fn relationship_is_contextual_and_reusable() {
        let mut r = PartyRegistry::default();
        r.insert_party(party("p-1")).unwrap();
        r.insert_party(party("p-2")).unwrap();
        r.insert_relationship(relationship()).unwrap();
        assert!(r.get_relationship(&"rel-1".into()).is_some());
    }

    #[test]
    fn self_relationship_rejected() {
        let mut x = relationship();
        x.to_party_id = x.from_party_id.clone();
        assert_eq!(
            x.validate(),
            Err("party relationship cannot self-reference")
        );
    }

    #[test]
    fn duplicate_party_rejected() {
        let mut r = PartyRegistry::default();
        r.insert_party(party("p-1")).unwrap();
        assert_eq!(r.insert_party(party("p-1")), Err("duplicate party id"));
    }

    #[test]
    fn public_enum_wire_values_are_snake_case() {
        assert_eq!(
            serde_json::to_string(&PartyKind::GovernmentEntity).unwrap(),
            "\"government_entity\""
        );
        assert_eq!(
            serde_json::to_string(&PartyStatus::Active).unwrap(),
            "\"active\""
        );
    }
}
