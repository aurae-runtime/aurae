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

mod error;

use crate::runtime::error::CellServiceError;
use anyhow::{anyhow, Context, Error};
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

mod cell_name;
mod free_cell;

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

/// CgroupTable is the in-memory store for the list of cgroups created with Aurae.
type CgroupTable = Arc<Mutex<HashMap<String, Cgroup>>>;

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

    fn aurae_process_pre_exec(exe: Executable) -> io::Result<()> {
        info!("CellService: aurae_process_pre_exec(): {}", exe.name);
        // Here we are executing as the new spawned pid.
        // This is a place where we can "hook" into all processes
        // started with Aurae in the future. Similar to kprobe/uprobe
        // in Linux or LD_PRELOAD in libc.
        Ok(())
    }

    // Here is where we define the "default" cgroup parameters for Aurae cells
    fn create_cgroup(
        &self,
        cell_name: &str,
        cpu_shares: u64,
    ) -> Result<Cgroup, Error> {
        let hierarchy = hierarchy();
        let cgroup: Cgroup = CgroupBuilder::new(cell_name)
            .cpu()
            .shares(cpu_shares) // Use a relative share of CPU compared to other cgroups
            .done()
            .build(hierarchy);

        let mut cgroup_cache =
            self.cgroup_table.lock().expect("lock cgroup_table");
        // Check if there was already a cgroup in the table with this cell name as a key.
        if let Some(_old_cgroup) =
            cgroup_cache.insert(cell_name.into(), cgroup.clone())
        {
            return Err(anyhow!("cgroup already exists for {cell_name}"));
        };
        Ok(cgroup)
    }

    fn remove_cgroup(&self, cell_name: &str) -> Result<(), Error> {
        let mut cgroup_cache =
            self.cgroup_table.lock().expect("lock cgroup table");
        let cgroup = cgroup_cache
            .remove(cell_name)
            .expect("find cell_name in cgroup_table");
        cgroup
            .delete()
            .context(format!("failed to delete {cell_name} from cgroup_table"))
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
            .ok_or(CellServiceError::MissingArgument { arg: "cell".into() })?;
        let cell_name = &cell.name;
        let cgroup =
            create_cgroup(cell_name, cell.cpu_shares).map_err(|e| {
                CellServiceError::Internal {
                    msg: format!("failed to create cgroup for {cell_name}"),
                    err: e.to_string(),
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
        remove_cgroup(&cell_name).map_err(|e| CellServiceError::Internal {
            msg: format!("failed to remove cgroup for {cell_name}"),
            err: e.to_string(),
        })?;
        Ok(Response::new(FreeCellResponse {}))
    }

    async fn start(
        &self,
        request: Request<StartCellRequest>,
    ) -> Result<Response<StartCellResponse>, Status> {
        let r = request.into_inner();
        let exe = r.executable.ok_or(CellServiceError::MissingArgument {
            arg: "executable".into(),
        })?;
        let exe_clone = exe.clone();
        let exe_command = exe.command;
        let cell_name = exe.cell_name;
        let cgroup =
            Cgroup::load(hierarchy(), format!("/sys/fs/cgroup/{}", cell_name));

        // Create the new child process
        info!("CellService: start() cell_name={cell_name} executable_name={:?} command={exe_command}", exe.name);
        let mut cmd = command_from_string(&exe_command).map_err(|e| {
            CellServiceError::Internal {
                msg: format!(
                    "failed to get command from string {}",
                    &exe_command
                ),
                err: e.to_string(),
            }
        })?;

        // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
        let post_cmd = unsafe {
            cmd.pre_exec(move || {
                CellService::aurae_process_pre_exec(exe_clone.clone())
            })
        };

        // Start the child process
        let child =
            post_cmd.spawn().map_err(|e| CellServiceError::Internal {
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

        // Cache the Child in ChildTable
        let mut cache = self.child_table.lock().map_err(|e| {
            CellServiceError::Internal {
                msg: "failed to lock child_table".into(),
                err: e.to_string(),
            }
        })?;

        // Check that we don't already have the child registered in the cache.
        if let Some(old_child) = cache.insert(cell_name.clone(), child) {
            return Err(CellServiceError::Internal {
                msg: format!(
                    "{} already exists in child_table with pid {:?}",
                    &cell_name,
                    old_child.id()
                ),
                err: "".into(),
            }
            .into());
        };

        Ok(Response::new(StartCellResponse {}))
    }

    async fn stop(
        &self,
        request: Request<StopCellRequest>,
    ) -> Result<Response<StopCellResponse>, Status> {
        let r = request.into_inner();
        let cell_name = r.cell_name;
        let executable_name = r.executable_name;
        let mut cache = self.child_table.lock().map_err(|e| {
            CellServiceError::Internal {
                msg: "failed to lock child table".into(),
                err: e.to_string(),
            }
        })?;
        let mut child =
            cache.remove(&cell_name).ok_or(CellServiceError::Internal {
                msg: format!("failed to find child for cell_name {cell_name}"),
                err: "".into(),
            })?;
        let child_id = child.id();
        info!(
            "CellService: stop() cell_name={:?} executable_name={:?} pid={child_id}",
            &cell_name,
            &executable_name,
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

        // Ok
        Ok(Response::new(StopCellResponse {}))
    }
}

// Here is where we define the "default" cgroup parameters for Aurae cells
fn create_cgroup(id: &str, cpu_shares: u64) -> Result<Cgroup, Error> {
    let hierarchy = hierarchy();
    let cgroup: Cgroup = CgroupBuilder::new(id)
        .cpu()
        .shares(cpu_shares) // Use x% of the CPU relative to other cgroups
        .done()
        .build(hierarchy);
    Ok(cgroup)
}

fn remove_cgroup(id: &str) -> Result<(), Error> {
    // TODO: create a cgroup_table mapping cell name to cgroup and do this instead
    //if let Err(err) = cgroup.delete() {
    //    return Err(Error::from(err))
    //}
    //Ok(())

    // The 'rmdir' command line tool from GNU coreutils calls the rmdir(2)
    // system call directly using the 'unistd.h' header file.

    // https://docs.rs/libc/latest/libc/fn.rmdir.html
    let path = std::ffi::CString::new(format!("/sys/fs/cgroup/{}", id))?;
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
    #[should_panic]
    fn test_attempt_to_remove_unknown_cgroup_fails() {
        let service = CellService::new();
        let id = "testing-aurae-removal";
        // TODO: check error type with unwrap_err().kind()
        assert!(service.remove_cgroup(id).is_err());
    }
}
