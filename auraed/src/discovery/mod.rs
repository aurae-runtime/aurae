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

use proto::discovery::{
    discovery_service_server, DiscoverRequest, DiscoverResponse,
};
use thiserror::Error;
use tonic::{Request, Response, Status};
use tracing::error;

pub(crate) type Result<T> = std::result::Result<T, DiscoveryServiceError>;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug, Error)]
pub(crate) enum DiscoveryServiceError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl From<DiscoveryServiceError> for Status {
    fn from(err: DiscoveryServiceError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            DiscoveryServiceError::IO(_) => Status::internal(msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveryService {}

impl DiscoveryService {
    pub fn new() -> Self {
        DiscoveryService {}
    }

    #[tracing::instrument(skip(self))]
    fn discover(&self, request: DiscoverRequest) -> Result<DiscoverResponse> {
        Ok(DiscoverResponse {
            healthy: true,
            version: VERSION.unwrap_or("unknown").into(),
        })
    }
}

#[tonic::async_trait]
impl discovery_service_server::DiscoveryService for DiscoveryService {
    async fn discover(
        &self,
        request: Request<DiscoverRequest>,
    ) -> std::result::Result<Response<DiscoverResponse>, Status> {
        let request = request.into_inner();
        Ok(Response::new(self.discover(request)?))
    }
}
