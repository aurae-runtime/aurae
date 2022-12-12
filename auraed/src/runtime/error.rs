use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CellServiceError {
    #[error("missing argument in request: {arg}")]
    MissingArgument { arg: String },

    #[error("internal error: {msg}: {err}")]
    Internal { msg: String, err: String },
}

impl From<CellServiceError> for Status {
    fn from(err: CellServiceError) -> Self {
        match err {
            CellServiceError::MissingArgument { arg } => {
                Self::failed_precondition(format!(
                    "bad request. expected {arg}"
                ))
            }
            CellServiceError::Internal { msg, err } => {
                Self::internal(format!("internal error: {msg}: {err}"))
            }
        }
    }
}
