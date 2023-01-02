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

#![allow(unused)]
#![allow(clippy::module_inception)]

use aurae_client::{runtime::pod_service::PodServiceClient, AuraeClient};
use aurae_proto::runtime::{
    pod_service_server, PodServiceAllocateRequest, PodServiceAllocateResponse,
    PodServiceFreeRequest, PodServiceFreeResponse, Pod, PodServiceStartRequest,
    PodServiceStartResponse, PodServiceStopRequest, PodServiceStopResponse,
};
// use std::sync::Arc;
// use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct PodService {
    // These are used for the cache as in the cells/executables
    //pods: Arc<Mutex<Pods>>,
    //containers: Arc<Mutex<Containers>>,
}

impl PodService {
    pub fn new() -> Self {
        PodService {}
    }
}

#[tonic::async_trait]
impl pod_service_server::PodService for PodService {
    async fn allocate(
        &self,
        request: Request<PodServiceAllocateRequest>,
    ) -> Result<Response<PodServiceAllocateResponse>, Status> {
        let _request = request.into_inner();
        Ok(Response::new(PodServiceAllocateResponse {}))
    }
    async fn free(
        &self,
        request: Request<PodServiceFreeRequest>,
    ) -> Result<Response<PodServiceFreeResponse>, Status> {
        let _request = request.into_inner();
        Ok(Response::new(PodServiceFreeResponse {}))
    }
    async fn start(
        &self,
        request: Request<PodServiceStartRequest>,
    ) -> Result<Response<PodServiceStartResponse>, Status> {
        let _request = request.into_inner();
        Ok(Response::new(PodServiceStartResponse {}))
    }
    async fn stop(
        &self,
        request: Request<PodServiceStopRequest>,
    ) -> Result<Response<PodServiceStopResponse>, Status> {
        let _request = request.into_inner();
        Ok(Response::new(PodServiceStopResponse {}))
    }
}
