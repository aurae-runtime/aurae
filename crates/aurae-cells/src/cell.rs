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

use crate::{CellName, CellSpec, CellsError, Result};
use aurae_client::AuraeConfig;
use aurae_executables::auraed::NestedAuraed;
use cgroups_rs::Cgroup;
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

// TODO: look into clippy warning
// TODO: remove #[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
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
            self.spec.cgroup_spec.clone().into_cgroup(&self.name);

        let auraed = NestedAuraed::new(self.spec.shared_namespaces.clone())
            .map_err(|e| CellsError::FailedToAllocateCell {
                cell_name: self.name.clone(),
                source: e,
            })?;

        let pid = auraed.pid();

        println!("auraed pid {}", pid);

        if let Err(e) = cgroup.add_task((pid.as_raw() as u64).into()) {
            // TODO: what if free also fails?
            let _ = self.free();

            return Err(CellsError::AbortedAllocateCell {
                cell_name: self.name.clone(),
                source: e,
            });
        }

        println!("inserted auraed pid {}", pid);

        self.state = CellState::Allocated { cgroup, nested_auraed: auraed };

        Ok(())
    }

    /// Deletes the underlying cgroup.
    /// A [Cell] should never be reused after calling [free].
    pub fn free(&mut self) -> Result<()> {
        // TODO In the future, use SIGINT intstead of SIGKILL once https://github.com/aurae-runtime/aurae/issues/199 is ready
        // TODO nested auraed should proxy (bus) POSIX signals to child executables
        if let CellState::Allocated { cgroup, nested_auraed } = &mut self.state
        {
            nested_auraed.kill().map_err(|e| {
                CellsError::FailedToKillCellChildren {
                    cell_name: self.name.clone(),
                    source: e,
                }
            })?;

            // TODO: The cgroup was made as {CellName}/_ to work around the limitations of v2 cgroups.
            //       But when we are deleting the cgroup, we are leaving behind a cgroup
            //       at {CellName}. We need to clean that up.
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

#[cfg(test)]
impl Drop for Cell {
    /// A [Cell] leaves a cgroup behind so we call [free] on drop
    fn drop(&mut self) {
        let _best_effort = self.free();
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
