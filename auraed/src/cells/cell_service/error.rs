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

use super::{cells::CellsError, executables::ExecutablesError};
use crate::observe::ObserveServiceError;
use client::ClientError;
use thiserror::Error;
use tonic::Status;
use tracing::error;

pub(crate) type Result<T> = std::result::Result<T, CellsServiceError>;

#[derive(Debug, Error)]
pub(crate) enum CellsServiceError {
    #[error(transparent)]
    CellsError(#[from] CellsError),
    #[error(transparent)]
    ExecutablesError(#[from] ExecutablesError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    ObserveServiceError(#[from] ObserveServiceError),
}

impl From<CellsServiceError> for Status {
    fn from(err: CellsServiceError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellsServiceError::CellsError(e) => match e {
                CellsError::CgroupIsNotACell { .. } => {
                    Status::failed_precondition(msg)
                }
                CellsError::CellExists { .. } => Status::already_exists(msg),
                CellsError::CellNotFound { .. }
                | CellsError::CgroupNotFound { .. } => Status::not_found(msg),
                CellsError::FailedToAllocateCell { .. }
                | CellsError::AbortedAllocateCell { .. }
                | CellsError::FailedToKillCellChildren { .. }
                | CellsError::FailedToFreeCell { .. } => Status::internal(msg),
                CellsError::CellNotAllocated { cell_name } => {
                    CellsServiceError::CellsError(CellsError::CellNotFound {
                        cell_name,
                    })
                    .into()
                }
            },
            CellsServiceError::ExecutablesError(e) => match e {
                ExecutablesError::ExecutableExists { .. } => {
                    Status::already_exists(msg)
                }
                ExecutablesError::ExecutableNotFound { .. } => {
                    Status::not_found(msg)
                }
                ExecutablesError::FailedToStartExecutable { .. }
                | ExecutablesError::FailedToStopExecutable { .. } => {
                    Status::internal(msg)
                }
            },
            CellsServiceError::Io(_) => Status::internal(msg),
            CellsServiceError::ClientError(e) => match e {
                ClientError::ConnectionError(_) => Status::unavailable(msg),
                ClientError::Other(_) => Status::unknown(msg),
            },
            CellsServiceError::ObserveServiceError(e) => e.into(),
        }
    }
}