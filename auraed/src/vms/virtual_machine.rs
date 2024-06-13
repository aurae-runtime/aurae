use anyhow::anyhow;
use std::{
    arch::x86_64::_CMP_LE_OQ,
    collections::HashMap,
    fmt,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use vmm::{
    api::ApiAction,
    config::{
        ConsoleConfig, ConsoleOutputMode, CpusConfig, DebugConsoleConfig,
        MemoryConfig, PayloadConfig, RngConfig,
    },
};

use crate::vms::manager::Manager;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct VmID(String);

impl VmID {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl ToString for VmID {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub struct VmSpec {
    pub memory_size: u32,
    pub vcpu_count: u32,
    pub kernel_image_path: PathBuf,
    pub kernel_args: Vec<String>,
    pub root_drive: RootDriveSpec,
    pub mounts: Vec<MountSpec>,
}

impl Into<vmm::vm_config::VmConfig> for VmSpec {
    fn into(self) -> vmm::vm_config::VmConfig {
        vmm::vm_config::VmConfig {
            cpus: CpusConfig::default(),
            memory: MemoryConfig::default(),
            payload: Some(PayloadConfig {
                firmware: None,
                kernel: Some(self.kernel_image_path),
                cmdline: Some(self.kernel_args.join(" ")),
                initramfs: Some(self.root_drive.host_path),
            }),
            rate_limit_groups: None,
            disks: None,
            net: None,
            rng: RngConfig::default(),
            balloon: None,
            fs: None,
            pmem: None,
            serial: ConsoleConfig {
                file: None,
                mode: ConsoleOutputMode::Null,
                iommu: false,
                socket: None,
            },
            console: ConsoleConfig {
                file: None,
                mode: ConsoleOutputMode::Tty,
                iommu: false,
                socket: None,
            },
            debug_console: DebugConsoleConfig::default(),
            devices: None,
            user_devices: None,
            vdpa: None,
            vsock: None,
            pvpanic: false,
            iommu: false,
            sgx_epc: None,
            numa: None,
            watchdog: false,
            pci_segments: None,
            platform: None,
            tpm: None,
            preserved_fds: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootDriveSpec {
    pub host_path: PathBuf,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct MountSpec {
    pub host_path: PathBuf,
    pub guest_path: PathBuf,
    pub fs_type: FilesystemType,
    pub read_only: bool,
}

#[derive(Debug, Default, Clone)]
pub enum FilesystemType {
    #[default]
    Ext4,
    Xfs,
    Btrfs,
}

#[derive(Clone)]
pub struct VirtualMachine {
    pub id: VmID,
    pub vm: VmSpec,
    manager: Arc<Mutex<Manager>>,
}

impl fmt::Debug for VirtualMachine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualMachine {{ id: {:?}, vm: {:?} }}", self.id, self.vm)
    }
}

impl VirtualMachine {
    pub fn new(id: VmID, spec: VmSpec) -> Result<Self, anyhow::Error> {
        let mut manager = Manager::new();
        manager.start()?;

        if let Some(sender) = &manager.sender {
            vmm::api::VmCreate
                .send(
                    manager.events.try_clone()?,
                    sender.clone(),
                    Arc::new(Mutex::new(spec.clone().into())),
                )
                .expect("Failed to send create request");
        } else {
            return Err(anyhow!("Virtual machine manager not initialized"));
        }

        Ok(VirtualMachine {
            id,
            vm: spec,
            manager: Arc::new(Mutex::new(manager)),
        })
    }

    pub fn start(&self) -> Result<(), anyhow::Error> {
        let manager = self
            .manager
            .lock()
            .map_err(|_| anyhow!("Failed to aquire lock for vm manager"))?;

        if let Some(sender) = &manager.sender {
            let _ = vmm::api::VmBoot
                .send(manager.events.try_clone()?, sender.clone(), ())
                .map_err(|e| anyhow!("Failed to send start request: {e}"))?;
        } else {
            return Err(anyhow!("Virtual machine manager not initialized"))?;
        }

        Ok(())
    }

    pub fn stop(&self) -> Result<(), anyhow::Error> {
        let manager = self
            .manager
            .lock()
            .map_err(|_| anyhow!("Failed to aquire lock for vm manager"))?;

        if let Some(sender) = &manager.sender {
            let _ = vmm::api::VmShutdown
                .send(manager.events.try_clone()?, sender.clone(), ())
                .map_err(|_| anyhow!("Failed to send stop request"))?;
        } else {
            return Err(anyhow!("Virtual machine manager not initialized"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::time::sleep;

    use crate::vms::virtual_machine::{
        RootDriveSpec, VirtualMachine, VmID, VmSpec,
    };

    #[test]
    fn test_create_vm() {
        let id = VmID::new("test_vm");
        let spec = VmSpec {
            memory_size: 1024,
            vcpu_count: 1,
            kernel_image_path: PathBuf::from("target/kernel/vmlinuz-5.15.68"),
            kernel_args: vec![
                "console=hvc0".to_string(),
                "root=/dev/vda1".to_string(),
            ],
            root_drive: RootDriveSpec {
                host_path: PathBuf::from("target/initramfs.zst"),
                read_only: false,
            },
            mounts: Vec::new(),
        };

        let vm = VirtualMachine::new(id.clone(), spec).unwrap();
        assert_eq!(vm.id, id);

        vm.start().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        vm.stop().unwrap();
    }
}
