/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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

use super::{cgroups::error::CgroupsError, CellName};
use std::io;
use thiserror::Error;
use tracing::error;

pub type Result<T> = std::result::Result<T, CellsError>;

#[derive(Error, Debug)]
pub enum CellsError {
    #[error("cell '{cell_name}' already exists'")]
    CellExists { cell_name: CellName },
    #[error("cell '{cell_name}' not found")]
    CellNotFound { cell_name: CellName },
    #[error("cell '{cell_name}' is not allocated")]
    CellNotAllocated { cell_name: CellName },
    #[error("cell '{cell_name}' could not be allocated: {source}")]
    FailedToAllocateCell { cell_name: CellName, source: io::Error },
    #[error("cell '{cell_name}' allocation was aborted: {source}")]
    AbortedAllocateCell { cell_name: CellName, source: CgroupsError },
    #[error("cell '{cell_name}' could not kill children: {source}")]
    FailedToKillCellChildren { cell_name: CellName, source: io::Error },
    #[error("cell '{cell_name}' could not be freed: {source}")]
    FailedToFreeCell { cell_name: CellName, source: CgroupsError },
    #[error(
        "cgroup '{cell_name}' exists on host, but is not controlled by auraed"
    )]
    CgroupIsNotACell { cell_name: CellName },
    #[error("cgroup '{cell_name}` not found on host")]
    CgroupNotFound { cell_name: CellName },
}
