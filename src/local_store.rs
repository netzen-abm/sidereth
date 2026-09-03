use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::persistence::{
    CaseStore, EventStore, IdempotencyClaim, IdempotencyStore, IncidentStore, Persisted,
    PersistenceError, Revision,
};
use crate::{Case, EventEnvelope, Id, Incident};

const SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct LocalFileStore {
    root: PathBuf,
}

impl LocalFileStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, PersistenceError> {
        let root = root.into();
        fs::create_dir_all(root.join("cases")).map_err(|_| PersistenceError::Unavailable)?;
        fs::create_dir_all(root.join("incidents")).map_err(|_| PersistenceError::Unavailable)?;
        fs::create_dir_all(root.join("events")).map_err(|_| PersistenceError::Unavailable)?;
        fs::create_dir_all(root.join("idempotency")).map_err(|_| PersistenceError::Unavailable)?;
        Ok(Self { root })
    }

    fn path(&self, kind: &str, id: &Id) -> Result<PathBuf, PersistenceError> {
        validate_id(id)?;
        Ok(self.root.join(kind).join(format!("{id}.json")))
    }

    fn temporary_path(path: &Path) -> Result<PathBuf, PersistenceError> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| PersistenceError::Unavailable)?
            .as_nanos();
        let pid = std::process::id();
        Ok(path.with_extension(format!("tmp-{pid}-{stamp}")))
    }

    fn write<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), PersistenceError> {
        let bytes =
            serde_json::to_vec(value).map_err(|_| PersistenceError::SerializationFailure)?;
        let temporary = Self::temporary_path(path)?;
        fs::write(&temporary, bytes).map_err(|_| PersistenceError::Unavailable)?;
        fs::rename(&temporary, path).map_err(|_| PersistenceError::Unavailable)
    }

    fn create<T: serde::Serialize>(
        &self,
        path: &Path,
        value: &Persisted<T>,
    ) -> Result<(), PersistenceError> {
        if path.exists() {
            return Err(PersistenceError::Duplicate);
        }
        let bytes =
            serde_json::to_vec(value).map_err(|_| PersistenceError::SerializationFailure)?;
        let temporary = Self::temporary_path(path)?;
        fs::write(&temporary, bytes).map_err(|_| PersistenceError::Unavailable)?;
        match fs::hard_link(&temporary, path) {
            Ok(()) => {
                let _ = fs::remove_file(&temporary);
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temporary);
                Err(PersistenceError::Duplicate)
            }
            Err(_) => {
                let _ = fs::remove_file(&temporary);
                Err(PersistenceError::Unavailable)
            }
        }
    }

    fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, PersistenceError> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path).map_err(|_| PersistenceError::Unavailable)?;
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| PersistenceError::IntegrityFailure)
    }

    fn validate_schema(version: u16) -> Result<(), PersistenceError> {
        if version != SCHEMA_VERSION {
            return Err(PersistenceError::UnsupportedSchemaVersion);
        }
        Ok(())
    }

    fn update<T>(
        &self,
        path: &Path,
        expected: Revision,
        value: T,
    ) -> Result<Revision, PersistenceError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let current: Persisted<T> = Self::read(path)?.ok_or(PersistenceError::NotFound)?;
        Self::validate_schema(current.schema_version)?;
        if current.revision != expected {
            return Err(PersistenceError::Conflict);
        }
        let revision = current.revision.next()?;
        Self::write(
            path,
            &Persisted {
                schema_version: SCHEMA_VERSION,
                revision,
                value,
            },
        )?;
        Ok(revision)
    }
}

fn validate_id(id: &str) -> Result<(), PersistenceError> {
    if id.is_empty() || id == "." || id == ".." || id.contains('/') || id.contains('\\') {
        return Err(PersistenceError::ValidationFailure);
    }
    Ok(())
}

impl CaseStore for LocalFileStore {
    fn get_case(&self, id: &Id) -> Result<Option<Persisted<Case>>, PersistenceError> {
        Self::read(&self.path("cases", id)?)
    }

