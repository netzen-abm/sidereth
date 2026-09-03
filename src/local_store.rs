use std::fs;
use std::path::{Path, PathBuf};

use crate::persistence::{
    CaseStore, EventStore, IdempotencyStore, IncidentStore, Persisted, Revision,
};
use crate::{Case, EventEnvelope, Id, Incident};

const SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct LocalFileStore {
    root: PathBuf,
}

impl LocalFileStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, &'static str> {
        let root = root.into();
        fs::create_dir_all(root.join("cases")).map_err(|_| "failed to create case store")?;
        fs::create_dir_all(root.join("incidents"))
            .map_err(|_| "failed to create incident store")?;
        fs::create_dir_all(root.join("events")).map_err(|_| "failed to create event store")?;
        fs::create_dir_all(root.join("idempotency"))
            .map_err(|_| "failed to create idempotency store")?;
        Ok(Self { root })
    }

    fn path(&self, kind: &str, id: &Id) -> Result<PathBuf, &'static str> {
        validate_id(id)?;
        Ok(self.root.join(kind).join(format!("{id}.json")))
    }

    fn write<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), &'static str> {
        let bytes = serde_json::to_vec(value).map_err(|_| "serialization failed")?;
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, bytes).map_err(|_| "write failed")?;
        fs::rename(&temporary, path).map_err(|_| "atomic rename failed")
    }

    fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, &'static str> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path).map_err(|_| "read failed")?;
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| "stored record is invalid")
    }

    fn create<T: serde::Serialize>(
        &self,
        path: &Path,
        value: &Persisted<T>,
    ) -> Result<(), &'static str> {
        if path.exists() {
            return Err("record already exists");
        }
        Self::write(path, value)
    }

    fn update<T>(
        &self,
        path: &Path,
        expected: Revision,
        value: T,
    ) -> Result<Revision, &'static str>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let current: Persisted<T> = Self::read(path)?.ok_or("record not found")?;
        if current.schema_version == 0 {
            return Err("schema version is required");
        }
        if current.revision != expected {
            return Err("revision conflict");
        }
        let revision = current.revision.next()?;
        Self::write(
            path,
            &Persisted {
                schema_version: SCHEMA_VERSION,
                revision: revision.clone(),
                value,
            },
        )?;
        Ok(revision)
    }
}

fn validate_id(id: &str) -> Result<(), &'static str> {
    if id.is_empty() {
        return Err("id is required");
    }
    if id == "." || id == ".." || id.contains('/') || id.contains('\\') {
        return Err("invalid id");
    }
    Ok(())
}

impl CaseStore for LocalFileStore {
    fn get_case(&self, id: &Id) -> Result<Option<Persisted<Case>>, &'static str> {
        Self::read(&self.path("cases", id)?)
    }

    fn create_case(&mut self, value: Persisted<Case>) -> Result<(), &'static str> {
        validate_id(&value.value.case_id)?;
        if value.schema_version == 0 {
            return Err("schema version is required");
        }
        self.create(&self.path("cases", &value.value.case_id)?, &value)
    }

    fn update_case(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Case,
    ) -> Result<Revision, &'static str> {
        if id != &value.case_id {
            return Err("id does not match value");
        }
        self.update(&self.path("cases", id)?, expected_revision, value)
    }
}

impl IncidentStore for LocalFileStore {
    fn get_incident(&self, id: &Id) -> Result<Option<Persisted<Incident>>, &'static str> {
        Self::read(&self.path("incidents", id)?)
    }

    fn create_incident(&mut self, value: Persisted<Incident>) -> Result<(), &'static str> {
        validate_id(&value.value.incident_id)?;
        if value.schema_version == 0 {
            return Err("schema version is required");
        }
        self.create(&self.path("incidents", &value.value.incident_id)?, &value)
    }

    fn update_incident(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Incident,
    ) -> Result<Revision, &'static str> {
        if id != &value.incident_id {
            return Err("id does not match value");
        }
        self.update(&self.path("incidents", id)?, expected_revision, value)
    }
}

impl EventStore for LocalFileStore {
    fn get_event(&self, id: &Id) -> Result<Option<Persisted<EventEnvelope>>, &'static str> {
        Self::read(&self.path("events", id)?)
    }

    fn append_event(&mut self, value: Persisted<EventEnvelope>) -> Result<(), &'static str> {
        value.value.validate()?;
        validate_id(&value.value.event_id)?;
        if value.schema_version == 0 {
            return Err("schema version is required");
        }
        self.create(&self.path("events", &value.value.event_id)?, &value)
    }
}

impl IdempotencyStore for LocalFileStore {
    fn lookup(&self, operation_id: &Id) -> Result<bool, &'static str> {
        Ok(self.path("idempotency", operation_id)?.exists())
    }

    fn record(&mut self, operation_id: Id) -> Result<(), &'static str> {
        let path = self.path("idempotency", &operation_id)?;
        if path.exists() {
            return Ok(());
        }
        Self::write(&path, &operation_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sidereth-{name}-{stamp}"))
    }

    #[test]
    fn case_survives_restart_and_revision_is_checked() {
        let path = root("case");
        {
            let mut store = LocalFileStore::open(&path).unwrap();
            let case = Case::new("case-1".into()).unwrap();
            let persisted = Persisted::new(SCHEMA_VERSION, case).unwrap();
            store.create_case(persisted).unwrap();
        }
        {
            let mut store = LocalFileStore::open(&path).unwrap();
            let saved = store.get_case(&"case-1".into()).unwrap().unwrap();
            assert_eq!(saved.revision.value, 0);
            let mut case = saved.value;
            case.transition(crate::CaseState::Active).unwrap();
            let revision = store
                .update_case(&"case-1".into(), Revision::initial(), case)
                .unwrap();
            assert_eq!(revision.value, 1);
            assert_eq!(
                store.update_case(
                    &"case-1".into(),
                    Revision::initial(),
                    Case::new("case-1".into()).unwrap(),
                ),
                Err("revision conflict")
            );
        }
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn duplicate_event_is_rejected() {
        let path = root("event");
        let mut store = LocalFileStore::open(&path).unwrap();
        let event = EventEnvelope {
            event_id: "event-1".into(),
            event_type: "case.created".into(),
            aggregate_type: "case".into(),
            aggregate_id: "case-1".into(),
            occurred_at: "2026-09-03T00:00:00Z".into(),
            actor_type: "user".into(),
            actor_id: "user-1".into(),
            schema_version: 1,
            payload: serde_json::Value::Null,
            source_refs: vec![],
            correlation_id: "corr-1".into(),
            causation_id: None,
        };
        let persisted = Persisted::new(SCHEMA_VERSION, event.clone()).unwrap();
        store.append_event(persisted.clone()).unwrap();
        assert_eq!(store.append_event(persisted), Err("record already exists"));
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn malformed_record_is_rejected() {
        let path = root("corrupt");
        let mut store = LocalFileStore::open(&path).unwrap();
        let case = Case::new("case-1".into()).unwrap();
        let persisted = Persisted::new(SCHEMA_VERSION, case).unwrap();
        store.create_case(persisted).unwrap();
        fs::write(path.join("cases").join("case-1.json"), b"not-json").unwrap();
        assert_eq!(
            store.get_case(&"case-1".into()),
            Err("stored record is invalid")
        );
        let _ = fs::remove_dir_all(path);
    }
}
