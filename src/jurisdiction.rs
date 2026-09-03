use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JurisdictionType {
    Territorial,
    SubjectMatter,
    Personal,
    Institutional,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JurisdictionStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Jurisdiction {
    pub jurisdiction_id: Id,
    pub jurisdiction_type: JurisdictionType,
    pub name: String,
    pub parent_id: Option<Id>,
    pub status: JurisdictionStatus,
}

impl Jurisdiction {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.jurisdiction_id.is_empty() {
            return Err("jurisdiction id is required");
        }
        if self.name.is_empty() {
            return Err("jurisdiction name is required");
        }
        if self.parent_id.as_ref() == Some(&self.jurisdiction_id) {
            return Err("jurisdiction cannot parent itself");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct JurisdictionRegistry {
    jurisdictions: std::collections::HashMap<Id, Jurisdiction>,
}

impl JurisdictionRegistry {
    pub fn insert(&mut self, jurisdiction: Jurisdiction) -> Result<(), &'static str> {
        jurisdiction.validate()?;
        if self
            .jurisdictions
            .contains_key(&jurisdiction.jurisdiction_id)
        {
            return Err("duplicate jurisdiction id");
        }
        self.jurisdictions
            .insert(jurisdiction.jurisdiction_id.clone(), jurisdiction);
        if let Err(error) = self.validate_hierarchy() {
            self.jurisdictions.remove(
                self.jurisdictions
                    .keys()
                    .find(|id| !self.has_unique_id(id))
                    .cloned()
                    .unwrap_or_default()
                    .as_str(),
            );
            return Err(error);
        }
        Ok(())
    }

    pub fn get(&self, id: &Id) -> Option<&Jurisdiction> {
        self.jurisdictions.get(id)
    }

    pub fn ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.jurisdictions.keys().cloned().collect();
        ids.sort();
        ids
    }

    fn has_unique_id(&self, _id: &Id) -> bool {
        true
    }

    fn validate_hierarchy(&self) -> Result<(), &'static str> {
        for jurisdiction in self.jurisdictions.values() {
            let mut current = jurisdiction.jurisdiction_id.clone();
            let mut visited = std::collections::HashSet::new();
            while let Some(parent) = self
                .jurisdictions
                .get(&current)
                .and_then(|value| value.parent_id.clone())
            {
                if !visited.insert(current.clone()) {
                    return Err("jurisdiction hierarchy cycle detected");
                }
                current = parent;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jurisdiction(id: &str) -> Jurisdiction {
        Jurisdiction {
            jurisdiction_id: id.into(),
            jurisdiction_type: JurisdictionType::Territorial,
            name: format!("Jurisdiction {id}"),
            parent_id: None,
            status: JurisdictionStatus::Active,
        }
    }

    #[test]
    fn valid_jurisdiction_passes() {
        assert!(jurisdiction("j-1").validate().is_ok());
    }

    #[test]
    fn empty_id_is_rejected() {
        let mut value = jurisdiction("j-1");
        value.jurisdiction_id.clear();
        assert_eq!(value.validate(), Err("jurisdiction id is required"));
    }

    #[test]
    fn self_parent_is_rejected() {
        let mut value = jurisdiction("j-1");
        value.parent_id = Some("j-1".into());
        assert_eq!(value.validate(), Err("jurisdiction cannot parent itself"));
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut registry = JurisdictionRegistry::default();
        registry.insert(jurisdiction("j-1")).unwrap();
        assert_eq!(registry.insert(jurisdiction("j-1")), Err("duplicate jurisdiction id"));
    }

    #[test]
    fn ids_are_stable() {
        let mut registry = JurisdictionRegistry::default();
        registry.insert(jurisdiction("j-2")).unwrap();
        registry.insert(jurisdiction("j-1")).unwrap();
        assert_eq!(registry.ids(), vec!["j-1", "j-2"]);
    }
}
