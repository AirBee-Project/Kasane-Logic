//! Error types for the encode_id_kv module.

use std::fmt;

/// Error type for EncodeIDKV operations
#[derive(Debug)]
pub enum EncodeIDKVError {
    /// Database error from redb
    DatabaseError(redb::DatabaseError),
    /// Transaction error from redb
    TransactionError(redb::TransactionError),
    /// Table error from redb
    TableError(redb::TableError),
    /// Storage error from redb
    StorageError(redb::StorageError),
    /// Commit error from redb
    CommitError(redb::CommitError),
    /// Map with the given name already exists
    MapAlreadyExists(String),
    /// Map with the given name does not exist
    MapNotFound(String),
    /// Lock poisoned error
    LockPoisoned,
}

impl fmt::Display for EncodeIDKVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeIDKVError::DatabaseError(e) => write!(f, "Database error: {}", e),
            EncodeIDKVError::TransactionError(e) => write!(f, "Transaction error: {}", e),
            EncodeIDKVError::TableError(e) => write!(f, "Table error: {}", e),
            EncodeIDKVError::StorageError(e) => write!(f, "Storage error: {}", e),
            EncodeIDKVError::CommitError(e) => write!(f, "Commit error: {}", e),
            EncodeIDKVError::MapAlreadyExists(name) => {
                write!(f, "Map '{}' already exists", name)
            }
            EncodeIDKVError::MapNotFound(name) => write!(f, "Map '{}' not found", name),
            EncodeIDKVError::LockPoisoned => write!(f, "Lock poisoned"),
        }
    }
}

impl std::error::Error for EncodeIDKVError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EncodeIDKVError::DatabaseError(e) => Some(e),
            EncodeIDKVError::TransactionError(e) => Some(e),
            EncodeIDKVError::TableError(e) => Some(e),
            EncodeIDKVError::StorageError(e) => Some(e),
            EncodeIDKVError::CommitError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<redb::DatabaseError> for EncodeIDKVError {
    fn from(e: redb::DatabaseError) -> Self {
        EncodeIDKVError::DatabaseError(e)
    }
}

impl From<redb::TransactionError> for EncodeIDKVError {
    fn from(e: redb::TransactionError) -> Self {
        EncodeIDKVError::TransactionError(e)
    }
}

impl From<redb::TableError> for EncodeIDKVError {
    fn from(e: redb::TableError) -> Self {
        EncodeIDKVError::TableError(e)
    }
}

impl From<redb::StorageError> for EncodeIDKVError {
    fn from(e: redb::StorageError) -> Self {
        EncodeIDKVError::StorageError(e)
    }
}

impl From<redb::CommitError> for EncodeIDKVError {
    fn from(e: redb::CommitError) -> Self {
        EncodeIDKVError::CommitError(e)
    }
}
