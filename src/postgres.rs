//! PostgreSQL persistence adapter for the provider-neutral unit-of-work contract.
//!
//! Resource reads and writes execute on the same SQL transaction. Optimistic
//! compare-and-set writes are enforced in SQL so concurrent writers cannot
//! silently overwrite a newer revision.

use std::sync::{Arc, Mutex};

use postgres::{Client, NoTls};
use serde_json::Value;

use crate::persistence::{
    PersistenceError, ResourceLink, ResourceRecord, ResourceWrite, ResourceWriteMode, Revision,
    UnitOfWork, UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use crate::{ResourceRef, ResourceType};

#[derive(Debug)]
pub struct PostgresUnitOfWorkFactory {
    connection_string: String,
}

impl PostgresUnitOfWorkFactory {
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }

    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

pub struct PostgresUnitOfWork {
    client: Arc<Mutex<Client>>,
    active: bool,
}

impl PostgresUnitOfWork {
    fn map_error(error: postgres::Error) -> PersistenceError {
        match error.code().map(|code| code.code()) {
            Some("23505") => PersistenceError::Duplicate,
            Some("23503") | Some("23514") => PersistenceError::IntegrityFailure,
            Some("40001") | Some("40P01") => PersistenceError::Conflict,
            Some("57014") => PersistenceError::Timeout,
            _ => PersistenceError::Unavailable,
        }
    }

    fn lock_client(&self) -> Result<std::sync::MutexGuard<'_, Client>, UnitOfWorkError> {
        self.client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))
    }
}

pub struct PostgresUnitOfWorkContext {
    client: Arc<Mutex<Client>>,
}

impl PostgresUnitOfWorkContext {
    fn resource_type_name(resource_type: ResourceType) -> &'static str {
        match resource_type {
            ResourceType::Case => "case",
            ResourceType::Incident => "incident",
            ResourceType::Event => "event",
            ResourceType::Authority => "authority",
            ResourceType::Jurisdiction => "jurisdiction",
            ResourceType::Party => "party",
            ResourceType::PartyRelationship => "party_relationship",
            ResourceType::Document => "document",
            ResourceType::Action => "action",
            ResourceType::Deadline => "deadline",
            ResourceType::Response => "response",
            ResourceType::Escalation => "escalation",
            ResourceType::Remedy => "remedy",
            ResourceType::Resolution => "resolution",
            ResourceType::Procedure => "procedure",
            ResourceType::ComplianceRequirement => "compliance_requirement",
            ResourceType::LegalSource => "legal_source",
            ResourceType::Timeline => "timeline",
            ResourceType::Evidence => "evidence",
            ResourceType::Audit => "audit",
            ResourceType::Provenance => "provenance",
            ResourceType::Idempotency => "idempotency",
            ResourceType::Other => "other",
        }
    }

    fn resource_ref(resource_type: &str, id: String) -> Result<ResourceRef, UnitOfWorkError> {
        let kind = match resource_type {
            "case" => ResourceType::Case,
            "incident" => ResourceType::Incident,
            "event" => ResourceType::Event,
            "authority" => ResourceType::Authority,
            "jurisdiction" => ResourceType::Jurisdiction,
            "party" => ResourceType::Party,
            "party_relationship" => ResourceType::PartyRelationship,
            "document" => ResourceType::Document,
            "action" => ResourceType::Action,
            "deadline" => ResourceType::Deadline,
            "response" => ResourceType::Response,
            "escalation" => ResourceType::Escalation,
            "remedy" => ResourceType::Remedy,
            "resolution" => ResourceType::Resolution,
            "procedure" => ResourceType::Procedure,
            "compliance_requirement" => ResourceType::ComplianceRequirement,
            "legal_source" => ResourceType::LegalSource,
            "timeline" => ResourceType::Timeline,
            "evidence" => ResourceType::Evidence,
            "audit" => ResourceType::Audit,
            "provenance" => ResourceType::Provenance,
            "idempotency" => ResourceType::Idempotency,
            "other" => ResourceType::Other,
            _ => return Err(UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure)),
        };
        ResourceRef::new(kind, id)
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure))
    }

    fn read_record(
        &self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
        let mut client = self
            .client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))?;
        let row = client
            .query_opt(
                "SELECT resource_type, resource_id, schema_version, revision, payload
                 FROM sidereth_resource_records
                 WHERE resource_type = $1 AND resource_id = $2",
                &[
                    &Self::resource_type_name(resource_ref.resource_type),
                    &resource_ref.id,
                ],
            )
            .map_err(PostgresUnitOfWork::map_error)?;
        row.map(|row| {
            let resource_type: String = row.get(0);
            let resource_id: String = row.get(1);
            let schema_version: i32 = row.get(2);
            let revision: i64 = row.get(3);
            let payload: Value = row.get(4);
            let resource_ref = Self::resource_ref(&resource_type, resource_id)?;
            let schema_version = u16::try_from(schema_version).map_err(|_| {
                UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure)
            })?;
            let revision = Revision::try_from(revision).map_err(|_| {
                UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure)
            })?;
            Ok(ResourceRecord {
                resource_ref,
                schema_version,
                revision,
                payload,
            })
        })
        .transpose()
    }

    fn write_record(&self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        let expected_revision = write.expected_revision;
        let mut client = self
            .client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))?;
        let resource_type = Self::resource_type_name(write.resource_ref.resource_type);
        let resource_id = write.resource_ref.id.clone();
        let schema_version = i32::from(write.schema_version);
        let payload = write.payload;

        match write.mode {
            ResourceWriteMode::Insert => {
                client
                    .execute(
                        "INSERT INTO sidereth_resource_records
                         (resource_type, resource_id, schema_version, revision, payload)
                         VALUES ($1, $2, $3, 1, $4)",
                        &[&resource_type, &resource_id, &schema_version, &payload],
                    )
                    .map_err(PostgresUnitOfWork::map_error)?;
            }
            ResourceWriteMode::Upsert => {
                if let Some(expected) = expected_revision {
                    let expected = i64::try_from(expected.get()).map_err(|_| {
                        UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure)
                    })?;
                    let updated = client
                        .execute(
                            "UPDATE sidereth_resource_records
                             SET schema_version = $3, revision = revision + 1, payload = $4, updated_at = CURRENT_TIMESTAMP
                             WHERE resource_type = $1 AND resource_id = $2 AND revision = $5",
                            &[&resource_type, &resource_id, &schema_version, &payload, &expected],
                        )
                        .map_err(PostgresUnitOfWork::map_error)?;
                    if updated == 0 {
                        let exists = client
                            .query_opt(
                                "SELECT revision FROM sidereth_resource_records WHERE resource_type = $1 AND resource_id = $2",
                                &[&resource_type, &resource_id],
                            )
                            .map_err(PostgresUnitOfWork::map_error)?;
                        return Err(UnitOfWorkError::Persistence(if exists.is_some() {
                            PersistenceError::Conflict
                        } else {
                            PersistenceError::NotFound
                        }));
                    }
                } else {
                    client
                        .execute(
                            "INSERT INTO sidereth_resource_records
                             (resource_type, resource_id, schema_version, revision, payload)
                             VALUES ($1, $2, $3, 1, $4)
                             ON CONFLICT (resource_type, resource_id)
                             DO UPDATE SET schema_version = EXCLUDED.schema_version,
                                           revision = sidereth_resource_records.revision + 1,
                                           payload = EXCLUDED.payload,
                                           updated_at = CURRENT_TIMESTAMP",
                            &[&resource_type, &resource_id, &schema_version, &payload],
                        )
                        .map_err(PostgresUnitOfWork::map_error)?;
                }
            }
        }
        Ok(())
    }

    fn link_record(&self, link: ResourceLink) -> Result<(), UnitOfWorkError> {
        let mut client = self
            .client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))?;
        client
            .execute(
                "INSERT INTO sidereth_resource_links
                 (source_type, source_id, relation, target_type, target_id)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT DO NOTHING",
                &[
                    &Self::resource_type_name(link.source.resource_type),
                    &link.source.id,
                    &link.relation,
                    &Self::resource_type_name(link.target.resource_type),
                    &link.target.id,
                ],
            )
            .map_err(PostgresUnitOfWork::map_error)?;
        Ok(())
    }
}

