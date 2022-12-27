pub use error::{ExecutablesError, Result};
pub use executable::Executable;
pub use executable_name::ExecutableName;
pub use executables::Executables;
use std::process::Command;

pub mod auraed;
mod error;
mod executable;
mod executable_name;
mod executables;
mod process;

pub struct ExecutableSpec {
    pub name: ExecutableName,
    pub description: String,
    pub command: Command,
}
