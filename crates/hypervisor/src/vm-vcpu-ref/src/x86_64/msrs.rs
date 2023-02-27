// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
use kvm_bindings::{kvm_msr_entry, Msrs};
use kvm_ioctls::Kvm;

use crate::x86_64::msr_index::*;

/// Errors associated with operations on MSRs.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Failed to initialize MSRS.
    CreateMsrs,
    /// Failed to get supported MSRs.
    GetSupportedMSR(kvm_ioctls::Error),
}
/// Specialized result type for operations on MSRs.
pub type Result<T> = std::result::Result<T, Error>;

/// Base MSR for APIC
const APIC_BASE_MSR: u32 = 0x800;

/// Number of APIC MSR indexes
const APIC_MSR_INDEXES: u32 = 0x400;

/// Custom MSRs fall in the range 0x4b564d00-0x4b564dff
const MSR_KVM_WALL_CLOCK_NEW: u32 = 0x4b56_4d00;
const MSR_KVM_SYSTEM_TIME_NEW: u32 = 0x4b56_4d01;
const MSR_KVM_ASYNC_PF_EN: u32 = 0x4b56_4d02;
const MSR_KVM_STEAL_TIME: u32 = 0x4b56_4d03;
const MSR_KVM_PV_EOI_EN: u32 = 0x4b56_4d04;

/// Taken from arch/x86/include/asm/msr-index.h
const MSR_IA32_SPEC_CTRL: u32 = 0x0000_0048;
const MSR_IA32_PRED_CMD: u32 = 0x0000_0049;

/// Creates and populates required MSR entries for booting Linux on X86_64.
///
/// # Example - Set boot MSRs
///
/// ```rust
/// use kvm_ioctls::Kvm;
/// use vm_vcpu_ref::x86_64::msrs::create_boot_msr_entries;
///
/// let kvm = Kvm::new().unwrap();
/// let vm = kvm.create_vm().unwrap();
/// let vcpu = vm.create_vcpu(0).unwrap();
///
/// vcpu.set_msrs(&create_boot_msr_entries().unwrap()).unwrap();
/// ```
pub fn create_boot_msr_entries() -> Result<Msrs> {
    let msr_entry_default = |msr| kvm_msr_entry {
        index: msr,
        data: 0x0,
        ..Default::default()
    };

    let raw_msrs = vec![
        msr_entry_default(MSR_IA32_SYSENTER_CS),
        msr_entry_default(MSR_IA32_SYSENTER_ESP),
        msr_entry_default(MSR_IA32_SYSENTER_EIP),
        // x86_64 specific msrs, we only run on x86_64 not x86.
        msr_entry_default(MSR_STAR),
        msr_entry_default(MSR_CSTAR),
        msr_entry_default(MSR_KERNEL_GS_BASE),
        msr_entry_default(MSR_SYSCALL_MASK),
        msr_entry_default(MSR_LSTAR),
        // end of x86_64 specific code
        msr_entry_default(MSR_IA32_TSC),
        kvm_msr_entry {
            index: MSR_IA32_MISC_ENABLE,
            data: u64::from(MSR_IA32_MISC_ENABLE_FAST_STRING),
            ..Default::default()
        },
    ];

    Msrs::from_entries(&raw_msrs).map_err(|_| Error::CreateMsrs)
}

/// MSR range
struct MsrRange {
    /// Base MSR address
    base: u32,
    /// Number of MSRs
    nmsrs: u32,
}

impl MsrRange {
    /// Returns whether `msr` is contained in this MSR range.
    fn contains(&self, msr: u32) -> bool {
        self.base <= msr && msr < self.base + self.nmsrs
    }
}

// Creates a MsrRange of one msr given as argument.
macro_rules! SINGLE_MSR {
    ($msr:expr) => {
        MsrRange {
            base: $msr,
            nmsrs: 1,
        }
    };
}

// Creates a MsrRange of with msr base and count given as arguments.
macro_rules! MSR_RANGE {
    ($first:expr, $count:expr) => {
        MsrRange {
            base: $first,
            nmsrs: $count,
        }
    };
}

