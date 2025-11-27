//! Multi-value Key-Value store for EncodeID mappings.
//!
//! Each EncodeID can map to multiple values.

use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use bincode::{Decode, Encode};
use redb::{Database, MultimapTableDefinition, ReadableDatabase, ReadableMultimapTable, ReadableTableMetadata};

use crate::encode_id::EncodeID;

use super::error::EncodeIDKVError;
use super::{bytes_to_encode_id, bytes_to_value, encode_id_to_bytes, value_to_bytes};

/// A multi-value Key-Value store for EncodeID mappings backed by redb.
///
/// This struct provides thread-safe access to a persistent Key-Value store
/// where each EncodeID can map to multiple values.
///
/// # Type Parameters
/// - `V`: The value type, must implement `Encode` and `Decode<()>` from bincode
///
/// # Example
/// ```no_run
/// use kasane_logic::encode_id_kv::EncodeIDKVMultiStore;
/// use std::sync::Arc;
/// use redb::Database;
///
/// let db = Arc::new(Database::create("my_db.redb").unwrap());
/// let store: EncodeIDKVMultiStore<String> = EncodeIDKVMultiStore::new(db, "my_multi_map").unwrap();
/// ```
pub struct EncodeIDKVMultiStore<V> {
    db: Arc<Database>,
    table_name: String,
    map_names: Arc<RwLock<HashSet<String>>>,
    _marker: PhantomData<V>,
}

impl<V: Encode + Decode<()> + Clone> EncodeIDKVMultiStore<V> {
    /// Create a new EncodeIDKVMultiStore with the given database and map name.
    ///
    /// # Arguments
    /// - `db`: The redb database instance wrapped in Arc for thread-safe sharing
    /// - `map_name`: The name of the map, must be unique across all stores using the same database
    ///
    /// # Errors
    /// Returns an error if the map already exists or if there's a database error.
    pub fn new(db: Arc<Database>, map_name: &str) -> Result<Self, EncodeIDKVError> {
        let table_name = format!("multi_{}", map_name);

        // Ensure the table can be created/opened
        {
            let write_txn = db.begin_write()?;
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&table_name);
            write_txn.open_multimap_table(table_def)?;
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
    pub fn create_map(
        &self,
        map_name: &str,
    ) -> Result<EncodeIDKVMultiStore<V>, EncodeIDKVError> {
        let mut names = self
            .map_names
            .write()
            .map_err(|_| EncodeIDKVError::LockPoisoned)?;
        if names.contains(map_name) {
            return Err(EncodeIDKVError::MapAlreadyExists(map_name.to_string()));
        }
        names.insert(map_name.to_string());

        let table_name = format!("multi_{}", map_name);
        {
            let write_txn = self.db.begin_write()?;
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&table_name);
            write_txn.open_multimap_table(table_def)?;
            write_txn.commit()?;
        }

        Ok(EncodeIDKVMultiStore {
            db: Arc::clone(&self.db),
            table_name,
            map_names: Arc::clone(&self.map_names),
            _marker: PhantomData,
        })
    }

