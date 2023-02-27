// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//! Reference VMM built with rust-vmm components and minimal glue.
#![deny(missing_docs)]

use std::convert::TryFrom;
#[cfg(target_arch = "aarch64")]
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, stdin, stdout};
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use event_manager::{EventManager, EventOps, Events, MutEventSubscriber, SubscriberOps};
use irq_allocator::IrqAllocator;
use kvm_bindings::KVM_API_VERSION;
use kvm_ioctls::{
    Cap::{self, Ioeventfd, Irqchip, Irqfd, UserMemory},
    Kvm,
};
use linux_loader::cmdline;
#[cfg(target_arch = "x86_64")]
use linux_loader::configurator::{
    self, linux::LinuxBootConfigurator, BootConfigurator, BootParams,
};
#[cfg(target_arch = "x86_64")]
use linux_loader::{bootparam::boot_params, cmdline::Cmdline};

use linux_loader::loader::{self, KernelLoader, KernelLoaderResult};
#[cfg(target_arch = "x86_64")]
use linux_loader::loader::{
    bzimage::BzImage,
    elf::{self, Elf},
    load_cmdline,
};
use vm_device::bus::{MmioAddress, MmioRange};
#[cfg(target_arch = "x86_64")]
use vm_device::bus::{PioAddress, PioRange};
use vm_device::device_manager::IoManager;
#[cfg(target_arch = "aarch64")]
use vm_device::device_manager::MmioManager;
#[cfg(target_arch = "x86_64")]
use vm_device::device_manager::PioManager;
#[cfg(target_arch = "aarch64")]
use vm_memory::GuestMemoryRegion;
use vm_memory::{GuestAddress, GuestMemory, GuestMemoryMmap};
#[cfg(target_arch = "x86_64")]
use vm_superio::I8042Device;
#[cfg(target_arch = "aarch64")]
use vm_superio::Rtc;
use vm_superio::Serial;
use vmm_sys_util::{epoll::EventSet, eventfd::EventFd, terminal::Terminal};

#[cfg(target_arch = "x86_64")]
use boot::build_bootparams;
pub use config::*;
use devices::virtio::block::{self, BlockArgs};
use devices::virtio::net::{self, NetArgs};
use devices::virtio::{Env, MmioConfig};

#[cfg(target_arch = "x86_64")]
use devices::legacy::I8042Wrapper;
use devices::legacy::{EventFdTrigger, SerialWrapper};
use vm_vcpu::vm::{self, ExitHandler, KvmVm, VmConfig};

#[cfg(target_arch = "aarch64")]
use devices::legacy::RtcWrapper;

#[cfg(target_arch = "aarch64")]
use arch::{FdtBuilder, AARCH64_FDT_MAX_SIZE, AARCH64_MMIO_BASE, AARCH64_PHYS_MEM_START};

use vm_allocator::{AddressAllocator, AllocPolicy, RangeInclusive};

use vm_vcpu::vm::MAX_IRQ;

mod boot;
mod config;
mod irq_allocator;

/// First address past 32 bits is where the MMIO gap ends.
#[cfg(target_arch = "x86_64")]
pub(crate) const MMIO_GAP_END: u64 = 1 << 32;
/// Size of the MMIO gap.
#[cfg(target_arch = "x86_64")]
pub(crate) const MMIO_GAP_SIZE: u64 = 768 << 20;
/// The start of the MMIO gap (memory area reserved for MMIO devices).
#[cfg(target_arch = "x86_64")]
pub(crate) const MMIO_GAP_START: u64 = MMIO_GAP_END - MMIO_GAP_SIZE;
/// Address of the zeropage, where Linux kernel boot parameters are written.
#[cfg(target_arch = "x86_64")]
const ZEROPG_START: u64 = 0x7000;
/// Address where the kernel command line is written.
#[cfg(target_arch = "x86_64")]
const CMDLINE_START: u64 = 0x0002_0000;

/// Default high memory start (1 MiB).
#[cfg(target_arch = "x86_64")]
pub const DEFAULT_HIGH_RAM_START: u64 = 0x0010_0000;

/// Default address for loading the kernel.
#[cfg(target_arch = "x86_64")]
pub const DEFAULT_KERNEL_LOAD_ADDR: u64 = DEFAULT_HIGH_RAM_START;
#[cfg(target_arch = "aarch64")]
/// Default address for loading the kernel.
pub const DEFAULT_KERNEL_LOAD_ADDR: u64 = AARCH64_PHYS_MEM_START;

/// Default kernel command line.
#[cfg(target_arch = "x86_64")]
pub const DEFAULT_KERNEL_CMDLINE: &str = "panic=1 pci=off";
#[cfg(target_arch = "aarch64")]
/// Default kernel command line.
pub const DEFAULT_KERNEL_CMDLINE: &str = "reboot=t panic=1 pci=off";
/// Default address allocator alignment. It needs to be a power of 2.
pub const DEFAULT_ADDRESSS_ALIGNEMNT: u64 = 4;
/// Default allocation policy for address allocator.
pub const DEFAULT_ALLOC_POLICY: AllocPolicy = AllocPolicy::FirstMatch;

/// IRQ line 4 is typically used for serial port 1.
// See more IRQ assignments & info: https://tldp.org/HOWTO/Serial-HOWTO-8.html
const SERIAL_IRQ: u32 = 4;

/// VMM memory related errors.
#[derive(Debug)]
pub enum MemoryError {
    /// Not enough memory slots.
    NotEnoughMemorySlots,
    /// AddressAllocatorError
    AddressAllocatorError(vm_allocator::Error),
    /// Failed to configure guest memory.
    VmMemory(vm_memory::Error),
}

