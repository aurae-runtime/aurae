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

use crate::new_client;
use crate::runtime::cell_service_client::CellServiceClient;
tonic::include_proto!("runtime");

#[derive(Debug, Clone)]
pub struct CellService {}

impl Default for CellService {
    fn default() -> Self {
        Self::new()
    }
}

impl CellService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn free(
        &mut self,
        _req: FreeCellRequest,
    ) -> anyhow::Result<FreeCellResponse> {
        todo!()
    }

    pub fn stop(
        &mut self,
        _req: StopCellRequest,
    ) -> anyhow::Result<StopCellResponse> {
        todo!()
    }

    pub fn start(
        &mut self,
        _req: StartCellRequest,
    ) -> anyhow::Result<StartCellResponse> {
        todo!()
    }

    pub fn allocate(
        &mut self,
        req: AllocateCellRequest,
    ) -> anyhow::Result<AllocateCellResponse> {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let client = rt.block_on(new_client()).expect("new client");
        let mut service = CellServiceClient::new(client.channel);
        let res = rt.block_on(service.allocate(req))?;
        Ok(res.into_inner())
    }
}
