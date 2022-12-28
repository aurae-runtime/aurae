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
    cells::Cells,
    error::CellsServiceError,
    executables::Executables,
    validation::{
        ValidatedAllocateCellRequest, ValidatedExecutable,
        ValidatedFreeCellRequest, ValidatedStartExecutableRequest,
        ValidatedStopExecutableRequest,
    },
    Result,
};
use ::validation::ValidatedType;
use aurae_client::{runtime::cell_service::CellServiceClient, AuraeClient};
use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse, Executable,
    FreeCellRequest, FreeCellResponse, StartExecutableRequest,
    StartExecutableResponse, StopExecutableRequest, StopExecutableResponse,
};
use iter_tools::Itertools;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, Clone)]
pub struct CellService {
    cells: Arc<Mutex<Cells>>,
    executables: Arc<Mutex<Executables>>,
}

impl CellService {
    pub fn new() -> Self {
        CellService {
            cells: Default::default(),
            executables: Default::default(),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn allocate(
        &self,
        request: ValidatedAllocateCellRequest,
    ) -> Result<AllocateCellResponse> {
        // Initialize the cell
        let ValidatedAllocateCellRequest { cell } = request;
        let cell_name = cell.name.clone();
        let cell_spec = cell.into();

        let mut cells = self.cells.lock().await;
        let cell = cells.allocate(cell_name, cell_spec)?;

        Ok(AllocateCellResponse {
            cell_name: cell.name().clone().into_inner(),
            cgroup_v2: cell.v2().expect("allocated cell returns `Some`"),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn free(
        &self,
        request: ValidatedFreeCellRequest,
    ) -> Result<FreeCellResponse> {
        let ValidatedFreeCellRequest { cell_name } = request;

        info!("CellService: free() cell_name={:?}", cell_name);
        let mut cells = self.cells.lock().await;
        cells.free(&cell_name)?;

        Ok(FreeCellResponse::default())
    }

    #[tracing::instrument(skip(self))]
    async fn start(
        &self,
        request: ValidatedStartExecutableRequest,
    ) -> std::result::Result<Response<StartExecutableResponse>, Status> {
        let ValidatedStartExecutableRequest { mut cell_name, executable } =
            request;

        info!(
            "CellService: start() cell_name={:?} executable={:?}",
            cell_name, executable
        );

        if cell_name.is_empty() {
            // we are in the correct cell
            let mut executables = self.executables.lock().await;
            let executable = executables
                .start(executable)
                .map_err(CellsServiceError::ExecutablesError)?;

            let pid = executable.pid().expect("pid").as_raw();
            Ok(Response::new(StartExecutableResponse { pid }))
        } else {
            // we are in a parent cell
            let child_cell_name = cell_name.pop_front().expect("len > 0");

            let mut cells = self.cells.lock().await;
            let client_config = cells
                .get(&child_cell_name, move |cell| cell.client_config())
                .map_err(CellsServiceError::CellsError)?;

            // TODO: Handle error
            let client = AuraeClient::new(client_config)
                .await
                .expect("failed to create AuraeClient");

            // TODO: This seems wrong.
            //  1. We are turning our validated request back into a normal message (nested auraed will revalidate for no reason).
            //  2. We've lost all the original request's metadata
            let ValidatedExecutable { name, command, description } = executable;

            client
                .start(StartExecutableRequest {
                    cell_name: cell_name.iter().join("/"),
                    executable: Some(Executable {
                        name: name.into_inner(),
                        command: command.into_string().expect("valid string"),
                        description,
                    }),
                })
                .await
        }
    }

    #[tracing::instrument(skip(self))]
    async fn stop(
        &self,
        request: ValidatedStopExecutableRequest,
    ) -> std::result::Result<Response<StopExecutableResponse>, Status> {
        let ValidatedStopExecutableRequest { mut cell_name, executable_name } =
            request;

        info!(
            "CellService: stop() cell_name={:?} executable_name={:?}",
            cell_name, executable_name,
        );

        if cell_name.is_empty() {
            // we are in the correct cell
            let mut executables = self.executables.lock().await;
            let _exit_status = executables
                .stop(&executable_name)
                .map_err(CellsServiceError::ExecutablesError)?;

            Ok(Response::new(StopExecutableResponse::default()))
        } else {
            // we are in a parent cell
            let child_cell_name = cell_name.pop_front().expect("len > 0");

            let mut cells = self.cells.lock().await;
            let client_config = cells
                .get(&child_cell_name, move |cell| cell.client_config())
                .map_err(CellsServiceError::CellsError)?;

            // TODO: Handle error
            let client = AuraeClient::new(client_config)
                .await
                .expect("failed to create AuraeClient");

            client
                .stop(StopExecutableRequest {
                    cell_name: cell_name.iter().join("/"),
                    executable_name: executable_name.into_inner(),
                })
                .await
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
    ) -> std::result::Result<Response<AllocateCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedAllocateCellRequest::validate(request, None)?;
        Ok(Response::new(self.allocate(request).await?))
    }

    async fn free(
        &self,
        request: Request<FreeCellRequest>,
    ) -> std::result::Result<Response<FreeCellResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedFreeCellRequest::validate(request, None)?;
        Ok(Response::new(self.free(request).await?))
    }

    async fn start(
        &self,
        request: Request<StartExecutableRequest>,
    ) -> std::result::Result<Response<StartExecutableResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStartExecutableRequest::validate(request, None)?;
        Ok(self.start(request).await?)
    }

    async fn stop(
        &self,
        request: Request<StopExecutableRequest>,
    ) -> std::result::Result<Response<StopExecutableResponse>, Status> {
        let request = request.into_inner();
        let request = ValidatedStopExecutableRequest::validate(request, None)?;
        Ok(self.stop(request).await?)
    }
}
