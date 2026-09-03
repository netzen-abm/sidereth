use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorityType {
    GovernmentBody,
    StatutoryOffice,
    Court,
    Tribunal,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorityStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Authority {
    pub authority_id: Id,
    pub name: String,
    pub authority_type: AuthorityType,
    pub jurisdiction_id: Id,
    pub status: AuthorityStatus,
}

impl Authority {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.authority_id.is_empty() {
            return Err("authority id is required");
        }
        if self.name.is_empty() {
            return Err("authority name is required");
        }
        if self.jurisdiction_id.is_empty() {
            return Err("authority jurisdiction is required");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityPower {
    pub power_id: Id,
    pub authority_id: Id,
    pub jurisdiction_id: Id,
    pub name: String,
    pub source_refs: Vec<Id>,
    pub status: AuthorityStatus,
}

impl AuthorityPower {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.power_id.is_empty() {
            return Err("power id is required");
        }
        if self.authority_id.is_empty() {
            return Err("power authority is required");
        }
        if self.jurisdiction_id.is_empty() {
            return Err("power jurisdiction is required");
        }
        if self.name.is_empty() {
            return Err("power name is required");
        }
        if self.source_refs.is_empty() {
            return Err("power requires a source reference");
        }
        if self.source_refs.iter().any(Id::is_empty) {
            return Err("power source reference cannot be empty");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AuthorityRegistry {
    authorities: HashMap<Id, Authority>,
    powers: HashMap<Id, AuthorityPower>,
}

impl AuthorityRegistry {
    pub fn insert_authority(&mut self, authority: Authority) -> Result<(), &'static str> {
        authority.validate()?;
        if self.authorities.contains_key(&authority.authority_id) {
            return Err("duplicate authority id");
        }
        self.authorities
            .insert(authority.authority_id.clone(), authority);
        Ok(())
    }

    pub fn insert_power(&mut self, power: AuthorityPower) -> Result<(), &'static str> {
        power.validate()?;
        if !self.authorities.contains_key(&power.authority_id) {
            return Err("power authority not found");
        }
        let authority = self.authorities.get(&power.authority_id).unwrap();
        if authority.jurisdiction_id != power.jurisdiction_id {
            return Err("power jurisdiction does not match authority");
        }
        if self.powers.contains_key(&power.power_id) {
            return Err("duplicate power id");
        }
        self.powers.insert(power.power_id.clone(), power);
        Ok(())
    }

    pub fn get_authority(&self, id: &Id) -> Option<&Authority> {
        self.authorities.get(id)
    }

    pub fn get_power(&self, id: &Id) -> Option<&AuthorityPower> {
        self.powers.get(id)
    }

    pub fn authority_ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.authorities.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn power_ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.powers.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authority() -> Authority {
        Authority {
            authority_id: "auth-1".into(),
            name: "Example Authority".into(),
            authority_type: AuthorityType::GovernmentBody,
            jurisdiction_id: "j-1".into(),
            status: AuthorityStatus::Active,
        }
    }

    fn power() -> AuthorityPower {
        AuthorityPower {
            power_id: "power-1".into(),
            authority_id: "auth-1".into(),
            jurisdiction_id: "j-1".into(),
            name: "Example power".into(),
            source_refs: vec!["law-1".into()],
            status: AuthorityStatus::Active,
        }
    }

    #[test]
    fn valid_authority_passes() {
        assert!(authority().validate().is_ok());
    }

    #[test]
    fn missing_jurisdiction_is_rejected() {
        let mut value = authority();
        value.jurisdiction_id.clear();
        assert_eq!(
            value.validate(),
            Err("authority jurisdiction is required")
        );
    }

    #[test]
    fn power_requires_authority() {
        let mut value = power();
        value.authority_id.clear();
        assert_eq!(value.validate(), Err("power authority is required"));
    }

    #[test]
    fn power_requires_source() {
        let mut value = power();
        value.source_refs.clear();
        assert_eq!(
            value.validate(),
            Err("power requires a source reference")
        );
    }

    #[test]
    fn authority_jurisdiction_mismatch_is_rejected() {
        let mut registry = AuthorityRegistry::default();
        registry.insert_authority(authority()).unwrap();
        let mut value = power();
        value.jurisdiction_id = "j-2".into();
        assert_eq!(
            registry.insert_power(value),
            Err("power jurisdiction does not match authority")
        );
    }

    #[test]
    fn unknown_authority_is_rejected() {
        let mut registry = AuthorityRegistry::default();
        assert_eq!(
            registry.insert_power(power()),
            Err("power authority not found")
        );
    }

    #[test]
    fn duplicate_authority_is_rejected() {
        let mut registry = AuthorityRegistry::default();
        registry.insert_authority(authority()).unwrap();
        assert_eq!(
            registry.insert_authority(authority()),
            Err("duplicate authority id")
        );
    }

    #[test]
    fn duplicate_power_is_rejected() {
        let mut registry = AuthorityRegistry::default();
        registry.insert_authority(authority()).unwrap();
        registry.insert_power(power()).unwrap();
        assert_eq!(
            registry.insert_power(power()),
            Err("duplicate power id")
        );
    }

    #[test]
    fn ids_are_stable() {
        let mut registry = AuthorityRegistry::default();
        let mut second = authority();
        second.authority_id = "auth-2".into();
        registry.insert_authority(second).unwrap();
        registry.insert_authority(authority()).unwrap();
        assert_eq!(registry.authority_ids(), vec!["auth-1", "auth-2"]);
    }
}
