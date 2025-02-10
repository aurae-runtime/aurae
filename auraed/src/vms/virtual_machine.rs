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
use crate::vms::manager::Manager;
use anyhow::anyhow;
use net_util::MacAddr;
use std::{
    fmt::{self, Display},
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
};
#[cfg(target_arch = "x86_64")]
use vmm::config::DebugConsoleConfig;
use vmm::{
    api::ApiAction,
    config::{
        default_console, default_serial, CpuFeatures, CpusConfig,
        HotplugMethod, MemoryConfig, PayloadConfig, RngConfig, VhostMode,
        DEFAULT_DISK_NUM_QUEUES, DEFAULT_DISK_QUEUE_SIZE,
        DEFAULT_MAX_PHYS_BITS, DEFAULT_NET_NUM_QUEUES, DEFAULT_NET_QUEUE_SIZE,
    },
    vm::VmState,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct VmID(String);

impl VmID {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Display for VmID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
        vmm::vm_config::VmConfig {
            cpus: CpusConfig {
                boot_vcpus: spec.vcpu_count as u8,
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
                initramfs: None,
            }),
            rate_limit_groups: None,
            disks: Some(spec.mounts.into_iter().map(Into::into).collect()),
            net: Some(spec.net.into_iter().map(Into::into).collect()),
            rng: RngConfig::default(),
            balloon: None,
            fs: None,
            pmem: None,
            serial: default_serial(),
            console: default_console(),
            #[cfg(target_arch = "x86_64")]
            debug_console: DebugConsoleConfig::default(),
            devices: None,
            user_devices: None,
            vdpa: None,
            vsock: None,
            pvpanic: false,
            iommu: false,
            #[cfg(target_arch = "x86_64")]
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
    pub status: VmStatus,
    manager: Arc<Mutex<Manager>>,
}

impl fmt::Debug for VirtualMachine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualMachine {{ id: {:?}, vm: {:?} }}", self.id, self.vm)
    }
}

#[derive(Debug, Clone)]
pub struct VmStatus(VmState);

impl Display for VmStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
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
            status: VmStatus(VmState::Created),
            manager: Arc::new(Mutex::new(manager)),
        })
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        if let VmState::Running = self.status.0 {
            return Err(anyhow!("Virtual machine already running"));
        }
        let manager = self
            .manager
            .lock()
            .map_err(|_| anyhow!("Failed to aquire lock for vm manager"))?;

        if let Some(sender) = &manager.sender {
            let _ = vmm::api::VmBoot
                .send(manager.events.try_clone()?, sender.clone(), ())
                .map_err(|e| anyhow!("Failed to send start request: {e}"))?;
            self.status = VmStatus(VmState::Running);
        } else {
            return Err(anyhow!("Virtual machine manager not initialized"))?;
        }

        // Update the VM with the network device information if it wasn't provided
        if self.vm.net.is_empty() {
            if let Some(net) = &self.info()?.net {
                self.vm.net = net
                    .iter()
                    .map(|n| NetSpec {
                        tap: n.tap.clone(),
                        ip: n.ip,
                        mask: n.mask,
                        mac: n.mac,
                        host_mac: n.host_mac,
                    })
                    .collect();
            }
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        if let VmState::Shutdown = self.status.0 {
            return Err(anyhow!("Virtual machine already stopped"));
        }
        let manager = self
            .manager
            .lock()
            .map_err(|_| anyhow!("Failed to aquire lock for vm manager"))?;

        if let Some(sender) = &manager.sender {
            let _ = vmm::api::VmShutdown
                .send(manager.events.try_clone()?, sender.clone(), ())
                .map_err(|e| anyhow!("Failed to send stop request: {e}"))?;
            self.status = VmStatus(VmState::Shutdown);
        } else {
            return Err(anyhow!("Virtual machine manager not initialized"));
        }

        Ok(())
    }

    pub fn delete(&mut self) -> Result<(), anyhow::Error> {
        if self.status.0 != VmState::Shutdown {
            self.stop()?;
        };
        let manager = self
            .manager
            .lock()
            .map_err(|_| anyhow!("Failed to aquire lock for vm manager"))?;

        if let Some(sender) = &manager.sender {
            let _ = vmm::api::VmDelete
                .send(manager.events.try_clone()?, sender.clone(), ())
                .map_err(|e| anyhow!("Failed to send destroy request: {e}"))?;
            return Ok(());
        }
        Err(anyhow!("Virtual machine manager not initialized"))
    }

    fn info(&self) -> Result<vmm::vm_config::VmConfig, anyhow::Error> {
        let manager = self
            .manager
            .lock()
            .map_err(|_| anyhow!("Failed to aquire lock for vm manager"))?;

        if let Some(sender) = &manager.sender {
            let res = vmm::api::VmInfo
                .send(manager.events.try_clone()?, sender.clone(), ())
                .map_err(|e| anyhow!("Failed to send info request: {e}"))?;
            let config = res
                .config
                .lock()
                .map_err(|_| anyhow!("Failed to aquire lock for vm config"))?;
            return Ok(config.clone());
        }
        Err(anyhow!("Virtual machine manager not initialized"))
    }

    /// Get a reference to the address of the TAP device for this VM
    pub fn tap(&self) -> Option<SocketAddr> {
        let manager = self.manager.lock().ok()?;

        // Retrieve config from the VMM
        let res = vmm::api::VmInfo
            .send(manager.events.try_clone().ok()?, manager.sender.clone()?, ())
            .ok()?;
        let config = res.config.lock().ok()?;
        let net = config.net.clone()?;

        let iface = net.first()?.tap.clone()?;
        let scope_id = nix::net::if_::if_nametoindex(iface.as_str()).ok()?;

        // TODO: Make this somehow configurable
        let addr: SocketAddr = format!("[fe80::2%{scope_id}]:8080")
            .parse()
            .expect("failed to parse socket address for aurae client");
        Some(addr)
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
    #[ignore]
    fn test_create_vm() {
        let id = VmID::new("test_vm");
        let spec = VmSpec {
            memory_size: 1024,
            vcpu_count: 4,
            kernel_image_path: PathBuf::from(
                "/var/lib/aurae/vm/kernel/vmlinux.bin",
            ),
            kernel_args: vec![
                "console=hvc0".to_string(),
                "root=/dev/vda1".to_string(),
            ],
            mounts: vec![MountSpec {
                host_path: PathBuf::from("/var/lib/aurae/vm/image/disk.raw"),
                read_only: false,
            }],
            net: vec![NetSpec {
                tap: Some("tap0".to_string()),
                ip: Ipv4Addr::new(192, 168, 249, 1),
                mask: Ipv4Addr::new(255, 255, 255, 255),
                mac: MacAddr::local_random(),
                host_mac: None,
            }],
        };

        let mut vm = VirtualMachine::new(id.clone(), spec).unwrap();
        assert_eq!(vm.id, id);

        assert!(vm.start().is_ok(), "{:?}", vm);

        // Give the VM some time to boot
        std::thread::sleep(std::time::Duration::from_secs(10));
        assert!(vm.stop().is_ok(), "{:?}", vm);

        std::thread::sleep(std::time::Duration::from_secs(5));
        assert!(vm.delete().is_ok(), "{:?}", vm);
    }
}
