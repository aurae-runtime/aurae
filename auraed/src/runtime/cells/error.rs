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

use aurae_cells::CellsError;
use tonic::Status;
use tracing::error;

pub(crate) type Result<T> = std::result::Result<T, CellsServiceError>;

pub(crate) struct CellsServiceError(CellsError);

impl From<CellsError> for CellsServiceError {
    fn from(value: CellsError) -> Self {
        Self(value)
    }
}

impl From<CellsServiceError> for Status {
    fn from(err: CellsServiceError) -> Self {
        let err = err.0;
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellsError::CellExists { .. }
            | CellsError::ExecutableExists { .. } => {
                Status::already_exists(msg)
            }
            CellsError::CellNotFound { .. }
            | CellsError::ExecutableNotFound { .. } => Status::not_found(msg),
            CellsError::FailedToAllocateCell { .. }
            | CellsError::AbortedAllocateCell { .. }
            | CellsError::FailedToFreeCell { .. }
            | CellsError::FailedToStartExecutable { .. }
            | CellsError::FailedToStopExecutable { .. }
            | CellsError::FailedToAddExecutableToCell { .. } => {
                Status::internal(msg)
            }
            CellsError::FailedToObtainLock() => {
                Status::aborted(err.to_string())
            }
            CellsError::CellNotAllocated { cell_name } => {
                CellsServiceError(CellsError::CellNotFound { cell_name }).into()
            }
        }
    }
}
