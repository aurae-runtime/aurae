use crate::cells::CellsError;
use log::error;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Error, Debug)]
pub(crate) enum RuntimeError {
    #[error("missing argument in request: {arg}")]
    MissingArgument { arg: String },

    #[error("internal error: {msg}: {err}")]
    Internal { msg: String, err: String },

    #[error("{resource} was not allocated")]
    Unallocated { resource: String },

    #[error(transparent)]
    CellsError(#[from] CellsError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<RuntimeError> for Status {
    fn from(err: RuntimeError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            RuntimeError::MissingArgument { .. } => {
                Self::failed_precondition(msg)
            }
            RuntimeError::Internal { .. }
            | RuntimeError::CellsError { .. }
            | RuntimeError::Other { .. } => Self::internal(msg),
            RuntimeError::Unallocated { .. } => Self::not_found(msg),
        }
    }
}