    /// Insert an EncodeID-value pair into the store.
    ///
    /// This will add the value to the set of values associated with the EncodeID.
    /// Duplicate values for the same key are allowed.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    /// - `value`: The value to store
    pub fn insert(&self, encode_id: &EncodeID, value: &V) -> Result<(), EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);
        let value_bytes = value_to_bytes(value);

        let write_txn = self.db.begin_write()?;
        {
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&self.table_name);
            let mut table = write_txn.open_multimap_table(table_def)?;
            table.insert(key_bytes.as_slice(), value_bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get all values associated with the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    ///
    /// # Returns
    /// A vector of values, empty if the key does not exist.
    pub fn get(&self, encode_id: &EncodeID) -> Result<Vec<V>, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);

        let read_txn = self.db.begin_read()?;
        let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
            MultimapTableDefinition::new(&self.table_name);
        let table = read_txn.open_multimap_table(table_def)?;

        let mut result = Vec::new();
        let values = table.get(key_bytes.as_slice())?;
        for value_result in values {
            let value_guard = value_result?;
            let value_bytes = value_guard.value();
            result.push(bytes_to_value(value_bytes));
        }

        Ok(result)
    }

    /// Remove all values associated with the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    ///
    /// # Returns
    /// A vector of removed values, empty if the key did not exist.
    pub fn remove_all(&self, encode_id: &EncodeID) -> Result<Vec<V>, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);

        // First, get all values
        let values = self.get(encode_id)?;

        if values.is_empty() {
            return Ok(values);
        }

        // Then remove them
        let write_txn = self.db.begin_write()?;
        {
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&self.table_name);
            let mut table = write_txn.open_multimap_table(table_def)?;
            table.remove_all(key_bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(values)
    }

    /// Remove a specific value associated with the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    /// - `value`: The value to remove
    ///
    /// # Returns
    /// `true` if the value was found and removed, `false` otherwise.
    pub fn remove(&self, encode_id: &EncodeID, value: &V) -> Result<bool, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);
        let value_bytes = value_to_bytes(value);

        let write_txn = self.db.begin_write()?;
        let removed;
        {
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&self.table_name);
            let mut table = write_txn.open_multimap_table(table_def)?;
            removed = table.remove(key_bytes.as_slice(), value_bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(removed)
    }

    /// Check if the store contains the given EncodeID.
    ///
    /// # Arguments
    /// - `encode_id`: The EncodeID key
    pub fn contains_key(&self, encode_id: &EncodeID) -> Result<bool, EncodeIDKVError> {
        let key_bytes = encode_id_to_bytes(encode_id);

        let read_txn = self.db.begin_read()?;
        let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
            MultimapTableDefinition::new(&self.table_name);
        let table = read_txn.open_multimap_table(table_def)?;

        let values = table.get(key_bytes.as_slice())?;
        for _ in values {
            return Ok(true);
        }
        Ok(false)
    }

    /// Get the number of entries (key-value pairs) in the store.
    /// Note: Each key can have multiple values, so this counts all key-value pairs.
    pub fn len(&self) -> Result<u64, EncodeIDKVError> {
        let read_txn = self.db.begin_read()?;
        let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
            MultimapTableDefinition::new(&self.table_name);
        let table = read_txn.open_multimap_table(table_def)?;

        Ok(table.len()?)
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> Result<bool, EncodeIDKVError> {
        Ok(self.len()? == 0)
    }

    /// Iterate over all key-value pairs in the store.
    ///
    /// # Returns
    /// A vector of (EncodeID, Vec<V>) pairs, where each key maps to all its values.
    pub fn iter(&self) -> Result<Vec<(EncodeID, Vec<V>)>, EncodeIDKVError> {
        let read_txn = self.db.begin_read()?;
        let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
            MultimapTableDefinition::new(&self.table_name);
        let table = read_txn.open_multimap_table(table_def)?;

        // Use a map to group values by key
        let mut result_map: std::collections::HashMap<Vec<u8>, Vec<V>> =
            std::collections::HashMap::new();

        for entry in table.iter()? {
            let (key_guard, values) = entry?;
            let key_bytes = key_guard.value().to_vec();

            let entry = result_map.entry(key_bytes).or_default();
            for value_result in values {
                let value_guard = value_result?;
                let value_bytes = value_guard.value();
                entry.push(bytes_to_value(value_bytes));
            }
        }

        let result: Vec<(EncodeID, Vec<V>)> = result_map
            .into_iter()
            .map(|(key_bytes, values)| (bytes_to_encode_id(&key_bytes), values))
            .collect();

        Ok(result)
    }

    /// Iterate over all key-value pairs in a flat structure.
    ///
    /// # Returns
    /// A vector of (EncodeID, V) pairs for each value in the store.
    pub fn iter_flat(&self) -> Result<Vec<(EncodeID, V)>, EncodeIDKVError> {
        let grouped = self.iter()?;
        let mut result = Vec::new();
        for (encode_id, values) in grouped {
            for value in values {
                result.push((encode_id.clone(), value));
            }
        }
        Ok(result)
    }

    /// Clear all entries from the store.
    pub fn clear(&self) -> Result<(), EncodeIDKVError> {
        // Collect all keys first
        let keys: Vec<Vec<u8>> = {
            let read_txn = self.db.begin_read()?;
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&self.table_name);
            let table = read_txn.open_multimap_table(table_def)?;
            let mut keys = Vec::new();
            for entry in table.iter()? {
                let (key_guard, _) = entry?;
                keys.push(key_guard.value().to_vec());
            }
            keys
        };

        // Remove all keys
        let write_txn = self.db.begin_write()?;
        {
            let table_def: MultimapTableDefinition<'_, &[u8], &[u8]> =
                MultimapTableDefinition::new(&self.table_name);
            let mut table = write_txn.open_multimap_table(table_def)?;
            for key in keys {
                table.remove_all(key.as_slice())?;
            }
        }
        write_txn.commit()?;

        Ok(())
    }
}

// Thread-safe: EncodeIDKVMultiStore can be sent between threads and shared safely
unsafe impl<V: Send> Send for EncodeIDKVMultiStore<V> {}
unsafe impl<V: Send + Sync> Sync for EncodeIDKVMultiStore<V> {}
