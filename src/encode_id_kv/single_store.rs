//! Single-value Key-Value store for EncodeID mappings.
//!
//! Each EncodeID maps to exactly one value.

use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use bincode::{Decode, Encode};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::encode_id::EncodeID;

use super::error::EncodeIDKVError;
use super::{bytes_to_encode_id, bytes_to_value, encode_id_to_bytes, value_to_bytes};

/// A single-value Key-Value store for EncodeID mappings backed by redb.
///
/// This struct provides thread-safe access to a persistent Key-Value store
/// where each EncodeID maps to exactly one value.
///
/// # Type Parameters
/// - `V`: The value type, must implement `Encode` and `Decode<()>` from bincode
///
/// # Example
/// ```no_run
/// use kasane_logic::encode_id_kv::EncodeIDKVStore;
/// use std::sync::Arc;
/// use redb::Database;
///
/// let db = Arc::new(Database::create("my_db.redb").unwrap());
/// let store: EncodeIDKVStore<String> = EncodeIDKVStore::new(db, "my_map").unwrap();
/// ```
pub struct EncodeIDKVStore<V> {
    db: Arc<Database>,
    table_name: String,
    map_names: Arc<RwLock<HashSet<String>>>,
    _marker: PhantomData<V>,
}

impl<V: Encode + Decode<()> + Clone> EncodeIDKVStore<V> {
    /// Create a new EncodeIDKVStore with the given database and map name.
    ///
    /// # Arguments
    /// - `db`: The redb database instance wrapped in Arc for thread-safe sharing
    /// - `map_name`: The name of the map, must be unique across all stores using the same database
    ///
    /// # Errors
    /// Returns an error if the map already exists or if there's a database error.
    pub fn new(db: Arc<Database>, map_name: &str) -> Result<Self, EncodeIDKVError> {
        let table_name = format!("single_{}", map_name);

        // Ensure the table can be created/opened
        {
            let write_txn = db.begin_write()?;
            let table_def: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new(&table_name);
            write_txn.open_table(table_def)?;
            write_txn.commit()?;
        }

        Ok(Self {
            db,
            table_name,
            map_names: Arc::new(RwLock::new(HashSet::from([map_name.to_string()]))),
            _marker: PhantomData,
        })
    }

    /// Create a new map within this store with the given name.
    ///
    /// # Arguments
    /// - `map_name`: The name of the new map, must be unique
    ///
    /// # Errors
    /// Returns an error if the map already exists.
    pub fn create_map(&self, map_name: &str) -> Result<EncodeIDKVStore<V>, EncodeIDKVError> {
        let mut names = self
            .map_names
            .write()
            .map_err(|_| EncodeIDKVError::LockPoisoned)?;
        if names.contains(map_name) {
            return Err(EncodeIDKVError::MapAlreadyExists(map_name.to_string()));
        }
        names.insert(map_name.to_string());

        let table_name = format!("single_{}", map_name);
        {
            let write_txn = self.db.begin_write()?;
            let table_def: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new(&table_name);
            write_txn.open_table(table_def)?;
            write_txn.commit()?;
        }

        Ok(EncodeIDKVStore {
            db: Arc::clone(&self.db),
            table_name,
            map_names: Arc::clone(&self.map_names),
            _marker: PhantomData,
        })
    }