    fn create_case(&mut self, value: Persisted<Case>) -> Result<(), PersistenceError> {
        validate_id(&value.value.case_id)?;
        Self::validate_schema(value.schema_version)?;
        self.create(&self.path("cases", &value.value.case_id)?, &value)
    }

    fn update_case(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Case,
    ) -> Result<Revision, PersistenceError> {
        if id != &value.case_id {
            return Err(PersistenceError::ValidationFailure);
        }
        self.update(&self.path("cases", id)?, expected_revision, value)
    }
}

impl IncidentStore for LocalFileStore {
    fn get_incident(&self, id: &Id) -> Result<Option<Persisted<Incident>>, PersistenceError> {
        Self::read(&self.path("incidents", id)?)
    }

    fn create_incident(&mut self, value: Persisted<Incident>) -> Result<(), PersistenceError> {
        validate_id(&value.value.incident_id)?;
        Self::validate_schema(value.schema_version)?;
        self.create(&self.path("incidents", &value.value.incident_id)?, &value)
    }

    fn update_incident(
        &mut self,
        id: &Id,
        expected_revision: Revision,
        value: Incident,
    ) -> Result<Revision, PersistenceError> {
        if id != &value.incident_id {
            return Err(PersistenceError::ValidationFailure);
        }
        self.update(&self.path("incidents", id)?, expected_revision, value)
    }
}

impl EventStore for LocalFileStore {
    fn get_event(&self, id: &Id) -> Result<Option<Persisted<EventEnvelope>>, PersistenceError> {
        Self::read(&self.path("events", id)?)
    }

    fn append_event(&mut self, value: Persisted<EventEnvelope>) -> Result<(), PersistenceError> {
        value
            .value
            .validate()
            .map_err(|_| PersistenceError::ValidationFailure)?;
        validate_id(&value.value.event_id)?;
        Self::validate_schema(value.schema_version)?;
        self.create(&self.path("events", &value.value.event_id)?, &value)
    }
}

impl IdempotencyStore for LocalFileStore {
    fn lookup(&self, operation_id: &Id) -> Result<bool, PersistenceError> {
        Ok(self.path("idempotency", operation_id)?.exists())
    }

    fn claim(&mut self, operation_id: Id) -> Result<IdempotencyClaim, PersistenceError> {
        let path = self.path("idempotency", &operation_id)?;
        let bytes = serde_json::to_vec(&operation_id)
            .map_err(|_| PersistenceError::SerializationFailure)?;
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Ok(IdempotencyClaim::AlreadyClaimed);
            }
            Err(_) => return Err(PersistenceError::Unavailable),
        };
        file.write_all(&bytes)
            .map_err(|_| PersistenceError::Unavailable)?;
        file.flush().map_err(|_| PersistenceError::Unavailable)?;
        Ok(IdempotencyClaim::Claimed)
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
                Err(PersistenceError::Conflict)
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
        let persisted = Persisted::new(SCHEMA_VERSION, event).unwrap();
        store.append_event(persisted.clone()).unwrap();
        assert_eq!(
            store.append_event(persisted),
            Err(PersistenceError::Duplicate)
        );
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
            Err(PersistenceError::IntegrityFailure)
        );
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn idempotency_claim_is_atomic_at_creation_boundary() {
        let path = root("idempotency");
        let mut store = LocalFileStore::open(&path).unwrap();
        assert_eq!(
            store.claim("operation-1".into()).unwrap(),
            IdempotencyClaim::Claimed
        );
        assert_eq!(
            store.claim("operation-1".into()).unwrap(),
            IdempotencyClaim::AlreadyClaimed
        );
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let path = root("schema");
        let mut store = LocalFileStore::open(&path).unwrap();
        let case = Case::new("case-1".into()).unwrap();
        let persisted = Persisted::new(2, case).unwrap();
        assert_eq!(
            store.create_case(persisted),
            Err(PersistenceError::UnsupportedSchemaVersion)
        );
        let _ = fs::remove_dir_all(path);
    }
}
