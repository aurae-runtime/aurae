use kvm_bindings::*;
use kvm_ioctls::VcpuFd;

use super::Error;

macro_rules! arm64_core_reg {
    ($reg: tt) => {
        KVM_REG_ARM64
            | KVM_REG_SIZE_U64
            | u64::from(KVM_REG_ARM_CORE)
            | ((offset__of!(kvm_bindings::user_pt_regs, $reg) / 4) as u64)
    };
}

// This macro gets the offset of a structure (i.e `str`) member (i.e `field`) without having
// an instance of that structure.
#[macro_export]
macro_rules! offset__of {
    ($str:ty, $($field:ident)+) => ({
        let tmp: std::mem::MaybeUninit<$str> = std::mem::MaybeUninit::uninit();
        // Safe because we are not using the value of tmp.
        let tmp = unsafe { tmp.assume_init() };
        let base = &tmp as *const _ as usize;
        let member =  &tmp.$($field)* as *const _ as usize;

        member - base
    });
}

// Compute the ID of a specific ARM64 system register similar to how
// the kernel C macro does.
// https://elixir.bootlin.com/linux/v4.20.17/source/arch/arm64/include/uapi/asm/kvm.h#L203
const fn arm64_sys_reg(op0: u64, op1: u64, crn: u64, crm: u64, op2: u64) -> u64 {
    KVM_REG_ARM64
        | KVM_REG_SIZE_U64
        | KVM_REG_ARM64_SYSREG as u64
        | ((op0 << KVM_REG_ARM64_SYSREG_OP0_SHIFT) & KVM_REG_ARM64_SYSREG_OP0_MASK as u64)
        | ((op1 << KVM_REG_ARM64_SYSREG_OP1_SHIFT) & KVM_REG_ARM64_SYSREG_OP1_MASK as u64)
        | ((crn << KVM_REG_ARM64_SYSREG_CRN_SHIFT) & KVM_REG_ARM64_SYSREG_CRN_MASK as u64)
        | ((crm << KVM_REG_ARM64_SYSREG_CRM_SHIFT) & KVM_REG_ARM64_SYSREG_CRM_MASK as u64)
        | ((op2 << KVM_REG_ARM64_SYSREG_OP2_SHIFT) & KVM_REG_ARM64_SYSREG_OP2_MASK as u64)
}

// The MPIDR_EL1 register ID is defined in the kernel:
// https://elixir.bootlin.com/linux/v4.20.17/source/arch/arm64/include/asm/sysreg.h#L135
const MPIDR_EL1: u64 = arm64_sys_reg(3, 0, 0, 0, 5);

// Get the values of all vCPU registers. The value of MPIDR register is also
// returned as the second element of the tuple as an optimization to prevent
// linear scan. This value is needed when saving the GIC state.
pub fn get_regs_and_mpidr(vcpu_fd: &VcpuFd) -> Result<(Vec<kvm_one_reg>, u64), Error> {
    // Get IDs of all registers available to the guest.
    // For ArmV8 there are less than 500 registers.
    let mut reg_id_list = RegList::new(500).map_err(Error::FamError)?;
    vcpu_fd
        .get_reg_list(&mut reg_id_list)
        .map_err(Error::VcpuGetRegList)?;

    let mut mpidr = None;
    let mut regs = Vec::with_capacity(reg_id_list.as_slice().len());
    for &id in reg_id_list.as_slice() {
        let addr = vcpu_fd.get_one_reg(id).map_err(Error::VcpuGetReg)?;
        regs.push(kvm_one_reg { id, addr });

        if id == MPIDR_EL1 {
            mpidr = Some(addr);
        }
    }

    if mpidr.is_none() {
        return Err(Error::VcpuGetMpidrReg);
    }

    // unwrap() is safe because of the is_none() check above
    Ok((regs, mpidr.unwrap()))
}