/// VMM errors.
#[derive(Debug)]
pub enum Error {
    /// Failed to create block device.
    Block(block::Error),
    /// Failed to write boot parameters to guest memory.
    #[cfg(target_arch = "x86_64")]
    BootConfigure(configurator::Error),
    /// Error configuring boot parameters.
    #[cfg(target_arch = "x86_64")]
    BootParam(boot::Error),
    /// Error configuring the kernel command line.
    Cmdline(cmdline::Error),
    /// Error setting up the serial device.
    SerialDevice(devices::legacy::SerialError),
    /// Event management error.
    EventManager(event_manager::Error),
    /// I/O error.
    IO(io::Error),
    /// Failed to load kernel.
    KernelLoad(loader::Error),
    /// Failed to create net device.
    Net(net::Error),
    /// Address stored in the rip registry does not fit in guest memory.
    RipOutOfGuestMemory,
    /// Invalid KVM API version.
    KvmApiVersion(i32),
    /// Unsupported KVM capability.
    KvmCap(Cap),
    /// Error issuing an ioctl to KVM.
    KvmIoctl(kvm_ioctls::Error),
    /// Memory error.
    Memory(MemoryError),
    /// VM errors.
    Vm(vm::Error),
    /// Exit event errors.
    ExitEvent(io::Error),
    #[cfg(target_arch = "x86_64")]
    /// Cannot retrieve the supported MSRs.
    GetSupportedMsrs(vm_vcpu_ref::x86_64::msrs::Error),
    #[cfg(target_arch = "aarch64")]
    /// Cannot setup the FDT for booting.
    SetupFdt(arch::Error),
    /// IrqAllocator error
    IrqAllocator(irq_allocator::Error),
}

impl std::convert::From<vm::Error> for Error {
    fn from(vm_error: vm::Error) -> Self {
        Self::Vm(vm_error)
    }
}

impl From<vm_allocator::Error> for crate::Error {
    fn from(error: vm_allocator::Error) -> Self {
        crate::Error::Memory(MemoryError::AddressAllocatorError(error))
    }
}

impl From<irq_allocator::Error> for crate::Error {
    fn from(error: irq_allocator::Error) -> Self {
        crate::Error::IrqAllocator(error)
    }
}

/// Dedicated [`Result`](https://doc.rust-lang.org/std/result/) type.
pub type Result<T> = std::result::Result<T, Error>;

type Block = block::Block<Arc<GuestMemoryMmap>>;
type Net = net::Net<Arc<GuestMemoryMmap>>;

/// A live VMM.
pub struct Vmm {
    vm: KvmVm<WrappedExitHandler>,
    kernel_cfg: KernelConfig,
    guest_memory: GuestMemoryMmap,
    address_allocator: AddressAllocator,
    irq_allocator: IrqAllocator,
    // The `device_mgr` is an Arc<Mutex> so that it can be shared between
    // the Vcpu threads, and modified when new devices are added.
    device_mgr: Arc<Mutex<IoManager>>,
    // Arc<Mutex<>> because the same device (a dyn DevicePio/DeviceMmio from IoManager's
    // perspective, and a dyn MutEventSubscriber from EventManager's) is managed by the 2 entities,
    // and isn't Copy-able; so once one of them gets ownership, the other one can't anymore.
    event_mgr: EventManager<Arc<Mutex<dyn MutEventSubscriber + Send>>>,
    exit_handler: WrappedExitHandler,
    block_devices: Vec<Arc<Mutex<Block>>>,
    net_devices: Vec<Arc<Mutex<Net>>>,
    // TODO: fetch the vcpu number from the `vm` object.
    // TODO-continued: this is needed to make the arm POC work as we need to create the FDT
    // TODO-continued: after the other resources are created.
    #[cfg(target_arch = "aarch64")]
    num_vcpus: u64,
    #[cfg(target_arch = "aarch64")]
    fdt_builder: FdtBuilder,
}

// The `VmmExitHandler` is used as the mechanism for exiting from the event manager loop.
// The Vm is notifying us through the `kick` method when it exited. Once the Vm finished
// the execution, it is time for the event manager loop to also exit. This way, we can
// terminate the VMM process cleanly.
struct VmmExitHandler {
    exit_event: EventFd,
    keep_running: AtomicBool,
}

// The wrapped exit handler is needed because the ownership of the inner `VmmExitHandler` is
// shared between the `KvmVm` and the `EventManager`. Clone is required for implementing the
// `ExitHandler` trait.
#[derive(Clone)]
struct WrappedExitHandler(Arc<Mutex<VmmExitHandler>>);

impl WrappedExitHandler {
    fn new() -> Result<WrappedExitHandler> {
        Ok(WrappedExitHandler(Arc::new(Mutex::new(VmmExitHandler {
            exit_event: EventFd::new(libc::EFD_NONBLOCK).map_err(Error::ExitEvent)?,
            keep_running: AtomicBool::new(true),
        }))))
    }

    fn keep_running(&self) -> bool {
        self.0.lock().unwrap().keep_running.load(Ordering::Acquire)
    }
}

impl ExitHandler for WrappedExitHandler {
    fn kick(&self) -> io::Result<()> {
        self.0.lock().unwrap().exit_event.write(1)
    }
}

impl MutEventSubscriber for VmmExitHandler {
    fn process(&mut self, events: Events, ops: &mut EventOps) {
        if events.event_set().contains(EventSet::IN) {
            self.keep_running.store(false, Ordering::Release);
        }
        if events.event_set().contains(EventSet::ERROR) {
            // We cannot do much about the error (besides log it).
            // TODO: log this error once we have a logger set up.
            let _ = ops.remove(Events::new(&self.exit_event, EventSet::IN));
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        ops.add(Events::new(&self.exit_event, EventSet::IN))
            .expect("Cannot initialize exit handler.");
    }
}

impl TryFrom<VMMConfig> for Vmm {
    type Error = Error;

