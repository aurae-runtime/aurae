use anyhow::anyhow;
use net_util::MacAddr;
use proto::vms::DriveMount;
use std::{
    fmt,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use vmm::{
    api::ApiAction,
    config::{
        ConsoleConfig, ConsoleOutputMode, CpuFeatures, CpusConfig,
        DebugConsoleConfig, DiskConfig, HotplugMethod, MemoryConfig,
        PayloadConfig, RngConfig, VhostMode, DEFAULT_DISK_NUM_QUEUES,
        DEFAULT_DISK_QUEUE_SIZE, DEFAULT_MAX_PHYS_BITS, DEFAULT_NET_NUM_QUEUES,
        DEFAULT_NET_QUEUE_SIZE,
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
    pub mounts: Vec<MountSpec>,
    pub net: Vec<NetSpec>,
}

impl From<VmSpec> for vmm::vm_config::VmConfig {
    fn from(spec: VmSpec) -> Self {
        let drives = spec.mounts.into_iter().map(Into::into).collect();
        vmm::vm_config::VmConfig {
            cpus: CpusConfig {
                boot_vcpus: 1,
                max_vcpus: spec.vcpu_count as u8,
                topology: None,
                kvm_hyperv: false,
                max_phys_bits: DEFAULT_MAX_PHYS_BITS,
                affinity: None,
                features: CpuFeatures::default(),
            },
            memory: MemoryConfig {
                size: (spec.memory_size << 20) as u64,
                mergeable: false,
                hotplug_method: HotplugMethod::default(),
                hotplug_size: None,
                hotplugged_size: None,
                shared: false,
                hugepages: false,
                hugepage_size: None,
                prefault: false,
                zones: None,
                thp: false,
            },
            payload: Some(PayloadConfig {
                firmware: None,
                kernel: Some(spec.kernel_image_path),
                cmdline: Some(spec.kernel_args.join(" ")),
                initramfs: Some(PathBuf::from("target/initramfs.zst")),
            }),
            rate_limit_groups: None,
            disks: Some(drives),
            net: Some(spec.net.into_iter().map(Into::into).collect()),
            rng: RngConfig::default(),
            balloon: None,
            fs: None,
            pmem: None,
            serial: ConsoleConfig {
                file: None,
                mode: ConsoleOutputMode::Tty,
                iommu: false,
                socket: None,
            },
            console: ConsoleConfig {
                file: None,
                mode: ConsoleOutputMode::Null,
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
pub struct NetSpec {
    pub tap: Option<String>,
    pub ip: Ipv4Addr,
    pub mask: Ipv4Addr,
    pub mac: MacAddr,
    pub host_mac: Option<MacAddr>,
}

impl From<NetSpec> for vmm::vm_config::NetConfig {
    fn from(spec: NetSpec) -> Self {
        vmm::vm_config::NetConfig {
            tap: spec.tap,
            ip: spec.ip,
            mask: spec.mask,
            mac: spec.mac,
            host_mac: spec.host_mac,
            mtu: None,
            iommu: false,
            num_queues: DEFAULT_NET_NUM_QUEUES,
            queue_size: DEFAULT_NET_QUEUE_SIZE,
            vhost_user: false,
            vhost_socket: None,
            vhost_mode: VhostMode::default(),
            id: None,
            fds: None,
            rate_limiter_config: None,
            pci_segment: 0,
            offload_tso: false,
            offload_ufo: false,
            offload_csum: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MountSpec {
    pub host_path: PathBuf,
    pub read_only: bool,
}

impl From<MountSpec> for vmm::vm_config::DiskConfig {
    fn from(spec: MountSpec) -> Self {
        vmm::vm_config::DiskConfig {
            path: Some(spec.host_path),
            readonly: spec.read_only,
            direct: false,
            iommu: false,
            num_queues: DEFAULT_DISK_NUM_QUEUES,
            queue_size: DEFAULT_DISK_QUEUE_SIZE,
            vhost_user: false,
            vhost_socket: None,
            rate_limit_group: None,
            rate_limiter_config: None,
            id: None,
            disable_io_uring: false,
            disable_aio: false,
            pci_segment: 0,
            serial: None,
            queue_affinity: None,
        }
    }
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
                .map_err(|e| anyhow!("Failed to send stop request: {e}"))?;
        } else {
            return Err(anyhow!("Virtual machine manager not initialized"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv4Addr, path::PathBuf};

    use net_util::MacAddr;

    use crate::vms::virtual_machine::{
        MountSpec, NetSpec, VirtualMachine, VmID, VmSpec,
    };

    #[test]
    fn test_create_vm_auraed_init() {
        let id = VmID::new("test_vm");
        let spec = VmSpec {
            memory_size: 2048,
            vcpu_count: 2,
            kernel_image_path: PathBuf::from("target/kernel/vmlinuz-5.15.68"),
            kernel_args: vec![
                "console=ttyS0".to_string(),
                "root=/dev/vda1".to_string(),
            ],
            mounts: vec![
                MountSpec {
                    host_path: PathBuf::from(
                        "target/disk/focal-server-cloudimg-amd64.raw",
                    ),
                    read_only: false,
                },
                MountSpec {
                    host_path: PathBuf::from(
                        "target/disk/ubuntu-cloudinit.img",
                    ),
                    read_only: false,
                },
            ],
            net: vec![NetSpec {
                tap: Some("tap0".to_string()),
                ip: Ipv4Addr::new(192, 168, 122, 1),
                mask: Ipv4Addr::new(255, 255, 255, 0),
                mac: MacAddr::local_random(),
                host_mac: Some(
                    MacAddr::parse_str("52:54:00:27:bb:eb")
                        .expect("Failed to parse MAC address"),
                ),
            }],
        };

        let vm = VirtualMachine::new(id.clone(), spec).unwrap();
        assert_eq!(vm.id, id);

        vm.start().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(120));
        vm.stop().unwrap();
    }
}
