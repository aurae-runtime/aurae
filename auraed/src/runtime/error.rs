use log::error;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CellServiceError {
    #[error("missing argument in request: {arg}")]
    MissingArgument { arg: String },

    #[error("internal error: {msg}: {err}")]
    Internal { msg: String, err: String },

    #[error("{resource} was not allocated")]
    Unallocated { resource: String },
}

impl From<CellServiceError> for Status {
    fn from(err: CellServiceError) -> Self {
        match err {
            CellServiceError::MissingArgument { arg } => {
                let msg = format!("missing argument in request: {arg}");
                error!("{msg}");
                Self::failed_precondition(msg)
            }
            CellServiceError::Internal { msg, err } => {
                let msg = format!("internal error: {msg}: {err}");
                error!("{msg}");
                Self::internal(msg)
            }
            CellServiceError::Unallocated { resource } => {
                let msg = format!("{resource} was not allocated before use");
                error!("{msg}");
                Self::not_found(msg)
            }
        }
    }
}
