/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
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
                Some(parent) => parent.to_child(&$cell_name).expect("child CellName"),
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

            // TODO: Should we remove the cell from the cache here if the call to allocate fails?
            cell.allocate()?;

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
                return Err(CellsError::CgroupIsNotACell {
                    cell_name: cell_name.clone(),
                });
            };

            let res = f(cell);

            if matches!(res, Err(CellsError::CellNotAllocated { .. })) {
                let _ = self.cache.remove(cell_name);
            }

            res
        })
    }

    fn get_all<F, R>(&self, f: F) -> Result<Vec<Result<R>>>
    where
        F: Fn(&Cell) -> Result<R>,
    {
        Ok(self
            .cache
            .values()
            .filter_map(|cell| {
                let cell_name = cell.name();
                if !Cgroup::exists(cell_name) {
                    return None;
                };

                let res = f(cell);

                if matches!(res, Err(CellsError::CellNotAllocated { .. })) {
                    return None;
                }

                Some(res)
            })
            .collect())
    }

    fn get_mut<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: FnOnce(&mut Cell) -> Result<R>,
    {
        self.handle_cgroup_does_not_exist(cell_name)?;

        let Some(cell) = self.cache.get_mut(cell_name) else {
            return Err(CellsError::CgroupIsNotACell {
                cell_name: cell_name.clone(),
            });
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

    fn get_all<F, R>(&self, f: F) -> Result<Vec<Result<R>>>
    where
        F: Fn(&Cell) -> Result<R>,
    {
        self.get_all(f)
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
    use crate::{AuraedRuntime, AURAED_RUNTIME};
    use test_helpers::*;

    #[test]
    fn test_allocate() {
        skip_if_not_root!("test_allocate");
        // Docker's seccomp security profile (https://docs.docker.com/engine/security/seccomp/) blocks clone
        skip_if_seccomp!("test_cant_unfree");

        let _ = AURAED_RUNTIME.set(AuraedRuntime::default());

        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        let cell_name = CellName::random_for_tests();
        let cell = CellSpec::new_for_tests();

        let _ = cells.allocate(cell_name.clone(), cell).expect("allocate");
        assert!(cells.cache.contains_key(&cell_name));
    }

    #[test]
    fn test_duplicate_allocate_is_error() {
        skip_if_not_root!("test_duplicate_allocate_is_error");
        // Docker's seccomp security profile (https://docs.docker.com/engine/security/seccomp/) blocks clone
        skip_if_seccomp!("test_cant_unfree");

        let _ = AURAED_RUNTIME.set(AuraedRuntime::default());

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

    #[test]
    fn test_get() {
        skip_if_not_root!("test_get");
        // Docker's seccomp security profile (https://docs.docker.com/engine/security/seccomp/) blocks clone
        skip_if_seccomp!("test_get");

        let _ = AURAED_RUNTIME.set(AuraedRuntime::default());

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

    #[test]
    fn test_free() {
        skip_if_not_root!("test_free");
        // Docker's seccomp security profile (https://docs.docker.com/engine/security/seccomp/) blocks clone
        skip_if_seccomp!("test_free");

        let _ = AURAED_RUNTIME.set(AuraedRuntime::default());

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

    struct Graph {
        name: CellName,
        children: Vec<Self>,
    }

    #[test]
    fn test_cell_graph_triple_nested() {
        skip_if_not_root!("test_cell_graph_triple_nested");
        skip_if_seccomp!("test_cell_graph_triple_nested");

        let _ = AURAED_RUNTIME.set(AuraedRuntime::default());

        let mut cells = Cells::default();
        assert!(cells.cache.is_empty());

        // Create grandparent cell
        let grandparent_cell_name = CellName::random_for_tests();
        let grandparent_cell = CellSpec::new_for_tests();
        let _ = cells
            .allocate(grandparent_cell_name.clone(), grandparent_cell)
            .expect("failed to allocate");

        // Create parent cell
        let parent_cell_name =
            CellName::random_child_for_tests(&grandparent_cell_name);
        let parent_cell = CellSpec::new_for_tests();
        let _ = cells
            .allocate(parent_cell_name.clone(), parent_cell)
            .expect("failed to allocate");

        // Create child cell
        let child_cell_name =
            CellName::random_child_for_tests(&parent_cell_name);
        let child_cell = CellSpec::new_for_tests();
        let _ = cells
            .allocate(child_cell_name.clone(), child_cell)
            .expect("failed to allocate");

        fn cell_fn(cell: &Cell) -> Result<Graph> {
            Ok(Graph {
                name: cell.name().clone(),
                children: CellsCache::get_all(cell, cell_fn)
                    .expect("get all failed")
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .collect(),
            })
        }

        let cells = cells.get_all(cell_fn).expect("failed to get all cells");

        assert_eq!(cells.len(), 1);
        let grandparent_cell = cells[0].as_ref().unwrap();
        assert_eq!(grandparent_cell.name, grandparent_cell_name);
        assert_eq!(grandparent_cell.children.len(), 1);

        let parent_cell = &grandparent_cell.children[0];
        assert_eq!(parent_cell.name, parent_cell_name);
        assert_eq!(parent_cell.children.len(), 1);

        let child_cell = &parent_cell.children[0];
        assert_eq!(child_cell.name, child_cell_name);
        assert_eq!(child_cell.children.len(), 0);
    }
}