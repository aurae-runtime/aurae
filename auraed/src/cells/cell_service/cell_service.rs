/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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
    cells::cgroups::{Limit, Weight},
    cells::{CellName, Cells, CellsCache, GraphNode},
    error::CellsServiceError,
    executables::Executables,
    validation::{
        ValidatedCellServiceAllocateRequest, ValidatedCellServiceFreeRequest,
        ValidatedCellServiceStartRequest, ValidatedCellServiceStopRequest,
    },
    Result,
};
use ::validation::ValidatedType;
use aurae_client::{
    cells::cell_service::CellServiceClient, AuraeClient, AuraeClientError,
};
use aurae_proto::cells::{
    cell_service_server, Cell, CellGraphNode, CellServiceAllocateRequest,
    CellServiceAllocateResponse, CellServiceFreeRequest,
    CellServiceFreeResponse, CellServiceListRequest, CellServiceListResponse,
    CellServiceStartRequest, CellServiceStartResponse, CellServiceStopRequest,
    CellServiceStopResponse, CpuController, CpusetController,
};
use backoff::backoff::Backoff;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status};
use tracing::{info, trace};

macro_rules! do_in_cell {
    ($self:ident, $cell_name:ident, $function:ident, $request:ident) => {{
        let mut cells = $self.cells.lock().await;

        let client_config = cells
            .get(&$cell_name, |cell| cell.client_config())
            .map_err(CellsServiceError::CellsError)?;

        let mut retry_strategy = backoff::ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_millis(50)) // 1st retry in 50ms
            .with_multiplier(10.0) // 10x the delay after 1st retry (500ms)
            .with_randomization_factor(0.5) // with a randomness of +/-50% (250-750ms)
            .with_max_interval(Duration::from_secs(3)) // but never delay more than 3s
            .with_max_elapsed_time(Some(Duration::from_secs(20))) // or 20s total
            .build();

        let client = loop {
            match AuraeClient::new(client_config.clone()).await {
                Ok(client) => break Ok(client),
                e @ Err(AuraeClientError::ConnectionError(_)) => {
                    trace!("aurae client failed to connect: {e:?}");
                    if let Some(delay) = retry_strategy.next_backoff() {
                        trace!("retrying in {delay:?}");
                        tokio::time::sleep(delay).await
                    } else {
                        break e
                    }
                }
                e => break e
            }
        }.map_err(CellsServiceError::from)?;

        backoff::future::retry(
            retry_strategy,
            || async {
                match client.$function($request.clone()).await {
                    Ok(res) => Ok(res),
                    Err(e) if e.code() == Code::Unknown && e.message() == "transport error" => {
                        Err(e)?;
                        unreachable!();
                    }
                    Err(e) => Err(backoff::Error::Permanent(e))
                }
            },
        )
        .await
    }};
}

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
        request: ValidatedCellServiceAllocateRequest,
    ) -> Result<CellServiceAllocateResponse> {
        // Initialize the cell
        let ValidatedCellServiceAllocateRequest { cell } = request;

        let cell_name = cell.name.clone();
        let cell_spec = cell.into();

        let mut cells = self.cells.lock().await;
        let cell = cells.allocate(cell_name, cell_spec)?;

        Ok(CellServiceAllocateResponse {
            cell_name: cell.name().clone().to_string(),
            cgroup_v2: cell.v2().expect("allocated cell returns `Some`"),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn free(
        &self,
        request: ValidatedCellServiceFreeRequest,
    ) -> Result<CellServiceFreeResponse> {
        let ValidatedCellServiceFreeRequest { cell_name } = request;

        info!("CellService: free() cell_name={cell_name:?}");

        let mut cells = self.cells.lock().await;
        cells.free(&cell_name)?;

        Ok(CellServiceFreeResponse::default())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn free_all(&self) -> Result<()> {
        let mut cells = self.cells.lock().await;

        // First try to gracefully free all cells.
        cells.broadcast_free();
        // The cells that remain failed to shut down for some reason.
        cells.broadcast_kill();

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn start(
        &self,
        request: ValidatedCellServiceStartRequest,
    ) -> std::result::Result<Response<CellServiceStartResponse>, Status> {
        let ValidatedCellServiceStartRequest { cell_name, executable } =
            request;

        assert!(matches!(cell_name, None));
        info!("CellService: start() executable={:?}", executable);

        let mut executables = self.executables.lock().await;
        let executable = executables
            .start(executable)
            .map_err(CellsServiceError::ExecutablesError)?;

        let pid = executable
            .pid()
            .map_err(CellsServiceError::Io)?
            .expect("pid")
            .as_raw();

        // TODO: either tell the [ObserveService] about this executable's log channels, or
        // provide a way for the observe service to extract the log channels from here.

        Ok(Response::new(CellServiceStartResponse { pid }))
    }

    #[tracing::instrument(skip(self))]
    async fn start_in_cell(
        &self,
        cell_name: &CellName,
        request: CellServiceStartRequest,
    ) -> std::result::Result<Response<CellServiceStartResponse>, Status> {
        do_in_cell!(self, cell_name, start, request)
    }

    #[tracing::instrument(skip(self))]
    async fn stop(
        &self,
        request: ValidatedCellServiceStopRequest,
    ) -> std::result::Result<Response<CellServiceStopResponse>, Status> {
        let ValidatedCellServiceStopRequest { cell_name, executable_name } =
            request;

        assert!(matches!(cell_name, None));
        info!("CellService: stop() executable_name={:?}", executable_name,);

        let mut executables = self.executables.lock().await;
        let _exit_status = executables
            .stop(&executable_name)
            .await
            .map_err(CellsServiceError::ExecutablesError)?;

        Ok(Response::new(CellServiceStopResponse::default()))
    }

    #[tracing::instrument(skip(self))]
    async fn stop_in_cell(
        &self,
        cell_name: &CellName,
        request: CellServiceStopRequest,
    ) -> std::result::Result<Response<CellServiceStopResponse>, Status> {
        do_in_cell!(self, cell_name, stop, request)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn stop_all(&self) -> Result<()> {
        let mut executables = self.executables.lock().await;
        executables.broadcast_stop().await;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list(&self) -> Result<CellServiceListResponse> {
        let cells = self.cells.lock().await;
        let graph =
            cells.cell_graph(&GraphNode { cell_info: None, children: vec![] });

        Ok(CellServiceListResponse {
            cells: graph.children.iter().map(map_graph_response).collect(),
        })
    }
}

fn map_graph_response(node: &GraphNode) -> CellGraphNode {
    let cell_info = node
        .cell_info
        .as_ref()
        .expect("cell_info should be present for every non-root node");

    let response = CellGraphNode {
        cell: Some(Cell {
            name: cell_info.cell_name.to_string(),
            cpu: cell_info.cpu.as_ref().map(|cpu| CpuController {
                weight: cpu.weight.map(Weight::into_inner),
                max: cpu.max.map(Limit::into_inner),
            }),
            cpuset: cell_info.cpuset.as_ref().map(|cpuset| CpusetController {
                cpus: cpuset.cpus.as_ref().map(|c| c.to_string()),
                mems: cpuset.mems.as_ref().map(|m| m.to_string()),
            }),
            isolate_network: cell_info.isolate_network,
            isolate_process: cell_info.isolate_process,
        }),
        children: node.children.iter().map(map_graph_response).collect(),
    };
    response
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
        request: Request<CellServiceAllocateRequest>,
    ) -> std::result::Result<Response<CellServiceAllocateResponse>, Status>
    {
        let request = request.into_inner();
        let request = ValidatedCellServiceAllocateRequest::validate(
            request.clone(),
            None,
        )?;

        Ok(Response::new(self.allocate(request).await?))
    }

    async fn free(
        &self,
        request: Request<CellServiceFreeRequest>,
    ) -> std::result::Result<Response<CellServiceFreeResponse>, Status> {
        let request = request.into_inner();
        let request =
            ValidatedCellServiceFreeRequest::validate(request.clone(), None)?;

        Ok(Response::new(self.free(request).await?))
    }

    async fn start(
        &self,
        request: Request<CellServiceStartRequest>,
    ) -> std::result::Result<Response<CellServiceStartResponse>, Status> {
        let request = request.into_inner();

        // We execute start if cell_name is none
        if request.cell_name.is_none() {
            let request =
                ValidatedCellServiceStartRequest::validate(request, None)?;
            Ok(self.start(request).await?)
        } else {
            // We are in a parent cell (or validation will fail)
            let validated = ValidatedCellServiceStartRequest::validate(
                request.clone(),
                None,
            )?;

            // validation has succeeded, so we can make assumptions about the request and use expect
            let cell_name = validated.cell_name.expect("cell name");
            let mut request = request;
            request.cell_name = None;
            self.start_in_cell(&cell_name, request).await
        }
    }

    async fn stop(
        &self,
        request: Request<CellServiceStopRequest>,
    ) -> std::result::Result<Response<CellServiceStopResponse>, Status> {
        let request = request.into_inner();

        // We execute stop if cell_name is none
        if request.cell_name.is_none() {
            let request =
                ValidatedCellServiceStopRequest::validate(request, None)?;
            Ok(self.stop(request).await?)
        } else {
            // We are in a parent cell (or validation will fail)
            let validated = ValidatedCellServiceStopRequest::validate(
                request.clone(),
                None,
            )?;

            // validation has succeeded, so we can make assumptions about the request and use expect
            let cell_name = validated.cell_name.expect("cell name");
            let mut request = request;
            request.cell_name = None;
            self.stop_in_cell(&cell_name, request).await
        }
    }

    async fn list(
        &self,
        _request: Request<CellServiceListRequest>,
    ) -> std::result::Result<Response<CellServiceListResponse>, Status> {
        Ok(Response::new(self.list().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::cell_service::validation::{
        ValidatedCell, ValidatedCpuController, ValidatedCpusetController,
    };
    use iter_tools::Itertools;

    // Ignored: requires sudo, which we don't have in CI
    #[ignore]
    #[tokio::test]
    async fn test_list() {
        let service = CellService::new();

        let parent_cell_name = format!("ae-test-{}", uuid::Uuid::new_v4());
        assert!(service
            .allocate(allocate_request(&parent_cell_name))
            .await
            .is_ok());

        let nested_cell_name =
            format!("{}/ae-test-{}", &parent_cell_name, uuid::Uuid::new_v4());
        assert!(service
            .allocate(allocate_request(&nested_cell_name))
            .await
            .is_ok());

        let cell_without_children_name =
            format!("ae-test-{}", uuid::Uuid::new_v4());
        assert!(service
            .allocate(allocate_request(&cell_without_children_name))
            .await
            .is_ok());

        let result = service.list().await;
        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.cells.len(), 2);

        let expected_root_cell_names =
            vec![&parent_cell_name, &nested_cell_name].sort();
        let actual_root_cell_names = list
            .cells
            .iter()
            .map(|c| c.cell.as_ref().unwrap().name.clone())
            .collect_vec()
            .sort();
        assert_eq!(actual_root_cell_names, expected_root_cell_names);

        let parent_cell = list
            .cells
            .iter()
            .find(|p| p.cell.as_ref().unwrap().name.eq(&parent_cell_name));
        assert!(parent_cell.is_some());

        let expected_nested_cell_names = vec![&nested_cell_name];
        let actual_nested_cell_names = parent_cell
            .unwrap()
            .children
            .iter()
            .map(|c| c.cell.as_ref().unwrap().name.as_str())
            .collect_vec();
        assert_eq!(actual_nested_cell_names, expected_nested_cell_names);
    }

    fn allocate_request(
        cell_name: &str,
    ) -> ValidatedCellServiceAllocateRequest {
        let cell = ValidatedCell {
            name: CellName::from(cell_name),
            cpu: Some(ValidatedCpuController { weight: None, max: None }),
            cpuset: Some(ValidatedCpusetController { cpus: None, mems: None }),
            isolate_process: false,
            isolate_network: false,
        };
        ValidatedCellServiceAllocateRequest { cell }
    }
}
