use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceState {
    Unknown,
    NotApplicable,
    Required,
    InProgress,
    Satisfied,
    Breached,
    Disputed,
    ReviewRequired,
}

impl ComplianceState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Unknown, Self::NotApplicable)
                | (Self::Unknown, Self::Required)
                | (Self::Unknown, Self::ReviewRequired)
                | (Self::Required, Self::InProgress)
                | (Self::Required, Self::Satisfied)
                | (Self::Required, Self::Disputed)
                | (Self::Required, Self::ReviewRequired)
                | (Self::InProgress, Self::Satisfied)
                | (Self::InProgress, Self::Breached)
                | (Self::InProgress, Self::Disputed)
                | (Self::InProgress, Self::ReviewRequired)
                | (Self::Satisfied, Self::Disputed)
                | (Self::Satisfied, Self::ReviewRequired)
                | (Self::Breached, Self::InProgress)
                | (Self::Breached, Self::Disputed)
                | (Self::Breached, Self::ReviewRequired)
                | (Self::Disputed, Self::InProgress)
                | (Self::Disputed, Self::Satisfied)
                | (Self::Disputed, Self::Breached)
                | (Self::Disputed, Self::ReviewRequired)
                | (Self::ReviewRequired, Self::InProgress)
                | (Self::ReviewRequired, Self::Satisfied)
                | (Self::ReviewRequired, Self::Breached)
                | (Self::ReviewRequired, Self::Disputed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplianceRequirement {
    pub requirement_id: Id,
    pub obligation_id: Id,
    pub description: String,
    pub state: ComplianceState,
    pub evidence_refs: Vec<Id>,
    pub source_refs: Vec<Id>,
}

impl ComplianceRequirement {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.requirement_id.is_empty() {
            return Err("requirement id is required");
        }
        if self.obligation_id.is_empty() {
            return Err("requirement obligation is required");
        }
        if self.description.is_empty() {
            return Err("requirement description is required");
        }
        if self.source_refs.is_empty() {
            return Err("requirement source references are required");
        }
        if matches!(self.state, ComplianceState::Satisfied) && self.evidence_refs.is_empty() {
            return Err("satisfied requirement evidence is required");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ComplianceRegistry {
    requirements: HashMap<Id, ComplianceRequirement>,
}

impl ComplianceRegistry {
    pub fn insert(&mut self, requirement: ComplianceRequirement) -> Result<(), &'static str> {
        requirement.validate()?;
        if self.requirements.contains_key(&requirement.requirement_id) {
            return Err("duplicate requirement id");
        }
        self.requirements
            .insert(requirement.requirement_id.clone(), requirement);
        Ok(())
    }

    pub fn transition(
        &mut self,
        requirement_id: &Id,
        next: ComplianceState,
        evidence_refs: Vec<Id>,
    ) -> Result<(), &'static str> {
        let requirement = self
            .requirements
            .get_mut(requirement_id)
            .ok_or("requirement not found")?;
        if !requirement.state.can_transition_to(&next) {
            return Err("invalid compliance state transition");
        }
        if matches!(next, ComplianceState::Satisfied) && evidence_refs.is_empty() {
            return Err("satisfied requirement evidence is required");
        }
        requirement.state = next;
        if !evidence_refs.is_empty() {
            requirement.evidence_refs = evidence_refs;
        }
        Ok(())
    }

    pub fn get(&self, id: &Id) -> Option<&ComplianceRequirement> {
        self.requirements.get(id)
    }

    pub fn ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.requirements.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn for_obligation(&self, obligation_id: &Id) -> Vec<&ComplianceRequirement> {
        let mut values: Vec<&ComplianceRequirement> = self
            .requirements
            .values()
            .filter(|item| &item.obligation_id == obligation_id)
            .collect();
        values.sort_by_key(|item| item.requirement_id.clone());
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement() -> ComplianceRequirement {
        ComplianceRequirement {
            requirement_id: "r-1".into(),
            obligation_id: "o-1".into(),
            description: "Submit the required document".into(),
            state: ComplianceState::Unknown,
            evidence_refs: vec![],
            source_refs: vec!["src-1".into()],
        }
    }

    #[test]
    fn valid_requirement_is_accepted() {
        assert!(requirement().validate().is_ok());
    }

    #[test]
    fn satisfied_requires_evidence() {
        let mut registry = ComplianceRegistry::default();
        registry.insert(requirement()).unwrap();
        registry
            .transition(&"r-1".into(), ComplianceState::Required, vec![])
            .unwrap();
        assert_eq!(
            registry.transition(&"r-1".into(), ComplianceState::Satisfied, vec![]),
            Err("satisfied requirement evidence is required")
        );
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut registry = ComplianceRegistry::default();
        registry.insert(requirement()).unwrap();
        assert_eq!(
            registry.insert(requirement()),
            Err("duplicate requirement id")
        );
    }

    #[test]
    fn state_transition_is_deterministic() {
        let mut registry = ComplianceRegistry::default();
        registry.insert(requirement()).unwrap();
        registry
            .transition(&"r-1".into(), ComplianceState::Required, vec![])
            .unwrap();
        registry
            .transition(
                &"r-1".into(),
                ComplianceState::Satisfied,
                vec!["ev-1".into()],
            )
            .unwrap();
        assert_eq!(
            registry.get(&"r-1".into()).unwrap().state,
            ComplianceState::Satisfied
        );
        assert_eq!(
            registry.get(&"r-1".into()).unwrap().evidence_refs,
            vec!["ev-1"]
        );
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut registry = ComplianceRegistry::default();
        registry.insert(requirement()).unwrap();
        assert_eq!(
            registry.transition(&"r-1".into(), ComplianceState::Breached, vec![]),
            Err("invalid compliance state transition")
        );
    }

    #[test]
    fn requirements_are_sorted() {
        let mut registry = ComplianceRegistry::default();
        let mut second = requirement();
        second.requirement_id = "r-2".into();
        registry.insert(second).unwrap();
        registry.insert(requirement()).unwrap();
        assert_eq!(registry.ids(), vec!["r-1", "r-2"]);
    }
}
