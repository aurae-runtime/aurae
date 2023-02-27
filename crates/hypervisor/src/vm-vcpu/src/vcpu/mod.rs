// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// Copyright 2017 The Chromium OS Authors. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
use libc::siginfo_t;
use std::cell::RefCell;
use std::ffi::c_void;
use std::io::{self, stdin};
use std::os::raw::c_int;
use std::result;
use std::sync::{Arc, Barrier, Condvar, Mutex};

#[cfg(target_arch = "x86_64")]
use kvm_bindings::{
    kvm_debugregs, kvm_fpu, kvm_lapic_state, kvm_mp_state, kvm_regs, kvm_sregs, kvm_vcpu_events,
    kvm_xcrs, kvm_xsave, CpuId, Msrs,
};
#[cfg(target_arch = "aarch64")]
use kvm_bindings::{
    kvm_mp_state, kvm_one_reg, kvm_vcpu_init, KVM_REG_ARM64, KVM_REG_ARM_CORE, KVM_REG_SIZE_U64,
    KVM_SYSTEM_EVENT_CRASH, KVM_SYSTEM_EVENT_RESET, KVM_SYSTEM_EVENT_SHUTDOWN,
};
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
use vm_device::bus::{MmioAddress, PioAddress};
use vm_device::device_manager::{IoManager, MmioManager, PioManager};
#[cfg(target_arch = "aarch64")]
use vm_memory::GuestMemoryRegion;
#[cfg(target_arch = "x86_64")]
use vm_memory::{Address, Bytes};
use vm_memory::{GuestAddress, GuestMemory, GuestMemoryError};
#[cfg(target_arch = "x86_64")]
use vm_vcpu_ref::x86_64::{
    gdt::{self, write_idt_value, Gdt, BOOT_GDT_OFFSET, BOOT_IDT_OFFSET},
    interrupts::{
        set_klapic_delivery_mode, DeliveryMode, APIC_LVT0_REG_OFFSET, APIC_LVT1_REG_OFFSET,
    },
    mptable, msr_index, msrs,
};
use vmm_sys_util::errno::Error as Errno;
use vmm_sys_util::signal::{register_signal_handler, SIGRTMIN};
use vmm_sys_util::terminal::Terminal;

use utils::debug;

#[cfg(target_arch = "aarch64")]
#[macro_use]
mod regs;

#[cfg(target_arch = "aarch64")]
use regs::*;

use crate::vm::VmRunState;
#[cfg(target_arch = "aarch64")]
use arch::{AARCH64_FDT_MAX_SIZE, AARCH64_PHYS_MEM_START};

/// Initial stack for the boot CPU.
#[cfg(target_arch = "x86_64")]
const BOOT_STACK_POINTER: u64 = 0x8ff0;
/// Address of the zeropage, where Linux kernel boot parameters are written.
#[cfg(target_arch = "x86_64")]
const ZEROPG_START: u64 = 0x7000;

// Initial pagetables.
#[cfg(target_arch = "x86_64")]
mod pagetable {
    pub const PML4_START: u64 = 0x9000;
    pub const PDPTE_START: u64 = 0xa000;
    pub const PDE_START: u64 = 0xb000;
}
#[cfg(target_arch = "x86_64")]
use pagetable::*;

#[cfg(target_arch = "x86_64")]
mod regs {
    pub const X86_CR0_PE: u64 = 0x1;
    pub const X86_CR0_PG: u64 = 0x8000_0000;
    pub const X86_CR4_PAE: u64 = 0x20;
}
#[cfg(target_arch = "x86_64")]
use regs::*;

#[cfg(target_arch = "aarch64")]
use kvm_bindings::{PSR_MODE_EL1h, PSR_A_BIT, PSR_D_BIT, PSR_F_BIT, PSR_I_BIT};

