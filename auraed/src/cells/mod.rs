pub(crate) use cell::Cell;
pub(crate) use cell_name::CellName;
pub(crate) use executable::Executable;
pub(crate) use executable_name::ExecutableName;
use std::io;
use thiserror::Error;

mod cell;
mod cell_name;
mod executable;
mod executable_name;

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
