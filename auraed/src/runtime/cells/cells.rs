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

use super::{Cell, CellName, CellsError, Result};
use std::collections::HashMap;

type Cache = HashMap<CellName, Cell>;

/// Cells is the in-memory store for the list of cells created with Aurae.
#[derive(Debug, Default)]
pub(crate) struct Cells {
    cache: Cache,
}

// TODO: add to the impl
// - Get Cgroup from cell_name
// - Get Cgroup from executable_name
// - Get Cgroup from pid
// - Get Cgroup and pids from executable_name

impl Cells {
    /// Add the [Cell] to the cache with key [CellName].
    /// Returns an error if a duplicate [CellName] already exists in the cache.
    pub fn allocate<T: Into<Cell>>(&mut self, cell: T) -> Result<&Cell> {
        let cell = cell.into();
        let cell_name = cell.name().clone();

        // TODO: replace with this when it becomes stable
        // cache.try_insert(cell_name.clone(), cgroup)

        // Check if there was already a cgroup in the table with this cell name as a key.
        if self.cache.contains_key(&cell_name) {
            return Err(CellsError::CellExists { cell_name });
        }

        // `or_insert` will always insert as we've already assured ourselves that the key does not exist.
        let cell = self.cache.entry(cell_name).or_insert(cell);
        cell.allocate();
        Ok(cell)
    }

    pub fn get<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: Fn(&Cell) -> Result<R>,
    {
        if let Some(cell) = self.cache.get(cell_name) {
            let res = f(cell);
            if matches!(res, Err(CellsError::CellUnallocated { .. })) {
                let _ = self.cache.remove(cell_name);
            }
            res
        } else {
            Err(CellsError::CellNotFound { cell_name: cell_name.clone() })
        }
    }

    pub fn get_mut<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: FnOnce(&mut Cell) -> Result<R>,
    {
        get_mut(&mut self.cache, cell_name, f)
    }

    /// Returns an error if the [CellName] does not exist in the cache.
    pub fn free(&mut self, cell_name: &CellName) -> Result<()> {
        get_mut(&mut self.cache, cell_name, |cell| cell.free())?;
        let _ = self.cache.remove(cell_name).ok_or_else(|| {
            CellsError::CellNotFound { cell_name: cell_name.clone() }
        })?;
        Ok(())
    }
}

// TODO: Someone make sure I'm (future-highway) making sense here.
/// auraed should never drop Cells unless it exits in which case, PID 1 is down anyway.
/// However, tests leave cells behind and this should clean them up.
impl Drop for Cells {
    fn drop(&mut self) {
        for cell in self.cache.values_mut() {
            let _ = cell.free();
        }
    }
}

fn get_mut<F, R>(cache: &mut Cache, cell_name: &CellName, f: F) -> Result<R>
where
    F: FnOnce(&mut Cell) -> Result<R>,
{
    if let Some(cell) = cache.get_mut(cell_name) {
        let res = f(cell);
        if matches!(res, Err(CellsError::CellUnallocated { .. })) {
            let _ = cache.remove(cell_name);
        }
        res
    } else {
        Err(CellsError::CellNotFound { cell_name: cell_name.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::cells::validation::ValidatedCell;
    use validation::ValidatedType;

    fn create_cell(name: Option<CellName>) -> Cell {
        let cell_name = name.unwrap_or_else(|| CellName::random_for_tests());

        let cell = aurae_proto::runtime::Cell {
            name: cell_name.into_inner(),
            cpu_cpus: "".to_string(),
            cpu_shares: 0,
            cpu_mems: "".to_string(),
            cpu_quota: 0,
            ns_share_mount: false,
            ns_share_uts: false,
            ns_share_ipc: false,
            ns_share_pid: false,
            ns_share_net: false,
            ns_share_cgroup: false,
        };
        let cell = ValidatedCell::validate(cell, None).expect("invalid cell");
        cell.into()
    }

    #[ignore]
    #[test]
    fn test_allocate() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell = create_cell(None);
        let cell_name = cell.name().clone();

        let _ = cells.allocate(cell).expect("allocate");
        assert!(cells.cache.contains_key(&cell_name));
    }

    #[ignore]
    #[test]
    fn test_duplicate_allocate_is_error() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name_in = CellName::random_for_tests();

        let cell_a = create_cell(Some(cell_name_in.clone()));
        let _ = cells.allocate(cell_a).expect("failed on first allocate");

        let cell_b = create_cell(Some(cell_name_in.clone()));
        assert!(matches!(
            cells.allocate(cell_b),
            Err(CellsError::CellExists { cell_name }) if cell_name == cell_name_in
        ));
    }

    #[ignore]
    #[test]
    fn test_get() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell = create_cell(None);
        let cell_name = cell.name().clone();
        let _ = cells.allocate(cell).expect("failed to allocate");

        cells.get(&cell_name, |_cell| Ok(())).expect("failed to get");
    }

    #[ignore]
    #[test]
    fn test_get_missing_errors() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name_in = CellName::random_for_tests();

        assert!(matches!(
            cells.get(&cell_name_in, |_cell| Ok(())),
            Err(CellsError::CellNotFound { cell_name }) if cell_name == cell_name_in
        ));
    }

    #[ignore]
    #[test]
    fn test_free() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell = create_cell(None);
        let cell_name = cell.name().clone();
        let _ = cells.allocate(cell).expect("failed to allocate");

        cells.free(&cell_name).expect("failed to free");
        assert!(cells.cache.is_empty());
    }

    #[ignore]
    #[test]
    fn test_free_missing_is_error() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name_in = CellName::random_for_tests();

        assert!(matches!(
            cells.free(&cell_name_in),
            Err(CellsError::CellNotFound { cell_name }) if cell_name == cell_name_in
        ));
    }
}
