use super::error::{PersistenceError, UnitOfWorkError};
use super::resource::{ResourceLink, ResourceRecord, ResourceWrite};

/// Provider-neutral read/write context used inside a unit of work.
/// Reads occur in the same transaction as subsequent CAS writes.
pub trait UnitOfWorkContext {
    fn read_resource(
        &mut self,
        resource_ref: &ResourceRef,
    ) -> Result<Option<ResourceRecord>, UnitOfWorkError>;
    fn write_resource(&mut self, write: ResourceWrite) -> Result<(), UnitOfWorkError>;
    fn link_resources(&mut self, link: ResourceLink) -> Result<(), UnitOfWorkError>;
}

/// Provider-neutral atomic boundary for a multi-resource workflow.
pub trait UnitOfWork {
    type Context: UnitOfWorkContext;

    fn execute<R, F>(&mut self, operation: F) -> Result<R, UnitOfWorkError>
    where
        F: FnOnce(&mut Self::Context) -> Result<R, UnitOfWorkError>;

    fn commit(self) -> Result<(), PersistenceError>;
    fn rollback(self) -> Result<(), PersistenceError>;
}

pub trait UnitOfWorkFactory {
    type Uow: UnitOfWork;

    fn begin(&mut self) -> Result<Self::Uow, PersistenceError>;
}

