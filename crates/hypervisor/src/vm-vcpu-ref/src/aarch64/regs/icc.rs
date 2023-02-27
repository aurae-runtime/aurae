// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use kvm_bindings::{KVM_DEV_ARM_VGIC_GRP_CPU_SYSREGS, KVM_DEV_ARM_VGIC_V3_MPIDR_MASK};
use kvm_bindings::{
    KVM_REG_ARM64_SYSREG_CRM_MASK, KVM_REG_ARM64_SYSREG_CRM_SHIFT, KVM_REG_ARM64_SYSREG_CRN_MASK,
    KVM_REG_ARM64_SYSREG_CRN_SHIFT, KVM_REG_ARM64_SYSREG_OP0_MASK, KVM_REG_ARM64_SYSREG_OP0_SHIFT,
    KVM_REG_ARM64_SYSREG_OP1_MASK, KVM_REG_ARM64_SYSREG_OP1_SHIFT, KVM_REG_ARM64_SYSREG_OP2_MASK,
    KVM_REG_ARM64_SYSREG_OP2_SHIFT,
};
use kvm_ioctls::DeviceFd;

use super::{
    get_reg_data, get_regs_data, set_reg_data, set_regs_data, Error, GicRegState, Result, SimpleReg,
};

const SYS_ICC_SRE_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 12, 12, 5);
const SYS_ICC_CTLR_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 12, 12, 4);
const SYS_ICC_IGRPEN0_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 12, 12, 6);
const SYS_ICC_IGRPEN1_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 12, 12, 7);
const SYS_ICC_PMR_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 4, 6, 0);
const SYS_ICC_BPR0_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 12, 8, 3);
const SYS_ICC_BPR1_EL1: SimpleReg = SimpleReg::gic_sys_reg(3, 0, 12, 12, 3);

static MAIN_GIC_ICC_REGS: &[SimpleReg] = &[
    SYS_ICC_SRE_EL1,
    SYS_ICC_CTLR_EL1,
    SYS_ICC_IGRPEN0_EL1,
    SYS_ICC_IGRPEN1_EL1,
    SYS_ICC_PMR_EL1,
    SYS_ICC_BPR0_EL1,
    SYS_ICC_BPR1_EL1,
];

const SYS_ICC_AP0R0_EL1: SimpleReg = SimpleReg::sys_icc_ap0rn_el1(0);
const SYS_ICC_AP0R1_EL1: SimpleReg = SimpleReg::sys_icc_ap0rn_el1(1);
const SYS_ICC_AP0R2_EL1: SimpleReg = SimpleReg::sys_icc_ap0rn_el1(2);
const SYS_ICC_AP0R3_EL1: SimpleReg = SimpleReg::sys_icc_ap0rn_el1(3);
const SYS_ICC_AP1R0_EL1: SimpleReg = SimpleReg::sys_icc_ap1rn_el1(0);
const SYS_ICC_AP1R1_EL1: SimpleReg = SimpleReg::sys_icc_ap1rn_el1(1);
const SYS_ICC_AP1R2_EL1: SimpleReg = SimpleReg::sys_icc_ap1rn_el1(2);
const SYS_ICC_AP1R3_EL1: SimpleReg = SimpleReg::sys_icc_ap1rn_el1(3);

static AP_GIC_ICC_REGS: &[SimpleReg] = &[
    SYS_ICC_AP0R0_EL1,
    SYS_ICC_AP0R1_EL1,
    SYS_ICC_AP0R2_EL1,
    SYS_ICC_AP0R3_EL1,
    SYS_ICC_AP1R0_EL1,
    SYS_ICC_AP1R1_EL1,
    SYS_ICC_AP1R2_EL1,
    SYS_ICC_AP1R3_EL1,
];

const ICC_CTLR_EL1_PRIBITS_SHIFT: u64 = 8;
const ICC_CTLR_EL1_PRIBITS_MASK: u64 = 7 << ICC_CTLR_EL1_PRIBITS_SHIFT;

/// Structure for serializing the state of the GIC ICC regs
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GicSysRegsState {
    main_icc_regs: Vec<GicRegState<u64>>,
    ap_icc_regs: Vec<Option<GicRegState<u64>>>,
}

