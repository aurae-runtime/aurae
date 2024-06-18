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

use std::{io, path::Path};
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

pub fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
