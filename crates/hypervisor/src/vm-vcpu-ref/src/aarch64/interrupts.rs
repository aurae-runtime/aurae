// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use kvm_bindings::{
    kvm_create_device, kvm_device_attr, kvm_device_type_KVM_DEV_TYPE_ARM_VGIC_V2,
    kvm_device_type_KVM_DEV_TYPE_ARM_VGIC_V3, KVM_DEV_ARM_VGIC_CTRL_INIT,
    KVM_DEV_ARM_VGIC_GRP_ADDR, KVM_DEV_ARM_VGIC_GRP_CTRL, KVM_DEV_ARM_VGIC_GRP_NR_IRQS,
    KVM_VGIC_V2_ADDR_TYPE_CPU, KVM_VGIC_V2_ADDR_TYPE_DIST, KVM_VGIC_V3_ADDR_TYPE_DIST,
    KVM_VGIC_V3_ADDR_TYPE_REDIST,
};
use kvm_ioctls::{DeviceFd, VmFd};

use super::regs::{
    convert_to_kvm_mpidrs, dist_regs, icc_regs, redist_regs, save_pending_tables, set_dist_regs,
    set_icc_regs, set_redist_regs, GicRegState, GicSysRegsState,
};

/// The minimum number of interrupts supported by the GIC.
// This is the minimum number of SPI interrupts aligned to 32 + 32 for the
// PPI (16) and GSI (16).
pub const MIN_NR_IRQS: u32 = 64;

const AARCH64_AXI_BASE: u64 = 0x40000000;

// These constants indicate the address space used by the ARM vGIC.
// TODO: find a way to export the registers base & lengths as part of the GIC.
const AARCH64_GIC_DIST_SIZE: u64 = 0x10000;
const AARCH64_GIC_CPUI_SIZE: u64 = 0x20000;

// These constants indicate the placement of the GIC registers in the physical
// address space.
const AARCH64_GIC_DIST_BASE: u64 = AARCH64_AXI_BASE - AARCH64_GIC_DIST_SIZE;
const AARCH64_GIC_CPUI_BASE: u64 = AARCH64_GIC_DIST_BASE - AARCH64_GIC_CPUI_SIZE;
const AARCH64_GIC_REDIST_SIZE: u64 = 0x20000;

/// Specifies the version of the GIC device
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GicVersion {
    // The following casts are safe because the device type has small values (i.e. 5 and 7)
    /// GICv2 identifier.
    V2 = kvm_device_type_KVM_DEV_TYPE_ARM_VGIC_V2 as isize,
    /// GICv3 identifier.
    V3 = kvm_device_type_KVM_DEV_TYPE_ARM_VGIC_V3 as isize,
}

/// Errors associated with operations related to the GIC.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// Error calling into KVM ioctl.
    #[error("Error calling into KVM ioctl: {0}")]
    Kvm(kvm_ioctls::Error),
    /// Error creating the GIC device.
    #[error("Error creating the GIC device: {0}")]
    CreateDevice(kvm_ioctls::Error),
    /// Error setting an attribute for the GIC device.
    #[error("Error setting an attribute ({0}) for the GIC device: {1}")]
    SetAttr(&'static str, kvm_ioctls::Error),
    /// Inconsisted vCPU count between GIC and vCPU states.
    #[error("Inconsisted vCPU count between the GIC and vCPU states")]
    InconsistentVcpuCount,
    /// Invalid state of the GIC system registers.
    #[error("Invalid state of the GIC system registers")]
    InvalidGicSysRegState,
}

impl From<kvm_ioctls::Error> for Error {
    fn from(inner: kvm_ioctls::Error) -> Self {
        Error::Kvm(inner)
    }
}

/// Specialized result type for operations on the GIC.
pub type Result<T> = std::result::Result<T, Error>;

/// High level wrapper for creating and managing the GIC device.
#[derive(Debug)]
pub struct Gic {
    version: GicVersion,
    device_fd: DeviceFd,
    num_irqs: u32,
    num_cpus: u8,
}

/// Configuration of the GIC device.
///
/// # Example
/// ```rust
/// use vm_vcpu_ref::aarch64::interrupts::{GicConfig, GicVersion};
///
/// // Create a default configuration for GICv2. We only care about setting the version.
/// let config = GicConfig {
///     version: Some(GicVersion::V2),
///     ..Default::default()
/// };
///
/// // Create a default configuration for GICv3. We also need to setup the cpu_num.
/// let config = GicConfig {
///     version: Some(GicVersion::V3),
///     num_cpus: 1,
///     ..Default::default()
/// };
///
/// // Create a default configuration for the 4 cpus. When creating the `Gic` from this
/// // configuration the GIC version will be selected depending on what's supported on the host.
/// let config = GicConfig {
///     num_cpus: 4,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct GicConfig {
    /// Number of IRQs that this GIC supports. The IRQ number can be a number divisible by 32 in
    /// the interval [[MIN_NR_IRQS], 1024].
    pub num_irqs: u32,
    /// Number of CPUs that this GIC supports. This is not used when configuring
    /// a GICv2.
    pub num_cpus: u8,
    /// Version of the GIC. When no version is specified, we try to create a GICv3, and fallback
    /// to GICv2 in case of failure.
    pub version: Option<GicVersion>,
}

