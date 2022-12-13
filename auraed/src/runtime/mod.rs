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
mod cgroup_table;
mod child_table;
mod cpu_cpus;
mod cpu_quota;
mod error;
mod executable_name;
mod validation;

use crate::runtime::cell_name::CellName;
use crate::runtime::executable_name::ExecutableName;
use crate::runtime::validation::{
    ValidatedAllocateCellRequest, ValidatedCell, ValidatedExecutable,
    ValidatedFreeCellRequest, ValidatedStartCellRequest,
    ValidatedStopCellRequest,
};
use crate::runtime::{
    cgroup_table::CgroupTable, child_table::ChildTable, error::CellServiceError,
};
use ::validation::ValidatedType;
use anyhow::{anyhow, Context, Error};
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use cgroups_rs::{cgroup_builder::CgroupBuilder, *};
use log::info;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;
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
    ) -> Result<Response<AllocateCellResponse>, Status> {
        // Initialize the cell
        let ValidatedAllocateCellRequest { cell } = request;

        info!("CellService: allocate() cell={:?}", cell);

        let cell_name = cell.name.clone();
        let cgroup = self.create_cgroup(cell).map_err(|e| {
            CellServiceError::Internal {
                msg: format!("failed to create cgroup for {cell_name}"),
                err: e.to_string(),
            }
        })?;

        Ok(Response::new(AllocateCellResponse {
            cell_name: cell_name.into_inner(),
            cgroup_v2: cgroup.v2(),
        }))
    }

    fn free(
        &self,
        request: ValidatedFreeCellRequest,
    ) -> Result<Response<FreeCellResponse>, Status> {
        let ValidatedFreeCellRequest { cell_name } = request;

        info!("CellService: free() cell_name={:?}", cell_name);
        self.remove_cgroup(&cell_name).map_err(|e| {
            CellServiceError::Internal {
                msg: format!("failed to remove cgroup for {cell_name}"),
                err: e.to_string(),
            }
        })?;

        Ok(Response::new(FreeCellResponse {}))
    }

    fn start(
        &self,
        request: ValidatedStartCellRequest,
    ) -> Result<Response<StartCellResponse>, Status> {
        let ValidatedStartCellRequest { executable } = request;

        let cgroup = Cgroup::load(
            hierarchy(),
            format!("/sys/fs/cgroup/{}", executable.cell_name),
        );

        // Create the new child process
        info!(
            "CellService: start() cell_name={} executable_name={} command={:?}",
            executable.cell_name, executable.name, executable.command
        );

        let ValidatedExecutable {
            name,
            mut command,
            description: _,
            cell_name,
        } = executable;

        // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
        let command = unsafe {
            command.pre_exec(move || CellService::aurae_process_pre_exec(&name))
        };

        // Start the child process
        let child =
            command.spawn().map_err(|e| CellServiceError::Internal {
                msg: "failed to spawn child process".into(),
                err: e.to_string(),
            })?;

        let cgroup_pid = CgroupPid::from(child.id() as u64);

        // Add the newly started child process to the cgroup
        cgroup.add_task(cgroup_pid).map_err(|e| {
            CellServiceError::Internal {
                msg: "failed to add child process to cgroup".into(),
                err: e.to_string(),
            }
        })?;
        info!("CellService: spawn() -> pid={:?}", &child.id());

        self.child_table.insert(cell_name.to_string(), child).map_err(|e| {
            CellServiceError::Internal {
                msg: format!("failed to insert {cell_name} into child_table"),
                err: e.to_string(),
            }
        })?;

        Ok(Response::new(StartCellResponse {}))
    }

    fn stop(
        &self,
        request: ValidatedStopCellRequest,
    ) -> Result<Response<StopCellResponse>, Status> {
        let ValidatedStopCellRequest { cell_name, executable_name } = request;

        let mut child = self.child_table.remove(&cell_name).map_err(|e| {
            CellServiceError::Internal {
                msg: format!(
                    "failed to remove child with cell_name {cell_name}"
                ),
                err: e.to_string(),
            }
        })?;

        let child_id = child.id();
        info!(
            "CellService: stop() cell_name={:?} executable_name={:?} pid={child_id}",
            cell_name,
            executable_name,
        );

        // TODO: check for
        child.kill().map_err(|e| CellServiceError::Internal {
            msg: format!("failed to kill child with pid {child_id}"),
            err: e.to_string(),
        })?;

        let exit_status =
            child.wait().map_err(|e| CellServiceError::Internal {
                msg: format!("failed to wait for child with pid {child_id}"),
                err: e.to_string(),
            })?;

        info!(
            "Child process with pid {child_id} exited with status {exit_status}",
        );

        Ok(Response::new(StopCellResponse {}))
    }

    // Here is where we define the "default" cgroup parameters for Aurae cells
    fn create_cgroup(&self, cell: ValidatedCell) -> Result<Cgroup, Error> {
        let ValidatedCell {
            name: _,
            cpu_cpus,
            cpu_shares,
            cpu_mems,
            cpu_quota,
        } = cell;

        let hierarchy = hierarchy();
        let cell_name = &cell.name;
        let cgroup: Cgroup = CgroupBuilder::new(cell_name)
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

        self.cgroup_table
            .insert(cell_name.to_string(), cgroup.clone())
            .map_err(|e| CellServiceError::Internal {
                msg: format!("failed to insert {cell_name} into cgroup_table"),
                err: e.to_string(),
            })?;

        Ok(cgroup)
    }

    fn remove_cgroup(&self, cell_name: &CellName) -> Result<(), Error> {
        self.cgroup_table
            .remove(cell_name)
            .map_err(|e| CellServiceError::Internal {
                msg: format!("failed to remove {cell_name} from cgroup_table"),
                err: e.to_string(),
            })?
            .delete()
            .context(format!("failed to delete {cell_name}"))
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
    ) -> Result<Response<AllocateCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedAllocateCellRequest::validate(request, None)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.allocate(request)
    }

    async fn free(
        &self,
        request: Request<FreeCellRequest>,
    ) -> Result<Response<FreeCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedFreeCellRequest::validate(request, None)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.free(request)
    }

    async fn start(
        &self,
        request: Request<StartCellRequest>,
    ) -> Result<Response<StartCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStartCellRequest::validate(request, None)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.start(request)
    }

    async fn stop(
        &self,
        request: Request<StopCellRequest>,
    ) -> Result<Response<StopCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStopCellRequest::validate(request, None)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
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
fn command_from_string(cmd: &str) -> Result<Command, Error> {
    let mut entries = cmd.split(' ');
    let base = match entries.next() {
        Some(base) => base,
        None => {
            return Err(anyhow!("empty base command string"));
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
        let id = CellName::new("testing-aurae-removal".into());
        // TODO: check error type with unwrap_err().kind()
        assert!(service.remove_cgroup(&id).is_err());
    }
}
