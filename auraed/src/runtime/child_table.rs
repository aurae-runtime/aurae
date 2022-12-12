use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    process::Child,
    sync::{Arc, Mutex},
};

/// ChildTable is the in-memory Arc<Mutex<HashMap<<>>> for the list of
/// child processes spawned with Aurae.
#[derive(Debug, Default, Clone)]
pub(crate) struct ChildTable {
    cache: Arc<Mutex<HashMap<String, Child>>>,
}

impl ChildTable {
    /// Store the [child] in the in-memory cache keyed by [cell_name].
    /// Returns an error if there is already a child keyed by that cell_name in
    /// the cache.
    pub(crate) fn insert(
        &self,
        cell_name: &String,
        child: Child,
    ) -> Result<()> {
        // Cache the Child in ChildTable
        let mut cache = self.cache.lock().map_err(|e| anyhow!("{e:?}"))?;

        // Check that we don't already have the child registered in the cache.
        if let Some(old_child) = cache.insert(cell_name.clone(), child) {
            return Err(anyhow!(format!(
                "{} already exists in child_table with pid {:?}",
                &cell_name,
                old_child.id()
            )));
        };
        Ok(())
    }

    pub(crate) fn remove(&self, cell_name: &String) -> Result<Child> {
        let mut cache = self.cache.lock().map_err(|e| anyhow!("{e:?}"))?;
        cache.remove(cell_name).ok_or_else(|| {
            anyhow!("failed to find child for cell_name {cell_name}")
        })
    }
}

// TODO: Create an impl for ChildTable that exposes this functionality:
// - List all pids given a cell_name
// - List all pids given a cell_name and a more granular executable_name
