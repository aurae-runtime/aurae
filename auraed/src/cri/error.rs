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

use client::ClientError;
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
    #[error("sandobx '{sandbox_id}' not in exited state")]
    SandboxNotExited { sandbox_id: String },
    #[error("Failed to kill sandbox '{sandbox_id}': {error}")]
    KillError { sandbox_id: String, error: String },
    #[error(transparent)]
    ClientError(#[from] ClientError),
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
            RuntimeServiceError::SandboxNotExited { .. } => {
                Status::failed_precondition(msg)
            }
            RuntimeServiceError::KillError { .. } => Status::internal(msg),
            RuntimeServiceError::ClientError(e) => match e {
                ClientError::ConnectionError(_) => Status::unavailable(msg),
                ClientError::Other(_) => Status::unknown(msg),
            },
        }
    }
}