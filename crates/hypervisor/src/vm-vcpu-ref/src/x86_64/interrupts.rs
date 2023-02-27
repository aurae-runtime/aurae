// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.
#![cfg(target_arch = "x86_64")]
use kvm_bindings::kvm_lapic_state;

// Helper function that writes the input (which must have 4 bytes) into an i32.
fn read_le_i32(input: &[i8]) -> i32 {
    assert_eq!(input.len(), 4);
    let mut array = [0u8; 4];
    for (byte, read) in array.iter_mut().zip(input.iter().cloned()) {
        *byte = read as u8;
    }
    i32::from_le_bytes(array)
}

// Helper function that writes bytes from an i32 into `buf`.
fn write_le_i32(buf: &mut [i8], n: i32) {
    for (byte, read) in buf.iter_mut().zip(i32::to_le_bytes(n).iter().cloned()) {
        *byte = read as i8;
    }
}

// Defines poached from apicdef.h kernel header.
/// The register offset corresponding to the APIC Local Vector Table for LINT0.
pub const APIC_LVT0_REG_OFFSET: usize = 0x350;
/// The register offset corresponding to the APIC Local Vector Table for LINT1.
pub const APIC_LVT1_REG_OFFSET: usize = 0x360;

/// Specifies the type of interrupt to be sent to the processor.
#[derive(Debug, PartialEq, Eq)]
// We user upper case acronyms so that we can stick to the names as they're
// defined in the Intel Manual.
#[allow(clippy::upper_case_acronyms)]
pub enum DeliveryMode {
    /// A fixed interrupt - delivers the interrupt specified in the vector field.
    Fixed = 0b000,
    /// System Management Interrupt.
    SMI = 0b010,
    /// Non Maskable Interrupt.
    NMI = 0b100,
    /// This type of interrupt causes the processor to perform an INIT.
    INIT = 0b101,
    /// External Interrupt.
    ExtINT = 0b111,
}

/// Errors associated with operations related to interrupts.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The register offset is invalid.
    #[error("The register offset is invalid.")]
    InvalidRegisterOffset,
}

/// Specialized result type for operations related to interrupts.
pub type Result<T> = std::result::Result<T, Error>;

/// Return the value of the register specified by `reg_offset`. Returns an error when the
/// offset is invalid.
pub fn get_klapic_reg(klapic: &kvm_lapic_state, reg_offset: usize) -> Result<i32> {
    let range = reg_offset..reg_offset + 4;
    let reg = klapic.regs.get(range).ok_or(Error::InvalidRegisterOffset)?;
    Ok(read_le_i32(reg))
}

/// Set the `value` of the register located at `reg_offset`. Returns an error when the offset is
/// invalid.
pub fn set_klapic_reg(klapic: &mut kvm_lapic_state, reg_offset: usize, value: i32) -> Result<()> {
    // The value that we are setting is a u32, which needs 4 bytes of space.
    // We're here creating a range that can fit the whole value.
    let range = reg_offset..reg_offset + 4;
    let reg = klapic
        .regs
        .get_mut(range)
        .ok_or(Error::InvalidRegisterOffset)?;
    write_le_i32(reg, value);
    Ok(())
}

fn set_apic_delivery_mode(reg: i32, mode: i32) -> i32 {
    ((reg) & !0x700) | ((mode) << 8)
}

/// Set the Local APIC delivery mode. Returns an error when the register configuration
/// is invalid.
///
/// **Note**: setting the Local APIC (using the kvm function `set_lapic`) MUST happen
/// after creating the IRQ chip. Otherwise, KVM returns an invalid argument (errno 22).
///
/// # Arguments
/// * `klapic`: The corresponding `kvm_lapic_state` for which to set the delivery mode.
/// * `reg_offset`: The offset that identifies the register for which to set the delivery mode.
///                 Available options exported by this module are: [APIC_LVT0_REG_OFFSET] and
///                 [APIC_LVT1_REG_OFFSET].
/// * `mode`: The APIC mode to set.
///
/// # Example - Configure LAPIC with KVM
/// ```rust
/// # use kvm_ioctls::Kvm;
/// use kvm_ioctls::{Error, VcpuFd};
/// use vm_vcpu_ref::x86_64::interrupts::{
///     set_klapic_delivery_mode, DeliveryMode, APIC_LVT0_REG_OFFSET, APIC_LVT1_REG_OFFSET,
/// };
///
/// fn configure_default_lapic(vcpu_fd: &mut VcpuFd) -> Result<(), Error> {
///     let mut klapic = vcpu_fd.get_lapic()?;
///
///     set_klapic_delivery_mode(&mut klapic, APIC_LVT0_REG_OFFSET, DeliveryMode::ExtINT).unwrap();
///     set_klapic_delivery_mode(&mut klapic, APIC_LVT1_REG_OFFSET, DeliveryMode::NMI).unwrap();
///     vcpu_fd.set_lapic(&klapic)
/// }
///
/// # let kvm = Kvm::new().unwrap();
/// # let vm = kvm.create_vm().unwrap();
/// # vm.create_irq_chip().unwrap();
/// # let mut vcpu = vm.create_vcpu(0).unwrap();
/// # configure_default_lapic(&mut vcpu).unwrap();
/// ```
pub fn set_klapic_delivery_mode(
    klapic: &mut kvm_lapic_state,
    reg_offset: usize,
    mode: DeliveryMode,
) -> Result<()> {
    let reg_value = get_klapic_reg(klapic, reg_offset)?;
    set_klapic_reg(
        klapic,
        reg_offset,
        set_apic_delivery_mode(reg_value, mode as i32),
    )
}

#[cfg(test)]
mod tests {
    use crate::x86_64::interrupts::{
        get_klapic_reg, set_klapic_delivery_mode, DeliveryMode, Error,
    };
    use kvm_bindings::kvm_lapic_state;

    #[test]
    fn test_reg_offset() {
        // The size of `regs` in `kvm_lapic_state` is 1024. Since we're setting a value of
        // 4 bytes, if we want to set it at offset = 1020 it should fit.
        let offset = 1020;
        let mut klapic = kvm_lapic_state::default();
        assert!(set_klapic_delivery_mode(&mut klapic, offset, DeliveryMode::ExtINT).is_ok());
        assert!(get_klapic_reg(&klapic, offset).is_ok());

        // Setting at the offset og 1021 does not work because 4 bytes don't fit.
        let offset = 1021;
        let mut klapic = kvm_lapic_state::default();
        assert_eq!(
            set_klapic_delivery_mode(&mut klapic, offset, DeliveryMode::ExtINT).unwrap_err(),
            Error::InvalidRegisterOffset
        );
        assert_eq!(
            get_klapic_reg(&klapic, offset).unwrap_err(),
            Error::InvalidRegisterOffset
        );
    }

    #[test]
    fn test_delivery_mode() {
        assert_eq!(DeliveryMode::Fixed as i32, 0);
        assert_eq!(DeliveryMode::SMI as i32, 0x2);
        assert_eq!(DeliveryMode::NMI as i32, 0x4);
        assert_eq!(DeliveryMode::INIT as i32, 0x5);
        assert_eq!(DeliveryMode::ExtINT as i32, 0x7);
    }
}
