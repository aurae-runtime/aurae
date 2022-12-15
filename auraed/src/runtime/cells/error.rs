use crate::runtime::cells::executable::{Executable, ExecutableError};
use crate::runtime::cells::{CellName, ExecutableName};
use log::error;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, CellError>;

#[derive(Error, Debug)]
pub(crate) enum CellError {
    #[error("cell '{cell_name}' already exists'")]
    Exists { cell_name: CellName },
    #[error("cell '{cell_name}' not found'")]
    NotFound { cell_name: CellName },
    #[error("cell '{cell_name}' could not be freed: {source}")]
    FailedToFree { cell_name: CellName, source: cgroups_rs::error::Error },
    #[error(
        "cell '{cell_name}' already has an executable '{executable_name}'"
    )]
    ExecutableExists { cell_name: CellName, executable_name: ExecutableName },
    #[error("cell '{cell_name} could not find executable '{executable_name}'")]
    ExecutableNotFound { cell_name: CellName, executable_name: ExecutableName },
    #[error("cell '{cell_name}': {source}")]
    ExecutableError { cell_name: CellName, source: ExecutableError },
    #[error("cell '{cell_name}' failed to add executable (executable:?)")]
    FailedToAddExecutable {
        cell_name: CellName,
        executable: Executable,
        source: cgroups_rs::error::Error,
    },
    // TODO: this error seems out of place
    #[error("failed to lock cells cache")]
    FailedToObtainLock(),
}

impl From<CellError> for Status {
    fn from(err: CellError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellError::Exists { .. } | CellError::ExecutableExists { .. } => {
                Status::already_exists(msg)
            }
            CellError::NotFound { .. }
            | CellError::ExecutableNotFound { .. } => Status::not_found(msg),
            // TODO (future-highway): I don't know what the conventions are of revealing
            //  messages that reveal the workings of the system to the api consumer
            //  in this type of application.
            //  For now, taking the safe route and not exposing the error messages for the below errors.
            CellError::FailedToObtainLock() => Status::aborted(""),
            CellError::ExecutableError { .. }
            | CellError::FailedToFree { .. }
            | CellError::FailedToAddExecutable { .. } => Status::internal(""),
        }
    }
}
