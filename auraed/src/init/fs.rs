/* -------------------------------------------------------------------------- *\
 *               Apache 2.0 License Copyright The Aurae Authors               *
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

use std::io;
use tracing::{error, info};

#[derive(thiserror::Error, Debug)]
pub(crate) enum FsError {
    #[error("Failed to mount {spec:?} due to error: {source}")]
    MountFailure { spec: MountSpec, source: io::Error },
}

#[derive(Debug)]
pub(crate) struct MountSpec {
    pub source: Option<&'static str>,
    pub target: &'static str,
    pub fstype: Option<&'static str>,
}

impl MountSpec {
    pub fn mount(self) -> Result<(), FsError> {
        info!("Mounting {}", self.target);

        if let Err(e) = nix::mount::mount(
            self.source,
            self.target,
            self.fstype,
            nix::mount::MsFlags::empty(),
            None::<&str>,
        ) {
            error!("Failed to mount {:?}", self);
            return Err(FsError::MountFailure {
                spec: self,
                source: io::Error::from_raw_os_error(e as i32),
            });
        }

        Ok(())
    }
}
