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

use proto::vms::{
    vm_service_server, VirtualMachine, VmServiceAllocateRequest,
    VmServiceAllocateResponse, VmServiceFreeRequest, VmServiceFreeResponse,
    VmServiceStartRequest, VmServiceStartResponse, VmServiceStopRequest,
    VmServiceStopResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct VmService {
    _vms: Arc<Mutex<VirtualMachine>>,
}

#[tonic::async_trait]
impl vm_service_server::VmService for VmService {
    async fn allocate(
        &self,
        _request: Request<VmServiceAllocateRequest>,
    ) -> Result<Response<VmServiceAllocateResponse>, Status> {
        todo!()
    }

    async fn free(
        &self,
        _request: Request<VmServiceFreeRequest>,
    ) -> Result<Response<VmServiceFreeResponse>, Status> {
        todo!()
    }

    async fn start(
        &self,
        _request: Request<VmServiceStartRequest>,
    ) -> Result<Response<VmServiceStartResponse>, Status> {
        todo!()
    }

    async fn stop(
        &self,
        _request: Request<VmServiceStopRequest>,
    ) -> Result<Response<VmServiceStopResponse>, Status> {
        todo!()
    }
}
