use crate::runtime::cells::cell::CellError;
use log::error;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, CellServiceError>;

#[derive(Error, Debug)]
pub(crate) enum CellServiceError {
    #[error(transparent)]
    CellError(#[from] CellError),
    #[error("failed to lock cells cache")]
    FailedToObtainLock(),
}

impl From<CellServiceError> for Status {
    fn from(err: CellServiceError) -> Self {
        match err {
            CellServiceError::CellError(err) => err.into(),
            CellServiceError::FailedToObtainLock() => Status::aborted(""),
        }
    }
}