// List of MSRs that can be serialized. List is sorted in ascending order of MSRs addresses.
static ALLOWED_MSR_RANGES: &[MsrRange] = &[
    SINGLE_MSR!(MSR_IA32_P5_MC_ADDR),
    SINGLE_MSR!(MSR_IA32_P5_MC_TYPE),
    SINGLE_MSR!(MSR_IA32_TSC),
    SINGLE_MSR!(MSR_IA32_PLATFORM_ID),
    SINGLE_MSR!(MSR_IA32_APICBASE),
    SINGLE_MSR!(MSR_IA32_EBL_CR_POWERON),
    SINGLE_MSR!(MSR_EBC_FREQUENCY_ID),
    SINGLE_MSR!(MSR_SMI_COUNT),
    SINGLE_MSR!(MSR_IA32_FEATURE_CONTROL),
    SINGLE_MSR!(MSR_IA32_TSC_ADJUST),
    SINGLE_MSR!(MSR_IA32_SPEC_CTRL),
    SINGLE_MSR!(MSR_IA32_PRED_CMD),
    SINGLE_MSR!(MSR_IA32_UCODE_WRITE),
    SINGLE_MSR!(MSR_IA32_UCODE_REV),
    SINGLE_MSR!(MSR_IA32_SMBASE),
    SINGLE_MSR!(MSR_FSB_FREQ),
    SINGLE_MSR!(MSR_PLATFORM_INFO),
    SINGLE_MSR!(MSR_PKG_CST_CONFIG_CONTROL),
    SINGLE_MSR!(MSR_IA32_MPERF),
    SINGLE_MSR!(MSR_IA32_APERF),
    SINGLE_MSR!(MSR_MTRRcap),
    SINGLE_MSR!(MSR_IA32_BBL_CR_CTL3),
    SINGLE_MSR!(MSR_IA32_SYSENTER_CS),
    SINGLE_MSR!(MSR_IA32_SYSENTER_ESP),
    SINGLE_MSR!(MSR_IA32_SYSENTER_EIP),
    SINGLE_MSR!(MSR_IA32_MCG_CAP),
    SINGLE_MSR!(MSR_IA32_MCG_STATUS),
    SINGLE_MSR!(MSR_IA32_MCG_CTL),
    SINGLE_MSR!(MSR_IA32_PERF_STATUS),
    SINGLE_MSR!(MSR_IA32_MISC_ENABLE),
    SINGLE_MSR!(MSR_MISC_FEATURE_CONTROL),
    SINGLE_MSR!(MSR_MISC_PWR_MGMT),
    SINGLE_MSR!(MSR_TURBO_RATIO_LIMIT),
    SINGLE_MSR!(MSR_TURBO_RATIO_LIMIT1),
    SINGLE_MSR!(MSR_IA32_DEBUGCTLMSR),
    SINGLE_MSR!(MSR_IA32_LASTBRANCHFROMIP),
    SINGLE_MSR!(MSR_IA32_LASTBRANCHTOIP),
    SINGLE_MSR!(MSR_IA32_LASTINTFROMIP),
    SINGLE_MSR!(MSR_IA32_LASTINTTOIP),
    SINGLE_MSR!(MSR_IA32_POWER_CTL),
    MSR_RANGE!(
        // IA32_MTRR_PHYSBASE0
        0x200, 0x100
    ),
    MSR_RANGE!(
        // MSR_CORE_C3_RESIDENCY
        // MSR_CORE_C6_RESIDENCY
        // MSR_CORE_C7_RESIDENCY
        MSR_CORE_C3_RESIDENCY,
        3
    ),
    MSR_RANGE!(MSR_IA32_MC0_CTL, 0x80),
    SINGLE_MSR!(MSR_RAPL_POWER_UNIT),
    MSR_RANGE!(
        // MSR_PKGC3_IRTL
        // MSR_PKGC6_IRTL
        // MSR_PKGC7_IRTL
        MSR_PKGC3_IRTL,
        3
    ),
    SINGLE_MSR!(MSR_PKG_POWER_LIMIT),
    SINGLE_MSR!(MSR_PKG_ENERGY_STATUS),
    SINGLE_MSR!(MSR_PKG_PERF_STATUS),
    SINGLE_MSR!(MSR_PKG_POWER_INFO),
    SINGLE_MSR!(MSR_DRAM_POWER_LIMIT),
    SINGLE_MSR!(MSR_DRAM_ENERGY_STATUS),
    SINGLE_MSR!(MSR_DRAM_PERF_STATUS),
    SINGLE_MSR!(MSR_DRAM_POWER_INFO),
    SINGLE_MSR!(MSR_CONFIG_TDP_NOMINAL),
    SINGLE_MSR!(MSR_CONFIG_TDP_LEVEL_1),
    SINGLE_MSR!(MSR_CONFIG_TDP_LEVEL_2),
    SINGLE_MSR!(MSR_CONFIG_TDP_CONTROL),
    SINGLE_MSR!(MSR_TURBO_ACTIVATION_RATIO),
    SINGLE_MSR!(MSR_IA32_TSCDEADLINE),
    MSR_RANGE!(APIC_BASE_MSR, APIC_MSR_INDEXES),
    SINGLE_MSR!(MSR_IA32_BNDCFGS),
    SINGLE_MSR!(MSR_KVM_WALL_CLOCK_NEW),
    SINGLE_MSR!(MSR_KVM_SYSTEM_TIME_NEW),
    SINGLE_MSR!(MSR_KVM_ASYNC_PF_EN),
    SINGLE_MSR!(MSR_KVM_STEAL_TIME),
    SINGLE_MSR!(MSR_KVM_PV_EOI_EN),
    SINGLE_MSR!(MSR_EFER),
    SINGLE_MSR!(MSR_STAR),
    SINGLE_MSR!(MSR_LSTAR),
    SINGLE_MSR!(MSR_CSTAR),
    SINGLE_MSR!(MSR_SYSCALL_MASK),
    SINGLE_MSR!(MSR_FS_BASE),
    SINGLE_MSR!(MSR_GS_BASE),
    SINGLE_MSR!(MSR_KERNEL_GS_BASE),
    SINGLE_MSR!(MSR_TSC_AUX),
];

