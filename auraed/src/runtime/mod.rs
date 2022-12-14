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

mod cell_name;
mod cell_table;
mod cgroup_table;
mod child_table;
mod cpu_cpus;
mod cpu_quota;
mod error;
mod executable_name;
mod validation;

use crate::runtime::cell_name::CellName;
use crate::runtime::error::Result;
use crate::runtime::executable_name::ExecutableName;
use crate::runtime::validation::{
    ValidatedAllocateCellRequest, ValidatedCell, ValidatedExecutable,
    ValidatedFreeCellRequest, ValidatedStartCellRequest,
    ValidatedStopCellRequest,
};
use crate::runtime::{
    cgroup_table::CgroupTable, child_table::ChildTable, error::RuntimeError,
};
use ::validation::ValidatedType;
use anyhow::anyhow;
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use cgroups_rs::{cgroup_builder::CgroupBuilder, *};
use log::info;
use std::{io, os::unix::process::CommandExt, process::Command};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct CellService {
    child_table: ChildTable,
    cgroup_table: CgroupTable,
}

impl CellService {
    pub fn new() -> Self {
        CellService {
            child_table: Default::default(),
            cgroup_table: Default::default(),
        }
    }

    fn aurae_process_pre_exec(exe_name: &ExecutableName) -> io::Result<()> {
        info!("CellService: aurae_process_pre_exec(): {exe_name}");
        // Here we are executing as the new spawned pid.
        // This is a place where we can "hook" into all processes
        // started with Aurae in the future. Similar to kprobe/uprobe
        // in Linux or LD_PRELOAD in libc.
        Ok(())
    }

    fn allocate(
        &self,
        request: ValidatedAllocateCellRequest,
    ) -> std::result::Result<Response<AllocateCellResponse>, Status> {
        // Initialize the cell
        let ValidatedAllocateCellRequest { cell } = request;

        info!("CellService: allocate() cell={:?}", cell);

        let cell_name = cell.name.clone();
        let cgroup = self.create_cgroup(cell)?;

        Ok(Response::new(AllocateCellResponse {
            cell_name: cell_name.into_inner(),
            cgroup_v2: cgroup.v2(),
        }))
    }

    fn free(
        &self,
        request: ValidatedFreeCellRequest,
    ) -> std::result::Result<Response<FreeCellResponse>, Status> {
        let ValidatedFreeCellRequest { cell_name } = request;

        info!("CellService: free() cell_name={:?}", cell_name);
        self.remove_cgroup(&cell_name)?;

        Ok(Response::new(FreeCellResponse::default()))
    }

    fn start(
        &self,
        request: ValidatedStartCellRequest,
    ) -> std::result::Result<Response<StartCellResponse>, Status> {
        let ValidatedStartCellRequest { cell_name, executables } = request;

        for executable in executables {
            let ValidatedExecutable {
                name: executable_name,
                mut command,
                description: _,
            } = executable;

            // Create the new child process
            info!(
                "CellService: start() cell_name={} executable_name={} command={:?}",
                cell_name, executable_name, command
            );

            // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
            let command = {
                let executable_name_clone = executable_name.clone();
                unsafe {
                    command.pre_exec(move || {
                        CellService::aurae_process_pre_exec(
                            &executable_name_clone,
                        )
                    })
                }
            };

            // Start the child process
            let child = command.spawn()?;

            let cgroup =
                self.cgroup_table.get(&cell_name)?.ok_or_else(|| {
                    RuntimeError::Unallocated { resource: "cgroup".into() }
                })?;

            // Add the newly started child process to the cgroup
            let cgroup_pid = CgroupPid::from(child.id() as u64);
            cgroup.add_task(cgroup_pid).map_err(RuntimeError::from)?;

            info!(
                "CellService: cell_name={cell_name} executable_name={executable_name} spawn() -> pid={:?}",
                &child.id()
            );

            self.child_table.insert(cell_name.clone(), child)?;
        }

        Ok(Response::new(StartCellResponse::default()))
    }

