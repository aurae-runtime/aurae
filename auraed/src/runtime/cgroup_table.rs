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
// - Get Cgroup and pids from exectuable_name

impl CgroupTable {
    pub(crate) fn insert(
        &self,
        cell_name: &str,
        cgroup: &Cgroup,
    ) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| anyhow!("failed to lock cgroup_table: {e:?}"))?;
        // Check if there was already a cgroup in the table with this cell name as a key.
        if let Some(_old_cgroup) =
            cache.insert(cell_name.into(), cgroup.clone())
        {
            return Err(anyhow!("cgroup already exists for {cell_name}"));
        };
        Ok(())
    }

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
