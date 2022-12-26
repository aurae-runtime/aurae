pub use error::{ExecutablesError, Result};
pub use executable::Executable;
pub use executable_name::ExecutableName;
pub use executables::Executables;
pub use shared_namespaces::SharedNamespaces;
use std::process::Command;

mod error;
mod executable;
mod executable_name;
mod executables;
mod shared_namespaces;

pub struct ExecutableSpec {
    pub name: ExecutableName,
    pub description: String,
    pub command: Command,
}