/// Errors encountered during vCPU operation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid number of vcpus specified in configuration.
    #[error("Invalid number of vcpus specified in configuration: {0}")]
    VcpuNumber(u8),
    /// Cannot get the supported MSRs.
    #[error("Cannot get the supported MSRs.")]
    #[cfg(target_arch = "x86_64")]
    GetSupportedMsrs(msrs::Error),
    /// Failed to operate on guest memory.
    #[error("Failed to operate on guest memory: {0}")]
    GuestMemory(GuestMemoryError),
    /// I/O Error.
    #[error("I/O Error: {0}")]
    IO(io::Error),
    /// Error issuing an ioctl to KVM.
    #[error("Error issuing an ioctl to KVM: {0}")]
    KvmIoctl(kvm_ioctls::Error),
    /// Failed to configure mptables.
    #[error("Failed to configure mptables.")]
    #[cfg(target_arch = "x86_64")]
    Mptable(mptable::Error),
    /// Failed to setup the GDT.
    #[error("Failed to setup the GDT.")]
    #[cfg(target_arch = "x86_64")]
    Gdt(gdt::Error),
    /// Failed to initialize MSRS.
    #[error("Failed to initialize MSRS.")]
    #[cfg(target_arch = "x86_64")]
    CreateMsrs(msrs::Error),
    /// Failed to configure MSRs.
    #[error("Failed to configure MSRs.")]
    #[cfg(target_arch = "x86_64")]
    SetModelSpecificRegistersCount,
    /// TLS already initialized.
    #[error("TLS already initialized.")]
    TlsInitialized,
    /// Unable to register signal handler.
    #[error("Unable to register signal handler: {0}")]
    RegisterSignalHandler(Errno),

    // These are all Save/Restore errors. Maybe it makes sense to move them
    // to a separate enum.
    #[error("FamError")]
    FamError(vmm_sys_util::fam::Error),
    /// Failed to get KVM vcpu debug regs.
    #[error("Failed to get KVM vcpu debug regs: {0}")]
    VcpuGetDebugRegs(kvm_ioctls::Error),
    /// Failed to get KVM vcpu lapic.
    #[error("Failed to get KVM vcpu lapic: {0}")]
    VcpuGetLapic(kvm_ioctls::Error),
    /// Failed to get KVM vcpu mp state.
    #[error("Failed to get KVM vcpu mp state: {0}")]
    VcpuGetMpState(kvm_ioctls::Error),
    /// The number of MSRS returned by the kernel is unexpected.
    #[error("The number of MSRS returned by the kernel is unexpected.")]
    VcpuGetMSRSIncomplete,
    /// Failed to get KVM vcpu msrs.
    #[error("Failed to get KVM vcpu msrs: {0}")]
    VcpuGetMsrs(kvm_ioctls::Error),
    /// Failed to get KVM vcpu regs.
    #[error("Failed to get KVM vcpu regs: {0}")]
    VcpuGetRegs(kvm_ioctls::Error),
    /// Failed to get KVM vcpu sregs.
    #[error("Failed to get KVM vcpu sregs: {0}")]
    VcpuGetSregs(kvm_ioctls::Error),
    /// Failed to get KVM vcpu event.
    #[error("Failed to get KVM vcpu event: {0}")]
    VcpuGetVcpuEvents(kvm_ioctls::Error),
    /// Failed to get KVM vcpu xcrs.
    #[error("Failed to get KVM vcpu xcrs: {0}")]
    VcpuGetXcrs(kvm_ioctls::Error),
    /// Failed to get KVM vcpu xsave.
    #[error("Failed to get KVM vcpu xsave: {0}")]
    VcpuGetXsave(kvm_ioctls::Error),
    /// Failed to get KVM vcpu cpuid.
    #[error("Failed to get KVM vcpu cpuid: {0}")]
    VcpuGetCpuid(kvm_ioctls::Error),
    /// Failed to get KVM TSC freq.
    #[error("Failed to get KVM TSC freq: {0}")]
    VcpuGetTSC(kvm_ioctls::Error),
    /// Failed to get KVM vcpu reglist.
    #[error("Failed to get KVM vcpu reglist: {0}")]
    VcpuGetRegList(kvm_ioctls::Error),
    /// Failed to get KVM vcpu reg.
    #[error("Failed to get KVM vcpu reg: {0}")]
    VcpuGetReg(kvm_ioctls::Error),
    /// Failed to get KVM vcpu MPIDR reg.
    #[error("Failed to get KVM vcpu MPIDR reg")]
    VcpuGetMpidrReg,
    /// Failed to set KVM vcpu cpuid.
    #[error("Failed to set KVM vcpu cpuid: {0}")]
    VcpuSetCpuid(kvm_ioctls::Error),
    /// Failed to set KVM vcpu debug regs.
    #[error("Failed to set KVM vcpu debug regs: {0}")]
    VcpuSetDebugRegs(kvm_ioctls::Error),
    /// Failed to set KVM vcpu lapic.
    #[error("Failed to set KVM vcpu lapic: {0}")]
    VcpuSetLapic(kvm_ioctls::Error),
    /// Failed to set KVM vcpu mp state.
    #[error("Failed to set KVM vcpu mp state: {0}")]
    VcpuSetMpState(kvm_ioctls::Error),
    /// Failed to set KVM vcpu msrs.
    #[error("Failed to set KVM vcpu msrs: {0}")]
    VcpuSetMsrs(kvm_ioctls::Error),
    /// Failed to set KVM vcpu regs.
    #[error("Failed to set KVM vcpu regs: {0}")]
    VcpuSetRegs(kvm_ioctls::Error),
    /// Failed to set KVM vcpu sregs.
    #[error("Failed to set KVM vcpu sregs: {0}")]
    VcpuSetSregs(kvm_ioctls::Error),
    /// Failed to set KVM vcpu event.
    #[error("Failed to set KVM vcpu event: {0}")]
    VcpuSetVcpuEvents(kvm_ioctls::Error),
    /// Failed to set KVM vcpu xcrs.
    #[error("Failed to set KVM vcpu xcrs: {0}")]
    VcpuSetXcrs(kvm_ioctls::Error),
    /// Failed to set KVM vcpu xsave.
    #[error("Failed to set KVM vcpu xsave: {0}")]
    VcpuSetXsave(kvm_ioctls::Error),
    /// Failed to set KVM vcpu reg.
    #[error("Failed to set KVM vcpu reg: {0}")]
    VcpuSetReg(kvm_ioctls::Error),
}

