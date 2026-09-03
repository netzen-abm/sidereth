use serde::{Deserialize, Serialize};

use crate::{Case, EventEnvelope, Id, Incident};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Revision {
    pub value: u64,
}

impl Revision {
    pub fn initial() -> Self {
        Self { value: 0 }
    }

    pub fn next(&self) -> Result<Self, &'static str> {
        self.value
            .checked_add(1)
            .map(|value| Self { value })
            .ok_or("revision overflow")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Persisted<T> {
    pub schema_version: u16,
    pub revision: Revision,
    pub value: T,
}

impl<T> Persisted<T> {
    pub fn new(schema_version: u16, value: T) -> Result<Self, &'static str> {
        if schema_version == 0 {
            return Err("schema version is required");
        }
        Ok(Self {
            schema_version,
            revision: Revision::initial(),
            value,
        })
    }
}

pub trait CaseStore {
    fn get_case(&self, id: &Id) -> Result<Option<Persisted<Case>>, &'static str>;
    fn create_case(&mut self, value: Persisted<Case>) -> Result<(), &'static str>;
    fn update_case(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Case,
    ) -> Result<Revision, &'static str>;
}

pub trait IncidentStore {
    fn get_incident(&self, id: &Id) -> Result<Option<Persisted<Incident>>, &'static str>;
    fn create_incident(&mut self, value: Persisted<Incident>) -> Result<(), &'static str>;
    fn update_incident(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Incident,
    ) -> Result<Revision, &'static str>;
}

pub trait EventStore {
    fn get_event(&self, id: &Id) -> Result<Option<Persisted<EventEnvelope>>, &'static str>;
    fn append_event(&mut self, value: Persisted<EventEnvelope>) -> Result<(), &'static str>;
}

pub trait Transaction {
    fn commit(self) -> Result<(), &'static str>;
    fn rollback(self) -> Result<(), &'static str>;
}

pub trait TransactionFactory {
    type Tx: Transaction;

    fn begin(&mut self) -> Result<Self::Tx, &'static str>;
}

pub trait IdempotencyStore {
    fn lookup(&self, operation_id: &Id) -> Result<bool, &'static str>;
    fn record(&mut self, operation_id: Id) -> Result<(), &'static str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_is_deterministic() {
        let revision = Revision::initial();
        assert_eq!(revision.next().unwrap().value, 1);
    }

    #[test]
    fn zero_schema_version_is_rejected() {
        let case = Case::new("case-1".into()).unwrap();
        assert_eq!(Persisted::new(0, case), Err("schema version is required"));
    }
}
