use anyhow::{anyhow, Result};
use cgroups_rs::Cgroup;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// CgroupTable is the in-memory store for the list of cgroups created with Aurae.
#[derive(Debug, Default, Clone)]
pub(crate) struct CgroupTable {
    cache: Arc<Mutex<HashMap<String, Cgroup>>>,
}

// TODO: add to the impl
// - Get Cgroup from cell_name
// - Get Cgroup from executable_name
// - Get Cgroup from pid
// - Get Cgroup and pids from executable_name

impl CgroupTable {
    /// Add the [cgroup] to the cache with key [cell_name].
    /// Note that this does not take ownership of the cgroup and instead clones it.
    /// The clone can be retrieved once it's removed from the cache.
    /// Returns an error if a duplicate [cell_name] already exists in the cache.
    pub(crate) fn insert(
        &self,
        cell_name: String,
        cgroup: Cgroup,
    ) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| anyhow!("failed to lock cgroup_table: {e:?}"))?;

        // TODO: replace with this when it becomes stable
        // cache.try_insert(cell_name.clone(), cgroup)

        // Check if there was already a cgroup in the table with this cell name as a key.
        if cache.contains_key(&cell_name) {
            return Err(anyhow!("cgroup already exists for {cell_name}"));
        }
        // Ignoring return value as we've already assured ourselves that the key does not exist.
        let _ = cache.insert(cell_name, cgroup);
        Ok(())
    }

    /// Return a clone of the cgroup keyed by [cell_name] from the cache or None if it is not found.
    /// Does not relinquish ownership.
    /// Returns an error if we fail to lock the cache.
    pub(crate) fn get(&self, cell_name: &str) -> Result<Option<Cgroup>> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| anyhow!("failed to lock cgroup_table: {e:?}"))?;
        let cgroup = cache.get(cell_name).cloned();
        Ok(cgroup)
    }

    /// Remove and return the cgroup keyed by [cell_name] from the cache.
    /// Returns an error if the cell_name does not exist in the cache.
    pub(crate) fn remove(&self, cell_name: &str) -> Result<Cgroup> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| anyhow!("failed to lock cgroup_table: {e:?}"))?;
        cache.remove(cell_name).ok_or_else(|| {
            anyhow!("failed to find {cell_name} in cgroup_table")
        })
    }
}

#[cfg(test)]
mod tests {
    use cgroups_rs::{cgroup_builder::CgroupBuilder, hierarchies};

    use super::*;

    #[test]
    fn test_insert() {
        let table = CgroupTable::default();
        let cgroup = CgroupBuilder::new("test-cell")
            .build(Box::new(hierarchies::V2::new()));
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table.insert("test".to_string(), cgroup).expect("inserted in table");
        {
            let mut cache = table.cache.lock().expect("lock table");
            assert!(cache.contains_key("test"));
            cache.clear();
        }
    }

    #[test]
    fn test_dublicate_insert_is_error() {
        let table = CgroupTable::default();
        let cgroup = CgroupBuilder::new("test-cell")
            .build(Box::new(hierarchies::V2::new()));
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table
            .insert("test".to_string(), cgroup.clone())
            .expect("inserted in table");
        assert!(table.insert("test".to_string(), cgroup).is_err());
        {
            let mut cache = table.cache.lock().expect("lock table");
            cache.clear();
        }
    }

    #[test]
    fn test_get() {
        let table = CgroupTable::default();
        let cgroup = CgroupBuilder::new("test-cell")
            .build(Box::new(hierarchies::V2::new()));
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table.insert("test".to_string(), cgroup).expect("inserted in table");
        assert!(table.get("test").expect("getting from cache").is_some());
        {
            let mut cache = table.cache.lock().expect("lock table");
            cache.clear();
        }
    }

    #[test]
    fn test_get_missing_returns_none() {
        let table = CgroupTable::default();
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }

        assert!(table.get("test").expect("getting from cache").is_none());
    }

    #[test]
    fn test_remove() {
        let table = CgroupTable::default();
        let cgroup = CgroupBuilder::new("test-cell")
            .build(Box::new(hierarchies::V2::new()));
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        table.insert("test".to_string(), cgroup).expect("inserted in table");
        let _ = table.remove("test").expect("removed from table");
        {
            let mut cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
            cache.clear();
        }
    }

    #[test]
    fn test_remove_missing_is_error() {
        let table = CgroupTable::default();
        {
            let cache = table.cache.lock().expect("lock table");
            assert!(cache.is_empty());
        }
        assert!(table.remove("test").is_err());
        {
            let mut cache = table.cache.lock().expect("lock table");
            cache.clear();
        }
    }
}