/// Dedicated Result type.
pub type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub struct VcpuConfig {
    pub id: u8,
    #[cfg(target_arch = "x86_64")]
    pub cpuid: CpuId,
    #[cfg(target_arch = "x86_64")]
    // This is just a workaround so that we can get a list of MSRS.
    // Just getting all the MSRS on a vcpu is not possible with KVM.
    pub msrs: Msrs,
}

#[derive(Clone)]
pub struct VcpuConfigList {
    pub configs: Vec<VcpuConfig>,
}

impl VcpuConfigList {
    /// Creates a default configuration list for vCPUs.
    pub fn new(_kvm: &Kvm, num_vcpus: u8) -> Result<Self> {
        if num_vcpus == 0 {
            return Err(Error::VcpuNumber(num_vcpus));
        }

        #[cfg(target_arch = "x86_64")]
        let base_cpuid = _kvm
            .get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)
            .map_err(Error::KvmIoctl)?;

        #[cfg(target_arch = "x86_64")]
        let supported_msrs = msrs::supported_guest_msrs(_kvm).map_err(Error::GetSupportedMsrs)?;

        let mut configs = Vec::new();
        for index in 0..num_vcpus {
            // Set CPUID.
            #[cfg(target_arch = "x86_64")]
            let mut cpuid = base_cpuid.clone();
            #[cfg(target_arch = "x86_64")]
            vm_vcpu_ref::x86_64::cpuid::filter_cpuid(_kvm, index, num_vcpus, &mut cpuid);

            #[cfg(target_arch = "x86_64")]
            let vcpu_config = VcpuConfig {
                cpuid,
                id: index,
                msrs: supported_msrs.clone(),
            };
            #[cfg(target_arch = "aarch64")]
            let vcpu_config = VcpuConfig { id: index };

            configs.push(vcpu_config);
        }

        Ok(VcpuConfigList { configs })
    }
}

/// Structure holding the kvm state for an x86_64 VCPU.
#[cfg(target_arch = "x86_64")]
#[derive(Clone)]
pub struct VcpuState {
    pub cpuid: CpuId,
    pub msrs: Msrs,
    pub debug_regs: kvm_debugregs,
    pub lapic: kvm_lapic_state,
    pub mp_state: kvm_mp_state,
    pub regs: kvm_regs,
    pub sregs: kvm_sregs,
    pub vcpu_events: kvm_vcpu_events,
    pub xcrs: kvm_xcrs,
    pub xsave: kvm_xsave,
    pub config: VcpuConfig,
}

#[cfg(target_arch = "aarch64")]
#[derive(Clone)]
pub struct VcpuState {
    pub mp_state: kvm_mp_state,
    pub regs: Vec<kvm_one_reg>,
    /// Cached value of MPIDR register. Even though it's stored
    /// in `regs`, searching for it is an expensive linear scan.
    pub mpidr: u64,
    pub config: VcpuConfig,
}

/// Represents the current run state of the VCPUs.
#[derive(Default)]
pub struct VcpuRunState {
    pub(crate) vm_state: Mutex<VmRunState>,
    condvar: Condvar,
}

impl VcpuRunState {
    pub fn set_and_notify(&self, state: VmRunState) {
        *self.vm_state.lock().unwrap() = state;
        self.condvar.notify_all();
    }
}

