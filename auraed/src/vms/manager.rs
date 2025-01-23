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
use std::sync::{
    mpsc::{channel, Sender},
    Arc,
};

use hypervisor::Hypervisor;
use libc::EFD_NONBLOCK;
use vmm::{api::ApiRequest, VmmThreadHandle};
use vmm_sys_util::eventfd::EventFd;

pub struct Manager {
    pub events: EventFd,
    pub sender: Option<Sender<ApiRequest>>,
    hypervisor: Arc<dyn Hypervisor>,
    debug: EventFd,
    vmm_thread: Option<VmmThreadHandle>,
}

impl Manager {
    pub fn new() -> Self {
        let debug =
            EventFd::new(EFD_NONBLOCK).expect("Failed to create event monitor");
        let api_evt =
            EventFd::new(EFD_NONBLOCK).expect("Failed to create API eventfd");

        let hypervisor =
            hypervisor::new().expect("Failed to instantiate hypervisor");

        Self {
            hypervisor,
            debug,
            sender: None,
            events: api_evt,
            vmm_thread: None,
        }
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        let (sender, receiver) = channel();
        self.sender = Some(sender.clone());

        let version =
            vmm::VmmVersionInfo::new("auraed", env!("CARGO_PKG_VERSION"));
        self.vmm_thread = Some(
            vmm::start_vmm_thread(
                version,
                &None,
                None,
                self.events.try_clone()?,
                sender,
                receiver,
                self.debug.try_clone()?,
                &seccompiler::SeccompAction::Allow,
                self.hypervisor.clone(),
                false
            )
            .expect("Failed to start VMM thread"),
        );
        Ok(())
    }
}
