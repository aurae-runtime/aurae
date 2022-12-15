use crate::runtime::cells::{CellName, ExecutableName};
use cgroups_rs::CgroupPid;
use log::error;
use std::io;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, CellsError>;

#[derive(Error, Debug)]
pub(crate) enum CellsError {
    #[error("cell '{cell_name}' already exists'")]
    CellExists { cell_name: CellName },
    #[error("cell '{cell_name}' not found'")]
    CellNotFound { cell_name: CellName },
    #[error("cell '{cell_name}' could not be freed: {source}")]
    FailedToFreeCell { cell_name: CellName, source: cgroups_rs::error::Error },
    #[error(
        "executable '{executable_name}' already exists in cell '{cell_name}'"
    )]
    ExecutableExists { cell_name: CellName, executable_name: ExecutableName },
    #[error("executable '{executable_name}' not found in '{cell_name}'")]
    ExecutableNotFound { cell_name: CellName, executable_name: ExecutableName },
    // TODO: spin out executable errors
    #[error("failed to spawn executable '{executable_name}'")]
    FailedToStartExecutable {
        cell_name: CellName,
        executable_name: ExecutableName,
        source: io::Error,
    },
    #[error("failed to add executable '{executable_name}' ({executable_pid:?}) to cell '{cell_name}`")]
    FailedToAddExecutableToCell {
        cell_name: CellName,
        executable_name: ExecutableName,
        executable_pid: CgroupPid,
        source: cgroups_rs::error::Error,
    },
    #[error("failed to stop executable '{executable_name}' ({executable_pid:?}) in cell '{cell_name}`")]
    FailedToStopExecutable {
        cell_name: CellName,
        executable_name: ExecutableName,
        executable_pid: CgroupPid,
        source: io::Error,
    },
    #[error("failed to lock cells cache")]
    FailedToObtainLock(),
}

impl From<CellsError> for Status {
    fn from(err: CellsError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellsError::CellExists { .. }
            | CellsError::ExecutableExists { .. } => {
                Status::already_exists(msg)
            }
            CellsError::CellNotFound { .. }
            | CellsError::ExecutableNotFound { .. } => Status::not_found(msg),
            // TODO (future-highway): I don't know what the conventions are of revealing
            //  messages that reveal the workings of the system to the api consumer
            //  in this type of application.
            //  For now, taking the safe route and not exposing the error messages for the below errors.
            CellsError::FailedToObtainLock() => Status::aborted(""),
            CellsError::FailedToStartExecutable { .. }
            | CellsError::FailedToStopExecutable { .. }
            | CellsError::FailedToFreeCell { .. }
            | CellsError::FailedToAddExecutableToCell { .. } => {
                Status::internal("")
            }
        }
    }
}