/// Struct for interacting with vCPUs.
///
/// This struct is a temporary (and quite terrible) placeholder until the
/// [`vmm-vcpu`](https://github.com/rust-vmm/vmm-vcpu) crate is stabilized.
pub struct KvmVcpu {
    /// KVM file descriptor for a vCPU.
    pub(crate) vcpu_fd: VcpuFd,
    /// Device manager for bus accesses.
    device_mgr: Arc<Mutex<IoManager>>,
    config: VcpuConfig,
    run_barrier: Arc<Barrier>,
    pub(crate) run_state: Arc<VcpuRunState>,
}

impl KvmVcpu {
    thread_local!(static TLS_VCPU_PTR: RefCell<Option<*const KvmVcpu>> = RefCell::new(None));

    /// Create a new vCPU.
    // This is needed so we can initialize the vcpu the same way on x86_64 and aarch64, but
    // have it as mutable on aarch64.
    #[allow(clippy::needless_late_init)]
    pub fn new<M: GuestMemory>(
        vm_fd: &VmFd,
        device_mgr: Arc<Mutex<IoManager>>,
        config: VcpuConfig,
        run_barrier: Arc<Barrier>,
        run_state: Arc<VcpuRunState>,
        memory: &M,
    ) -> Result<Self> {
        #[cfg(target_arch = "x86_64")]
        let vcpu;
        #[cfg(target_arch = "aarch64")]
        let mut vcpu;

        vcpu = KvmVcpu {
            vcpu_fd: vm_fd
                .create_vcpu(config.id.into())
                .map_err(Error::KvmIoctl)?,
            device_mgr,
            config,
            run_barrier,
            run_state,
        };

        #[cfg(target_arch = "x86_64")]
        {
            vcpu.configure_cpuid(&vcpu.config.cpuid)?;
            vcpu.configure_msrs()?;
            vcpu.configure_sregs(memory)?;
            vcpu.configure_lapic()?;
            vcpu.configure_fpu()?;
        }

        #[cfg(target_arch = "aarch64")]
        {
            vcpu.init(vm_fd)?;
            vcpu.configure_regs(memory)?;
        }

        Ok(vcpu)
    }

    #[cfg(target_arch = "x86_64")]
    // Set the state of this `KvmVcpu`. Errors returned from this function
    // MUST not be ignored because they can lead to undefined behavior when
    // the state of the vCPU is only partially set.
    fn set_state(&mut self, state: VcpuState) -> Result<()> {
        self.vcpu_fd
            .set_cpuid2(&state.cpuid)
            .map_err(Error::VcpuSetCpuid)?;
        self.vcpu_fd
            .set_mp_state(state.mp_state)
            .map_err(Error::VcpuSetMpState)?;
        self.vcpu_fd
            .set_regs(&state.regs)
            .map_err(Error::VcpuSetRegs)?;
        self.vcpu_fd
            .set_sregs(&state.sregs)
            .map_err(Error::VcpuSetSregs)?;
        self.vcpu_fd
            .set_xsave(&state.xsave)
            .map_err(Error::VcpuSetXsave)?;
        self.vcpu_fd
            .set_xcrs(&state.xcrs)
            .map_err(Error::VcpuSetXcrs)?;
        self.vcpu_fd
            .set_debug_regs(&state.debug_regs)
            .map_err(Error::VcpuSetDebugRegs)?;
        self.vcpu_fd
            .set_lapic(&state.lapic)
            .map_err(Error::VcpuSetLapic)?;
        self.vcpu_fd
            .set_msrs(&state.msrs)
            .map_err(Error::VcpuSetMsrs)?;
        self.vcpu_fd
            .set_vcpu_events(&state.vcpu_events)
            .map_err(Error::VcpuSetVcpuEvents)?;
        Ok(())
    }

    #[cfg(target_arch = "aarch64")]
    fn set_state(&mut self, state: VcpuState) -> Result<()> {
        for reg in state.regs {
            self.vcpu_fd
                .set_one_reg(reg.id, reg.addr)
                .map_err(Error::VcpuSetReg)?;
        }

        self.vcpu_fd
            .set_mp_state(state.mp_state)
            .map_err(Error::VcpuSetMpState)?;

        Ok(())
    }

    /// Create a vCPU from a previously saved state.
    pub fn from_state<M: GuestMemory>(
        vm_fd: &VmFd,
        device_mgr: Arc<Mutex<IoManager>>,
        state: VcpuState,
        run_barrier: Arc<Barrier>,
        run_state: Arc<VcpuRunState>,
    ) -> Result<Self> {
        let mut vcpu = KvmVcpu {
            vcpu_fd: vm_fd
                .create_vcpu(state.config.id.into())
                .map_err(Error::KvmIoctl)?,
            device_mgr,
            config: state.config.clone(),
            run_barrier,
            run_state,
        };

        #[cfg(target_arch = "aarch64")]
        vcpu.init(vm_fd)?;

        vcpu.set_state(state)?;
        Ok(vcpu)
    }

