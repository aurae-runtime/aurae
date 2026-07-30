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

use super::{Cell, CellName, CellSpec, Result};

/// Common interface for both `Cells` (the daemon's cell collection) and
/// `Cell` (forwarding to its child collection). The recursive structure
/// of nested cells means we walk the same operations through whichever
/// type happens to hold the next level.
///
/// `allocate` and `free` are async because cell creation and teardown
/// perform rtnetlink and IPAM operations under a tokio executor.
/// `get`/`get_all` stay sync — they don't talk to the kernel.
///
/// `broadcast_free` and `broadcast_kill` aren't part of this trait —
/// they're inherent methods on `Cells` only, called once at daemon
/// shutdown rather than recursing through the cell tree.
pub(crate) trait CellsCache {
    /// Calls [Cell::allocate] on a new [Cell] and adds it to it's cache with key [CellName].
    ///
    /// # Errors
    /// * If cell exists -> [CellsError::CellExists]
    /// * If a cell is not in cache but cgroup exists on fs -> [CellsError::CgroupIsNotACell]
    /// * If cell fails to allocate (see [Cell::allocate])
    async fn allocate(
        &mut self,
        cell_name: CellName,
        cell_spec: CellSpec,
    ) -> Result<&Cell>;

    /// Calls [Cell::free] on a [Cell] and removes it from the cache.
    ///
    /// # Errors
    /// * If cell is not cached and cgroup does not exist -> [CellsError::CellNotFound]
    /// * If cell is cached and cgroup does not exist -> [CellsError::CgroupNotFound]
    ///     - note: cell will be removed from cache
    /// * If cell is not cached and cgroup exists on fs -> [CellsError::CgroupIsNotACell]
    /// * If cell fails to free (see [Cell::free])
    async fn free(&mut self, cell_name: &CellName) -> Result<()>;

    fn get<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: Fn(&Cell) -> Result<R>;

    fn get_all<F, R>(&self, f: F) -> Result<Vec<Result<R>>>
    where
        F: Fn(&Cell) -> Result<R>;
}
