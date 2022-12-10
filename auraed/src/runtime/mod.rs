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

#![allow(dead_code)]

use anyhow::Error;
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::*;
use log::{error, info};
use std::io;
use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct CellService {}

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
        let cgroup = create_cgroup(cell_name, cell.cpu_shares).expect("create");
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
        let cell_name = r.executable.expect("executable").cell_name;
        info!("CellService: start() cell_name={:?}", cell_name);
        Ok(Response::new(StartCellResponse {}))
    }

    async fn stop(
        &self,
        request: Request<StopCellRequest>,
    ) -> Result<Response<StopCellResponse>, Status> {
        let r = request.into_inner();
        let cell_name = r.cell_name;
        let executable_name = r.executable_name;
        info!(
            "CellService: stop() cell_name={:?} executable_name={:?}",
            cell_name, executable_name
        );
        Ok(Response::new(StopCellResponse {}))
    }
}

// Here is where we define the "default" cgroup parameters for Aurae cells
fn create_cgroup(id: &str, cpu_shares: u64) -> Result<Cgroup, Error> {
    let hierarchy = hierarchies::auto(); // v1/v2 cgroup switch automatically
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_remove_cgroup() {
        let id = "testing-aurae";
        let cgroup = create_cgroup(&id, 2).expect("create");
        println!("Created cgroup: {}", id);
        remove_cgroup(&id).expect("remove");
    }
}