    #[cfg(target_arch = "aarch64")]
    fn configure_regs<M: GuestMemory>(&mut self, guest_mem: &M) -> Result<()> {
        // set up registers
        let mut data: u64;
        let mut reg_id: u64;

        // All interrupts masked
        data = (PSR_D_BIT | PSR_A_BIT | PSR_I_BIT | PSR_F_BIT | PSR_MODE_EL1h).into();
        reg_id = arm64_core_reg!(pstate);
        self.vcpu_fd
            .set_one_reg(reg_id, data)
            .map_err(Error::VcpuSetReg)?;

        // Other cpus are powered off initially
        if self.config.id == 0 {
            /* X0 -- fdt address */
            let mut fdt_offset: u64 = guest_mem.iter().map(|region| region.len()).sum();
            fdt_offset = fdt_offset - AARCH64_FDT_MAX_SIZE - 0x10000;
            data = (AARCH64_PHYS_MEM_START + fdt_offset) as u64;
            // hack -- can't get this to do offsetof(regs[0]) but luckily it's at offset 0
            reg_id = arm64_core_reg!(regs);
            self.vcpu_fd
                .set_one_reg(reg_id, data)
                .map_err(Error::VcpuSetReg)?;
        }

        Ok(())
    }

    #[cfg(target_arch = "aarch64")]
    fn init(&mut self, vm_fd: &VmFd) -> Result<()> {
        let mut kvi: kvm_vcpu_init = kvm_vcpu_init::default();
        vm_fd
            .get_preferred_target(&mut kvi)
            .map_err(Error::KvmIoctl)?;

        kvi.features[0] |= 1 << kvm_bindings::KVM_ARM_VCPU_PSCI_0_2;
        // Non-boot cpus are powered off initially.
        if self.config.id > 0 {
            kvi.features[0] |= 1 << kvm_bindings::KVM_ARM_VCPU_POWER_OFF;
        }

        self.vcpu_fd.vcpu_init(&kvi).map_err(Error::KvmIoctl)?;

        Ok(())
    }

    /// Set CPUID.
    #[cfg(target_arch = "x86_64")]
    fn configure_cpuid(&self, cpuid: &CpuId) -> Result<()> {
        self.vcpu_fd.set_cpuid2(cpuid).map_err(Error::KvmIoctl)
    }

    /// Configure MSRs.
    #[cfg(target_arch = "x86_64")]
    fn configure_msrs(&self) -> Result<()> {
        let msrs = msrs::create_boot_msr_entries().map_err(Error::CreateMsrs)?;
        self.vcpu_fd
            .set_msrs(&msrs)
            .map_err(Error::KvmIoctl)
            .and_then(|msrs_written| {
                if msrs_written as u32 != msrs.as_fam_struct_ref().nmsrs {
                    Err(Error::SetModelSpecificRegistersCount)
                } else {
                    Ok(())
                }
            })
    }

    /// Configure regs.
    #[cfg(target_arch = "x86_64")]
    fn configure_regs(&self, instruction_pointer: GuestAddress) -> Result<()> {
        let regs = kvm_regs {
            // EFLAGS (RFLAGS in 64-bit mode) always has bit 1 set.
            // See https://software.intel.com/sites/default/files/managed/39/c5/325462-sdm-vol-1-2abcd-3abcd.pdf#page=79
            // Section "EFLAGS Register"
            rflags: 0x0000_0000_0000_0002u64,
            rip: instruction_pointer.raw_value(),
            // Starting stack pointer.
            rsp: BOOT_STACK_POINTER,
            // Frame pointer. It gets a snapshot of the stack pointer (rsp) so that when adjustments are
            // made to rsp (i.e. reserving space for local variables or pushing values on to the stack),
            // local variables and function parameters are still accessible from a constant offset from rbp.
            rbp: BOOT_STACK_POINTER,
            // Must point to zero page address per Linux ABI. This is x86_64 specific.
            rsi: ZEROPG_START,
            ..Default::default()
        };
        self.vcpu_fd.set_regs(&regs).map_err(Error::KvmIoctl)
    }

