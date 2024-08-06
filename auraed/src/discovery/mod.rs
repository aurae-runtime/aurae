/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
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

#[cfg(test)]
mod tests {
    use proto::discovery::DiscoverRequest;

    use crate::discovery::{DiscoveryService, VERSION};

    #[test]
    fn test_discover() {
        let resp = DiscoveryService::new().discover(DiscoverRequest {});
        assert!(resp.is_ok());

        let resp = resp.unwrap();

        assert!(resp.healthy);
        assert_eq!(resp.version, VERSION.expect("valid version"));
    }
}