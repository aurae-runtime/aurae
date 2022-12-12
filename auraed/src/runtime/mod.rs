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

use anyhow::{anyhow, Context, Error};
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse, Executable,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::*;
use log::info;
use std::collections::HashMap;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};

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
        let cell = r.cell.expect("cell");
        let cell_name = &cell.name;
        let cgroup = self
            .create_cgroup(cell_name, cell.cpu_shares)
            .expect("create cgroup");
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
        self.remove_cgroup(&cell_name).expect("remove cgroup");
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
        let child_table = self.child_table.clone();
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
        let mut cache = child_table.lock().expect("locking child_table mutex");
        let _ = cache.insert(cell_name, child);
        drop(cache);

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
        let child_table = self.child_table.clone();
        let mut cache = child_table.lock().expect("locking child_table mutex");
        let mut child = cache.remove(&cell_name).expect("getting child");
        info!(
            "CellService: stop() cell_name={:?} executable_name={:?} pid={:?}",
            &cell_name,
            &executable_name,
            &child.id()
        );
        let _ = child.kill().expect("killing child");
        let _ = child.wait().expect("waiting child");
        drop(cache);

        // Ok
        Ok(Response::new(StopCellResponse {}))
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
