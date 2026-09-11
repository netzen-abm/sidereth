//! PostgreSQL persistence adapter for the provider-neutral unit-of-work contract.
//!
//! Resource reads and writes execute on the same SQL transaction. Optimistic
//! compare-and-set writes are enforced in SQL so concurrent writers cannot
//! silently overwrite a newer revision.

use std::sync::{Arc, Mutex};

use postgres::{Client, NoTls};
use serde_json::Value;

use crate::persistence::{
    PersistenceError, ResourceLink, ResourceLinkClass, ResourceRecord, ResourceWrite,
    ResourceWriteMode, Revision, UnitOfWork, UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
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
            ResourceType::Observation => "observation",
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
            "observation" => ResourceType::Observation,
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
            _ => {
                return Err(UnitOfWorkError::Persistence(
                    PersistenceError::IntegrityFailure,
                ))
            }
        };
        ResourceRef::new(kind, id)
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure))
    }

    fn semantic_class_name(class: Option<ResourceLinkClass>) -> Option<&'static str> {
        match class {
            Some(ResourceLinkClass::Strong) => Some("strong"),
            Some(ResourceLinkClass::Forward) => Some("forward"),
            Some(ResourceLinkClass::External) => Some("external"),
            Some(ResourceLinkClass::Legacy) | None => None,
        }
    }

    fn resource_exists(
        client: &mut postgres::Client,
        resource_ref: &ResourceRef,
    ) -> Result<bool, UnitOfWorkError> {
        let resource_type = Self::resource_type_name(resource_ref.resource_type);
        client
            .query_opt(
                "SELECT 1 FROM sidereth_resource_records WHERE resource_type = $1 AND resource_id = $2",
                &[&resource_type, &resource_ref.id],
            )
            .map(|row| row.is_some())
            .map_err(|error| UnitOfWorkError::Persistence(PostgresUnitOfWork::map_error(error)))
    }
}

impl UnitOfWorkContext for PostgresUnitOfWorkContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError> {
        let resource_type = Self::resource_type_name(resource_ref.resource_type);
        let mut client = self
            .client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))?;
        let row = client
            .query_opt(
                "SELECT resource_type, resource_id, schema_version, revision, payload
             FROM sidereth_resource_records
             WHERE resource_type = $1 AND resource_id = $2",
                &[&resource_type, &resource_ref.id],
            )
            .map_err(|error| UnitOfWorkError::Persistence(PostgresUnitOfWork::map_error(error)))?;
        let Some(row) = row else {
            return Ok(None);
        };
        let stored_type: &str = row.get(0);
        let schema_version: i32 = row.get(2);
        let revision: i64 = row.get(3);
        if !(1..=i32::from(u16::MAX)).contains(&schema_version) || revision < 0 {
            return Err(UnitOfWorkError::Persistence(
                PersistenceError::IntegrityFailure,
            ));
        }
        let revision = u64::try_from(revision)
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::IntegrityFailure))?;
        Ok(Some(ResourceRecord {
            resource_ref: Self::resource_ref(stored_type, row.get(1))?,
            schema_version: schema_version as u16,
            revision: Revision { value: revision },
            payload: row.get(4),
        }))
    }

    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        let resource_type = Self::resource_type_name(write.resource_ref.resource_type);
        let id = write.resource_ref.id;
        let schema_version = i32::from(write.schema_version);
        let payload = write.payload;
        let mut client = self
            .client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))?;
        match (write.mode, write.expected_revision) {
            (ResourceWriteMode::Insert, Some(_)) => Err(UnitOfWorkError::InvalidOperation),
            (ResourceWriteMode::Insert, None) => client
                .execute(
                    "INSERT INTO sidereth_resource_records
                    (resource_type, resource_id, schema_version, revision, payload)
                 VALUES ($1, $2, $3, 0, $4)",
                    &[&resource_type, &id, &schema_version, &payload],
                )
                .map(|_| ())
                .map_err(|error| {
                    UnitOfWorkError::Persistence(PostgresUnitOfWork::map_error(error))
                }),
            (ResourceWriteMode::Upsert, Some(expected)) => {
                let expected_revision = i64::try_from(expected.value)
                    .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Conflict))?;
                let next_revision = expected.next().map_err(UnitOfWorkError::Persistence)?;
                let next_revision = i64::try_from(next_revision.value)
                    .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Conflict))?;
                let affected = client
                    .execute(
                        "UPDATE sidereth_resource_records
                     SET schema_version = $3,
                         revision = $4,
                         payload = $5,
                         updated_at = CURRENT_TIMESTAMP
                     WHERE resource_type = $1
                       AND resource_id = $2
                       AND revision = $6",
                        &[
                            &resource_type,
                            &id,
                            &schema_version,
                            &next_revision,
                            &payload,
                            &expected_revision,
                        ],
                    )
                    .map_err(|error| {
                        UnitOfWorkError::Persistence(PostgresUnitOfWork::map_error(error))
                    })?;
                if affected == 0 {
                    return Err(UnitOfWorkError::Persistence(PersistenceError::Conflict));
                }
                Ok(())
            }
            (ResourceWriteMode::Upsert, None) => client
                .execute(
                    "INSERT INTO sidereth_resource_records
                    (resource_type, resource_id, schema_version, revision, payload)
                 VALUES ($1, $2, $3, 0, $4)
                 ON CONFLICT (resource_type, resource_id)
                 DO UPDATE SET schema_version = EXCLUDED.schema_version,
                               revision = sidereth_resource_records.revision + 1,
                               payload = EXCLUDED.payload,
                               updated_at = CURRENT_TIMESTAMP",
                    &[&resource_type, &id, &schema_version, &payload],
                )
                .map(|_| ())
                .map_err(|error| {
                    UnitOfWorkError::Persistence(PostgresUnitOfWork::map_error(error))
                }),
        }
    }

    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError> {
        let class = link.semantic_class();
        let source_type = Self::resource_type_name(link.source_ref.resource_type);
        let target_type = Self::resource_type_name(link.target_ref.resource_type);
        let semantic_class = Self::semantic_class_name(class);
        let mut client = self
            .client
            .lock()
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))?;

        if class == Some(ResourceLinkClass::Strong)
            && (!Self::resource_exists(&mut client, &link.source_ref)?
                || !Self::resource_exists(&mut client, &link.target_ref)?)
        {
            return Err(UnitOfWorkError::Persistence(
                PersistenceError::IntegrityFailure,
            ));
        }

        let affected = client
            .execute(
                "INSERT INTO sidereth_resource_links
                (source_type, source_id, relation, target_type, target_id, semantic_class)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (source_type, source_id, relation, target_type, target_id)
             DO UPDATE SET semantic_class = EXCLUDED.semantic_class
             WHERE sidereth_resource_links.semantic_class IS NULL
                OR sidereth_resource_links.semantic_class = EXCLUDED.semantic_class",
                &[
                    &source_type,
                    &link.source_ref.id,
                    &link.relation,
                    &target_type,
                    &link.target_ref.id,
                    &semantic_class,
                ],
            )
            .map_err(|error| UnitOfWorkError::Persistence(PostgresUnitOfWork::map_error(error)))?;

        if affected == 0 {
            return Err(UnitOfWorkError::Persistence(PersistenceError::Conflict));
        }
        Ok(())
    }
}

