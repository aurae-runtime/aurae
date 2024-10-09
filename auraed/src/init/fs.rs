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

use lazy_static::lazy_static;
use nix::{mount::MsFlags, sys::stat::Mode};
use std::io;
use tracing::{error, info};

#[derive(thiserror::Error, Debug)]
pub(crate) enum FsError {
    #[error("Failed to mount {spec:?} due to error: {source}")]
    MountFailure { spec: MountSpec, source: io::Error },
    #[error(transparent)]
    FileCreationFailure(#[from] nix::errno::Errno),
}

lazy_static! {
    pub static ref CHMOD_0755: Mode = Mode::S_IRWXU
        | Mode::S_IRGRP
        | Mode::S_IXGRP
        | Mode::S_IROTH
        | Mode::S_IXOTH;
    pub static ref CHMOD_0555: Mode = Mode::S_IRUSR
        | Mode::S_IXUSR
        | Mode::S_IRGRP
        | Mode::S_IXGRP
        | Mode::S_IROTH
        | Mode::S_IXOTH;
    pub static ref CHMOD_1777: Mode =
        Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO | Mode::S_ISVTX;
    pub static ref COMMON_MNT_FLAGS: MsFlags =
        MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID;
    pub static ref CGROUP_MNT_FLAGS: MsFlags = MsFlags::MS_NODEV
        | MsFlags::MS_NOEXEC
        | MsFlags::MS_NOSUID
        | MsFlags::MS_RELATIME;
}

#[derive(Debug)]
pub(crate) struct MountSpec {
    pub source: Option<&'static str>,
    pub target: &'static str,
    pub fstype: Option<&'static str>,
    pub flags: MsFlags,
    pub data: Option<&'static str>,
}

impl MountSpec {
    pub fn mount(self) -> Result<(), FsError> {
        info!("Mounting {}", self.target);

        if let Err(e) = nix::mount::mount(
            self.source,
            self.target,
            self.fstype,
            self.flags,
            self.data,
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
