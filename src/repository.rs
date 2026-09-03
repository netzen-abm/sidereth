use std::collections::HashMap;

use crate::{Case, EventEnvelope, Id, Incident};

pub trait CaseRepository {
    fn get(&self, id: &Id) -> Result<Option<Case>, &'static str>;
    fn save(&mut self, case: Case) -> Result<(), &'static str>;
}

pub trait IncidentRepository {
    fn get(&self, id: &Id) -> Result<Option<Incident>, &'static str>;
    fn save(&mut self, incident: Incident) -> Result<(), &'static str>;
}

pub trait EventRepository {
    fn get(&self, id: &Id) -> Result<Option<EventEnvelope>, &'static str>;
    fn append(&mut self, event: EventEnvelope) -> Result<(), &'static str>;
}

#[derive(Debug, Default)]
pub struct InMemoryRepositories {
    cases: HashMap<Id, Case>,
    incidents: HashMap<Id, Incident>,
    events: HashMap<Id, EventEnvelope>,
}

impl CaseRepository for InMemoryRepositories {
    fn get(&self, id: &Id) -> Result<Option<Case>, &'static str> {
        Ok(self.cases.get(id).cloned())
    }

    fn save(&mut self, case: Case) -> Result<(), &'static str> {
        if case.id.is_empty() {
            return Err("case id is required");
        }
        if self.cases.contains_key(&case.id) {
            return Err("case already exists");
        }
        self.cases.insert(case.id.clone(), case);
        Ok(())
    }
}

impl IncidentRepository for InMemoryRepositories {
    fn get(&self, id: &Id) -> Result<Option<Incident>, &'static str> {
        Ok(self.incidents.get(id).cloned())
    }

    fn save(&mut self, incident: Incident) -> Result<(), &'static str> {
        if incident.id.is_empty() {
            return Err("incident id is required");
        }
        if self.incidents.contains_key(&incident.id) {
            return Err("incident already exists");
        }
        self.incidents.insert(incident.id.clone(), incident);
        Ok(())
    }
}

impl EventRepository for InMemoryRepositories {
    fn get(&self, id: &Id) -> Result<Option<EventEnvelope>, &'static str> {
        Ok(self.events.get(id).cloned())
    }

    fn append(&mut self, event: EventEnvelope) -> Result<(), &'static str> {
        event.validate()?;
        if self.events.contains_key(&event.event_id) {
            return Err("event already exists");
        }
        self.events.insert(event.event_id.clone(), event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case() -> Case {
        Case {
            id: "case-1".into(),
            title: "Test".into(),
            jurisdiction_id: None,
            authority_id: None,
            state: crate::CaseState::Draft,
        }
    }

    #[test]
    fn case_can_be_saved_and_loaded() {
        let mut repo = InMemoryRepositories::default();
        CaseRepository::save(&mut repo, case()).unwrap();
        let saved = CaseRepository::get(&repo, &"case-1".into())
            .unwrap()
            .unwrap();
        assert_eq!(saved.id, "case-1");
    }

    #[test]
    fn duplicate_case_is_rejected() {
        let mut repo = InMemoryRepositories::default();
        CaseRepository::save(&mut repo, case()).unwrap();
        assert_eq!(
            CaseRepository::save(&mut repo, case()),
            Err("case already exists")
        );
    }

    #[test]
    fn missing_case_returns_none() {
        let repo = InMemoryRepositories::default();
        assert!(CaseRepository::get(&repo, &"missing".into())
            .unwrap()
            .is_none());
    }
}
