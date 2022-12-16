/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use super::{Cell, CellError, CellName, CellServiceError, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// CellsTable is the in-memory store for the list of cells created with Aurae.
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
    /// Add the [cell] to the cache with key [cell_name].
    /// Returns an error if a duplicate [cell_name] already exists in the cache.
    pub(crate) fn insert(&self, cell_name: CellName, cell: Cell) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| CellServiceError::FailedToObtainLock())?;

        // TODO: replace with this when it becomes stable
        // cache.try_insert(cell_name.clone(), cgroup)

        // Check if there was already a cgroup in the table with this cell name as a key.
        if cache.contains_key(&cell_name) {
            return Err(CellError::CellExists { cell_name }.into());
        }
        // Ignoring return value as we've already assured ourselves that the key does not exist.
        let _ = cache.insert(cell_name, cell);
        Ok(())
    }

    pub(crate) fn contains(&self, cell_name: &CellName) -> Result<bool> {
        let cache = self
            .cache
            .lock()
            .map_err(|_| CellServiceError::FailedToObtainLock())?;

        Ok(cache.contains_key(cell_name))
    }

    pub(crate) fn get_then<F, R>(&self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: FnOnce(&mut Cell) -> Result<R>,
    {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| CellServiceError::FailedToObtainLock())?;

        if let Some(cell) = cache.get_mut(cell_name) {
            f(cell)
        } else {
            Err(CellError::CellNotFound { cell_name: cell_name.clone() }.into())
        }
    }

    /// Remove and return the cgroup keyed by [cell_name] from the cache.
    /// Returns an error if the cell_name does not exist in the cache.
    pub(crate) fn remove(&self, cell_name: &CellName) -> Result<Cell> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| CellServiceError::FailedToObtainLock())?;

        cache.remove(cell_name).ok_or_else(|| {
            CellError::CellNotFound { cell_name: cell_name.clone() }.into()
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
