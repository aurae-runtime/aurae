use std::collections::HashMap;

use anyhow::anyhow;
use vmm_sys_util::signal::block_signal;

use super::virtual_machine::{VirtualMachine, VmID, VmSpec};

type Cache = HashMap<VmID, VirtualMachine>;

/// The in-memory cache of virtual machines ([VirtualMachine]) created with Aurae.
#[derive(Debug)]
pub struct VirtualMachines {
    cache: Cache,
}

impl VirtualMachines {
    /// Create a new instance of the virtual machines cache.
    pub fn new() -> Self {
        unsafe {
            let _ = libc::signal(libc::SIGCHLD, libc::SIG_IGN);
        }

        // Before we start any threads, mask the signals we'll be
        // installing handlers for, to make sure they only ever run on the
        // dedicated signal handling thread we'll start in a bit.
        for sig in &vmm::vm::Vm::HANDLED_SIGNALS {
            if let Err(e) = block_signal(*sig) {
                eprintln!("Error blocking signals: {e}");
            }
        }

        for sig in &vmm::Vmm::HANDLED_SIGNALS {
            if let Err(e) = block_signal(*sig) {
                eprintln!("Error blocking signals: {e}");
            }
        }

        Self { cache: Cache::new() }
    }

    /// Create a new virtual machine
    pub fn create(
        &mut self,
        id: VmID,
        spec: VmSpec,
    ) -> Result<VirtualMachine, anyhow::Error> {
        if let Some(vm) = self.cache.get(&id) {
            return Err(anyhow!(
                "Virtual machine with ID '{:?}' already exists: {:?}",
                &id,
                vm.vm,
            ));
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

    /// Start a virtual machine by its ID
    pub fn start(&mut self, id: &VmID) -> Result<(), anyhow::Error> {
        if let Some(vm) = self.cache.get_mut(id) {
            vm.start()?;
            Ok(())
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
}
