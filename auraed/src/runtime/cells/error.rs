/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use crate::runtime::cells::{CellName, ExecutableName};
use cgroups_rs::CgroupPid;
use log::error;
use std::io;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, CellsError>;

#[derive(Error, Debug)]
pub(crate) enum CellsError {
    #[error("cell '{cell_name}' already exists'")]
    CellExists { cell_name: CellName },
    #[error("cell '{cell_name}' not found'")]
    CellNotFound { cell_name: CellName },
    #[error("cell '{cell_name}' could not be freed: {source}")]
    FailedToFreeCell { cell_name: CellName, source: cgroups_rs::error::Error },
    #[error(
        "cell '{cell_name}' already has an executable '{executable_name}'"
    )]
    ExecutableExists { cell_name: CellName, executable_name: ExecutableName },
    #[error("cell '{cell_name} could not find executable '{executable_name}'")]
    ExecutableNotFound { cell_name: CellName, executable_name: ExecutableName },
    #[error("cell '{cell_name}' failed to start executable '{executable_name}' ({command:?}) due to: {source}")]
    FailedToStartExecutable {
        cell_name: CellName,
        executable_name: ExecutableName,
        command: String,
        args: Vec<String>,
        source: io::Error,
    },
    #[error("cell '{cell_name}' failed to stop executable '{executable_name}' ({executable_pid:?}) due to: {source}")]
    FailedToStopExecutable {
        cell_name: CellName,
        executable_name: ExecutableName,
        executable_pid: CgroupPid,
        source: io::Error,
    },
    #[error(
        "cell '{cell_name}' failed to add executable (executable:?): {source}"
    )]
    FailedToAddExecutableToCell {
        cell_name: CellName,
        executable_name: ExecutableName,
        source: cgroups_rs::error::Error,
    },
    #[error("failed to lock cells table")]
    FailedToObtainLock(),
}

impl From<CellsError> for Status {
    fn from(err: CellsError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellsError::CellExists { .. }
            | CellsError::ExecutableExists { .. } => {
                Status::already_exists(msg)
            }
            CellsError::CellNotFound { .. }
            | CellsError::ExecutableNotFound { .. } => Status::not_found(msg),
            CellsError::FailedToFreeCell { .. }
            | CellsError::FailedToStartExecutable { .. }
            | CellsError::FailedToStopExecutable { .. }
            | CellsError::FailedToAddExecutableToCell { .. } => {
                Status::internal(msg)
            }
            CellsError::FailedToObtainLock() => {
                Status::aborted(err.to_string())
            }
        }
    }
}
