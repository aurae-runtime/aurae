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

use crate::cells::cell_service::cells::CellName;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CgroupsError>;

#[derive(Error, Debug)]
pub enum CgroupsError {
    #[error("cgroup '{cell_name}' creation failed: {source}")]
    CreateCgroup { cell_name: CellName, source: anyhow::Error },
    #[error("cgroup '{cell_name}' failed to add task: {source}")]
    AddTaskToCgroup { cell_name: CellName, source: anyhow::Error },
    #[error("cgroup '{cell_name}' deletion failed: {source}")]
    DeleteCgroup { cell_name: CellName, source: anyhow::Error },
    #[error("cgroup '{cell_name}' failed to read stats: {source}")]
    ReadStats { cell_name: CellName, source: anyhow::Error },
}