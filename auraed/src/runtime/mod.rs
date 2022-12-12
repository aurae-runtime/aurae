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

use anyhow::{anyhow, Error};
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse, Executable,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::*;
use log::{error, info};
use std::collections::HashMap;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};

// TODO: Create an impl for ChildTable that exposes this functionality:
// - List all pids given a cell_name
// - List all pids given a cell_name and a more granular executable_name
// - Get Cgroup from cell_name
// - Get Cgroup from executable_name
// - Get Cgroup from pid
// - Get Cgroup and pids from exectuable_name

/// ChildTable is the in-memory Arc<Mutex<HashMap<<>>> for the list of
/// child processes spawned with Aurae.
type ChildTable = Arc<Mutex<HashMap<String, Child>>>;

#[derive(Debug, Clone)]
pub struct CellService {
    child_table: ChildTable,
}

impl CellService {
    pub fn new() -> Self {
        CellService { child_table: Default::default() }
    }

    fn aurae_process_pre_exec(exe: Executable) -> io::Result<()> {
        info!("CellService: aurae_process_pre_exec(): {}", exe.name);
        // Here we are executing as the new spawned pid.
        // This is a place where we can "hook" into all processes
        // started with Aurae in the future. Similar to kprobe/uprobe
        // in Linux or LD_PRELOAD in libc.
        Ok(())
    }
}

#[derive(Debug)]
pub enum CellServiceError {
    InvalidArgument { arg: String },
    Internal { msg: String },
}

impl From<CellServiceError> for Status {
    fn from(err: CellServiceError) -> Self {
        match err {
            CellServiceError::InvalidArgument { arg } => {
                Self::failed_precondition(format!(
                    "bad request. expected {arg}"
                ))
            }
            CellServiceError::Internal { msg } => {
                Self::internal(format!("internal error: {msg}"))
            }
        }
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
        // Initialize the cell
        let r = request.into_inner();
        let cell = r
            .cell
            .ok_or(CellServiceError::InvalidArgument { arg: "cell".into() })?;
        let cell_name = &cell.name;
        let cgroup =
            create_cgroup(cell_name, cell.cpu_shares).map_err(|e| {
                CellServiceError::Internal {
                    msg: format!(
                        "failed to create cgroup for {cell_name}: {e:?}"
                    ),
                }
            })?;
        info!("CellService: allocate() cell_name={:?}", cell_name);
        Ok(Response::new(AllocateCellResponse {
            cell_name: cell_name.to_string(),
            cgroup_v2: cgroup.v2(),
        }))
    }

    async fn free(
        &self,
        request: Request<FreeCellRequest>,
    ) -> Result<Response<FreeCellResponse>, Status> {
        // Initialize the cell
        let r = request.into_inner();
        let cell_name = r.cell_name;
        info!("CellService: free() cell_name={:?}", cell_name);
        remove_cgroup(&cell_name).expect("remove");
        Ok(Response::new(FreeCellResponse {}))
    }

    async fn start(
        &self,
        request: Request<StartCellRequest>,
    ) -> Result<Response<StartCellResponse>, Status> {
        let r = request.into_inner();
        let exe = r.executable.expect("executable");
        let exe_clone = exe.clone();
        let cell_name = exe.cell_name;
        let cgroup =
            Cgroup::load(hierarchy(), format!("/sys/fs/cgroup/{}", cell_name));

        // Create the new child process
        info!("CellService: start() cell_name={:?} executable_name={:?} command={:?}", cell_name, exe.name, exe.command);
        let mut cmd =
            command_from_string(&exe.command).expect("command from string");

        // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
        let post_cmd = unsafe {
            cmd.pre_exec(move || {
                CellService::aurae_process_pre_exec(exe_clone.clone())
            })
        };

        // Start the child process
        let child = post_cmd.spawn().expect("spawning command");
        let cgroup_pid = CgroupPid::from(child.id() as u64);

        // Add the newly started child process to the cgroup
        cgroup.add_task(cgroup_pid).expect("adding executable to cell");
        info!("CellService: spawn() -> pid={:?}", &child.id());

        // Cache the Child in ChildTable
        let mut cache =
            self.child_table.lock().expect("locking child_table mutex");
        let _ = cache
            .insert(cell_name, child)
            .expect("insert child to child_table");

        // Ok
        Ok(Response::new(StartCellResponse {}))
    }

    async fn stop(
        &self,
        request: Request<StopCellRequest>,
    ) -> Result<Response<StopCellResponse>, Status> {
        let r = request.into_inner();
        let cell_name = r.cell_name;
        let executable_name = r.executable_name;
        let mut cache =
            self.child_table.lock().expect("locking child_table mutex");
        let mut child =
            cache.remove(&cell_name).expect("removing and retrieving child");
        info!(
            "CellService: stop() cell_name={:?} executable_name={:?} pid={:?}",
            &cell_name,
            &executable_name,
            &child.id()
        );
        child.kill().expect("killing child");
        // TODO: something with the exit status
        let _ = child.wait().expect("waiting child");

        // Ok
        Ok(Response::new(StopCellResponse {}))
    }
}

// Here is where we define the "default" cgroup parameters for Aurae cells
fn create_cgroup(id: &str, cpu_shares: u64) -> Result<Cgroup, Error> {
    let hierarchy = hierarchy();
    let cgroup: Cgroup = CgroupBuilder::new(id)
        .cpu()
        .shares(cpu_shares) // Use 10% of the CPU relative to other cgroups
        .done()
        .build(hierarchy);
    Ok(cgroup)
}

fn remove_cgroup(id: &str) -> Result<(), Error> {
    // The 'rmdir' command line tool from GNU coreutils calls the rmdir(2)
    // system call directly using the 'unistd.h' header file.

    // https://docs.rs/libc/latest/libc/fn.rmdir.html
    let path = std::ffi::CString::new(format!("/sys/fs/cgroup/{}", id))
        .expect("valid CString");
    let ret = unsafe { libc::rmdir(path.as_ptr()) };
    if ret < 0 {
        let error = io::Error::last_os_error();
        error!("Failed to remove cgroup ({})", error);
        Err(Error::from(error))
    } else {
        Ok(())
    }
}

fn hierarchy() -> Box<dyn Hierarchy> {
    hierarchies::auto() // v1/v2 cgroup switch automatically
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

    #[test]
    fn test_create_remove_cgroup() {
        let id = "testing-aurae";
        let _cgroup = create_cgroup(id, 2).expect("create");
        println!("Created cgroup: {}", id);
        remove_cgroup(id).expect("remove");
    }
}
