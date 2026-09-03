use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionState {
    Open,
    Resolved,
    PartiallyResolved,
    Unresolved,
    Closed,
    Reopened,
}

impl ResolutionState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::Resolved)
                | (Self::Open, Self::PartiallyResolved)
                | (Self::Open, Self::Unresolved)
                | (Self::Resolved, Self::Reopened)
                | (Self::PartiallyResolved, Self::Reopened)
                | (Self::Unresolved, Self::Reopened)
                | (Self::Resolved, Self::Closed)
                | (Self::PartiallyResolved, Self::Closed)
                | (Self::Unresolved, Self::Closed)
                | (Self::Reopened, Self::Resolved)
                | (Self::Reopened, Self::PartiallyResolved)
                | (Self::Reopened, Self::Unresolved)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resolution {
    pub resolution_id: Id,
    pub case_id: Id,
    pub remedy_id: Option<Id>,
    pub escalation_id: Option<Id>,
    pub response_id: Option<Id>,
    pub requested_outcome: String,
    pub recorded_outcome: Option<String>,
    pub state: ResolutionState,
    pub source_refs: Vec<Id>,
    pub evidence_refs: Vec<Id>,
}

impl Resolution {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.resolution_id.is_empty() {
            return Err("resolution id is required");
        }
        if self.case_id.is_empty() {
            return Err("resolution case is required");
        }
        if self.requested_outcome.is_empty() {
            return Err("requested outcome is required");
        }
        if self.source_refs.is_empty() {
            return Err("resolution source references are required");
        }
        if matches!(
            self.state,
            ResolutionState::Resolved
                | ResolutionState::PartiallyResolved
                | ResolutionState::Closed
        ) && self.recorded_outcome.is_none()
        {
            return Err("recorded outcome is required for completed resolution");
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ResolutionRegistry {
    resolutions: HashMap<Id, Resolution>,
}

impl ResolutionRegistry {
    pub fn insert(&mut self, resolution: Resolution) -> Result<(), &'static str> {
        resolution.validate()?;
        if self.resolutions.contains_key(&resolution.resolution_id) {
            return Err("duplicate resolution id");
        }
        self.resolutions
            .insert(resolution.resolution_id.clone(), resolution);
        Ok(())
    }

    pub fn transition(
        &mut self,
        resolution_id: &Id,
        next: ResolutionState,
    ) -> Result<(), &'static str> {
        let resolution = self
            .resolutions
            .get_mut(resolution_id)
            .ok_or("resolution not found")?;
        if !resolution.state.can_transition_to(&next) {
            return Err("invalid resolution state transition");
        }
        if matches!(
            next,
            ResolutionState::Resolved
                | ResolutionState::PartiallyResolved
                | ResolutionState::Closed
        ) && resolution.recorded_outcome.is_none()
        {
            return Err("recorded outcome is required for completed resolution");
        }
        resolution.state = next;
        Ok(())
    }

    pub fn get(&self, id: &Id) -> Option<&Resolution> {
        self.resolutions.get(id)
    }

    pub fn ids(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = self.resolutions.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolution() -> Resolution {
        Resolution {
            resolution_id: "res-1".into(),
            case_id: "case-1".into(),
            remedy_id: Some("rem-1".into()),
            escalation_id: Some("esc-1".into()),
            response_id: Some("resp-1".into()),
            requested_outcome: "Review the decision".into(),
            recorded_outcome: None,
            state: ResolutionState::Open,
            source_refs: vec!["src-1".into()],
            evidence_refs: vec!["ev-1".into()],
        }
    }

    #[test]
    fn requested_and_recorded_outcomes_are_distinct() {
        let mut value = resolution();
        value.recorded_outcome = Some("Decision reviewed".into());
        let mut registry = ResolutionRegistry::default();
        registry.insert(value).unwrap();
        registry
            .transition(&"res-1".into(), ResolutionState::Resolved)
            .unwrap();
        assert_eq!(
            registry
                .get(&"res-1".into())
                .unwrap()
                .recorded_outcome
                .as_deref(),
            Some("Decision reviewed")
        );
    }

    #[test]
    fn completed_resolution_requires_recorded_outcome() {
        let mut registry = ResolutionRegistry::default();
        registry.insert(resolution()).unwrap();
        assert_eq!(
            registry.transition(&"res-1".into(), ResolutionState::Resolved),
            Err("recorded outcome is required for completed resolution")
        );
    }

    #[test]
    fn invalid_resolution_transition_is_rejected() {
        let mut registry = ResolutionRegistry::default();
        registry.insert(resolution()).unwrap();
        assert_eq!(
            registry.transition(&"res-1".into(), ResolutionState::Closed),
            Err("invalid resolution state transition")
        );
    }

    #[test]
    fn duplicate_resolution_ids_are_rejected() {
        let mut registry = ResolutionRegistry::default();
        registry.insert(resolution()).unwrap();
        assert_eq!(
            registry.insert(resolution()),
            Err("duplicate resolution id")
        );
    }
}
