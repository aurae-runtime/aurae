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

use aurae_client::AuraeClientError;
use thiserror::Error;
use tonic::Status;
use tracing::error;

pub(crate) type Result<T> = std::result::Result<T, RuntimeServiceError>;

#[derive(Debug, Error)]
pub enum RuntimeServiceError {
    #[error("sandbox '{sandbox_id}' already exists")]
    SandboxExists { sandbox_id: String },
    #[error("sandbox '{sandbox_id}' not found")]
    SandboxNotFound { sandbox_id: String },
    #[error("Failed to kill sandbox '{sandbox_id}': {error}")]
    KillError { sandbox_id: String, error: String },
    #[error(transparent)]
    AuraeClientError(#[from] AuraeClientError),
}

impl From<RuntimeServiceError> for Status {
    fn from(err: RuntimeServiceError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            RuntimeServiceError::SandboxExists { .. } => {
                Status::already_exists(msg)
            }
            RuntimeServiceError::SandboxNotFound { .. } => {
                Status::not_found(msg)
            }
            RuntimeServiceError::KillError { .. } => Status::internal(msg),
            RuntimeServiceError::AuraeClientError(e) => match e {
                AuraeClientError::ConnectionError(_) => {
                    Status::unavailable(msg)
                }
                AuraeClientError::Other(_) => Status::unknown(msg),
            },
        }
    }
}