    fn try_from(config: VMMConfig) -> Result<Self> {
        let kvm = Kvm::new().map_err(Error::KvmIoctl)?;

        // Check that the KVM on the host is supported.
        let kvm_api_ver = kvm.get_api_version();
        if kvm_api_ver != KVM_API_VERSION as i32 {
            return Err(Error::KvmApiVersion(kvm_api_ver));
        }
        Vmm::check_kvm_capabilities(&kvm)?;

        let guest_memory = Vmm::create_guest_memory(&config.memory_config)?;
        let address_allocator = Vmm::create_address_allocator(&config.memory_config)?;
        let device_mgr = Arc::new(Mutex::new(IoManager::new()));

        // Create the KvmVm.
        let vm_config = VmConfig::new(&kvm, config.vcpu_config.num, MAX_IRQ)?;
        let wrapped_exit_handler = WrappedExitHandler::new()?;
        let vm = KvmVm::new(
            &kvm,
            vm_config,
            &guest_memory,
            wrapped_exit_handler.clone(),
            device_mgr.clone(),
        )?;

        let mut event_manager = EventManager::<Arc<Mutex<dyn MutEventSubscriber + Send>>>::new()
            .map_err(Error::EventManager)?;
        event_manager.add_subscriber(wrapped_exit_handler.0.clone());
        #[cfg(target_arch = "aarch64")]
        let fdt_builder = FdtBuilder::new();

        let irq_allocator = IrqAllocator::new(SERIAL_IRQ, vm.max_irq())?;

        let mut vmm = Vmm {
            vm,
            guest_memory,
            address_allocator,
            irq_allocator,
            device_mgr,
            event_mgr: event_manager,
            kernel_cfg: config.kernel_config,
            exit_handler: wrapped_exit_handler,
            block_devices: Vec::new(),
            net_devices: Vec::new(),
            #[cfg(target_arch = "aarch64")]
            num_vcpus: config.vcpu_config.num as u64,
            #[cfg(target_arch = "aarch64")]
            fdt_builder,
        };
        vmm.add_serial_console()?;
        #[cfg(target_arch = "x86_64")]
        vmm.add_i8042_device()?;
        #[cfg(target_arch = "aarch64")]
        vmm.add_rtc_device()?;

        // Adding the virtio devices. We'll come up with a cleaner abstraction for `Env`.
        if let Some(cfg) = config.block_config.as_ref() {
            vmm.add_block_device(cfg)?;
        }

        if let Some(cfg) = config.net_config.as_ref() {
            vmm.add_net_device(cfg)?;
        }

        Ok(vmm)
    }
}

impl Vmm {
    /// Run the VMM.
    pub fn run(&mut self) -> Result<()> {
        let load_result = self.load_kernel()?;
        #[cfg(target_arch = "x86_64")]
        let kernel_load_addr = self.compute_kernel_load_addr(&load_result)?;
        #[cfg(target_arch = "aarch64")]
        let kernel_load_addr = load_result.kernel_load;
        #[cfg(target_arch = "aarch64")]
        self.setup_fdt()?;

        if stdin().lock().set_raw_mode().is_err() {
            eprintln!("Failed to set raw mode on terminal. Stdin will echo.");
        }

        self.vm.run(Some(kernel_load_addr)).map_err(Error::Vm)?;
        loop {
            match self.event_mgr.run() {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to handle events: {:?}", e),
            }
            if !self.exit_handler.keep_running() {
                break;
            }
        }
        self.vm.shutdown();

        Ok(())
    }

    // Create guest memory regions.
    fn create_guest_memory(memory_config: &MemoryConfig) -> Result<GuestMemoryMmap> {
        let mem_size = ((memory_config.size_mib as u64) << 20) as usize;
        let mem_regions = Vmm::create_memory_regions(mem_size);

        // Create guest memory from regions.
        GuestMemoryMmap::from_ranges(&mem_regions)
            .map_err(|e| Error::Memory(MemoryError::VmMemory(e)))
    }

    fn create_memory_regions(mem_size: usize) -> Vec<(GuestAddress, usize)> {
        #[cfg(target_arch = "x86_64")]
        // On x86_64, they surround the MMIO gap (dedicated space for MMIO device slots) if the
        // configured memory size exceeds the latter's starting address.
        match mem_size.checked_sub(MMIO_GAP_START as usize) {
            // Guest memory fits before the MMIO gap.
            None | Some(0) => vec![(GuestAddress(0), mem_size)],
            // Guest memory extends beyond the MMIO gap.
            Some(remaining) => vec![
                (GuestAddress(0), MMIO_GAP_START as usize),
                (GuestAddress(MMIO_GAP_END), remaining),
            ],
        }

        #[cfg(target_arch = "aarch64")]
        vec![(GuestAddress(AARCH64_PHYS_MEM_START), mem_size)]
    }

    fn create_address_allocator(memory_config: &MemoryConfig) -> Result<AddressAllocator> {
        let mem_size = (memory_config.size_mib as u64) << 20;
        #[cfg(target_arch = "x86_64")]
        let start_addr = MMIO_GAP_START;
        #[cfg(target_arch = "aarch64")]
        let start_addr = AARCH64_MMIO_BASE;
        let address_allocator = AddressAllocator::new(start_addr, mem_size)?;
        Ok(address_allocator)
    }

    // Load the kernel into guest memory.
    #[cfg(target_arch = "x86_64")]
    fn load_kernel(&mut self) -> Result<KernelLoaderResult> {
        let mut kernel_image = File::open(&self.kernel_cfg.path).map_err(Error::IO)?;
        let zero_page_addr = GuestAddress(ZEROPG_START);

        // Load the kernel into guest memory.
        let kernel_load = match Elf::load(
            &self.guest_memory,
            None,
            &mut kernel_image,
            Some(GuestAddress(self.kernel_cfg.load_addr)),
        ) {
            Ok(result) => result,
            Err(loader::Error::Elf(elf::Error::InvalidElfMagicNumber)) => BzImage::load(
                &self.guest_memory,
                None,
                &mut kernel_image,
                Some(GuestAddress(self.kernel_cfg.load_addr)),
            )
            .map_err(Error::KernelLoad)?,
            Err(e) => {
                return Err(Error::KernelLoad(e));
            }
        };

        // Generate boot parameters.
        let mut bootparams = build_bootparams(
            &self.guest_memory,
            &kernel_load,
            GuestAddress(self.kernel_cfg.load_addr),
            GuestAddress(MMIO_GAP_START),
            GuestAddress(MMIO_GAP_END),
        )
        .map_err(Error::BootParam)?;

        // Add the kernel command line to the boot parameters.
        bootparams.hdr.cmd_line_ptr = CMDLINE_START as u32;
        bootparams.hdr.cmdline_size = self.kernel_cfg.cmdline.as_str().len() as u32 + 1;

        // Load the kernel command line into guest memory.
        let mut cmdline = Cmdline::new(4096);
        cmdline
            .insert_str(self.kernel_cfg.cmdline.as_str())
            .map_err(Error::Cmdline)?;

        load_cmdline(
            &self.guest_memory,
            GuestAddress(CMDLINE_START),
            // Safe because we know the command line string doesn't contain any 0 bytes.
            &cmdline,
        )
        .map_err(Error::KernelLoad)?;

        // Write the boot parameters in the zeropage.
        LinuxBootConfigurator::write_bootparams::<GuestMemoryMmap>(
            &BootParams::new::<boot_params>(&bootparams, zero_page_addr),
            &self.guest_memory,
        )
        .map_err(Error::BootConfigure)?;

        Ok(kernel_load)
    }