    /// Configure sregs.
    #[cfg(target_arch = "x86_64")]
    fn configure_sregs<M: GuestMemory>(&self, guest_memory: &M) -> Result<()> {
        let mut sregs = self.vcpu_fd.get_sregs().map_err(Error::KvmIoctl)?;

        // Global descriptor tables.
        let gdt_table = Gdt::default();

        // The following unwraps are safe because we know that the default GDT has 4 segments.
        let code_seg = gdt_table.create_kvm_segment_for(1).unwrap();
        let data_seg = gdt_table.create_kvm_segment_for(2).unwrap();
        let tss_seg = gdt_table.create_kvm_segment_for(3).unwrap();

        // Write segments to guest memory.
        gdt_table.write_to_mem(guest_memory).map_err(Error::Gdt)?;
        sregs.gdt.base = BOOT_GDT_OFFSET as u64;
        sregs.gdt.limit = std::mem::size_of_val(&gdt_table) as u16 - 1;

        write_idt_value(0, guest_memory).map_err(Error::Gdt)?;
        sregs.idt.base = BOOT_IDT_OFFSET as u64;
        sregs.idt.limit = std::mem::size_of::<u64>() as u16 - 1;

        sregs.cs = code_seg;
        sregs.ds = data_seg;
        sregs.es = data_seg;
        sregs.fs = data_seg;
        sregs.gs = data_seg;
        sregs.ss = data_seg;
        sregs.tr = tss_seg;

        // 64-bit protected mode.
        sregs.cr0 |= X86_CR0_PE;
        sregs.efer |= (msr_index::EFER_LME | msr_index::EFER_LMA) as u64;

        // Start page table configuration.
        // Puts PML4 right after zero page but aligned to 4k.
        let boot_pml4_addr = GuestAddress(PML4_START);
        let boot_pdpte_addr = GuestAddress(PDPTE_START);
        let boot_pde_addr = GuestAddress(PDE_START);

        // Entry covering VA [0..512GB).
        guest_memory
            .write_obj(boot_pdpte_addr.raw_value() | 0x03, boot_pml4_addr)
            .map_err(Error::GuestMemory)?;

        // Entry covering VA [0..1GB).
        guest_memory
            .write_obj(boot_pde_addr.raw_value() | 0x03, boot_pdpte_addr)
            .map_err(Error::GuestMemory)?;

        // 512 2MB entries together covering VA [0..1GB).
        // This assumes that the CPU supports 2MB pages (/proc/cpuinfo has 'pse').
        for i in 0..512 {
            guest_memory
                .write_obj((i << 21) + 0x83u64, boot_pde_addr.unchecked_add(i * 8))
                .map_err(Error::GuestMemory)?;
        }

        sregs.cr3 = boot_pml4_addr.raw_value();
        sregs.cr4 |= X86_CR4_PAE;
        sregs.cr0 |= X86_CR0_PG;

        self.vcpu_fd.set_sregs(&sregs).map_err(Error::KvmIoctl)
    }

    /// Configure FPU.
    #[cfg(target_arch = "x86_64")]
    fn configure_fpu(&self) -> Result<()> {
        let fpu = kvm_fpu {
            fcw: 0x37f,
            mxcsr: 0x1f80,
            ..Default::default()
        };
        self.vcpu_fd.set_fpu(&fpu).map_err(Error::KvmIoctl)
    }

    /// Configures LAPICs. LAPIC0 is set for external interrupts, LAPIC1 is set for NMI.
    #[cfg(target_arch = "x86_64")]
    fn configure_lapic(&self) -> Result<()> {
        let mut klapic = self.vcpu_fd.get_lapic().map_err(Error::KvmIoctl)?;

        // The following unwraps are safe because we are using valid values for all parameters
        // (using defines from the crate for APIC_LV*). If these end up being wrong,
        // it is a programming error in which case we want to fail fast.
        set_klapic_delivery_mode(&mut klapic, APIC_LVT0_REG_OFFSET, DeliveryMode::ExtINT).unwrap();
        set_klapic_delivery_mode(&mut klapic, APIC_LVT1_REG_OFFSET, DeliveryMode::NMI).unwrap();

        self.vcpu_fd.set_lapic(&klapic).map_err(Error::KvmIoctl)
    }

    pub(crate) fn setup_signal_handler() -> Result<()> {
        extern "C" fn handle_signal(_: c_int, _: *mut siginfo_t, _: *mut c_void) {
            KvmVcpu::set_local_immediate_exit(1);
        }
        #[allow(clippy::identity_op)]
        register_signal_handler(SIGRTMIN() + 0, handle_signal)
            .map_err(Error::RegisterSignalHandler)?;
        Ok(())
    }

    fn init_tls(&mut self) -> Result<()> {
        Self::TLS_VCPU_PTR.with(|vcpu| {
            if vcpu.borrow().is_none() {
                *vcpu.borrow_mut() = Some(self as *const KvmVcpu);
                Ok(())
            } else {
                Err(Error::TlsInitialized)
            }
        })?;
        Ok(())
    }

