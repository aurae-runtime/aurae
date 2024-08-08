use std::{collections::HashMap, net::Ipv4Addr};

use anyhow::anyhow;
use net_util::MacAddr;
use tracing::error;
use vmm_sys_util::{rand, signal::block_signal};

use super::virtual_machine::{NetSpec, VirtualMachine, VmID, VmSpec};

type Cache = HashMap<VmID, VirtualMachine>;

/// The in-memory cache of virtual machines ([VirtualMachine]) created with Aurae.
#[derive(Debug)]
pub struct VirtualMachines {
    cache: Cache,
}

impl Default for VirtualMachines {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachines {
    /// Create a new instance of the virtual machines cache.
    pub fn new() -> Self {
        unsafe {
            let _ = libc::signal(libc::SIGCHLD, libc::SIG_IGN);
        }

        // Mask the signals handled by the Cloud Hyupervisor VMM so they only run on the dedicated signal handling thread
        for sig in &vmm::vm::Vm::HANDLED_SIGNALS {
            if let Err(e) = block_signal(*sig) {
                error!("Error blocking signals: {e}");
            }
        }

        for sig in &vmm::Vmm::HANDLED_SIGNALS {
            if let Err(e) = block_signal(*sig) {
                error!("Error blocking signals: {e}");
            }
        }

        Self { cache: Cache::new() }
    }

    /// Allocate an IP address for a new virtual machine
    ///
    /// Use the hard-coded Cloud Hypervisor default address as starting IP
    /// https://github.com/cloud-hypervisor/cloud-hypervisor/blob/165c2c476f752909aba41d4e319f12ade20b72d3/vmm/src/vm_config.rs#L313-L319
    fn allocate_ip(&self) -> Ipv4Addr {
        Ipv4Addr::new(192, 168, 249, self.cache.len() as u8 + 1)
    }

    /// Create a new virtual machine
    pub fn create(
        &mut self,
        id: VmID,
        mut spec: VmSpec,
    ) -> Result<VirtualMachine, anyhow::Error> {
        if let Some(vm) = self.cache.get(&id) {
            return Err(anyhow!(
                "Virtual machine with ID '{:?}' already exists: {:?}",
                &id,
                vm.vm,
            ));
        }

        // Populate the default network configuration if it's empty
        if spec.net.is_empty() {
            spec.net.push(NetSpec {
                tap: Some(format!(
                    "auraed-{}",
                    rand::rand_alphanumerics(6).into_string().map_err(
                        |_| anyhow!("Error generating TAP device name")
                    )?,
                )),
                ip: self.allocate_ip(),
                mask: Ipv4Addr::new(255, 255, 255, 0),
                mac: MacAddr::local_random(),
                host_mac: None,
            });
        }

        let vm = VirtualMachine::new(id.clone(), spec)?;
        let _ = self.cache.insert(id, vm.clone()).is_none();
        Ok(vm)
    }

    /// Stop a virtual machine by its ID
    pub fn stop(&mut self, id: &VmID) -> Result<(), anyhow::Error> {
        if let Some(vm) = self.cache.get_mut(id) {
            vm.stop()?;
            Ok(())
        } else {
            Err(anyhow!("Virtual machine with ID '{:?}' not found", id))
        }
    }

    /// Start a virtual machine by its ID, returning the addres of its TAP device
    pub fn start(&mut self, id: &VmID) -> Result<String, anyhow::Error> {
        if let Some(vm) = self.cache.get_mut(id) {
            vm.start()?;
            match vm.tap() {
                Some(tap) => Ok(tap.to_string()),
                None => Ok("".into()),
            }
        } else {
            Err(anyhow!("Virtual machine with ID '{:?}' not found", id))
        }
    }

    /// Delete a virtual machine by its ID
    pub fn delete(&mut self, id: &VmID) -> Result<(), anyhow::Error> {
        if let Some(vm) = self.cache.get_mut(id) {
            vm.delete()?;
            let _ = self.cache.remove(id);
            Ok(())
        } else {
            Err(anyhow!("Virtual machine with ID '{:?}' not found", id))
        }
    }

    /// List all virtual machines
    pub fn list(&self) -> Vec<VirtualMachine> {
        self.cache.values().cloned().collect()
    }
}