    #[cfg(target_arch = "aarch64")]
    fn load_kernel(&mut self) -> Result<KernelLoaderResult> {
        let mut kernel_image = File::open(&self.kernel_cfg.path).map_err(Error::IO)?;
        linux_loader::loader::pe::PE::load(
            &self.guest_memory,
            Some(GuestAddress(self.kernel_cfg.load_addr)),
            &mut kernel_image,
            None,
        )
        .map_err(Error::KernelLoad)
    }

    // Create and add a serial console to the VMM.
    fn add_serial_console(&mut self) -> Result<()> {
        // Create the serial console.
        let interrupt_evt = EventFdTrigger::new(libc::EFD_NONBLOCK).map_err(Error::IO)?;
        let serial = Arc::new(Mutex::new(SerialWrapper(Serial::new(
            interrupt_evt.try_clone().map_err(Error::IO)?,
            stdout(),
        ))));

        // Register its interrupt fd with KVM.
        self.vm.register_irqfd(&interrupt_evt, SERIAL_IRQ)?;

        self.kernel_cfg
            .cmdline
            .insert_str("console=ttyS0")
            .map_err(Error::Cmdline)?;

        #[cfg(target_arch = "aarch64")]
        self.kernel_cfg
            .cmdline
            .insert_str(&format!("earlycon=uart,mmio,0x{:08x}", AARCH64_MMIO_BASE))
            .map_err(Error::Cmdline)?;

        // Put it on the bus.
        // Safe to use unwrap() because the device manager is instantiated in new(), there's no
        // default implementation, and the field is private inside the VMM struct.
        #[cfg(target_arch = "x86_64")]
        {
            let range = PioRange::new(PioAddress(0x3f8), 0x8).unwrap();
            self.device_mgr
                .lock()
                .unwrap()
                .register_pio(range, serial.clone())
                .unwrap();
        }

        #[cfg(target_arch = "aarch64")]
        {
            let range = self.address_allocator.allocate(
                0x1000,
                DEFAULT_ADDRESSS_ALIGNEMNT,
                AllocPolicy::ExactMatch(AARCH64_MMIO_BASE),
            )?;
            self.fdt_builder
                .with_serial_console(range.start(), range.len());
            let range = mmio_from_range(&range);
            self.device_mgr
                .lock()
                .unwrap()
                .register_mmio(range, serial.clone())
                .unwrap();
        }

        // Hook it to event management.
        self.event_mgr.add_subscriber(serial);

        Ok(())
    }

    // Create and add a i8042 device to the VMM.
    #[cfg(target_arch = "x86_64")]
    fn add_i8042_device(&mut self) -> Result<()> {
        let reset_evt = EventFdTrigger::new(libc::EFD_NONBLOCK).map_err(Error::IO)?;
        let i8042_device = Arc::new(Mutex::new(I8042Wrapper(I8042Device::new(
            reset_evt.try_clone().map_err(Error::IO)?,
        ))));
        self.vm.register_irqfd(&reset_evt, 1)?;
        let range = PioRange::new(PioAddress(0x060), 0x5).unwrap();

        self.device_mgr
            .lock()
            .unwrap()
            .register_pio(range, i8042_device)
            .unwrap();
        Ok(())
    }

    #[cfg(target_arch = "aarch64")]
    fn add_rtc_device(&mut self) -> Result<()> {
        let rtc = Arc::new(Mutex::new(RtcWrapper(Rtc::new())));
        let range = self.address_allocator.allocate(
            0x1000,
            DEFAULT_ADDRESSS_ALIGNEMNT,
            DEFAULT_ALLOC_POLICY,
        )?;
        self.fdt_builder.with_rtc(range.start(), range.len());
        let range = mmio_from_range(&range);
        self.device_mgr
            .lock()
            .unwrap()
            .register_mmio(range, rtc)
            .unwrap();
        Ok(())
    }

    // All methods that add a virtio device use hardcoded addresses and interrupts for now, and
    // only support a single device. We need to expand this, but it looks like a good match if we
    // can do it after figuring out how to better separate concerns and make the VMM agnostic of
    // the actual device types.
    fn add_block_device(&mut self, cfg: &BlockConfig) -> Result<()> {
        let mem = Arc::new(self.guest_memory.clone());
        let range = self.address_allocator.allocate(
            0x1000,
            DEFAULT_ADDRESSS_ALIGNEMNT,
            DEFAULT_ALLOC_POLICY,
        )?;
        let irq = self.irq_allocator.next_irq()?;
        let mmio_range = mmio_from_range(&range);
        let mmio_cfg = MmioConfig {
            range: mmio_range,
            gsi: irq,
        };

        let mut guard = self.device_mgr.lock().unwrap();

        let mut env = Env {
            mem,
            vm_fd: self.vm.vm_fd(),
            event_mgr: &mut self.event_mgr,
            mmio_mgr: guard.deref_mut(),
            mmio_cfg,
            kernel_cmdline: &mut self.kernel_cfg.cmdline,
        };

        let args = BlockArgs {
            file_path: PathBuf::from(&cfg.path),
            read_only: false,
            root_device: true,
            advertise_flush: true,
        };

        // We can also hold this somewhere if we need to keep the handle for later.
        let block = Block::new(&mut env, &args).map_err(Error::Block)?;
        #[cfg(target_arch = "aarch64")]
        self.fdt_builder
            .add_virtio_device(range.start(), range.len(), irq);
        self.block_devices.push(block);

        Ok(())
    }

