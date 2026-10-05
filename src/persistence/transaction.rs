use super::error::PersistenceError;

pub trait Transaction {
    fn commit(self) -> Result<(), PersistenceError>;
    fn rollback(self) -> Result<(), PersistenceError>;
}

pub trait TransactionFactory {
    type Tx: Transaction;

    fn begin(&mut self) -> Result<Self::Tx, PersistenceError>;
}
