use log::error;
use std::io;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, CellsError>;

#[derive(Error, Debug)]
pub(crate) enum CellsError {
    // TODO: define the errors better
    #[error(transparent)]
    CgroupsError(#[from] cgroups_rs::error::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<CellsError> for Status {
    fn from(err: CellsError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellsError::CgroupsError { .. }
            | CellsError::Io { .. }
            | CellsError::Other { .. } => Self::internal(msg),
        }
    }
}