impl Default for GicConfig {
    fn default() -> Self {
        GicConfig {
            num_irqs: MIN_NR_IRQS,
            num_cpus: 0,
            version: None,
        }
    }
}

/// Structure used for serializing the state of the GIC registers
#[derive(Clone, Debug)]
pub struct GicState {
    dist: Vec<GicRegState<u32>>,
    gic_vcpu_states: Vec<GicVcpuState>,
}

/// Structure used for serializing the state of the GIC registers for a specific vCPU
#[derive(Clone, Debug)]
pub struct GicVcpuState {
    redist: Vec<GicRegState<u32>>,
    icc: GicSysRegsState,
}

impl Gic {
    /// Create a new GIC based on the passed configuration.
    ///
    /// # Arguments
    /// * `config`: The GIC configuration. More details are available in the
    ///             [`GicConfig`] definition.
    /// * `vm_fd`: A reference to the KVM specific file descriptor. This is used
    ///            for creating and configuring the GIC in KVM.
    /// # Example
    /// ```rust
    /// use kvm_ioctls::Kvm;
    /// use vm_vcpu_ref::aarch64::interrupts::{Gic, GicConfig};
    ///
    /// let kvm = Kvm::new().unwrap();
    /// let vm = kvm.create_vm().unwrap();
    /// let _vcpu = vm.create_vcpu(0).unwrap();
    ///
    /// let gic_config = GicConfig {
    ///     num_cpus: 1,
    ///     ..Default::default()
    /// };
    ///
    /// let gic = Gic::new(gic_config, &vm).unwrap();
    /// let device_fd = gic.device_fd();
    /// ```
    pub fn new(config: GicConfig, vm_fd: &VmFd) -> Result<Gic> {
        let (version, device_fd) = match config.version {
            Some(version) => (version, Gic::create_device(vm_fd, version)?),
            None => {
                // First try to create a GICv3, if that does not work, update the version
                // and try to create a V2 instead.
                let mut version = GicVersion::V3;
                let device_fd = Gic::create_device(vm_fd, GicVersion::V3).or_else(|_| {
                    version = GicVersion::V2;
                    Gic::create_device(vm_fd, GicVersion::V2)
                })?;
                (version, device_fd)
            }
        };

        let mut gic = Gic {
            num_irqs: config.num_irqs,
            num_cpus: config.num_cpus,
            version,
            device_fd,
        };

        gic.configure_device()?;
        Ok(gic)
    }

    // Helper function that sets the required attributes for the device.
    fn configure_device(&mut self) -> Result<()> {
        match self.version {
            GicVersion::V2 => {
                self.set_distr_attr(KVM_VGIC_V2_ADDR_TYPE_DIST as u64)?;
                self.set_cpu_attr()?;
            }
            GicVersion::V3 => {
                self.set_redist_attr()?;
                self.set_distr_attr(KVM_VGIC_V3_ADDR_TYPE_DIST as u64)?;
            }
        }
        self.set_interrupts_attr()?;
        self.finalize_gic()
    }

    // Create the device FD corresponding to the GIC version specified as parameter.
    fn create_device(vm_fd: &VmFd, version: GicVersion) -> Result<DeviceFd> {
        let mut create_device_attr = kvm_create_device {
            type_: version as u32,
            fd: 0,
            flags: 0,
        };
        vm_fd
            .create_device(&mut create_device_attr)
            .map_err(Error::CreateDevice)
    }

    // Helper function for setting the _ADDR_TYPE_DIST. The attribute type depends on the
    // type of GIC, so we are passing it as a parameter so that we don't need to do
    // a match on the version again.
    fn set_distr_attr(&mut self, attr_type: u64) -> Result<()> {
        let dist_if_addr: u64 = AARCH64_GIC_DIST_BASE;
        let raw_dist_if_addr = &dist_if_addr as *const u64;
        let dist_attr = kvm_device_attr {
            group: KVM_DEV_ARM_VGIC_GRP_ADDR,
            addr: raw_dist_if_addr as u64,
            attr: attr_type,
            flags: 0,
        };
        self.device_fd
            .set_device_attr(&dist_attr)
            .map_err(|e| Error::SetAttr("dist", e))
    }