impl UnitOfWork for PostgresUnitOfWork {
    type Context = PostgresUnitOfWorkContext;

    fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
    where
        F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>,
    {
        if !self.active {
            return Err(UnitOfWorkError::Persistence(PersistenceError::Conflict));
        }
        let mut context = PostgresUnitOfWorkContext {
            client: Arc::clone(&self.client),
        };
        operation(&mut context)
    }

    fn commit(mut self) -> Result<(), PersistenceError> {
        if !self.active {
            return Err(PersistenceError::Conflict);
        }
        let result = {
            let mut client = self
                .client
                .lock()
                .map_err(|_| PersistenceError::Unavailable)?;
            client.batch_execute("COMMIT").map_err(Self::map_error)
        };
        result?;
        self.active = false;
        Ok(())
    }

    fn rollback(mut self) -> Result<(), PersistenceError> {
        self.rollback_in_place()
    }
}

impl PostgresUnitOfWork {
    fn rollback_in_place(&mut self) -> Result<(), PersistenceError> {
        if !self.active {
            return Ok(());
        }
        let result = {
            let mut client = self
                .lock_client()
                .map_err(|_| PersistenceError::Unavailable)?;
            let result = client.batch_execute("ROLLBACK").map_err(Self::map_error);
            drop(client);
            result
        };
        result?;
        self.active = false;
        Ok(())
    }
}

impl Drop for PostgresUnitOfWork {
    fn drop(&mut self) {
        if self.active {
            let _ = self.rollback_in_place();
        }
    }
}

impl UnitOfWorkFactory for PostgresUnitOfWorkFactory {
    type Uow = PostgresUnitOfWork;

    fn begin(&mut self) -> Result<Self::Uow, PersistenceError> {
        let mut client = Client::connect(&self.connection_string, NoTls)
            .map_err(PostgresUnitOfWork::map_error)?;
        client
            .batch_execute("BEGIN")
            .map_err(PostgresUnitOfWork::map_error)?;
        Ok(PostgresUnitOfWork {
            client: Arc::new(Mutex::new(client)),
            active: true,
        })
    }
}

pub fn to_json<T: serde::Serialize>(value: &T) -> Result<Value, PersistenceError> {
    serde_json::to_value(value).map_err(|_| PersistenceError::SerializationFailure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_preserves_connection_configuration() {
        let factory = PostgresUnitOfWorkFactory::new("host=localhost user=sidereth");
        assert_eq!(factory.connection_string(), "host=localhost user=sidereth");
    }
}