impl UnitOfWorkContext for PostgresUnitOfWorkContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
        self.read_record(resource_ref)
    }

    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        self.write_record(write)
    }

    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError> {
        self.link_record(link)
    }
}

impl UnitOfWork for PostgresUnitOfWork {
    fn execute<T, F>(&mut self, operation: F) -> Result<T, UnitOfWorkError>
    where
        F: FnOnce(&mut dyn UnitOfWorkContext) -> Result<T, UnitOfWorkError>,
    {
        if !self.active {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        let mut client = self.lock_client()?;
        client.batch_execute("BEGIN").map_err(Self::map_error)?;
        drop(client);

        let mut context = PostgresUnitOfWorkContext {
            client: Arc::clone(&self.client),
        };
        let result = operation(&mut context);
        let mut client = self.lock_client()?;
        match result {
            Ok(value) => {
                client.batch_execute("COMMIT").map_err(Self::map_error)?;
                self.active = false;
                Ok(value)
            }
            Err(error) => {
                client.batch_execute("ROLLBACK").map_err(Self::map_error)?;
                self.active = false;
                Err(error)
            }
        }
    }

    fn commit(&mut self) -> Result<(), UnitOfWorkError> {
        if !self.active {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        let mut client = self.lock_client()?;
        client.batch_execute("COMMIT").map_err(Self::map_error)?;
        self.active = false;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), UnitOfWorkError> {
        if !self.active {
            return Err(UnitOfWorkError::InvalidOperation);
        }
        let mut client = self.lock_client()?;
        client.batch_execute("ROLLBACK").map_err(Self::map_error)?;
        self.active = false;
        Ok(())
    }
}

impl UnitOfWorkFactory for PostgresUnitOfWorkFactory {
    type Uow = PostgresUnitOfWork;

    fn begin(&mut self) -> Result<Self::Uow, UnitOfWorkError> {
        let client = Client::connect(&self.connection_string, NoTls)
            .map_err(PostgresUnitOfWork::map_error)?;
        Ok(PostgresUnitOfWork {
            client: Arc::new(Mutex::new(client)),
            active: true,
        })
    }
}
