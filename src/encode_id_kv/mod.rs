//! Key-Value store backed implementation for EncodeID mappings using redb.
//!
//! This module provides a persistent Key-Value store for mapping EncodeIDs to values,
//! similar to `EncodeIDMap` but backed by redb for persistence and thread-safe access.
//!
//! ## Features
//! - Persistent storage using redb
//! - Thread-safe access via RwLock
//! - Multiple internal maps with user-defined names
//! - Support for both single-value and multi-value modes
//!
//! ## Types
//! - `EncodeIDKVStore<V>`: Single value per EncodeID
//! - `EncodeIDKVMultiStore<V>`: Multiple values per EncodeID
//!
//! ## Example
//! ```no_run
//! use kasane_logic::encode_id_kv::{EncodeIDKVStore, EncodeIDKVMultiStore};
//! use std::sync::Arc;
//! use redb::Database;
//!
//! // Create or open a database
//! let db = Arc::new(Database::create("my_db.redb").unwrap());
//!
//! // Create a single-value store
//! let single_store: EncodeIDKVStore<String> = EncodeIDKVStore::new(Arc::clone(&db), "my_map").unwrap();
//!
//! // Create a multi-value store  
//! let multi_store: EncodeIDKVMultiStore<i32> = EncodeIDKVMultiStore::new(db, "my_multi_map").unwrap();
//! ```

mod error;
mod multi_store;
mod single_store;

pub use error::EncodeIDKVError;
pub use multi_store::EncodeIDKVMultiStore;
pub use single_store::EncodeIDKVStore;

// Re-export redb Database for convenience
pub use redb::Database;

use bincode::{Decode, Encode};

use crate::encode_id::EncodeID;

/// Serialize EncodeID to bytes for storage in redb
pub(crate) fn encode_id_to_bytes(id: &EncodeID) -> Vec<u8> {
    let config = bincode::config::standard();
    bincode::encode_to_vec(id, config).expect("Failed to encode EncodeID")
}

/// Deserialize EncodeID from bytes
pub(crate) fn bytes_to_encode_id(bytes: &[u8]) -> EncodeID {
    let config = bincode::config::standard();
    let (id, _): (EncodeID, _) =
        bincode::decode_from_slice(bytes, config).expect("Failed to decode EncodeID");
    id
}

/// Serialize value to bytes for storage in redb
pub(crate) fn value_to_bytes<V: Encode>(value: &V) -> Vec<u8> {
    let config = bincode::config::standard();
    bincode::encode_to_vec(value, config).expect("Failed to encode value")
}

/// Deserialize value from bytes
pub(crate) fn bytes_to_value<V: Decode<()>>(bytes: &[u8]) -> V {
    let config = bincode::config::standard();
    let (value, _): (V, _) =
        bincode::decode_from_slice(bytes, config).expect("Failed to decode value");
    value
}