    fn add_net_device(&mut self, cfg: &NetConfig) -> Result<()> {
        let mem = Arc::new(self.guest_memory.clone());
        let range = self.address_allocator.allocate(
            0x1000,
            DEFAULT_ADDRESSS_ALIGNEMNT,
            DEFAULT_ALLOC_POLICY,
        )?;
        let mmio_range = mmio_from_range(&range);
        let irq = self.irq_allocator.next_irq()?;
        let mmio_cfg = MmioConfig {
            range: mmio_range,
            gsi: irq,
        };

        let mut guard = self.device_mgr.lock().unwrap();

        let mut env = Env {
            mem,
            vm_fd: self.vm.vm_fd(),
            event_mgr: &mut self.event_mgr,
            mmio_mgr: guard.deref_mut(),
            mmio_cfg,
            kernel_cmdline: &mut self.kernel_cfg.cmdline,
        };

        let args = NetArgs {
            tap_name: cfg.tap_name.clone(),
        };

        // We can also hold this somewhere if we need to keep the handle for later.
        let net = Net::new(&mut env, &args).map_err(Error::Net)?;
        self.net_devices.push(net);
        #[cfg(target_arch = "aarch64")]
        self.fdt_builder
            .add_virtio_device(range.start(), range.len(), irq);
        Ok(())
    }

    // Helper function that computes the kernel_load_addr needed by the
    // VcpuState when creating the Vcpus.
    #[cfg(target_arch = "x86_64")]
    fn compute_kernel_load_addr(&self, kernel_load: &KernelLoaderResult) -> Result<GuestAddress> {
        // If the kernel format is bzImage, the real-mode code is offset by
        // 0x200, so that's where we have to point the rip register for the
        // first instructions to execute.
        // See https://www.kernel.org/doc/html/latest/x86/boot.html#memory-layout
        //
        // The kernel is a bzImage kernel if the protocol >= 2.00 and the 0x01
        // bit (LOAD_HIGH) in the loadflags field is set.
        let mut kernel_load_addr = self
            .guest_memory
            .check_address(kernel_load.kernel_load)
            .ok_or(Error::RipOutOfGuestMemory)?;
        if let Some(hdr) = kernel_load.setup_header {
            if hdr.version >= 0x200 && hdr.loadflags & 0x1 == 0x1 {
                // Yup, it's bzImage.
                kernel_load_addr = self
                    .guest_memory
                    .checked_offset(kernel_load_addr, 0x200)
                    .ok_or(Error::RipOutOfGuestMemory)?;
            }
        }

        Ok(kernel_load_addr)
    }

    fn check_kvm_capabilities(kvm: &Kvm) -> Result<()> {
        let capabilities = vec![Irqchip, Ioeventfd, Irqfd, UserMemory];

        // Check that all desired capabilities are supported.
        if let Some(c) = capabilities
            .iter()
            .find(|&capability| !kvm.check_extension(*capability))
        {
            Err(Error::KvmCap(*c))
        } else {
            Ok(())
        }
    }

    #[cfg(target_arch = "aarch64")]
    // TODO: move this where it makes sense from a config point of view as we add all
    // needed stuff in FDT.
    fn setup_fdt(&mut self) -> Result<()> {
        let mem_size: u64 = self.guest_memory.iter().map(|region| region.len()).sum();
        let fdt_offset = mem_size - AARCH64_FDT_MAX_SIZE - 0x10000;
        let cmdline = &self.kernel_cfg.cmdline;
        let fdt = self
            .fdt_builder
            .with_cmdline(cmdline.as_str().to_string())
            .with_num_vcpus(self.num_vcpus.try_into().unwrap())
            .with_mem_size(mem_size)
            .create_fdt()
            .map_err(Error::SetupFdt)?;
        fdt.write_to_mem(&self.guest_memory, fdt_offset)
            .map_err(Error::SetupFdt)?;
        Ok(())
    }
}

fn mmio_from_range(range: &RangeInclusive) -> MmioRange {
    // The following unwrap is safe because the address allocator makes
    // sure that the address is available and correct
    MmioRange::new(MmioAddress(range.start()), range.len()).unwrap()
}
#[cfg(test)]
mod tests {
    #[cfg(target_arch = "x86_64")]
    use linux_loader::elf::Elf64_Ehdr;
    #[cfg(target_arch = "x86_64")]
    use linux_loader::loader::{self, bootparam::setup_header, elf::PvhBootCapability};
    use std::io::ErrorKind;
    #[cfg(target_arch = "x86_64")]
    use std::path::Path;
    use std::path::PathBuf;
    #[cfg(target_arch = "x86_64")]
    use vm_memory::{
        bytes::{ByteValued, Bytes},
        Address, GuestAddress, GuestMemory,
    };

    use vmm_sys_util::{tempdir::TempDir, tempfile::TempFile};

    use super::*;
    use utils::resource_download::s3_download;

    const MEM_SIZE_MIB: u32 = 1024;
    const NUM_VCPUS: u8 = 1;

    #[cfg(target_arch = "x86_64")]
    fn default_bzimage_path() -> PathBuf {
        let tags = r#"
        {
            "halt_after_boot": true,
            "image_format": "bzimage"
        }
        "#;
        s3_download("kernel", Some(tags)).unwrap()
    }

    fn default_elf_path() -> PathBuf {
        let tags = r#"
        {
            "halt_after_boot": true,
            "image_format": "elf"
        }
        "#;
        s3_download("kernel", Some(tags)).unwrap()
    }

    #[cfg(target_arch = "aarch64")]
    fn default_pe_path() -> PathBuf {
        let tags = r#"
        {
            "halt_after_boot": true,
            "image_format": "pe"
        }
        "#;
        s3_download("kernel", Some(tags)).unwrap()
    }

