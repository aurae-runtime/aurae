use crate::runtime::cells::{Cell, CellName};
use crate::runtime::error::Result;
use anyhow::anyhow;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// CgroupTable is the in-memory store for the list of cgroups created with Aurae.
#[derive(Debug, Default, Clone)]
pub(crate) struct CellsTable {
    // TODO (future-highway): would a RWLock be more performant?
    cache: Arc<Mutex<HashMap<CellName, Cell>>>,
}

// TODO: add to the impl
// - Get Cgroup from cell_name
// - Get Cgroup from executable_name
// - Get Cgroup from pid
// - Get Cgroup and pids from executable_name

impl CellsTable {
    /// Add the [cgroup] to the cache with key [cell_name].
    /// Note that this does not take ownership of the cgroup and instead clones it.
    /// The clone can be retrieved once it's removed from the cache.
    /// Returns an error if a duplicate [cell_name] already exists in the cache.
    pub(crate) fn insert(&self, cell_name: CellName, cell: Cell) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("failed to lock cells cache"))?;

        // TODO: replace with this when it becomes stable
        // cache.try_insert(cell_name.clone(), cgroup)

        // Check if there was already a cgroup in the table with this cell name as a key.
        if cache.contains_key(&cell_name) {
            return Err(anyhow!("cell '{cell_name}' already exists").into());
        }
        // Ignoring return value as we've already assured ourselves that the key does not exist.
        let _ = cache.insert(cell_name, cell);
        Ok(())
    }

    pub(crate) fn contains(&self, cell_name: &CellName) -> Result<bool> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| anyhow!("failed to lock cells cache: {e:?}"))?;

        Ok(cache.contains_key(cell_name))
    }

    // /// Return a clone of the cgroup keyed by [cell_name] from the cache or None if it is not found.
    // /// Does not relinquish ownership.
    // /// Returns an error if we fail to lock the cache.
    // pub(crate) fn get(&self, cell_name: &CellName) -> Result<Option<Cgroup>> {
    //     let cache = self
    //         .cache
    //         .lock()
    //         .map_err(|e| anyhow!("failed to lock cgroup_table: {e:?}"))?;
    //     let cgroup = cache.get(cell_name).cloned();
    //     Ok(cgroup)
    // }

    pub(crate) fn get_then<F, R>(&self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: FnOnce(&mut Cell) -> Result<R>,
    {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| anyhow!("failed to lock cgroup_table: {e:?}"))?;

        if let Some(cell) = cache.get_mut(cell_name) {
            f(cell)
        } else {
            Err(anyhow!("failed to find '{cell_name}' in cells").into())
        }
    }

    /// Remove and return the cgroup keyed by [cell_name] from the cache.
    /// Returns an error if the cell_name does not exist in the cache.
    pub(crate) fn remove(&self, cell_name: &CellName) -> Result<Cell> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("failed to lock cgroup cache"))?;
        cache.remove(cell_name).ok_or_else(|| {
            anyhow!("failed to find '{cell_name}' in cells").into()
        })
    }
}

#[cfg(test)]
mod tests {
    // TODO (future-highway): These tests need to be updated.
    // use cgroups_rs::{cgroup_builder::CgroupBuilder, hierarchies};
    //
    // use super::*;

    // #[test]
    // fn test_insert() {
    //     let table = CellsTable::default();
    //     let cgroup = CgroupBuilder::new("test-cell")
    //         .build(Box::new(hierarchies::V2::new()));
    //     {
    //         let cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //     }
    //     table.insert("test".into(), cgroup).expect("inserted in table");
    //     {
    //         let mut cache = table.cache.lock().expect("lock table");
    //         assert!(cache.contains_key(&"test".into()));
    //         cache.clear();
    //     }
    // }
    //
    // #[test]
    // fn test_duplicate_insert_is_error() {
    //     let table = CellsTable::default();
    //     let cgroup = CgroupBuilder::new("test-cell")
    //         .build(Box::new(hierarchies::V2::new()));
    //     {
    //         let cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //     }
    //     table.insert("test".into(), cgroup.clone()).expect("inserted in table");
    //     assert!(table.insert("test".into(), cgroup).is_err());
    //     {
    //         let mut cache = table.cache.lock().expect("lock table");
    //         cache.clear();
    //     }
    // }
    //
    // #[test]
    // fn test_get() {
    //     let table = CellsTable::default();
    //     let cgroup = CgroupBuilder::new("test-cell")
    //         .build(Box::new(hierarchies::V2::new()));
    //     {
    //         let cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //     }
    //     table.insert("test".into(), cgroup).expect("inserted in table");
    //     assert!(table
    //         .get(&"test".into())
    //         .expect("getting from cache")
    //         .is_some());
    //     {
    //         let mut cache = table.cache.lock().expect("lock table");
    //         cache.clear();
    //     }
    // }
    //
    // #[test]
    // fn test_get_missing_returns_none() {
    //     let table = CellsTable::default();
    //     {
    //         let cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //     }
    //
    //     assert!(table
    //         .get(&"test".into())
    //         .expect("getting from cache")
    //         .is_none());
    // }
    //
    // #[test]
    // fn test_remove() {
    //     let table = CellsTable::default();
    //     let cgroup = CgroupBuilder::new("test-cell")
    //         .build(Box::new(hierarchies::V2::new()));
    //     {
    //         let cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //     }
    //     table.insert("test".into(), cgroup).expect("inserted in table");
    //     let _ = table.remove(&"test".into()).expect("removed from table");
    //     {
    //         let mut cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //         cache.clear();
    //     }
    // }
    //
    // #[test]
    // fn test_remove_missing_is_error() {
    //     let table = CellsTable::default();
    //     {
    //         let cache = table.cache.lock().expect("lock table");
    //         assert!(cache.is_empty());
    //     }
    //     assert!(table.remove(&"test".into()).is_err());
    //     {
    //         let mut cache = table.cache.lock().expect("lock table");
    //         cache.clear();
    //     }
    // }
}