/// Specifies whether a particular MSR should be included in vcpu serialization.
///
/// # Arguments
///
/// * `index` - The index of the MSR that is checked whether it's needed for serialization.
fn msr_should_serialize(index: u32) -> bool {
    // Denied MSRs not exported by Linux: IA32_FEATURE_CONTROL and IA32_MCG_CTL
    if index == MSR_IA32_FEATURE_CONTROL || index == MSR_IA32_MCG_CTL {
        return false;
    };
    ALLOWED_MSR_RANGES.iter().any(|range| range.contains(index))
}

/// Returns the list of supported, serializable MSRs.
///
/// # Arguments
///
/// * `kvm_fd` - Structure that holds the KVM's fd.
pub fn supported_guest_msrs(kvm_fd: &Kvm) -> Result<Msrs> {
    let mut msr_list = kvm_fd
        .get_msr_index_list()
        .map_err(Error::GetSupportedMSR)?;

    msr_list.retain(|msr_index| msr_should_serialize(*msr_index));

    let mut msrs =
        Msrs::new(msr_list.as_fam_struct_ref().nmsrs as usize).map_err(|_| Error::CreateMsrs)?;
    let indices = msr_list.as_slice();
    let msr_entries = msrs.as_mut_slice();
    // We created the msrs from the msr_list. If the size is not the same,
    // there is a fatal programming error.
    assert_eq!(indices.len(), msr_entries.len());
    for (pos, index) in indices.iter().enumerate() {
        msr_entries[pos].index = *index;
    }

    Ok(msrs)
}

#[cfg(test)]
mod tests {
    use crate::x86_64::msrs::{create_boot_msr_entries, supported_guest_msrs};
    use kvm_ioctls::Kvm;

    #[test]
    fn test_create_boot_msrs() {
        // This is a rather dummy test to check that creating the MSRs that we
        // need for booting can be initialized into the `Msrs` type without
        // yielding any error.
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        let vcpu = vm.create_vcpu(0).unwrap();

        let boot_msrs = create_boot_msr_entries().unwrap();
        assert!(vcpu.set_msrs(&boot_msrs).is_ok())
    }

    #[test]
    fn test_supported_guest_msrs() {
        // This is a rather dummy test to check that with a basic initialization
        // we don't hit any errors. There is not much we can test here.
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        let vcpu = vm.create_vcpu(0).unwrap();

        let mut msrs = supported_guest_msrs(&kvm).unwrap();
        let expected_nmsrs = msrs.as_fam_struct_ref().nmsrs as usize;
        let actual_nmsrs = vcpu.get_msrs(&mut msrs).unwrap();
        assert_eq!(expected_nmsrs, actual_nmsrs);
    }
}