    fn default_vmm_config() -> VMMConfig {
        VMMConfig {
            kernel_config: KernelConfig {
                #[cfg(target_arch = "x86_64")]
                path: default_elf_path(),
                #[cfg(target_arch = "aarch64")]
                path: default_pe_path(),
                load_addr: DEFAULT_KERNEL_LOAD_ADDR,
                cmdline: KernelConfig::default_cmdline(),
            },
            memory_config: MemoryConfig {
                size_mib: MEM_SIZE_MIB,
            },
            vcpu_config: VcpuConfig { num: NUM_VCPUS },
            block_config: None,
            net_config: None,
        }
    }

    fn default_exit_handler() -> WrappedExitHandler {
        WrappedExitHandler(Arc::new(Mutex::new(VmmExitHandler {
            keep_running: AtomicBool::default(),
            exit_event: EventFd::new(libc::EFD_NONBLOCK).unwrap(),
        })))
    }

    // Returns a VMM which only has the memory configured. The purpose of the mock VMM
    // is to give a finer grained control to test individual private functions in the VMM.
    fn mock_vmm(vmm_config: VMMConfig) -> Vmm {
        let kvm = Kvm::new().unwrap();
        let guest_memory = Vmm::create_guest_memory(&vmm_config.memory_config).unwrap();

        let address_allocator = Vmm::create_address_allocator(&vmm_config.memory_config).unwrap();
        // Create the KvmVm.
        let vm_config = VmConfig::new(&kvm, vmm_config.vcpu_config.num, MAX_IRQ).unwrap();

        let device_mgr = Arc::new(Mutex::new(IoManager::new()));
        let exit_handler = default_exit_handler();
        let vm = KvmVm::new(
            &kvm,
            vm_config,
            &guest_memory,
            exit_handler.clone(),
            device_mgr.clone(),
        )
        .unwrap();
        #[cfg(target_arch = "aarch64")]
        let fdt_builder = FdtBuilder::new();
        let irq_allocator = IrqAllocator::new(SERIAL_IRQ, vm.max_irq()).unwrap();
        Vmm {
            vm,
            guest_memory,
            address_allocator,
            irq_allocator,
            device_mgr,
            event_mgr: EventManager::new().unwrap(),
            kernel_cfg: vmm_config.kernel_config,
            exit_handler,
            block_devices: Vec::new(),
            net_devices: Vec::new(),
            #[cfg(target_arch = "aarch64")]
            num_vcpus: vmm_config.vcpu_config.num as u64,
            #[cfg(target_arch = "aarch64")]
            fdt_builder,
        }
    }

