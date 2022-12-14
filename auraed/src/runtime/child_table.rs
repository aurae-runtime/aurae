use crate::runtime::cell_name::CellName;
use crate::runtime::error::Result;
use anyhow::anyhow;
use std::{
    collections::HashMap,
    process::Child,
    sync::{Arc, Mutex},
};

/// ChildTable is the in-memory Arc<Mutex<HashMap<<>>> for the list of
/// child processes spawned with Aurae.
#[derive(Debug, Default, Clone)]
pub(crate) struct ChildTable {
    cache: Arc<Mutex<HashMap<CellName, Child>>>,
}

impl ChildTable {
    /// Store the [child] in the in-memory cache keyed by [cell_name].
    /// Note that this explicitly takes ownership of the Child which will be returned
    /// when it is removed from the cache.
    /// Returns an error if there is already a child keyed by that cell_name in
    /// the cache.
    pub(crate) fn insert(
        &self,
        cell_name: CellName,
        child: Child,
    ) -> Result<()> {
        // Cache the Child in ChildTable
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("failed to lock child cache"))?;

        // TODO: replace with this when it becomes stable
        // cache.try_insert(cell_name.clone(), child)

        // Check if there was already a cgroup in the table with this cell name as a key.
        if cache.contains_key(&cell_name) {
            return Err(anyhow!("child already exists for {cell_name}").into());
        }
        // Ignoring return value as we've already assured ourselves that the key does not exist.
        let _ = cache.insert(cell_name, child);
        Ok(())
    }

    /// Remove and return the Child process inserted with key [cell_name].
    /// Returns an error if the process cannot be found.
    pub(crate) fn remove(&self, cell_name: &CellName) -> Result<Child> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("failed to lock child cache"))?;
        cache.remove(cell_name).ok_or_else(|| {
            anyhow!("failed to find child for cell_name {cell_name}").into()
        })
    }
}

// TODO: Create an impl for ChildTable that exposes this functionality:
// - List all pids given a cell_name
// - List all pids given a cell_name and a more granular executable_name

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_insert() {
        let table = ChildTable::default();
        let child = Command::new("sleep")
            .arg("3000")
            .spawn()
            .expect("failed to execute child");
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table.insert("test".into(), child).expect("inserted in table");
        {
            let mut cache = table.cache.lock().expect("lock table");
            assert!(cache.contains_key(&"test".into()));
            cache.clear();
        }
    }

    #[test]
    fn test_duplicate_insert_is_error() {
        let table = ChildTable::default();
        let child = Command::new("sleep")
            .arg("3000")
            .spawn()
            .expect("failed to execute child");
        let child2 = Command::new("sleep")
            .arg("3000")
            .spawn()
            .expect("failed to execute child2");
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table.insert("test".into(), child).expect("inserted in table");
        assert!(table.insert("test".into(), child2).is_err());
        {
            let mut cache = table.cache.lock().expect("lock table");
            cache.clear();
        }
    }

    #[test]
    fn test_remove() {
        let table = ChildTable::default();
        let child = Command::new("sleep")
            .arg("3000")
            .spawn()
            .expect("failed to execute child");
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table.insert("test".into(), child).expect("inserted in table");
        let _ = table.remove(&"test".into()).expect("removed from table");
        {
            let mut cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
            cache.clear();
        }
    }

    #[test]
    fn test_remove_missing_is_error() {
        let table = ChildTable::default();
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        assert!(table.remove(&"test".into()).is_err());
        {
            let mut cache = table.cache.lock().expect("lock table");
            cache.clear();
        }
    }
}
