// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use kvm_bindings::{
    kvm_device_attr, KVM_DEV_ARM_VGIC_GRP_CTRL, KVM_DEV_ARM_VGIC_SAVE_PENDING_TABLES,
};
use kvm_ioctls::DeviceFd;
use std::iter::StepBy;
use std::ops::Range;

use super::interrupts::{Error, Result};
pub use dist::{dist_regs, set_dist_regs};
pub use icc::{icc_regs, set_icc_regs, GicSysRegsState};
pub use redist::{redist_regs, set_redist_regs};

mod dist;
mod icc;
mod redist;

/// Generic GIC register state,
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GicRegState<T> {
    pub(crate) chunks: Vec<T>,
}

/// Function that flushes RDIST pending tables into guest RAM.
///
/// The tables get flushed to guest RAM whenever the VM gets stopped.
pub fn save_pending_tables(fd: &DeviceFd) -> Result<()> {
    let init_gic_attr = kvm_device_attr {
        group: KVM_DEV_ARM_VGIC_GRP_CTRL,
        attr: KVM_DEV_ARM_VGIC_SAVE_PENDING_TABLES as u64,
        addr: 0,
        flags: 0,
    };
    fd.set_device_attr(&init_gic_attr)?;
    Ok(())
}

/// Process the content of the MPIDR_EL1 register in order to be able to pass it to KVM
///
/// The kernel expects to find the four affinity levels of the MPIDR in the first 32 bits of the
/// VGIC register attribute:
/// https://elixir.free-electrons.com/linux/v4.14.203/source/virt/kvm/arm/vgic/vgic-kvm-device.c#L445.
///
/// The format of the MPIDR_EL1 register is:
/// | 39 .... 32 | 31 .... 24 | 23 .... 16 | 15 .... 8 | 7 .... 0 |
/// |    Aff3    |    Other   |    Aff2    |    Aff1   |   Aff0   |
///
/// The KVM mpidr format is:
/// | 63 .... 56 | 55 .... 48 | 47 .... 40 | 39 .... 32 |
/// |    Aff3    |    Aff2    |    Aff1    |    Aff0    |
/// As specified in the linux kernel: Documentation/virt/kvm/devices/arm-vgic-v3.rst
pub fn convert_to_kvm_mpidrs(mut mpidrs: Vec<u64>) -> Vec<u64> {
    for mpidr in mpidrs.iter_mut() {
        let cpu_affid = ((*mpidr & 0xFF_0000_0000) >> 8) | (*mpidr & 0xFF_FFFF);
        *mpidr = cpu_affid << 32;
    }
    mpidrs
}

// Helper trait for working with the different types of the GIC registers
// in a unified manner.
trait MmioReg {
    fn range(&self) -> Range<u64>;

    fn iter<T>(&self) -> StepBy<Range<u64>>
    where
        Self: Sized,
    {
        self.range().step_by(std::mem::size_of::<T>())
    }
}

fn set_regs_data<'a, Reg, RegChunk>(
    fd: &DeviceFd,
    regs: impl Iterator<Item = &'a Reg>,
    group: u32,
    data: &[GicRegState<RegChunk>],
    mpidr: u64,
    mpidr_mask: u64,
) -> Result<()>
where
    Reg: MmioReg + 'a,
    RegChunk: Clone,
{
    for (reg, reg_data) in regs.zip(data) {
        set_reg_data(fd, reg, group, reg_data, mpidr, mpidr_mask)?;
    }
    Ok(())
}

fn set_reg_data<Reg, RegChunk>(
    fd: &DeviceFd,
    reg: &Reg,
    group: u32,
    data: &GicRegState<RegChunk>,
    mpidr: u64,
    mpidr_mask: u64,
) -> Result<()>
where
    Reg: MmioReg,
    RegChunk: Clone,
{
    for (offset, val) in reg.iter::<RegChunk>().zip(&data.chunks) {
        let mut tmp = (*val).clone();
        fd.set_device_attr(&kvm_device_attr(group, offset, &mut tmp, mpidr, mpidr_mask))?;
    }

    Ok(())
}

fn get_regs_data<'a, Reg, RegChunk>(
    fd: &DeviceFd,
    regs: impl Iterator<Item = &'a Reg>,
    group: u32,
    mpidr: u64,
    mpidr_mask: u64,
) -> Result<Vec<GicRegState<RegChunk>>>
where
    Reg: MmioReg + 'a,
    RegChunk: Default,
{
    let mut data = Vec::new();
    for reg in regs {
        data.push(get_reg_data(fd, reg, group, mpidr, mpidr_mask)?);
    }

    Ok(data)
}

fn get_reg_data<Reg, RegChunk>(
    fd: &DeviceFd,
    reg: &Reg,
    group: u32,
    mpidr: u64,
    mpidr_mask: u64,
) -> Result<GicRegState<RegChunk>>
where
    Reg: MmioReg,
    RegChunk: Default,
{
    let mut data = Vec::with_capacity(reg.iter::<RegChunk>().count());
    for offset in reg.iter::<RegChunk>() {
        let mut val = RegChunk::default();
        fd.get_device_attr(&mut kvm_device_attr(
            group, offset, &mut val, mpidr, mpidr_mask,
        ))?;
        data.push(val);
    }

    Ok(GicRegState { chunks: data })
}

fn kvm_device_attr<RegChunk>(
    group: u32,
    offset: u64,
    val: &mut RegChunk,
    mpidr: u64,
    mpidr_mask: u64,
) -> kvm_device_attr {
    kvm_device_attr {
        group,
        attr: (mpidr & mpidr_mask) | offset,
        addr: val as *mut RegChunk as u64,
        flags: 0,
    }
}

/// Structure representing a simple register.
#[derive(PartialEq, Eq)]
struct SimpleReg {
    /// The offset from the component address. The register is memory mapped here.
    offset: u64,
    /// Size in bytes.
    size: u16,
}

impl SimpleReg {
    const fn new(offset: u64, size: u16) -> Self {
        Self { offset, size }
    }
}

impl MmioReg for SimpleReg {
    fn range(&self) -> Range<u64> {
        // It's technically possible for this addition to overflow.
        // However, SimpleReg is only used to define register descriptors
        // with constant offsets and sizes, so any overflow would be detected
        // during testing.
        self.offset..self.offset + u64::from(self.size)
    }
}
