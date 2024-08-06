/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

use super::ExecutableName;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ExecutablesError>;

#[derive(Error, Debug)]
pub enum ExecutablesError {
    #[error("executable '{executable_name}' exists")]
    ExecutableExists { executable_name: ExecutableName },
    #[error("executable '{executable_name}' not found")]
    ExecutableNotFound { executable_name: ExecutableName },
    #[error("executable '{executable_name}' failed to start: {source}")]
    FailedToStartExecutable {
        executable_name: ExecutableName,
        source: io::Error,
    },
    #[error("executable '{executable_name}' failed to stop: {source}")]
    FailedToStopExecutable {
        executable_name: ExecutableName,
        source: io::Error,
    },
}