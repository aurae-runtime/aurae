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

use super::{
    validation::ValidatedCell, CellName, CellsError, Executable,
    ExecutableName, Result,
};
use cgroups_rs::{
    cgroup_builder::CgroupBuilder, hierarchies, Cgroup, Hierarchy,
};
use log::info;
use std::{collections::HashMap, process::ExitStatus};

#[derive(Debug)]
pub(crate) struct Cell {
    name: CellName,
    state: CellState,
}

#[derive(Debug)]
enum CellState {
    Unallocated(ValidatedCell),
    Allocated {
        cgroup: Cgroup,
        executables: HashMap<ExecutableName, Executable>,
    },
}

impl Cell {
    pub fn new(cell_spec: ValidatedCell) -> Self {
        Self {
            name: cell_spec.name.clone(),
            state: CellState::Unallocated(cell_spec),
        }
    }

    /// Creates the underlying cgroup. Does nothing if the `Cell` has already been allocated.
    // Here is where we define the "default" cgroup parameters for Aurae cells
    pub fn allocate(&mut self) {
        if let CellState::Unallocated(spec) = &self.state {
            self.state = {
                let ValidatedCell {
                    name,
                    cpu_cpus,
                    cpu_shares,
                    cpu_mems,
                    cpu_quota,
                } = (*spec).clone();

                let hierarchy = hierarchy();
                let cgroup = CgroupBuilder::new(&name)
                    // CPU Controller
                    .cpu()
                    .shares(cpu_shares.into_inner())
                    .mems(cpu_mems.into_inner())
                    .period(1000000) // microseconds in a second
                    .quota(cpu_quota.into_inner())
                    .cpus(cpu_cpus.into_inner())
                    .done()
                    // Final Build
                    .build(hierarchy);

                CellState::Allocated { cgroup, executables: Default::default() }
            }
        }
    }

    pub fn free(mut self) -> Result<()> {
        if let CellState::Allocated { cgroup, executables: _ } = &mut self.state
        {
            cgroup.delete().map_err(|e| CellsError::FailedToFreeCell {
                cell_name: self.name.clone(),
                source: e,
            })?;
        }

        Ok(())
    }

    pub fn start_executable<T: Into<Executable>>(
        &mut self,
        executable: T,
    ) -> Result<()> {
        match &mut self.state {
            CellState::Unallocated(_) => {
                Err(CellsError::CellNotFound { cell_name: self.name.clone() })
            }
            CellState::Allocated { cgroup, executables } => {
                let executable = executable.into();

                // TODO: replace with try_insert when it becomes stable
                // Check if there was already an executable with the same name.
                if executables.contains_key(&executable.name) {
                    return Err(CellsError::ExecutableExists {
                        cell_name: self.name.clone(),
                        executable_name: executable.name,
                    });
                }

                let executable_name = executable.name.clone();

                // Ignoring return value as we've already assured ourselves that the key does not exist.
                let _ = executables.insert(executable_name.clone(), executable);

                // Start the child process
                if let Some(executable) = executables.get_mut(&executable_name)
                {
                    let pid = executable.start().map_err(|e| {
                        CellsError::FailedToStartExecutable {
                            cell_name: self.name.clone(),
                            executable_name: executable.name.clone(),
                            command: executable.command.clone(),
                            args: executable.args.clone(),
                            source: e,
                        }
                    })?;

                    // TODO: We've inserted the executable into our in-memory cache, and started it,
                    //   but we've failed to move it to the Cell...bad...solution?
                    if let Err(e) = cgroup.add_task(pid.pid.into()) {
                        return Err(CellsError::FailedToAddExecutableToCell {
                            cell_name: self.name.clone(),
                            executable_name,
                            source: e,
                        });
                    }

                    info!(
                        "Cells: cell_name={} executable_name={executable_name} spawn() -> pid={pid:?}",
                        self.name
                    );
                } else {
                    unreachable!("executable is guaranteed to be in the HashMap; we just inserted and there is a MutexGuard");
                };

                Ok(())
            }
        }
    }

    pub fn stop_executable(
        &mut self,
        executable_name: &ExecutableName,
    ) -> Result<Option<ExitStatus>> {
        match &mut self.state {
            CellState::Unallocated(_) => {
                Err(CellsError::CellNotFound { cell_name: self.name.clone() })
            }
            CellState::Allocated { executables, .. } => {
                if let Some(executable) = executables.get_mut(executable_name) {
                    match executable.kill() {
                        Ok(exit_status) => {
                            let _ = executables
                                .remove(executable_name)
                                .expect("asserted above");

                            Ok(exit_status)
                        }
                        Err(e) => Err(CellsError::FailedToStopExecutable {
                            cell_name: self.name.clone(),
                            executable_name: executable.name.clone(),
                            executable_pid: executable.pid().expect("pid"),
                            source: e,
                        }),
                    }
                } else {
                    Err(CellsError::ExecutableNotFound {
                        cell_name: self.name.clone(),
                        executable_name: executable_name.clone(),
                    })
                }
            }
        }
    }

    /// Returns `None` if the `Cell` has not been allocated.
    pub fn v2(&self) -> Option<bool> {
        match &self.state {
            CellState::Allocated { cgroup, .. } => Some(cgroup.v2()),
            _ => None,
        }
    }
}

fn hierarchy() -> Box<dyn Hierarchy> {
    // Auraed will assume the V2 cgroup hierarchy by default.
    // For now we do not change this, albeit in theory we could
    // likely create backwards compatability for V1 hierarchy.
    //
    // For now, we simply... don't.
    // hierarchies::auto() // Uncomment to auto detect Cgroup hierarchy
    // hierarchies::V2
    Box::new(hierarchies::V2::new())
}
