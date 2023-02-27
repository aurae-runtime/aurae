// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// Copyright 2017 The Chromium OS Authors. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use kvm_bindings::CpuId;
use kvm_ioctls::{Cap::TscDeadlineTimer, Kvm};

// CPUID bits in ebx, ecx, and edx.
const EBX_CLFLUSH_CACHELINE: u32 = 8; // Flush a cache line size.
const EBX_CLFLUSH_SIZE_SHIFT: u32 = 8; // Bytes flushed when executing CLFLUSH.
const EBX_CPU_COUNT_SHIFT: u32 = 16; // Index of this CPU.
const EBX_CPUID_SHIFT: u32 = 24; // Index of this CPU.
const ECX_EPB_SHIFT: u32 = 3; // "Energy Performance Bias" bit.
const ECX_TSC_DEADLINE_TIMER_SHIFT: u32 = 24; // TSC deadline mode of APIC timer
const ECX_HYPERVISOR_SHIFT: u32 = 31; // Flag to be set when the cpu is running on a hypervisor.
const EDX_HTT_SHIFT: u32 = 28; // Hyper Threading Enabled.

/// Updates the passed `cpuid` such that it can be used for configuring a vCPU
/// for running.
///
/// # Example
///
/// We are recommending the `cpuid` to be created from the supported CPUID on
/// the running host.
///
/// ```rust
/// use kvm_bindings::CpuId;
/// use kvm_ioctls::{Error, Kvm};
/// use vm_vcpu_ref::x86_64::cpuid::filter_cpuid;
///
/// fn default_cpuid(cpu_index: u8, num_vcpus: u8) -> Result<CpuId, Error> {
///     let kvm = Kvm::new()?;
///     let mut cpuid = kvm.get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)?;
///     filter_cpuid(&kvm, cpu_index, num_vcpus, &mut cpuid);
///     Ok(cpuid)
/// }
///
/// # default_cpuid(0, 1).unwrap();
/// ```
pub fn filter_cpuid(kvm: &Kvm, vcpu_id: u8, cpu_count: u8, cpuid: &mut CpuId) {
    for entry in cpuid.as_mut_slice().iter_mut() {
        match entry.function {
            0x01 => {
                // X86 hypervisor feature.
                if entry.index == 0 {
                    entry.ecx |= 1 << ECX_HYPERVISOR_SHIFT;
                }
                if kvm.check_extension(TscDeadlineTimer) {
                    entry.ecx |= 1 << ECX_TSC_DEADLINE_TIMER_SHIFT;
                }
                entry.ebx = ((vcpu_id as u32) << EBX_CPUID_SHIFT) as u32
                    | (EBX_CLFLUSH_CACHELINE << EBX_CLFLUSH_SIZE_SHIFT);
                if cpu_count > 1 {
                    entry.ebx |= (cpu_count as u32) << EBX_CPU_COUNT_SHIFT;
                    entry.edx |= 1 << EDX_HTT_SHIFT;
                }
            }
            0x06 => {
                // Clear X86 EPB feature. No frequency selection in the hypervisor.
                entry.ecx &= !(1 << ECX_EPB_SHIFT);
            }
            0x0B => {
                // EDX bits 31..0 contain x2APIC ID of current logical processor.
                entry.edx = vcpu_id as u32;
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vmm_sys_util::fam::FamStruct;

    #[test]
    fn test_filter_cpuid() {
        // This is a bit of an artificial test because there's not much we can
        // validate at the unit test level.
        let vcpu_id = 0;
        let kvm = Kvm::new().unwrap();

        let mut cpuid = kvm
            .get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)
            .unwrap();
        let before_len = cpuid.as_fam_struct_ref().len();
        filter_cpuid(&kvm, vcpu_id, 1, &mut cpuid);

        // Check that no new entries than the supported ones are added.
        assert_eq!(cpuid.as_fam_struct_ref().len(), before_len);

        // Check that setting this cpuid to a vcpu does not yield an error.
        let vm = kvm.create_vm().unwrap();
        let vcpu = vm.create_vcpu(0).unwrap();
        vcpu.set_cpuid2(&cpuid).unwrap();
    }
}
