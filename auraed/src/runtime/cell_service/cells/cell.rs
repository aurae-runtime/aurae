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

use super::{CellName, CellSpec, CellsError, Cgroup, Result};
use crate::runtime::cell_service::executables::auraed::NestedAuraed;
use aurae_client::AuraeConfig;
use std::io;
use std::process::ExitStatus;
use tracing::info;

// We should not be able to change a cell after it has been created.
// You must free the cell and create a new one if you want to change anything about the cell.
// In order to facilitate that immutability:
// NEVER MAKE THE FIELDS PUB (OF ANY KIND)
#[derive(Debug)]
pub struct Cell {
    name: CellName,
    spec: CellSpec,
    state: CellState,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum CellState {
    Unallocated,
    Allocated { cgroup: Cgroup, nested_auraed: NestedAuraed },
    Freed,
}

impl Cell {
    pub fn new(name: CellName, cell_spec: CellSpec) -> Self {
        Self { name, spec: cell_spec, state: CellState::Unallocated }
    }

    /// Creates the underlying cgroup.
    /// Does nothing if [Cell] has been previously allocated.
    // Here is where we define the "default" cgroup parameters for Aurae cells
    pub fn allocate(&mut self) -> Result<()> {
        let CellState::Unallocated = &self.state else {
            return Ok(());
        };

        let cgroup: Cgroup =
            Cgroup::new(self.name.clone(), self.spec.cgroup_spec.clone());

        let auraed = NestedAuraed::new(&self.name, self.spec.iso_ctl.clone())
            .map_err(|e| CellsError::FailedToAllocateCell {
            cell_name: self.name.clone(),
            source: e,
        })?;

        let pid = auraed.pid();

        if let Err(e) = cgroup.add_task((pid.as_raw() as u64).into()) {
            // TODO: what if free also fails?
            let _ = self.free();

            return Err(CellsError::AbortedAllocateCell {
                cell_name: self.name.clone(),
                source: e,
            });
        }

        info!(
            "Attach nested Auraed pid {} to cgroup {}",
            pid.clone(),
            self.name.clone()
        );

        self.state = CellState::Allocated { cgroup, nested_auraed: auraed };

        Ok(())
    }

    /// Signals the [NestedAuraed] to gracefully shut down, and deletes the underlying cgroup.
    /// The [Cell::state] will be set to [CellState::Freed] regardless of it's state prior to this call.
    /// A [Cell] should never be reused once in the [CellState::Freed] state.
    pub fn free(&mut self) -> Result<()> {
        // TODO https://github.com/aurae-runtime/aurae/issues/199 &&
        //      aurae.io/signals, which is more accurate
        // TODO nested auraed should proxy (bus) POSIX signals to child executables
        self.do_free(|nested_auraed| nested_auraed.shutdown())
    }

    /// Sends a [SIGKILL] to the [NestedAuraed], and deletes the underlying cgroup.
    /// The [Cell::state] will be set to [CellState::Freed] regardless of it's state prior to this call.
    /// A [Cell] should never be reused once in the [CellState::Freed] state.
    pub fn kill(&mut self) -> Result<()> {
        self.do_free(|nested_auraed| nested_auraed.kill())
    }

    fn do_free<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(&mut NestedAuraed) -> io::Result<ExitStatus>,
    {
        if let CellState::Allocated { cgroup, nested_auraed } = &mut self.state
        {
            let _exit_status = f(nested_auraed).map_err(|e| {
                CellsError::FailedToKillCellChildren {
                    cell_name: self.name.clone(),
                    source: e,
                }
            })?;

            cgroup.delete().map_err(|e| CellsError::FailedToFreeCell {
                cell_name: self.name.clone(),
                source: e,
            })?;
        }

        // set cell state to freed, independent of the current state
        self.state = CellState::Freed;

        Ok(())
    }

    // NOTE: Having this function return the AuraeClient means we need to make it async,
    // or we need to make [AuraeClient::new] not async.
    pub fn client_config(&self) -> Result<AuraeConfig> {
        let CellState::Allocated { nested_auraed, .. } = &self.state else {
            return Err(CellsError::CellNotAllocated {
                cell_name: self.name.clone(),
            })
        };

        Ok(nested_auraed.client_config.clone())
    }

    /// Returns the [CellName] of the [Cell]
    pub fn name(&self) -> &CellName {
        &self.name
    }

    /// Returns [None] if the [Cell] is not allocated.
    pub fn v2(&self) -> Option<bool> {
        info!("{:?}", self);
        match &self.state {
            CellState::Allocated { cgroup, .. } => Some(cgroup.v2()),
            _ => None,
        }
    }
}

impl Drop for Cell {
    /// During normal behavior, cells are freed before being dropped,
    /// but cache reconciliation may result in a drop in other circumstances.
    /// Here we have a chance to clean up, no matter the circumstance.   
    fn drop(&mut self) {
        // We use kill here to be aggressive in cleaning up if anything has been left behind.
        let _best_effort = self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_cant_unfree() {
        let cell_name = CellName::random_for_tests();
        let mut cell = Cell::new(cell_name, CellSpec::new_for_tests());
        assert!(matches!(cell.state, CellState::Unallocated));

        cell.allocate().expect("failed to allocate");
        assert!(matches!(cell.state, CellState::Allocated { .. }));

        cell.free().expect("failed to free");
        assert!(matches!(cell.state, CellState::Freed));

        // Calling allocate again should do nothing
        cell.allocate().expect("failed to allocate 2");
        assert!(matches!(cell.state, CellState::Freed));
    }
}
