//! PostgreSQL persistence adapter for the provider-neutral unit-of-work contract.
//!
//! This adapter intentionally stores canonical resource payloads as JSONB at the
//! infrastructure boundary. Domain validation remains in the Rust core; the
//! database provides atomic commit/rollback and durable storage.

use postgres::{Client, NoTls};
use serde_json::Value;

use crate::persistence::{
    PersistenceError, ResourceLink, ResourceWrite, ResourceWriteMode, UnitOfWork,
    UnitOfWorkContext, UnitOfWorkError, UnitOfWorkFactory,
};
use crate::ResourceType;

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
    client: Option<Client>,
    active: bool,
}

impl PostgresUnitOfWork {
    fn map_error(_: postgres::Error) -> PersistenceError {
        PersistenceError::Unavailable
    }

    fn client_mut(&mut self) -> Result<&mut Client, UnitOfWorkError> {
        self.client
            .as_mut()
            .ok_or(UnitOfWorkError::Persistence(PersistenceError::Conflict))
    }
}

pub struct PostgresUnitOfWorkContext<'a> {
    client: &'a mut Client,
}

impl<'a> PostgresUnitOfWorkContext<'a> {
    fn resource_type_name(resource_type: ResourceType) -> &'static str {
        match resource_type {
            ResourceType::Case => "case",
            ResourceType::Incident => "incident",
            ResourceType::Event => "event",
            ResourceType::Authority => "authority",
            ResourceType::Jurisdiction => "jurisdiction",
            ResourceType::Party => "party",
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
            ResourceType::Other => "other",
        }
    }
}

impl UnitOfWorkContext for PostgresUnitOfWorkContext<'_> {
    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError> {
        let resource_type = Self::resource_type_name(write.resource_ref.resource_type);
        let id = write.resource_ref.id;
        let schema_version = i32::from(write.schema_version);
        let payload = write.payload;

        match write.mode {
            ResourceWriteMode::Insert => self
                .client
                .execute(
                    "INSERT INTO sidereth_resource_records
                        (resource_type, resource_id, schema_version, payload)
                     VALUES ($1, $2, $3, $4)",
                    &[&resource_type, &id, &schema_version, &payload],
                )
                .map(|_| ())
                .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Duplicate)),
            ResourceWriteMode::Upsert => self
                .client
                .execute(
                    "INSERT INTO sidereth_resource_records
                        (resource_type, resource_id, schema_version, payload)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (resource_type, resource_id)
                     DO UPDATE SET schema_version = EXCLUDED.schema_version,
                                   payload = EXCLUDED.payload,
                                   updated_at = CURRENT_TIMESTAMP",
                    &[&resource_type, &id, &schema_version, &payload],
                )
                .map(|_| ())
                .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable)),
        }
    }

    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError> {
        let source_type = Self::resource_type_name(link.source_ref.resource_type);
        let target_type = Self::resource_type_name(link.target_ref.resource_type);
        self.client
            .execute(
                "INSERT INTO sidereth_resource_links
                    (source_type, source_id, relation, target_type, target_id)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (source_type, source_id, relation, target_type, target_id)
                 DO NOTHING",
                &[
                    &source_type,
                    &link.source_ref.id,
                    &link.relation,
                    &target_type,
                    &link.target_ref.id,
                ],
            )
            .map(|_| ())
            .map_err(|_| UnitOfWorkError::Persistence(PersistenceError::Unavailable))
    }
}

impl UnitOfWork for PostgresUnitOfWork {
    type Context = PostgresUnitOfWorkContext<'static>;

    fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
    where
        F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>,
    {
        // The context lifetime is tied to the temporary borrow of this UoW. The
        // adapter never stores the context, so the transmute only expresses the
        // associated-type shape required by the provider-neutral trait.
        let client = self.client_mut()?;
        let context = PostgresUnitOfWorkContext { client };
        let context = unsafe {
            std::mem::transmute::<PostgresUnitOfWorkContext<'_>, PostgresUnitOfWorkContext<'static>>(context)
        };
        let result = operation(&mut { context });
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                let _ = self.rollback_in_place();
                Err(error)
            }
        }
    }

    fn commit(mut self) -> Result<(), PersistenceError> {
        let client = self
            .client
            .take()
            .ok_or(PersistenceError::Conflict)?;
        let mut client = client;
        client
            .batch_execute("COMMIT")
            .map_err(Self::map_error)?;
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
        if let Some(client) = self.client.as_mut() {
            client.batch_execute("ROLLBACK").map_err(Self::map_error)?;
        }
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
            client: Some(client),
            active: true,
        })
    }
}

/// Small helper for adapters that need to persist a typed JSON payload.
pub fn to_json<T: serde::Serialize>(value: &T) -> Result<Value, PersistenceError> {
    serde_json::to_value(value).map_err(|_| PersistenceError::SerializationFailure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_preserves_connection_configuration() {
        let factory = PostgresUnitOfWorkFactory::new("host=localhost user=sidereth");
        assert_eq!(
            factory.connection_string(),
            "host=localhost user=sidereth"
        );
    }
}
