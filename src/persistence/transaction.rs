use super::error::PersistenceError;
use super::idempotency::IdempotencyClaim;
use crate::Id;

pub trait Transaction {
    fn commit(self) -> Result<(), PersistenceError>;
    fn rollback(self) -> Result<(), PersistenceError>;
}

pub trait TransactionFactory {
    type Tx: Transaction;

    fn begin(&mut self) -> Result<Self::Tx, PersistenceError>;
}

pub trait IdempotencyStore {
    fn lookup(&self, operation_id: &Id) -> Result<bool, PersistenceError>;
    fn claim(&mut self, operation_id: Id) -> Result<IdempotencyClaim, PersistenceError>;
}