    fn set_local_immediate_exit(value: u8) {
        Self::TLS_VCPU_PTR.with(|v| {
            if let Some(vcpu) = *v.borrow() {
                // The block below modifies a mmaped memory region (`kvm_run` struct) which is valid
                // as long as the `VMM` is still in scope. This function is called in response to
                // SIGRTMIN(), while the vCPU threads are still active. Their termination are
                // strictly bound to the lifespan of the `VMM` and it precedes the `VMM` dropping.
                unsafe {
                    let vcpu_ref = &*vcpu;
                    vcpu_ref.vcpu_fd.set_kvm_immediate_exit(value);
                };
            }
        });
    }

    /// vCPU emulation loop.
    ///
    /// # Arguments
    ///
    /// * `instruction_pointer`: Represents the start address of the vcpu. This can be None
    /// when the IP is specified using the platform dependent registers.
    #[allow(clippy::if_same_then_else)]
    pub fn run(&mut self, instruction_pointer: Option<GuestAddress>) -> Result<()> {
        if let Some(ip) = instruction_pointer {
            #[cfg(target_arch = "x86_64")]
            self.configure_regs(ip)?;
            #[cfg(target_arch = "aarch64")]
            if self.config.id == 0 {
                let data = ip.0;
                let reg_id = arm64_core_reg!(pc);
                self.vcpu_fd
                    .set_one_reg(reg_id, data)
                    .map_err(Error::VcpuSetReg)?;
            }
        }
        self.init_tls()?;

        self.run_barrier.wait();
        'vcpu_run: loop {
            let mut interrupted_by_signal = false;
            match self.vcpu_fd.run() {
                Ok(exit_reason) => {
                    // println!("{:#?}", exit_reason);
                    match exit_reason {
                        VcpuExit::Shutdown | VcpuExit::Hlt => {
                            println!("Guest shutdown: {:?}. Bye!", exit_reason);
                            if stdin().lock().set_canon_mode().is_err() {
                                eprintln!("Failed to set canon mode. Stdin will not echo.");
                            }
                            self.run_state.set_and_notify(VmRunState::Exiting);
                            break;
                        }
                        VcpuExit::IoOut(addr, data) => {
                            if (0x3f8..(0x3f8 + 8)).contains(&addr) {
                                // Write at the serial port.
                                if self
                                    .device_mgr
                                    .lock()
                                    .unwrap()
                                    .pio_write(PioAddress(addr), data)
                                    .is_err()
                                {
                                    debug!("Failed to write to serial port");
                                }
                            } else if addr == 0x060 || addr == 0x061 || addr == 0x064 {
                                // Write at the i8042 port.
                                //i8042 is registered at port 0x64.
                                // See https://wiki.osdev.org/%228042%22_PS/2_Controller#PS.2F2_Controller_IO_Ports
                                #[cfg(target_arch = "x86_64")]
                                if self
                                    .device_mgr
                                    .lock()
                                    .unwrap()
                                    .pio_write(PioAddress(addr), data)
                                    .is_err()
                                {
                                    debug!("Failed to write to i8042 port")
                                }
                            } else if (0x070..=0x07f).contains(&addr) {
                                // Write at the RTC port.
                            } else {
                                // Write at some other port.
                            }
                        }
                        VcpuExit::IoIn(addr, data) => {
                            if (0x3f8..(0x3f8 + 8)).contains(&addr) {
                                // Read from the serial port.
                                if self
                                    .device_mgr
                                    .lock()
                                    .unwrap()
                                    .pio_read(PioAddress(addr), data)
                                    .is_err()
                                {
                                    debug!("Failed to read from serial port");
                                }
                            } else {
                                // Read from some other port.
                            }
                        }
                        VcpuExit::MmioRead(addr, data) => {
                            if self
                                .device_mgr
                                .lock()
                                .unwrap()
                                .mmio_read(MmioAddress(addr), data)
                                .is_err()
                            {
                                debug!("Failed to read from mmio addr={} data={:#?}", addr, data);
                            }
                        }
                        VcpuExit::MmioWrite(addr, data) => {
                            if self
                                .device_mgr
                                .lock()
                                .unwrap()
                                .mmio_write(MmioAddress(addr), data)
                                .is_err()
                            {
                                debug!("Failed to write to mmio");
                            }
                        }
                        #[cfg(target_arch = "aarch64")]
                        VcpuExit::SystemEvent(type_, flags) => match type_ {
                            KVM_SYSTEM_EVENT_SHUTDOWN
                            | KVM_SYSTEM_EVENT_RESET
                            | KVM_SYSTEM_EVENT_CRASH => {
                                println!("Exit reason: {:#?}", VcpuExit::SystemEvent(type_, flags));
                                if stdin().lock().set_canon_mode().is_err() {
                                    eprintln!("Failed to set canon mode. Stdin will not echo.");
                                }
                                self.run_state.set_and_notify(VmRunState::Exiting);
                                break;
                            }
                            _ => {
                                // Unknown system event type
                                debug!("Unknown system event type: {:#?}", type_)
                            }
                        },
                        _other => {
                            // Unhandled KVM exit.
                            debug!("Unhandled vcpu exit: {:#?}", _other);
                        }
                    }
                }
                Err(e) => {
                    // During boot KVM can exit with `EAGAIN`. In that case, do not
                    // terminate the run loop.
                    match e.errno() {
                        libc::EAGAIN => {}
                        libc::EINTR => {
                            interrupted_by_signal = true;
                        }
                        _ => {
                            debug!("Emulation error: {}", e);
                            break;
                        }
                    }
                }
            }

            if interrupted_by_signal {
                self.vcpu_fd.set_kvm_immediate_exit(0);
                let mut run_state_lock = self.run_state.vm_state.lock().unwrap();
                loop {
                    match *run_state_lock {
                        VmRunState::Running => {
                            // The VM state is running, so we need to exit from this loop,
                            // and enter the kvm run loop.
                            break;
                        }
                        VmRunState::Suspending => {
                            // The VM is suspending. We run this loop until we get a different
                            // state.
                        }
                        VmRunState::Exiting => {
                            // The VM is exiting. We also exit from this VCPU thread.
                            break 'vcpu_run;
                        }
                    }
                    // Give ownership of our exclusive lock to the condition variable that will
                    // block. When the condition variable is notified, `wait` will unblock and
                    // return a new exclusive lock.
                    run_state_lock = self.run_state.condvar.wait(run_state_lock).unwrap();
                }
            }
        }

