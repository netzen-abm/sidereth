use super::error::PersistenceError;
use super::resource::Persisted;

pub trait CaseStore {
    fn get_case(&self, id: &Id) -> Result<Option<Persisted<Case>>, PersistenceError>;
    fn create_case(&mut self, value: Persisted<Case>) -> Result<(), PersistenceError>;
    fn update_case(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Case,
    ) -> Result<Revision, PersistenceError>;
}

pub trait IncidentStore {
    fn get_incident(&self, id: &Id) -> Result<Option<Persisted<Incident>>, PersistenceError>;
    fn create_incident(&mut self, value: Persisted<Incident>) -> Result<(), PersistenceError>;
    fn update_incident(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Incident,
    ) -> Result<Revision, PersistenceError>;
}

pub trait EventStore {
    fn get_event(&self, id: &Id) -> Result<Option<Persisted<EventEnvelope>>, PersistenceError>;
    fn append_event(&mut self, value: Persisted<EventEnvelope>) -> Result<(), PersistenceError>;
}
