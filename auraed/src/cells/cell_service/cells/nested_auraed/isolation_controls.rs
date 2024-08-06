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

use libc::c_char;
use std::io::{self};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct IsolationControls {
    pub isolate_process: bool,
    pub isolate_network: bool,
}

#[derive(Default)]
pub(crate) struct Isolation {
    name: String,
}

impl Isolation {
    pub fn new(name: String) -> Isolation {
        Isolation { name }
    }
    pub fn setup(&mut self, iso_ctl: &IsolationControls) -> io::Result<()> {
        // The only setup we will need to do is for isolate_process at this time.
        // We can exit quickly if we are sharing the process controls with the host.
        if !iso_ctl.isolate_process {
            return Ok(());
        }

        // Bind mount root:root with MS_REC and MS_PRIVATE flags
        // We are not sharing the mounts at this point (in other words we are in a new mount namespace)
        nix::mount::mount(
            None::<&str>, // ignored
            "/",
            None::<&str>, // ignored
            nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
            None::<&str>, // ignored
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
        info!("Isolation: Mounted root dir (/) in cell");
        Ok(())
    }

    pub fn isolate_process(
        &mut self,
        iso_ctl: &IsolationControls,
    ) -> io::Result<()> {
        if !iso_ctl.isolate_process {
            return Ok(());
        }

        //Mount proc in the new pid and mount namespace
        let target = PathBuf::from("/proc");
        nix::mount::mount(
            Some("/proc"),
            &target,
            Some("proc"),
            nix::mount::MsFlags::empty(),
            None::<&str>,
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        // We are in a new UTS namespace so we manage hostname and domainname.
        // hostname and domainname both allow null bytes and are not required to be null terminated.
        if unsafe {
            #[allow(trivial_casts)]
            libc::sethostname(
                self.name.as_ptr() as *const c_char,
                self.name.len(),
            )
        } == -1
        {
            return Err(io::Error::last_os_error());
        }

        // Set domainname
        if unsafe {
            #[allow(trivial_casts)]
            libc::setdomainname(
                self.name.as_ptr() as *const c_char,
                self.name.len(),
            )
        } == -1
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn isolate_network(
        &mut self,
        iso_ctl: &IsolationControls,
    ) -> io::Result<()> {
        if !iso_ctl.isolate_network {
            return Ok(());
        }
        // Insert pre_exec network logic here
        Ok(())
    }
}