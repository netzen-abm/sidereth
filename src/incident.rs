use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IncidentState {
    Open,
    Recorded,
    UnderReview,
    Resolved,
    Closed,
}

impl IncidentState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::Recorded)
                | (Self::Recorded, Self::UnderReview)
                | (Self::UnderReview, Self::Resolved)
                | (Self::Resolved, Self::Closed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Incident {
    pub incident_id: Id,
    pub state: IncidentState,
}

impl Incident {
    pub fn new(incident_id: Id) -> Result<Self, &'static str> {
        if incident_id.is_empty() {
            return Err("incident id is required");
        }
        Ok(Self {
            incident_id,
            state: IncidentState::Open,
        })
    }

    pub fn transition(&mut self, next: IncidentState) -> Result<(), &'static str> {
        if !self.state.can_transition_to(&next) {
            return Err("invalid incident state transition");
        }
        self.state = next;
        Ok(())
    }
}
