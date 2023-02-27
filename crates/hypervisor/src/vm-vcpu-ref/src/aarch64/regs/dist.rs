// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use kvm_bindings::KVM_DEV_ARM_VGIC_GRP_DIST_REGS;
use kvm_ioctls::DeviceFd;
use std::ops::Range;

use super::{get_regs_data, set_regs_data, GicRegState, MmioReg, Result, SimpleReg};

// As per virt/kvm/arm/vgic/vgic-kvm-device.c we need
// the number of interrupts our GIC will support to be:
// * bigger than 32
// * less than 1023 and
// * a multiple of 32.
/// The highest usable SPI on aarch64.
const IRQ_MAX: u32 = 128;

/// First usable interrupt on aarch64.
const IRQ_BASE: u32 = 32;

// Distributor registers as detailed at page 456 from
// https://developer.arm.com/documentation/ihi0069/c/.
// Address offsets are relative to the Distributor base
// address defined by the system memory map.
const GICD_CTLR: DistReg = DistReg::simple(0x0, 4);
const GICD_STATUSR: DistReg = DistReg::simple(0x0010, 4);
const GICD_IGROUPR: DistReg = DistReg::shared_irq(0x0080, 1);
const GICD_ISENABLER: DistReg = DistReg::shared_irq(0x0100, 1);
const GICD_ICENABLER: DistReg = DistReg::shared_irq(0x0180, 1);
const GICD_ISPENDR: DistReg = DistReg::shared_irq(0x0200, 1);
const GICD_ICPENDR: DistReg = DistReg::shared_irq(0x0280, 1);
const GICD_ISACTIVER: DistReg = DistReg::shared_irq(0x0300, 1);
const GICD_ICACTIVER: DistReg = DistReg::shared_irq(0x0380, 1);
const GICD_IPRIORITYR: DistReg = DistReg::shared_irq(0x0400, 8);
const GICD_ICFGR: DistReg = DistReg::shared_irq(0x0C00, 2);
const GICD_IROUTER: DistReg = DistReg::shared_irq(0x6000, 64);

static VGIC_DIST_REGS: &[DistReg] = &[
    GICD_CTLR,
    GICD_STATUSR,
    GICD_ICENABLER,
    GICD_ISENABLER,
    GICD_IGROUPR,
    GICD_IROUTER,
    GICD_ICFGR,
    GICD_ICPENDR,
    GICD_ISPENDR,
    GICD_ICACTIVER,
    GICD_ISACTIVER,
    GICD_IPRIORITYR,
];

/// Get distributor registers.
pub fn dist_regs(fd: &DeviceFd) -> Result<Vec<GicRegState<u32>>> {
    get_regs_data(
        fd,
        VGIC_DIST_REGS.iter(),
        KVM_DEV_ARM_VGIC_GRP_DIST_REGS,
        0,
        0,
    )
}

/// Set distributor registers.
pub fn set_dist_regs(fd: &DeviceFd, dist: &[GicRegState<u32>]) -> Result<()> {
    set_regs_data(
        fd,
        VGIC_DIST_REGS.iter(),
        KVM_DEV_ARM_VGIC_GRP_DIST_REGS,
        dist,
        0,
        0,
    )
}

enum DistReg {
    Simple(SimpleReg),
    SharedIrq(SharedIrqReg),
}

impl DistReg {
    const fn simple(offset: u64, size: u16) -> DistReg {
        DistReg::Simple(SimpleReg { offset, size })
    }

    const fn shared_irq(offset: u64, bits_per_irq: u8) -> DistReg {
        DistReg::SharedIrq(SharedIrqReg {
            offset,
            bits_per_irq,
        })
    }
}

impl MmioReg for DistReg {
    fn range(&self) -> Range<u64> {
        match self {
            DistReg::Simple(reg) => reg.range(),
            DistReg::SharedIrq(reg) => reg.range(),
        }
    }
}

/// Some registers have variable lengths since they dedicate a specific number of bits to
/// each interrupt. So, their length depends on the number of interrupts.
/// (i.e the ones that are represented as GICD_REG<n>) in the documentation mentioned above.
struct SharedIrqReg {
    /// The offset from the component address. The register is memory mapped here.
    offset: u64,
    /// Number of bits per interrupt.
    bits_per_irq: u8,
}

impl MmioReg for SharedIrqReg {
    fn range(&self) -> Range<u64> {
        // The ARM® TrustZone® implements a protection logic which contains a
        // read-as-zero/write-ignore (RAZ/WI) policy.
        // The first part of a shared-irq register, the one corresponding to the
        // SGI and PPI IRQs (0-32) is RAZ/WI, so we skip it.
        //
        // It's technically possible for this operation to overflow.
        // However, SharedIrqReg is only used to define register descriptors
        // with constant offsets and bits_per_irq, so any overflow would be detected
        // during testing.
        let start = self.offset + u64::from(IRQ_BASE) * u64::from(self.bits_per_irq) / 8;

        let size_in_bits = u64::from(self.bits_per_irq) * u64::from(IRQ_MAX - IRQ_BASE);
        let mut size_in_bytes = size_in_bits / 8;
        if size_in_bits % 8 > 0 {
            size_in_bytes += 1;
        }

        start..start + size_in_bytes
    }
}