        Ok(())
    }

    /// Pause the vcpu. If the vcpu is already paused, this is a no-op.
    pub fn pause(&mut self) -> Result<()> {
        todo!()
    }

    #[cfg(target_arch = "x86_64")]
    pub fn save_state(&mut self) -> Result<VcpuState> {
        let mp_state = self.vcpu_fd.get_mp_state().map_err(Error::VcpuGetMpState)?;
        let regs = self.vcpu_fd.get_regs().map_err(Error::VcpuGetRegs)?;
        let sregs = self.vcpu_fd.get_sregs().map_err(Error::VcpuGetSregs)?;
        let xsave = self.vcpu_fd.get_xsave().map_err(Error::VcpuGetXsave)?;
        let xcrs = self.vcpu_fd.get_xcrs().map_err(Error::VcpuGetXcrs)?;
        let debug_regs = self
            .vcpu_fd
            .get_debug_regs()
            .map_err(Error::VcpuGetDebugRegs)?;
        let lapic = self.vcpu_fd.get_lapic().map_err(Error::VcpuGetLapic)?;

        let mut msrs = self.config.msrs.clone();
        let num_msrs = self.config.msrs.as_fam_struct_ref().nmsrs as usize;
        let nmsrs = self
            .vcpu_fd
            .get_msrs(&mut msrs)
            .map_err(Error::VcpuGetMsrs)?;
        if nmsrs != num_msrs {
            return Err(Error::VcpuGetMSRSIncomplete);
        }
        let vcpu_events = self
            .vcpu_fd
            .get_vcpu_events()
            .map_err(Error::VcpuGetVcpuEvents)?;

        let cpuid = self
            .vcpu_fd
            .get_cpuid2(kvm_bindings::KVM_MAX_CPUID_ENTRIES)
            .map_err(Error::VcpuGetCpuid)?;

        Ok(VcpuState {
            cpuid,
            msrs,
            debug_regs,
            lapic,
            mp_state,
            regs,
            sregs,
            vcpu_events,
            xcrs,
            xsave,
            config: self.config.clone(),
        })
    }

    #[cfg(target_arch = "aarch64")]
    pub fn save_state(&mut self) -> Result<VcpuState> {
        let mp_state = self.vcpu_fd.get_mp_state().map_err(Error::VcpuGetMpState)?;
        let (regs, mpidr) = get_regs_and_mpidr(&self.vcpu_fd)?;

        Ok(VcpuState {
            mp_state,
            regs,
            mpidr,
            config: self.config.clone(),
        })
    }
}

impl Drop for KvmVcpu {
    fn drop(&mut self) {
        Self::TLS_VCPU_PTR.with(|v| {
            *v.borrow_mut() = None;
        });
    }
}
