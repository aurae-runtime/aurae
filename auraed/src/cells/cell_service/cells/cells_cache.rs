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
/* -------------------------------------------------------------------------- *\
 *          Apache 2.0 License Copyright © 2022-2023 The Aurae Authors        *
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

use super::{Cell, CellName, CellSpec, Result};

pub trait CellsCache {
    /// Calls [Cell::allocate] on a new [Cell] and adds it to it's cache with key [CellName].
    ///
    /// # Errors
    /// * If cell exists -> [CellsError::CellExists]
    /// * If a cell is not in cache but cgroup exists on fs -> [CellsError::CgroupIsNotACell]
    /// * If cell fails to allocate (see [Cell::allocate])
    fn allocate(
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
    fn free(&mut self, cell_name: &CellName) -> Result<()>;

    fn get<F, R>(&mut self, cell_name: &CellName, f: F) -> Result<R>
    where
        F: Fn(&Cell) -> Result<R>;

    fn get_all<F, R>(&self, f: F) -> Result<Vec<Result<R>>>
    where
        F: Fn(&Cell) -> Result<R>;

    /// Calls [Cell::Free] on all cells in the cache, ignoring any errors.
    /// Successfully freed cells will be removed from the cache.
    fn broadcast_free(&mut self);

    /// Sends a [SIGKILL] to all Cells, ignoring any errors.
    fn broadcast_kill(&mut self);
}