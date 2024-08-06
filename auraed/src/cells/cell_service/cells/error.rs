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