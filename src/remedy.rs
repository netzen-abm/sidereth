use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplicabilityStatus {
    Unverified,
    Verified,
    Uncertain,
    ReviewRequired,
}

impl ApplicabilityStatus {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Unverified, Self::Verified)
                | (Self::Unverified, Self::Uncertain)
                | (Self::Unverified, Self::ReviewRequired)
                | (Self::Verified, Self::Uncertain)
                | (Self::Verified, Self::ReviewRequired)
                | (Self::Uncertain, Self::Verified)
                | (Self::Uncertain, Self::ReviewRequired)
                | (Self::ReviewRequired, Self::Verified)
                | (Self::ReviewRequired, Self::Uncertain)
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RemedyState {
    Candidate,
    Requested,
    UnderReview,
    Submitted,
    Granted,
    Denied,
    Withdrawn,
    Expired,
}

impl RemedyState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Candidate, Self::Requested)
                | (Self::Candidate, Self::UnderReview)
                | (Self::Candidate, Self::Withdrawn)
                | (Self::Requested, Self::UnderReview)
                | (Self::Requested, Self::Withdrawn)
                | (Self::UnderReview, Self::Submitted)
                | (Self::UnderReview, Self::Withdrawn)
                | (Self::Submitted, Self::Granted)
                | (Self::Submitted, Self::Denied)
                | (Self::Submitted, Self::Expired)
                | (Self::Granted, Self::Expired)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Remedy {
    pub remedy_id: Id,
    pub case_id: Id,
    pub escalation_id: Option<Id>,
    pub category: String,
    pub description: String,
    pub state: RemedyState,
    pub applicability: ApplicabilityStatus,
    pub source_refs: Vec<Id>,
    pub evidence_refs: Vec<Id>,
}

impl Remedy {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.remedy_id.is_empty() {
            return Err("remedy id is required");
        }
        if self.case_id.is_empty() {
            return Err("remedy case is required");
        }
        if self.category.is_empty() {
            return Err("remedy category is required");
        }
        if self.description.is_empty() {
            return Err("remedy description is required");
        }
        if self.source_refs.is_empty() {
            return Err("remedy source references are required");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct RemedyRegistry {
    remedies: HashMap<Id, Remedy>,
}

impl RemedyRegistry {
    pub fn insert(&mut self, remedy: Remedy) -> Result<(), &'static str> {
        remedy.validate()?;
        if self.remedies.contains_key(&remedy.remedy_id) {
            return Err("duplicate remedy id");
        }
        self.remedies.insert(remedy.remedy_id.clone(), remedy);
        Ok(())
    }

    pub fn transition(
        &mut self,
        remedy_id: &Id,
        next: RemedyState,
    ) -> Result<(), &'static str> {
        let remedy = self
            .remedies
            .get_mut(remedy_id)
            .ok_or("remedy not found")?;
        if !remedy.state.can_transition_to(&next) {
            return Err("invalid remedy state transition");
        }
        remedy.state = next;
        Ok(())
    }

    pub fn transition_applicability(
        &mut self,
        remedy_id: &Id,
        next: ApplicabilityStatus,
    ) -> Result<(), &'static str> {
        let remedy = self
            .remedies
            .get_mut(remedy_id)
            .ok_or("remedy not found")?;
        if !remedy.applicability.can_transition_to(&next) {
            return Err("invalid remedy applicability transition");
        }
        remedy.applicability = next;
        Ok(())
    }

    pub fn get(&self, id: &Id) -> Option<&Remedy> {
        self.remedies.get(id)
    }

    pub fn ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.remedies.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remedy() -> Remedy {
        Remedy {
            remedy_id: "rem-1".into(),
            case_id: "case-1".into(),
            escalation_id: Some("esc-1".into()),
            category: "review".into(),
            description: "Review the disputed decision".into(),
            state: RemedyState::Candidate,
            applicability: ApplicabilityStatus::Unverified,
            source_refs: vec!["src-1".into()],
            evidence_refs: vec!["ev-1".into()],
        }
    }

    #[test]
    fn remedy_lifecycle_is_deterministic() {
        let mut registry = RemedyRegistry::default();
        registry.insert(remedy()).unwrap();
        registry
            .transition(&"rem-1".into(), RemedyState::Requested)
            .unwrap();
        registry
            .transition(&"rem-1".into(), RemedyState::UnderReview)
            .unwrap();
        assert_eq!(
            registry.get(&"rem-1".into()).unwrap().state,
            RemedyState::UnderReview
        );
    }

    #[test]
    fn applicability_requires_explicit_transition() {
        let mut registry = RemedyRegistry::default();
        registry.insert(remedy()).unwrap();
        registry
            .transition_applicability(&"rem-1".into(), ApplicabilityStatus::Verified)
            .unwrap();
        assert_eq!(
            registry.get(&"rem-1".into()).unwrap().applicability,
            ApplicabilityStatus::Verified
        );
    }

    #[test]
    fn invalid_remedy_transition_is_rejected() {
        let mut registry = RemedyRegistry::default();
        registry.insert(remedy()).unwrap();
        assert_eq!(
            registry.transition(&"rem-1".into(), RemedyState::Granted),
            Err("invalid remedy state transition")
        );
    }

    #[test]
    fn duplicate_remedy_ids_are_rejected() {
        let mut registry = RemedyRegistry::default();
        registry.insert(remedy()).unwrap();
        assert_eq!(
            registry.insert(remedy()),
            Err("duplicate remedy id")
        );
    }
}