    // Helper function for setting the `KVM_VGIC_V2_ADDR_TYPE_CPU`. This can only be used with
    // VGIC 2. We're not doing a check here because KVM will return an error if used with V3,
    // and this function is private, so calling it with a v3 GIC can only happen if we have
    // a programming error.
    fn set_cpu_attr(&mut self) -> Result<()> {
        let cpu_if_addr: u64 = AARCH64_GIC_CPUI_BASE;
        let raw_cpu_if_addr = &cpu_if_addr as *const u64;
        let cpu_if_attr = kvm_device_attr {
            group: KVM_DEV_ARM_VGIC_GRP_ADDR,
            attr: KVM_VGIC_V2_ADDR_TYPE_CPU as u64,
            addr: raw_cpu_if_addr as u64,
            flags: 0,
        };

        self.device_fd
            .set_device_attr(&cpu_if_attr)
            .map_err(|e| Error::SetAttr("cpu", e))
    }

    fn set_redist_attr(&mut self) -> Result<()> {
        // The following arithmetic operations are safe because the `num_cpus` can be maximum
        // u8::MAX, which multiplied by the `AARCH64_GIC_REDIST_SIZE` results in a number that is
        // an order of maginitude smaller that `AARCH64_GIC_DIST_BASE`.
        let redist_addr: u64 =
            AARCH64_GIC_DIST_BASE - (AARCH64_GIC_REDIST_SIZE * self.num_cpus as u64);
        let raw_redist_addr = &redist_addr as *const u64;
        let redist_attr = kvm_device_attr {
            group: KVM_DEV_ARM_VGIC_GRP_ADDR,
            attr: KVM_VGIC_V3_ADDR_TYPE_REDIST as u64,
            addr: raw_redist_addr as u64,
            flags: 0,
        };

        self.device_fd
            .set_device_attr(&redist_attr)
            .map_err(|e| Error::SetAttr("redist", e))
    }

    fn set_interrupts_attr(&mut self) -> Result<()> {
        let nr_irqs_ptr = &self.num_irqs as *const u32;
        let nr_irqs_attr = kvm_device_attr {
            group: KVM_DEV_ARM_VGIC_GRP_NR_IRQS,
            addr: nr_irqs_ptr as u64,
            ..Default::default()
        };

        self.device_fd
            .set_device_attr(&nr_irqs_attr)
            .map_err(|e| Error::SetAttr("irq", e))
    }

    fn finalize_gic(&mut self) -> Result<()> {
        let init_gic_attr = kvm_device_attr {
            group: KVM_DEV_ARM_VGIC_GRP_CTRL,
            attr: KVM_DEV_ARM_VGIC_CTRL_INIT as u64,
            ..Default::default()
        };

        self.device_fd
            .set_device_attr(&init_gic_attr)
            .map_err(|e| Error::SetAttr("finalize", e))
    }

    /// Return the `DeviceFd` associated with this GIC.
    pub fn device_fd(&self) -> &DeviceFd {
        &self.device_fd
    }

    /// Returns the version of this GIC.
    pub fn version(&self) -> GicVersion {
        self.version
    }

    /// Save the state of this GIC.
    pub fn save_state(&self, vcpu_mpidrs: Vec<u64>) -> Result<GicState> {
        let fd = self.device_fd();

        let kvm_mpidrs = convert_to_kvm_mpidrs(vcpu_mpidrs);

        // Flush redistributors pending tables to guest RAM.
        save_pending_tables(fd)?;

        let mut gic_vcpu_states = Vec::with_capacity(kvm_mpidrs.len());
        for mpidr in kvm_mpidrs {
            gic_vcpu_states.push(GicVcpuState {
                redist: redist_regs(fd, mpidr)?,
                icc: icc_regs(fd, mpidr)?,
            })
        }

        Ok(GicState {
            dist: dist_regs(fd)?,
            gic_vcpu_states,
        })
    }

