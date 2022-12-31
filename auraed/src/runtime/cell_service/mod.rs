pub use cell_service::CellService;
pub use cells::CELLS;
use error::Result;

#[allow(clippy::module_inception)]
mod cell_service;
mod cells;
mod error;
mod executables;
mod validation;