impl SimpleReg {
    const fn gic_sys_reg(op0: u64, op1: u64, crn: u64, crm: u64, op2: u64) -> SimpleReg {
        let offset = (((op0 as u64) << KVM_REG_ARM64_SYSREG_OP0_SHIFT)
            & KVM_REG_ARM64_SYSREG_OP0_MASK as u64)
            | (((op1 as u64) << KVM_REG_ARM64_SYSREG_OP1_SHIFT)
                & KVM_REG_ARM64_SYSREG_OP1_MASK as u64)
            | (((crn as u64) << KVM_REG_ARM64_SYSREG_CRN_SHIFT)
                & KVM_REG_ARM64_SYSREG_CRN_MASK as u64)
            | (((crm as u64) << KVM_REG_ARM64_SYSREG_CRM_SHIFT)
                & KVM_REG_ARM64_SYSREG_CRM_MASK as u64)
            | (((op2 as u64) << KVM_REG_ARM64_SYSREG_OP2_SHIFT)
                & KVM_REG_ARM64_SYSREG_OP2_MASK as u64);

        SimpleReg { offset, size: 8 }
    }

    const fn sys_icc_ap0rn_el1(n: u64) -> SimpleReg {
        Self::gic_sys_reg(3, 0, 12, 8, 4 | n)
    }

    const fn sys_icc_ap1rn_el1(n: u64) -> SimpleReg {
        Self::gic_sys_reg(3, 0, 12, 9, n)
    }
}

/// Get vCPU GIC system registers.
pub fn icc_regs(fd: &DeviceFd, mpidr: u64) -> Result<GicSysRegsState> {
    let main_icc_regs = get_regs_data(
        fd,
        MAIN_GIC_ICC_REGS.iter(),
        KVM_DEV_ARM_VGIC_GRP_CPU_SYSREGS,
        mpidr,
        KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
    )?;

    let num_priority_bits = num_priority_bits(fd, mpidr)?;

    let mut ap_icc_regs = Vec::with_capacity(AP_GIC_ICC_REGS.len());
    for reg in AP_GIC_ICC_REGS {
        if is_ap_reg_available(reg, num_priority_bits) {
            ap_icc_regs.push(Some(get_reg_data(
                fd,
                reg,
                KVM_DEV_ARM_VGIC_GRP_CPU_SYSREGS,
                mpidr,
                KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
            )?));
        } else {
            ap_icc_regs.push(None);
        }
    }

    Ok(GicSysRegsState {
        main_icc_regs,
        ap_icc_regs,
    })
}

/// Set vCPU GIC system registers.
pub fn set_icc_regs(fd: &DeviceFd, state: &GicSysRegsState, mpidr: u64) -> Result<()> {
    set_regs_data(
        fd,
        MAIN_GIC_ICC_REGS.iter(),
        KVM_DEV_ARM_VGIC_GRP_CPU_SYSREGS,
        &state.main_icc_regs,
        mpidr,
        KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
    )?;

    let num_priority_bits = num_priority_bits(fd, mpidr)?;

    for (reg, maybe_reg_data) in AP_GIC_ICC_REGS.iter().zip(&state.ap_icc_regs) {
        if is_ap_reg_available(reg, num_priority_bits) != maybe_reg_data.is_some() {
            return Err(Error::InvalidGicSysRegState);
        }

        if let Some(reg_data) = maybe_reg_data {
            set_reg_data(
                fd,
                reg,
                KVM_DEV_ARM_VGIC_GRP_CPU_SYSREGS,
                reg_data,
                mpidr,
                KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
            )?;
        }
    }

    Ok(())
}

fn num_priority_bits(fd: &DeviceFd, mpidr: u64) -> Result<u64> {
    let reg_val: u64 = get_reg_data(
        fd,
        &SYS_ICC_CTLR_EL1,
        KVM_DEV_ARM_VGIC_GRP_CPU_SYSREGS,
        mpidr,
        KVM_DEV_ARM_VGIC_V3_MPIDR_MASK as u64,
    )?
    .chunks[0];

    Ok(((reg_val & ICC_CTLR_EL1_PRIBITS_MASK) >> ICC_CTLR_EL1_PRIBITS_SHIFT) + 1)
}

fn is_ap_reg_available(reg: &SimpleReg, num_priority_bits: u64) -> bool {
    // As per ARMv8 documentation:
    // https://developer.arm.com/documentation/ihi0069/c/
    // page 178,
    // ICC_AP0R1_EL1 is only implemented in implementations that support 6 or more bits of
    // priority.
    // ICC_AP0R2_EL1 and ICC_AP0R3_EL1 are only implemented in implementations that support
    // 7 bits of priority.
    if (reg == &SYS_ICC_AP0R1_EL1 || reg == &SYS_ICC_AP1R1_EL1) && num_priority_bits < 6 {
        return false;
    }
    if (reg == &SYS_ICC_AP0R2_EL1
        || reg == &SYS_ICC_AP0R3_EL1
        || reg == &SYS_ICC_AP1R2_EL1
        || reg == &SYS_ICC_AP1R3_EL1)
        && num_priority_bits != 7
    {
        return false;
    }

    true
}
