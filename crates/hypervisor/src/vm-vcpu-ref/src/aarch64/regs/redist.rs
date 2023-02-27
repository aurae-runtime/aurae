// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use kvm_bindings::{KVM_DEV_ARM_VGIC_GRP_REDIST_REGS, KVM_DEV_ARM_VGIC_V3_MPIDR_MASK};
use kvm_ioctls::DeviceFd;

use super::{get_regs_data, set_regs_data, GicRegState, Result, SimpleReg};

// Relevant PPI redistributor registers that we want to save/restore.
const GICR_CTLR: SimpleReg = SimpleReg::new(0x0000, 4);
const GICR_STATUSR: SimpleReg = SimpleReg::new(0x0010, 4);
const GICR_WAKER: SimpleReg = SimpleReg::new(0x0014, 4);
const GICR_PROPBASER: SimpleReg = SimpleReg::new(0x0070, 8);
const GICR_PENDBASER: SimpleReg = SimpleReg::new(0x0078, 8);

// Relevant SGI redistributor registers that we want to save/restore.
const GICR_SGI_OFFSET: u64 = 0x0001_0000;
const GICR_IGROUPR0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0080, 4);
const GICR_ISENABLER0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0100, 4);
const GICR_ICENABLER0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0180, 4);
const GICR_ISPENDR0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0200, 4);
const GICR_ICPENDR0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0280, 4);
const GICR_ISACTIVER0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0300, 4);
const GICR_ICACTIVER0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0380, 4);
const GICR_IPRIORITYR0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0400, 32);
const GICR_ICFGR0: SimpleReg = SimpleReg::new(GICR_SGI_OFFSET + 0x0C00, 8);

// List with relevant redistributor registers and SGI associated redistributor
// registers that we will be restoring.
static VGIC_RDIST_AND_SGI_REGS: &[SimpleReg] = &[
    GICR_CTLR,
    GICR_STATUSR,
    GICR_WAKER,
    GICR_PROPBASER,
    GICR_PENDBASER,
    GICR_IGROUPR0,
    GICR_ICENABLER0,
    GICR_ISENABLER0,
    GICR_ICFGR0,
    GICR_ICPENDR0,
    GICR_ISPENDR0,
    GICR_ICACTIVER0,
    GICR_ISACTIVER0,
    GICR_IPRIORITYR0,
];

/// Get vCPU redistributor registers.
pub fn redist_regs(fd: &DeviceFd, mpidr: u64) -> Result<Vec<GicRegState<u32>>> {
    get_regs_data(
        fd,
        VGIC_RDIST_AND_SGI_REGS.iter(),
        KVM_DEV_ARM_VGIC_GRP_REDIST_REGS,
        mpidr,
        KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
    )
}

/// Set vCPU redistributor registers.
pub fn set_redist_regs(fd: &DeviceFd, redist: &[GicRegState<u32>], mpidr: u64) -> Result<()> {
    set_regs_data(
        fd,
        VGIC_RDIST_AND_SGI_REGS.iter(),
        KVM_DEV_ARM_VGIC_GRP_REDIST_REGS,
        redist,
        mpidr,
        KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
    )?;
    Ok(())
}