    /// Insert an EncodeID-value pair into the store.
    ///
    /// If the EncodeID already exists, its value will be replaced.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    /// - `value`: The value to store
    pub fn insert(&self, encode_id: &EncodeID, value: &V) -> Result<(), EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);
        let value_bytes = value_to_bytes(value);

        let write_txn = self.db.begin_write()?;
        {
            let table_def: TableDefinition<'_, &[u8], &[u8]> =
                TableDefinition::new(&self.table_name);
            let mut table = write_txn.open_table(table_def)?;
            table.insert(key_bytes.as_slice(), value_bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get the value associated with the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    ///
    /// # Returns
    /// - `Some(value)` if the key exists
    /// - `None` if the key does not exist
    pub fn get(&self, encode_id: &EncodeID) -> Result<Option<V>, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);

        let read_txn = self.db.begin_read()?;
        let table_def: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new(&self.table_name);
        let table = read_txn.open_table(table_def)?;

        match table.get(key_bytes.as_slice())? {
            Some(value_guard) => {
                let value_bytes = value_guard.value();
                Ok(Some(bytes_to_value(value_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Remove the value associated with the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    ///
    /// # Returns
    /// - `Some(value)` if the key existed and was removed
    /// - `None` if the key did not exist
    pub fn remove(&self, encode_id: &EncodeID) -> Result<Option<V>, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);

        let write_txn = self.db.begin_write()?;
        let removed_value;
        {
            let table_def: TableDefinition<'_, &[u8], &[u8]> =
                TableDefinition::new(&self.table_name);
            let mut table = write_txn.open_table(table_def)?;
            removed_value = match table.remove(key_bytes.as_slice())? {
                Some(value_guard) => {
                    let value_bytes = value_guard.value();
                    Some(bytes_to_value(value_bytes))
                }
                None => None,
            };
        }
        write_txn.commit()?;

        Ok(removed_value)
    }

    /// Check if the store contains the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    pub fn contains(&self, encode_id: &EncodeID) -> Result<bool, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);

        let read_txn = self.db.begin_read()?;
        let table_def: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new(&self.table_name);
        let table = read_txn.open_table(table_def)?;

        Ok(table.get(key_bytes.as_slice())?.is_some())
    }

    /// Get the number of entries in the store.
    pub fn len(&self) -> Result<u64, EncodeIDKVError> {
        let read_txn = self.db.begin_read()?;
        let table_def: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new(&self.table_name);
        let table = read_txn.open_table(table_def)?;

        Ok(table.len()?)
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> Result<bool, EncodeIDKVError> {
        Ok(self.len()? == 0)
    }

    /// Iterate over all key-value pairs in the store.
    ///
    /// # Returns
    /// A vector of (EncodeID, V) pairs.
    pub fn iter(&self) -> Result<Vec<(EncodeID, V)>, EncodeIDKVError> {
        let read_txn = self.db.begin_read()?;
        let table_def: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new(&self.table_name);
        let table = read_txn.open_table(table_def)?;

        let mut result = Vec::new();
        for entry in table.iter()? {
            let (key_guard, value_guard) = entry?;
            let key_bytes = key_guard.value();
            let value_bytes = value_guard.value();
            let encode_id = bytes_to_encode_id(key_bytes);
            let value: V = bytes_to_value(value_bytes);
            result.push((encode_id, value));
        }

        Ok(result)
    }

    /// Clear all entries from the store.
    pub fn clear(&self) -> Result<(), EncodeIDKVError> {
        // Collect all keys first using a read transaction
        let keys: Vec<Vec<u8>> = {
            let read_txn = self.db.begin_read()?;
            let table_def: TableDefinition<'_, &[u8], &[u8]> =
                TableDefinition::new(&self.table_name);
            let table = read_txn.open_table(table_def)?;
            let mut keys = Vec::new();
            for entry in table.iter()? {
                let (key_guard, _) = entry?;
                keys.push(key_guard.value().to_vec());
            }
            keys
        };

        // Then remove all keys in a write transaction
        let write_txn = self.db.begin_write()?;
        {
            let table_def: TableDefinition<'_, &[u8], &[u8]> =
                TableDefinition::new(&self.table_name);
            let mut table = write_txn.open_table(table_def)?;
            for key in keys {
                table.remove(key.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

// Thread-safe: EncodeIDKVStore can be sent between threads and shared safely
unsafe impl<V: Send> Send for EncodeIDKVStore<V> {}
unsafe impl<V: Send + Sync> Sync for EncodeIDKVStore<V> {}
