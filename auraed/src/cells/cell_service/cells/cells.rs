/* -------------------------------------------------------------------------- *\
 *               Apache 2.0 License Copyright The Aurae Authors               *
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

use super::{cgroups::Cgroup, Cell, CellName, CellSpec, CellsError, Result};
use crate::cells::cell_service::cells::cells_cache::CellsCache;
use std::collections::HashMap;
use tracing::warn;

macro_rules! proxy_if_needed {
    ($self:ident, $cell_name:ident, $call:ident($($arg:ident),*), $expr:expr) => {
        if !$cell_name.is_child($self.parent.as_ref()) {
            // we are not in the direct parent
            let child_cell_name = match &$self.parent {
                None => $cell_name.to_root(),
                Some(parent) => parent.to_child(&$cell_name),
            };

            // we require that all ancestor cells exist
            let Some(child) = $self.cache.get_mut(&child_cell_name) else {
                                        return Err(CellsError::CellNotFound {
                                            cell_name: child_cell_name,
                                        })
                                    };

            CellsCache::$call(child, $($arg),*)
        } else {
            $expr
        }
    };
}

type Cache = HashMap<CellName, Cell>;

/// The in-memory cache of cells ([Cell]) created with Aurae.
#[derive(Debug, Default)]
pub struct Cells {
    parent: Option<CellName>,
    cache: Cache,
}

// TODO: add to the impl
// [x] Get Cgroup from cell_name
// [ ] Get Cgroup from executable_name
// [ ] Get Cgroup from pid
// [ ] Get Cgroup and pids from executable_name

impl Cells {
    pub fn new(parent: CellName) -> Self {
        Self { parent: Some(parent), ..Self::default() }
    }

    fn allocate(
        &mut self,
        cell_name: CellName,
        cell_spec: CellSpec,
    ) -> Result<&Cell> {
        proxy_if_needed!(self, cell_name, allocate(cell_name, cell_spec), {
            if Cgroup::exists(&cell_name) {
                return if self.cache.contains_key(&cell_name) {
                    Err(CellsError::CellExists { cell_name })
                } else {
                    Err(CellsError::CgroupIsNotACell {
                        cell_name: cell_name.clone(),
                    })
                };
            }

            // From here, we know the cgroup doesn't exist, so remove from cache if it does
            if let Some(_removed) = self.cache.remove(&cell_name) {
                // TODO: Should we not remove the cell (that has no cgroup) from the cache and
                //       force the user to call Free? Free will also return an error, but we may be
                //       calling other logic in free that we want to run.
                warn!("Found cached cell ('{cell_name}') without cgroup. Did you forget to call free on the cell?");
            }

            let cell = self
                .cache
                .entry(cell_name.clone())
                .or_insert_with(|| Cell::new(cell_name, cell_spec));

            cell.allocate()?;

            let cell = cell;
            Ok(cell)
        })
    }

    fn free(&mut self, cell_name: &CellName) -> Result<()> {
        proxy_if_needed!(self, cell_name, free(cell_name), {
            self.handle_cgroup_does_not_exist(cell_name)?;
            self.get_mut(cell_name, |cell| cell.free())?;
            let _ = self.cache.remove(cell_name);
            Ok(())
        })
    }

    fn get<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: Fn(&Cell) -> Result<R>,
    {
        proxy_if_needed!(self, cell_name, get(cell_name, f), {
            self.handle_cgroup_does_not_exist(cell_name)?;

            let Some(cell) = self.cache.get(cell_name) else {
                return Err(CellsError::CgroupIsNotACell { cell_name: cell_name.clone() });
            };

            let res = f(cell);

            if matches!(res, Err(CellsError::CellNotAllocated { .. })) {
                let _ = self.cache.remove(cell_name);
            }

            res
        })
    }

    fn get_mut<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: FnOnce(&mut Cell) -> Result<R>,
    {
        self.handle_cgroup_does_not_exist(cell_name)?;

        let Some(cell) = self.cache.get_mut(cell_name) else {
            return Err(CellsError::CgroupIsNotACell { cell_name: cell_name.clone() });
        };

        let res = f(cell);

        if matches!(res, Err(CellsError::CellNotAllocated { .. })) {
            let _ = self.cache.remove(cell_name);
        }

        res
    }

    fn handle_cgroup_does_not_exist(
        &mut self,
        cell_name: &CellName,
    ) -> Result<()> {
        if Cgroup::exists(cell_name) {
            return Ok(());
        }

        let Some(_removed) = self.cache.remove(cell_name) else {
            // Cell doesn't exist & cgroup doesn't exist
            return Err(CellsError::CellNotFound {
                cell_name: cell_name.clone(),
            });
        };

        // Cell exist, but cgroup doesn't
        Err(CellsError::CgroupNotFound { cell_name: cell_name.clone() })
    }

    fn broadcast_free(&mut self) {
        let freed_cells = self.do_broadcast(|cell| cell.free());

        for cell_name in freed_cells {
            let _ = self.cache.remove(&cell_name);
        }
    }

    fn broadcast_kill(&mut self) {
        let killed_cells = self.do_broadcast(|cell| cell.kill());

        for cell_name in killed_cells {
            let _ = self.cache.remove(&cell_name);
        }
    }

    fn do_broadcast<F>(&mut self, f: F) -> Vec<CellName>
    where
        F: Fn(&mut Cell) -> Result<()>,
    {
        self.cache
            .values_mut()
            .flat_map(|cell| {
                f(cell)?;

                // We clone here because we need a way to reference the cell for the loop
                // to remove it from the cache. Instead of cloning, we could make [Cell::state]
                // `pub(crate)` and check the state of the cell, removing the ones in the
                // [CellState::Freed] state, but that would expose internal functionality of the cell.
                // We could also create and `is_freed` fn on the cell.
                Ok::<_, CellsError>(cell.name().clone())
            })
            .collect()
    }
}

impl CellsCache for Cells {
    fn allocate(
        &mut self,
        cell_name: CellName,
        cell_spec: CellSpec,
    ) -> Result<&Cell> {
        self.allocate(cell_name, cell_spec)
    }

    fn free(&mut self, cell_name: &CellName) -> Result<()> {
        self.free(cell_name)
    }

    fn get<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: Fn(&Cell) -> Result<R>,
    {
        self.get(cell_name, f)
    }

    fn broadcast_free(&mut self) {
        self.broadcast_free()
    }

    fn broadcast_kill(&mut self) {
        self.broadcast_kill()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ignored: requires sudo, which we don't have in CI
    #[ignore]
    #[test]
    fn test_allocate() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name = CellName::random_for_tests();
        let cell = CellSpec::new_for_tests();

        let _ = cells.allocate(cell_name.clone(), cell).expect("allocate");
        assert!(cells.cache.contains_key(&cell_name));
    }

    // Ignored: requires sudo, which we don't have in CI
    #[ignore]
    #[test]
    fn test_duplicate_allocate_is_error() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name_in = CellName::random_for_tests();

        let cell_a = CellSpec::new_for_tests();
        let _ = cells
            .allocate(cell_name_in.clone(), cell_a)
            .expect("failed on first allocate");

        let cell_b = CellSpec::new_for_tests();
        assert!(matches!(
            cells.allocate(cell_name_in.clone(), cell_b),
            Err(CellsError::CellExists { cell_name }) if cell_name == cell_name_in
        ));
    }

    // Ignored: requires sudo, which we don't have in CI
    #[ignore]
    #[test]
    fn test_get() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name = CellName::random_for_tests();
        let cell = CellSpec::new_for_tests();
        let _ = cells
            .allocate(cell_name.clone(), cell)
            .expect("failed to allocate");

        cells.get(&cell_name, |_cell| Ok(())).expect("failed to get");
    }

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

    // Ignored: requires sudo, which we don't have in CI
    #[ignore]
    #[test]
    fn test_free() {
        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name = CellName::random_for_tests();
        let cell = CellSpec::new_for_tests();
        let _ = cells
            .allocate(cell_name.clone(), cell)
            .expect("failed to allocate");

        cells.free(&cell_name).expect("failed to free");
        assert!(cells.cache.is_empty());
    }

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