    /// Restore the state of GIC.
    pub fn restore_state(&self, state: &GicState, vcpu_mpidrs: Vec<u64>) -> Result<()> {
        if vcpu_mpidrs.len() != state.gic_vcpu_states.len() {
            return Err(Error::InconsistentVcpuCount);
        }

        let kvm_mpidrs = convert_to_kvm_mpidrs(vcpu_mpidrs);

        let fd = self.device_fd();
        set_dist_regs(fd, &state.dist)?;

        for (mpidr, vcpu_state) in kvm_mpidrs.iter().zip(&state.gic_vcpu_states) {
            set_redist_regs(fd, &vcpu_state.redist, *mpidr)?;
            set_icc_regs(fd, &vcpu_state.icc, *mpidr)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::aarch64::interrupts::{
        Error, Gic, GicConfig, GicVersion, KVM_DEV_ARM_VGIC_GRP_NR_IRQS, MIN_NR_IRQS,
    };
    use kvm_bindings::kvm_device_attr;
    use kvm_ioctls::{Kvm, VmFd};

    use super::GicState;

    #[test]
    fn test_create_device() {
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        let config = GicConfig {
            num_irqs: MIN_NR_IRQS * 2,
            num_cpus: 4u8,
            ..Default::default()
        };
        let mut vcpus = Vec::new();
        // For the creation of GICv2 to work we need to first create the vcpus.
        for i in 0..config.num_cpus {
            vcpus.push(vm.create_vcpu(i as u64).unwrap());
        }

        let gic = Gic::new(config.clone(), &vm).unwrap();
        assert_eq!(gic.num_irqs, config.num_irqs);
        assert_eq!(gic.num_cpus, config.num_cpus);

        // Retrieve the number of IRQs as defined in the device to check the setters in builder.
        // This value should be saved in the address provided to the ioctl.
        let mut data: u32 = 0;
        let mut nr_irqs_attr = kvm_device_attr {
            group: KVM_DEV_ARM_VGIC_GRP_NR_IRQS,
            addr: &mut data as *const u32 as u64,
            ..Default::default()
        };
        gic.device_fd().get_device_attr(&mut nr_irqs_attr).unwrap();
        assert_eq!(data, config.num_irqs);
    }

    #[test]
    fn test_restore_with_invalid_vcpu_count() {
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        let config = GicConfig {
            ..Default::default()
        };
        let gic = Gic::new(config, &vm).unwrap();

        // This is a completely synthetic test that does not expect
        // any real data; only the lengths of mpidrs and gic_vcpu_states
        // are compared.
        let dummy_gic_state = GicState {
            dist: vec![],
            // Zero vCPUs.
            gic_vcpu_states: vec![],
        };
        // One vCPU.
        let mpidrs = vec![1];
        let res = gic.restore_state(&dummy_gic_state, mpidrs);
        assert_eq!(res, Err(Error::InconsistentVcpuCount));
    }

    #[test]
    fn test_create_failed() {
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        let _vcpu = vm.create_vcpu(0).unwrap();
        let config = GicConfig {
            num_cpus: 1,
            // This number of IRQs is invalid because it's less than the minimum supported
            // (< AARCH64_GIC_NR_IRQS)
            num_irqs: 63,
            ..Default::default()
        };

        assert_eq!(
            Gic::new(config, &vm).unwrap_err(),
            Error::SetAttr("irq", kvm_ioctls::Error::new(22))
        );

        // We cannot check that setting a wrong number of vCPUs fails because in the case of
        // V2 GIC the number of vCPUs is not used for setting the attributes.
    }

    // Helper function that tries to create a device with the version specified as a parameter.
    // This is needed because we need to check if the creation fails due to the GIC emulation
    // not being available on the host we're running.
    fn checked_create_device(vm: &mut VmFd, version: GicVersion) {
        let gic_config = GicConfig {
            version: Some(version),
            num_cpus: 1,
            ..Default::default()
        };
        match Gic::new(gic_config, vm) {
            Ok(gic) => {
                assert_eq!(gic.version(), version);
            }
            Err(Error::CreateDevice(e)) if e.errno() == 19 => {
                // When the host supports GICv3, it does not necessarily also support GICv2.
                // So that the tests don't fail in the CI we need to just exit the test.
                // KVM will return erno 19 (No Such Device) in that case.
            }
            Err(e) => {
                panic!("Unexpected error: {:#?}", e);
            }
        };
    }

    #[test]
    fn test_create_with_version() {
        let kvm = Kvm::new().unwrap();
        let mut vm = kvm.create_vm().unwrap();
        // For the creation of GICv2 to work we need to first create the vcpus.
        vm.create_vcpu(0).unwrap();

        checked_create_device(&mut vm, GicVersion::V2);
        checked_create_device(&mut vm, GicVersion::V3);
    }

    #[test]
    fn test_create_with_max_values() {
        {
            let kvm = Kvm::new().unwrap();
            let vm = kvm.create_vm().unwrap();
            // For the creation of GICv2 to work we need to first create the vcpus.
            vm.create_vcpu(0).unwrap();
            let gic_config = GicConfig {
                num_cpus: u8::MAX,
                ..Default::default()
            };

            assert!(Gic::new(gic_config, &vm).is_ok());
        }

        {
            let kvm = Kvm::new().unwrap();
            let vm = kvm.create_vm().unwrap();
            // For the creation of GICv2 to work we need to first create the vcpus.
            vm.create_vcpu(0).unwrap();
            let gic_config = GicConfig {
                num_irqs: u32::MAX,
                ..Default::default()
            };
            // This fails because the maximum number of IRQs is 1024.
            assert_eq!(
                Gic::new(gic_config, &vm).unwrap_err(),
                Error::SetAttr("irq", kvm_ioctls::Error::new(22))
            );
        }
    }
}
