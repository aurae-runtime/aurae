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

use crate::runtime::cells::executable::ExecutableError;
use crate::runtime::cells::{
    validation::ValidatedCell, CellName, Executable, ExecutableName,
};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::{hierarchies, Cgroup, Hierarchy};
use log::{error, info};
use std::collections::HashMap;
use std::process::{Command, ExitStatus};
use thiserror::Error;
use tonic::Status;

type Result<T> = std::result::Result<T, CellError>;

#[derive(Debug)]
pub(crate) struct Cell {
    name: CellName,
    cgroup: Cgroup,
    executables: HashMap<ExecutableName, Executable>,
}

impl Cell {
    // TODO: This fn signature ties cells module to runtime module (refactor to better solution)
    // Here is where we define the "default" cgroup parameters for Aurae cells
    pub fn allocate(cell_spec: ValidatedCell) -> Self {
        let ValidatedCell { name, cpu_cpus, cpu_shares, cpu_mems, cpu_quota } =
            cell_spec;

        let hierarchy = hierarchy();
        let cgroup: Cgroup = CgroupBuilder::new(&name)
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

        Self { name, cgroup, executables: Default::default() }
    }

    pub fn free(self) -> Result<()> {
        self.cgroup.delete().map_err(|e| CellError::FailedToFreeCell {
            cell_name: self.name.clone(),
            source: e,
        })?;

        Ok(())
    }

    pub fn start_executable(
        &mut self,
        exe_name: ExecutableName,
        command: Command,
        args: Vec<String>,
        description: String,
    ) -> Result<()> {
        // Check if there was already an executable with the same name.
        if self.executables.contains_key(&exe_name) {
            return Err(CellError::ExecutableExists {
                cell_name: self.name.clone(),
                executable_name: exe_name,
            });
        }

        // Start the child process
        let exe =
            Executable::start(exe_name.clone(), command, args, description)
                .map_err(|e| CellError::ExecutableError {
                    cell_name: self.name.clone(),
                    source: e,
                })?;

        // Add the newly started child process to the cgroup
        let exe_pid = exe.pid();
        if let Err(e) = self.cgroup.add_task(exe.pid()) {
            return Err(CellError::FailedToAddExecutable {
                cell_name: self.name.clone(),
                executable: exe,
                source: e,
            });
        }

        info!(
            "Cells: cell_name={} executable_name={exe_name} spawn() -> pid={}",
            self.name, exe_pid.pid
        );

        // Ignoring return value as we've already assured ourselves that the key does not exist.
        let _ = self.executables.insert(exe_name, exe);

        Ok(())
    }

    pub fn stop_executable(
        &mut self,
        exe_name: &ExecutableName,
    ) -> Result<ExitStatus> {
        if let Some(mut exe) = self.executables.remove(exe_name) {
            match exe.kill() {
                Ok(exit_status) => Ok(exit_status),
                Err(e) => {
                    // Failed to kill, put it back in cache
                    let _ = self.executables.insert(exe_name.clone(), exe);

                    Err(CellError::ExecutableError {
                        cell_name: self.name.clone(),
                        source: e,
                    })
                }
            }
        } else {
            Err(CellError::ExecutableNotFound {
                cell_name: self.name.clone(),
                executable_name: exe_name.clone(),
            })
        }
    }

    pub fn v2(&self) -> bool {
        self.cgroup.v2()
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

#[derive(Error, Debug)]
pub(crate) enum CellError {
    #[error("cell '{cell_name}' already exists'")]
    CellExists { cell_name: CellName },
    #[error("cell '{cell_name}' not found'")]
    CellNotFound { cell_name: CellName },
    #[error("cell '{cell_name}' could not be freed: {source}")]
    FailedToFreeCell { cell_name: CellName, source: cgroups_rs::error::Error },
    #[error(
        "cell '{cell_name}' already has an executable '{executable_name}'"
    )]
    ExecutableExists { cell_name: CellName, executable_name: ExecutableName },
    #[error("cell '{cell_name} could not find executable '{executable_name}'")]
    ExecutableNotFound { cell_name: CellName, executable_name: ExecutableName },
    #[error("cell '{cell_name}': {source}")]
    ExecutableError { cell_name: CellName, source: ExecutableError },
    #[error("cell '{cell_name}' failed to add executable (executable:?)")]
    FailedToAddExecutable {
        cell_name: CellName,
        executable: Executable,
        source: cgroups_rs::error::Error,
    },
}

impl From<CellError> for Status {
    fn from(err: CellError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellError::CellExists { .. }
            | CellError::ExecutableExists { .. } => Status::already_exists(msg),
            CellError::CellNotFound { .. }
            | CellError::ExecutableNotFound { .. } => Status::not_found(msg),
            // TODO (future-highway): I don't know what the conventions are of revealing
            //  messages that reveal the workings of the system to the api consumer
            //  in this type of application.
            //  For now, taking the safe route and not exposing the error messages for the below errors.
            CellError::ExecutableError { .. }
            | CellError::FailedToFreeCell { .. }
            | CellError::FailedToAddExecutable { .. } => Status::internal(""),
        }
    }
}