    fn stop(
        &self,
        request: ValidatedStopCellRequest,
    ) -> std::result::Result<Response<StopCellResponse>, Status> {
        let ValidatedStopCellRequest { cell_name, executable_name } = request;

        let mut child = self.child_table.remove(&cell_name)?;

        let child_id = child.id();
        info!(
            "CellService: stop() cell_name={:?} executable_name={:?} pid={child_id}",
            cell_name,
            executable_name,
        );

        child.kill()?;

        let exit_status = child.wait()?;

        info!(
            "Child process with pid {child_id} exited with status {exit_status}",
        );

        Ok(Response::new(StopCellResponse::default()))
    }

    // Here is where we define the "default" cgroup parameters for Aurae cells
    fn create_cgroup(&self, cell: ValidatedCell) -> Result<Cgroup> {
        let ValidatedCell {
            name: cell_name,
            cpu_cpus,
            cpu_shares,
            cpu_mems,
            cpu_quota,
        } = cell;

        let hierarchy = hierarchy();
        let cgroup: Cgroup = CgroupBuilder::new(&cell_name)
            // CPU Controller
            .cpu()
            .shares(cpu_shares)
            .mems(cpu_mems)
            .period(1000000) // microseconds in a second
            .quota(cpu_quota.into_inner())
            .cpus(cpu_cpus.into_inner())
            .done()
            // Final Build
            .build(hierarchy);

        self.cgroup_table.insert(cell_name, cgroup.clone())?;

        Ok(cgroup)
    }

    fn remove_cgroup(&self, cell_name: &CellName) -> Result<()> {
        self.cgroup_table
            .remove(cell_name)?
            .delete()
            .map_err(RuntimeError::from)
    }
}

/// ### Mapping cgroup options to the Cell API
///
/// Here we *only* expose options from the CgroupBuilder
/// as our features in Aurae need them! We do not try to
/// "map" everything as much as we start with a few small
/// features and add as needed.
///
// Example builder options can be found: https://github.com/kata-containers/cgroups-rs/blob/main/tests/builder.rs
// Cgroup documentation: https://man7.org/linux/man-pages/man7/cgroups.7.html
#[tonic::async_trait]
impl cell_service_server::CellService for CellService {
    async fn allocate(
        &self,
        request: Request<AllocateCellRequest>,
    ) -> std::result::Result<Response<AllocateCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedAllocateCellRequest::validate(request, None)?;
        self.allocate(request)
    }

    async fn free(
        &self,
        request: Request<FreeCellRequest>,
    ) -> std::result::Result<Response<FreeCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedFreeCellRequest::validate(request, None)?;
        self.free(request)
    }

    async fn start(
        &self,
        request: Request<StartCellRequest>,
    ) -> std::result::Result<Response<StartCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStartCellRequest::validate(request, None)?;
        self.start(request)
    }

    async fn stop(
        &self,
        request: Request<StopCellRequest>,
    ) -> std::result::Result<Response<StopCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStopCellRequest::validate(request, None)?;
        self.stop(request)
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

/// A deterministic function used to take an arbitrary shell string and attempt
/// to convert to a Command which can be .spawn()'ed later.
fn command_from_string(cmd: &str) -> Result<Command> {
    let mut entries = cmd.split(' ');
    let base = match entries.next() {
        Some(base) => base,
        None => {
            return Err(anyhow!("empty base command string").into());
        }
    };

    let mut command = Command::new(base);
    for ent in entries {
        if ent != base {
            let _ = command.arg(ent);
        }
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: run this in a way that creating cgroups works
    #[test]
    fn test_create_remove_cgroup() {
        // let service = CellService::new();
        // let id = "testing-aurae";
        // let _cgroup = service.create_cgroup(id, 2).expect("create cgroup");
        // println!("Created cgroup: {}", id);
        // service.remove_cgroup(id).expect("remove cgroup");
    }

    #[test]
    fn test_attempt_to_remove_unknown_cgroup_fails() {
        let service = CellService::new();
        let cell_name = "testing-aurae-removal".into();
        // TODO: check error type with unwrap_err().kind()
        assert!(service.remove_cgroup(&cell_name).is_err());
    }
}