    // Return the address where an ELF file should be loaded, as specified in its header.
    #[cfg(target_arch = "x86_64")]
    fn elf_load_addr(elf_path: &Path) -> GuestAddress {
        let mut elf_file = File::open(elf_path).unwrap();
        let mut ehdr = Elf64_Ehdr::default();
        ehdr.as_bytes()
            .read_from(0, &mut elf_file, std::mem::size_of::<Elf64_Ehdr>())
            .unwrap();
        GuestAddress(ehdr.e_entry)
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_compute_kernel_load_addr() {
        let vmm_config = default_vmm_config();
        let vmm = mock_vmm(vmm_config);

        // ELF (vmlinux) kernel scenario: happy case
        let mut kern_load = KernelLoaderResult {
            kernel_load: GuestAddress(DEFAULT_HIGH_RAM_START), // 1 MiB.
            kernel_end: 0,                                     // doesn't matter.
            setup_header: None,
            pvh_boot_cap: PvhBootCapability::PvhEntryNotPresent,
        };
        let actual_kernel_load_addr = vmm.compute_kernel_load_addr(&kern_load).unwrap();
        let expected_load_addr = kern_load.kernel_load;
        assert_eq!(actual_kernel_load_addr, expected_load_addr);

        kern_load.kernel_load = GuestAddress(vmm.guest_memory.last_addr().raw_value() + 1);
        assert!(matches!(
            vmm.compute_kernel_load_addr(&kern_load),
            Err(Error::RipOutOfGuestMemory)
        ));

        // bzImage kernel scenario: happy case
        // The difference is that kernel_load.setup_header is no longer None, because we found one
        // while parsing the bzImage file.
        kern_load.kernel_load = GuestAddress(0x0100_0000); // 1 MiB.
        kern_load.setup_header = Some(setup_header {
            version: 0x0200, // 0x200 (v2.00) is the minimum.
            loadflags: 1,
            ..Default::default()
        });
        let expected_load_addr = kern_load.kernel_load.unchecked_add(0x200);
        let actual_kernel_load_addr = vmm.compute_kernel_load_addr(&kern_load).unwrap();
        assert_eq!(expected_load_addr, actual_kernel_load_addr);

        // bzImage kernel scenario: error case: kernel_load + 0x200 (512 - size of bzImage header)
        // falls out of guest memory
        kern_load.kernel_load = GuestAddress(vmm.guest_memory.last_addr().raw_value() - 511);
        assert!(matches!(
            vmm.compute_kernel_load_addr(&kern_load),
            Err(Error::RipOutOfGuestMemory)
        ));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_load_kernel() {
        // Test Case: load a valid elf.
        let mut vmm_config = default_vmm_config();
        vmm_config.kernel_config.path = default_elf_path();
        // ELF files start with a header that tells us where they want to be loaded.
        let kernel_load = elf_load_addr(&vmm_config.kernel_config.path);
        let mut vmm = mock_vmm(vmm_config);
        let kernel_load_result = vmm.load_kernel().unwrap();
        assert_eq!(kernel_load_result.kernel_load, kernel_load);
        assert!(kernel_load_result.setup_header.is_none());

        // Test case: load a valid bzImage.
        let mut vmm_config = default_vmm_config();
        vmm_config.kernel_config.path = default_bzimage_path();
        let mut vmm = mock_vmm(vmm_config);
        let kernel_load_result = vmm.load_kernel().unwrap();
        assert_eq!(
            kernel_load_result.kernel_load,
            GuestAddress(DEFAULT_HIGH_RAM_START)
        );
        assert!(kernel_load_result.setup_header.is_some());
    }

    #[test]
    fn test_load_kernel_errors() {
        // Test case: kernel file does not exist.
        let mut vmm_config = default_vmm_config();
        vmm_config.kernel_config.path = PathBuf::from(TempFile::new().unwrap().as_path());
        let mut vmm = mock_vmm(vmm_config);
        assert!(
            matches!(vmm.load_kernel().unwrap_err(), Error::IO(e) if e.kind() == ErrorKind::NotFound)
        );

        // Test case: kernel image is invalid.
        let mut vmm_config = default_vmm_config();
        let temp_file = TempFile::new().unwrap();
        vmm_config.kernel_config.path = PathBuf::from(temp_file.as_path());
        let mut vmm = mock_vmm(vmm_config);

        let err = vmm.load_kernel().unwrap_err();
        #[cfg(target_arch = "x86_64")]
        assert!(matches!(
            err,
            Error::KernelLoad(loader::Error::Bzimage(
                loader::bzimage::Error::InvalidBzImage
            ))
        ));
        #[cfg(target_arch = "aarch64")]
        assert!(matches!(
            err,
            Error::KernelLoad(loader::Error::Pe(
                loader::pe::Error::InvalidImageMagicNumber
            ))
        ));

        // Test case: kernel path doesn't point to a file.
        let mut vmm_config = default_vmm_config();
        let temp_dir = TempDir::new().unwrap();
        vmm_config.kernel_config.path = PathBuf::from(temp_dir.as_path());
        let mut vmm = mock_vmm(vmm_config);
        let err = vmm.load_kernel().unwrap_err();

        #[cfg(target_arch = "x86_64")]
        assert!(matches!(
            err,
            Error::KernelLoad(loader::Error::Elf(loader::elf::Error::ReadElfHeader))
        ));
        #[cfg(target_arch = "aarch64")]
        assert!(matches!(
            err,
            Error::KernelLoad(loader::Error::Pe(loader::pe::Error::ReadImageHeader))
        ));
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_load_kernel() {
        // Test case: Loading the default & valid image is ok.
        let vmm_config = default_vmm_config();
        let mut vmm = mock_vmm(vmm_config);
        assert!(vmm.load_kernel().is_ok());
    }

    #[test]
    fn test_cmdline_updates() {
        let mut vmm_config = default_vmm_config();
        vmm_config.kernel_config.path = default_elf_path();
        let mut vmm = mock_vmm(vmm_config);
        assert_eq!(vmm.kernel_cfg.cmdline.as_str(), DEFAULT_KERNEL_CMDLINE);
        vmm.add_serial_console().unwrap();
        #[cfg(target_arch = "x86_64")]
        assert!(vmm.kernel_cfg.cmdline.as_str().contains("console=ttyS0"));
        #[cfg(target_arch = "aarch64")]
        assert!(vmm
            .kernel_cfg
            .cmdline
            .as_str()
            .contains("earlycon=uart,mmio"));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_create_guest_memory() {
        // Guest memory ends exactly at the MMIO gap: should succeed (last addressable value is
        // MMIO_GAP_START - 1). There should be 1 memory region.
        let mut mem_cfg = MemoryConfig {
            size_mib: (MMIO_GAP_START >> 20) as u32,
        };
        let guest_mem = Vmm::create_guest_memory(&mem_cfg).unwrap();
        assert_eq!(guest_mem.num_regions(), 1);
        assert_eq!(guest_mem.last_addr(), GuestAddress(MMIO_GAP_START - 1));

        // Guest memory ends exactly past the MMIO gap: not possible because it's specified in MiB.
        // But it can end 1 MiB within the MMIO gap. Should succeed.
        // There will be 2 regions, the 2nd ending at `size_mib << 20 + MMIO_GAP_SIZE`.
        mem_cfg.size_mib = (MMIO_GAP_START >> 20) as u32 + 1;
        let guest_mem = Vmm::create_guest_memory(&mem_cfg).unwrap();
        assert_eq!(guest_mem.num_regions(), 2);
        assert_eq!(
            guest_mem.last_addr(),
            GuestAddress(MMIO_GAP_START + MMIO_GAP_SIZE + (1 << 20) - 1)
        );

        // Guest memory ends exactly at the MMIO gap end: should succeed. There will be 2 regions,
        // the 2nd ending at `size_mib << 20 + MMIO_GAP_SIZE`.
        mem_cfg.size_mib = ((MMIO_GAP_START + MMIO_GAP_SIZE) >> 20) as u32;
        let guest_mem = Vmm::create_guest_memory(&mem_cfg).unwrap();
        assert_eq!(guest_mem.num_regions(), 2);
        assert_eq!(
            guest_mem.last_addr(),
            GuestAddress(MMIO_GAP_START + 2 * MMIO_GAP_SIZE - 1)
        );

        // Guest memory ends 1 MiB past the MMIO gap end: should succeed. There will be 2 regions,
        // the 2nd ending at `size_mib << 20 + MMIO_GAP_SIZE`.
        mem_cfg.size_mib = ((MMIO_GAP_START + MMIO_GAP_SIZE) >> 20) as u32 + 1;
        let guest_mem = Vmm::create_guest_memory(&mem_cfg).unwrap();
        assert_eq!(guest_mem.num_regions(), 2);
        assert_eq!(
            guest_mem.last_addr(),
            GuestAddress(MMIO_GAP_START + 2 * MMIO_GAP_SIZE + (1 << 20) - 1)
        );

        // Guest memory size is 0: should fail, rejected by vm-memory with EINVAL.
        mem_cfg.size_mib = 0u32;
        assert!(matches!(
            Vmm::create_guest_memory(&mem_cfg),
            Err(Error::Memory(MemoryError::VmMemory(vm_memory::Error::MmapRegion(vm_memory::mmap::MmapRegionError::Mmap(e)))))
            if e.kind() == ErrorKind::InvalidInput
        ));
    }
    #[test]
    fn test_create_vcpus() {
        // The scopes force the created vCPUs to unmap their kernel memory at the end.
        let mut vmm_config = default_vmm_config();
        vmm_config.vcpu_config = VcpuConfig { num: 0 };

        // Creating 0 vCPUs throws an error.
        {
            assert!(matches!(
                Vmm::try_from(vmm_config.clone()),
                Err(Error::Vm(vm::Error::CreateVmConfig(
                    vm_vcpu::vcpu::Error::VcpuNumber(0)
                )))
            ));
        }

        // Creating one works.
        vmm_config.vcpu_config = VcpuConfig { num: 1 };
        {
            assert!(Vmm::try_from(vmm_config.clone()).is_ok());
        }

        // Creating 254 also works (that's the maximum number on x86 when using MP Table).
        vmm_config.vcpu_config = VcpuConfig { num: 254 };
        Vmm::try_from(vmm_config).unwrap();
    }

    #[test]
    // FIXME: We cannot run this on aarch64 because we do not have an image that just runs and
    // FIXME-continued: halts afterwards. Once we have this, we need to update `default_vmm_config`
    // FIXME-continued: and have a default PE image on aarch64.
    fn test_add_block() {
        let vmm_config = default_vmm_config();
        let mut vmm = mock_vmm(vmm_config);

        let tempfile = TempFile::new().unwrap();
        let block_config = BlockConfig {
            path: tempfile.as_path().to_path_buf(),
        };

        assert!(vmm.add_block_device(&block_config).is_ok());
        assert_eq!(vmm.block_devices.len(), 1);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(vmm.fdt_builder.virtio_device_len(), 1);
        assert!(vmm.kernel_cfg.cmdline.as_str().contains("virtio"));

        let invalid_block_config = BlockConfig {
            // Let's create the tempfile directly here so that it gets out of scope immediately
            // and delete the underlying file.
            path: TempFile::new().unwrap().as_path().to_path_buf(),
        };

        let err = vmm.add_block_device(&invalid_block_config).unwrap_err();
        assert!(
            matches!(err, Error::Block(block::Error::OpenFile(io_err)) if io_err.kind() == ErrorKind::NotFound)
        );
    }

    #[test]
    // FIXME: We cannot run this on aarch64 because we do not have an image that just runs and
    // FIXME-continued: halts afterwards. Once we have this, we need to update `default_vmm_config`
    // FIXME-continued: and have a default PE image on aarch64.
    fn test_add_net() {
        let vmm_config = default_vmm_config();
        let mut vmm = mock_vmm(vmm_config);

        // The device only attempts to open the tap interface during activation, so we can
        // specify any name here for now.
        let cfg = NetConfig {
            tap_name: "imaginary_tap".to_owned(),
        };

        {
            assert!(vmm.add_net_device(&cfg).is_ok());
            assert_eq!(vmm.net_devices.len(), 1);
            #[cfg(target_arch = "aarch64")]
            assert_eq!(vmm.fdt_builder.virtio_device_len(), 1);
            assert!(vmm.kernel_cfg.cmdline.as_str().contains("virtio"));
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_setup_fdt() {
        let vmm_config = default_vmm_config();
        let mut vmm = mock_vmm(vmm_config);

        {
            let result = vmm.setup_fdt();
            assert!(result.is_ok());
        }

        {
            let mem_size: u64 = vmm.guest_memory.iter().map(|region| region.len()).sum();
            let fdt_offset = mem_size + AARCH64_FDT_MAX_SIZE;
            let cmdline = &vmm.kernel_cfg.cmdline;
            let fdt = vmm
                .fdt_builder
                .with_cmdline(cmdline.as_str().to_string())
                .with_num_vcpus(vmm.num_vcpus.try_into().unwrap())
                .with_mem_size(mem_size)
                .with_serial_console(0x40000000, 0x1000)
                .with_rtc(0x40001000, 0x1000)
                .create_fdt()
                .unwrap();
            assert!(fdt.write_to_mem(&vmm.guest_memory, fdt_offset).is_err());
        }
    }
    #[test]
    fn test_address_alloc() {
        let memory_config = MemoryConfig {
            size_mib: MEM_SIZE_MIB,
        };
        #[cfg(target_arch = "x86_64")]
        let start_addr = MMIO_GAP_START;
        #[cfg(target_arch = "aarch64")]
        let start_addr = AARCH64_MMIO_BASE;
        let mut address_alloc = Vmm::create_address_allocator(&memory_config).unwrap();

        // Trying to allocate at an address before the base address.
        let alloc_err = address_alloc
            .allocate(100, DEFAULT_ADDRESSS_ALIGNEMNT, AllocPolicy::ExactMatch(0))
            .unwrap_err();
        assert_eq!(alloc_err, vm_allocator::Error::ResourceNotAvailable);
        let alloc_err = address_alloc
            .allocate(
                100,
                DEFAULT_ADDRESSS_ALIGNEMNT,
                AllocPolicy::ExactMatch(start_addr - DEFAULT_ADDRESSS_ALIGNEMNT),
            )
            .unwrap_err();
        assert_eq!(alloc_err, vm_allocator::Error::ResourceNotAvailable);

        // Testing it outside the available range
        let outside_avail_range =
            start_addr + ((MEM_SIZE_MIB as u64) << 20) + DEFAULT_ADDRESSS_ALIGNEMNT;
        let alloc_err = address_alloc
            .allocate(
                100,
                DEFAULT_ADDRESSS_ALIGNEMNT,
                AllocPolicy::ExactMatch(outside_avail_range),
            )
            .unwrap_err();
        assert_eq!(alloc_err, vm_allocator::Error::ResourceNotAvailable);

        // Trying to add more than available range.
        let alloc_err = address_alloc
            .allocate(
                ((MEM_SIZE_MIB as u64) << 20) + 2,
                DEFAULT_ADDRESSS_ALIGNEMNT,
                DEFAULT_ALLOC_POLICY,
            )
            .unwrap_err();
        assert_eq!(alloc_err, vm_allocator::Error::ResourceNotAvailable);
    }
}
