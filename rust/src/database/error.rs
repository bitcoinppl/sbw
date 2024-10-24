use super::{
    global_config::GlobalConfigTableError, global_flag::GlobalFlagTableError,
    wallet::WalletTableError,
};

type Error = DatabaseError;

#[derive(Debug, Clone, Hash, Eq, PartialEq, uniffi::Error, thiserror::Error)]
pub enum SerdeError {
    #[error("failed to serialize: {0}")]
    SerializationError(String),

    #[error("failed to deserialize: {0}")]
    DeserializationError(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, uniffi::Error, thiserror::Error)]
pub enum DatabaseError {
    #[error("failed to open database: {0}")]
    DatabaseAccess(String),

    #[error("failed to open table: {0}")]
    TableAccess(String),

    #[error(transparent)]
    Wallets(#[from] WalletTableError),

    #[error(transparent)]
    GlobalFlag(#[from] GlobalFlagTableError),

    #[error(transparent)]
    GlobalConfig(#[from] GlobalConfigTableError),

    #[error("unable to serialize or deserialize: {0}")]
    Serialization(#[from] SerdeError),
}

impl From<redb::TransactionError> for Error {
    fn from(error: redb::TransactionError) -> Self {
        Self::DatabaseAccess(error.to_string())
    }
}

impl From<redb::TableError> for Error {
    fn from(error: redb::TableError) -> Self {
        Self::TableAccess(error.to_string())
    }
}

impl From<redb::StorageError> for Error {
    fn from(error: redb::StorageError) -> Self {
        Self::TableAccess(error.to_string())
    }
}
