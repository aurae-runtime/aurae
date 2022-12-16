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

use super::validation::{
    ValidatedAllocateCellRequest, ValidatedExecutable,
    ValidatedFreeCellRequest, ValidatedStartCellRequest,
    ValidatedStopCellRequest,
};
use super::{Cell, CellsTable, Result};
use crate::runtime::cells::error::CellsError;
use ::validation::ValidatedType;
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use log::info;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct CellService {
    cells: CellsTable,
}

impl CellService {
    pub fn new() -> Self {
        CellService { cells: Default::default() }
    }

    fn allocate(
        &self,
        request: ValidatedAllocateCellRequest,
    ) -> Result<AllocateCellResponse> {
        // Initialize the cell
        let ValidatedAllocateCellRequest { cell } = request;
        info!("CellService: allocate() cell={:?}", cell);

        if self.cells.contains(&cell.name)? {
            return Err(CellsError::CellExists { cell_name: cell.name });
        }

        let cell_name = cell.name.clone();

        // TODO: We allocate and then insert, which could fail, losing the ref to the cell
        let cell = Cell::allocate(cell);
        let cgroup_v2 = cell.v2();
        self.cells.insert(cell_name.clone(), cell)?;

        Ok(AllocateCellResponse {
            cell_name: cell_name.into_inner(),
            cgroup_v2,
        })
    }

    fn free(
        &self,
        request: ValidatedFreeCellRequest,
    ) -> Result<FreeCellResponse> {
        let ValidatedFreeCellRequest { cell_name } = request;

        info!("CellService: free() cell_name={:?}", cell_name);
        // TODO: We remove and then free, which could fail, losing the ref to the cell
        self.cells.remove(&cell_name)?.free()?;

        Ok(FreeCellResponse::default())
    }

    fn start(
        &self,
        request: ValidatedStartCellRequest,
    ) -> Result<StartCellResponse> {
        let ValidatedStartCellRequest { cell_name, executables } = request;

        for executable in executables {
            let ValidatedExecutable {
                name: executable_name,
                command,
                args,
                description,
            } = executable;

            // Create the new child process
            info!(
                "CellService: start() cell_name={} executable_name={} command={:?}",
                cell_name, executable_name, command
            );

            self.cells.get_then(&cell_name, move |cell| {
                cell.start_executable(
                    executable_name,
                    command,
                    args,
                    description,
                )
                .map_err(CellsError::from)
            })?;
        }

        Ok(StartCellResponse::default())
    }

    fn stop(
        &self,
        request: ValidatedStopCellRequest,
    ) -> Result<StopCellResponse> {
        let ValidatedStopCellRequest { cell_name, executable_name } = request;

        info!(
            "CellService: stop() cell_name={:?} executable_name={:?}",
            cell_name, executable_name,
        );

        let _exit_status = self.cells.get_then(&cell_name, move |cell| {
            cell.stop_executable(&executable_name).map_err(CellsError::from)
        })?;

        Ok(StopCellResponse::default())
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
        Ok(Response::new(self.allocate(request)?))
    }

    async fn free(
        &self,
        request: Request<FreeCellRequest>,
    ) -> std::result::Result<Response<FreeCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedFreeCellRequest::validate(request, None)?;
        Ok(Response::new(self.free(request)?))
    }

    async fn start(
        &self,
        request: Request<StartCellRequest>,
    ) -> std::result::Result<Response<StartCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStartCellRequest::validate(request, None)?;
        Ok(Response::new(self.start(request)?))
    }

    async fn stop(
        &self,
        request: Request<StopCellRequest>,
    ) -> std::result::Result<Response<StopCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStopCellRequest::validate(request, None)?;
        Ok(Response::new(self.stop(request)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::cells::CellName;

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
    fn test_attempt_to_remove_unknown_cell_fails() {
        let service = CellService::new();
        let cell_name = CellName::random();
        // TODO: check error type with unwrap_err().kind()
        assert!(service.free(ValidatedFreeCellRequest { cell_name }).is_err());
    }
}
