// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

//! [Virtual Function I/O (VFIO) API](https://www.kernel.org/doc/Documentation/vfio.txt)
//!
//! Many modern system now provide DMA and interrupt remapping facilities to help ensure I/O
//! devices behave within the boundaries they've been allotted. This includes x86 hardware with
//! AMD-Vi and Intel VT-d, POWER systems with Partitionable Endpoints (PEs) and embedded PowerPC
//! systems such as Freescale PAMU. The VFIO driver is an IOMMU/device agnostic framework for
//! exposing direct device access to userspace, in a secure, IOMMU protected environment.
//! In other words, the VFIO framework allows safe, non-privileged, userspace drivers.
//!
//! Why do we want that?  Virtual machines often make use of direct device access ("device
//! assignment") when configured for the highest possible I/O performance. From a device and host
//! perspective, this simply turns the VM into a userspace driver, with the benefits of
//! significantly reduced latency, higher bandwidth, and direct use of bare-metal device drivers.
//!
//! Devices are the main target of any I/O driver.  Devices typically create a programming
//! interface made up of I/O access, interrupts, and DMA.  Without going into the details of each
//! of these, DMA is by far the most critical aspect for maintaining a secure environment as
//! allowing a device read-write access to system memory imposes the greatest risk to the overall
//! system integrity.
//!
//! To help mitigate this risk, many modern IOMMUs now incorporate isolation properties into what
//! was, in many cases, an interface only meant for translation (ie. solving the addressing
//! problems of devices with limited address spaces).  With this, devices can now be isolated
//! from each other and from arbitrary memory access, thus allowing things like secure direct
//! assignment of devices into virtual machines.
//!
//! While for the most part an IOMMU may have device level granularity, any system is susceptible
//! to reduced granularity. The IOMMU API therefore supports a notion of IOMMU groups. A group is
//! a set of devices which is isolatable from all other devices in the system. Groups are therefore
//! the unit of ownership used by VFIO.
//!
//! While the group is the minimum granularity that must be used to ensure secure user access, it's
//! not necessarily the preferred granularity. In IOMMUs which make use of page tables, it may be
//! possible to share a set of page tables between different groups, reducing the overhead both to
//! the platform (reduced TLB thrashing, reduced duplicate page tables), and to the user
//! (programming only a single set of translations). For this reason, VFIO makes use of a container
//! class, which may hold one or more groups. A container is created by simply opening the
//! /dev/vfio/vfio character device.
//!
//! This crate is a safe wrapper around the Linux kernel's VFIO interfaces, which offering safe
//! wrappers for:
//! - [VFIO Container](struct.VfioContainer.html) using the `VfioContainer` structure
//! - [VFIO Device](struct.VfioDevice.html) using the `VfioDevice` structure
//!
//! # Platform support
//!
//! - x86_64
//!
//! **NOTE:** The list of available ioctls is not exhaustive.

#![deny(missing_docs)]

#[macro_use]
extern crate vmm_sys_util;

use std::io;
use thiserror::Error;
use vmm_sys_util::errno::Error as SysError;

mod fam;
mod vfio_device;
mod vfio_ioctls;

pub use vfio_device::{
    VfioContainer, VfioDevice, VfioDeviceFd, VfioGroup, VfioIrq, VfioRegion, VfioRegionInfoCap,
    VfioRegionInfoCapNvlink2Lnkspd, VfioRegionInfoCapNvlink2Ssatgt, VfioRegionInfoCapSparseMmap,
    VfioRegionInfoCapType, VfioRegionSparseMmapArea,
};

/// Error codes for VFIO operations.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum VfioError {
    #[error("failed to open /dev/vfio/vfio container: {0}")]
    OpenContainer(#[source] io::Error),
    #[error("failed to open /dev/vfio/{1} group: {0}")]
    OpenGroup(#[source] io::Error, String),
    #[error("failed to get Group Status")]
    GetGroupStatus,
    #[error("group is not viable")]
    GroupViable,
    #[error("vfio API version doesn't match with VFIO_API_VERSION defined in vfio-bindings")]
    VfioApiVersion,
    #[error("failed to check VFIO extension")]
    VfioExtension,
    #[error("invalid VFIO type")]
    VfioInvalidType,
    #[error("container doesn't support VfioType1V2 IOMMU driver type")]
    VfioType1V2,
    #[error("failed to add vfio group into vfio container")]
    GroupSetContainer,
    #[error("failed to unset vfio container")]
    UnsetContainer,
    #[error("failed to set container's IOMMU driver type as VfioType1V2: {0}")]
    ContainerSetIOMMU(#[source] SysError),
    #[error("failed to get vfio device fd: {0}")]
    GroupGetDeviceFD(#[source] SysError),
    #[error("failed to set vfio device's attribute: {0}")]
    SetDeviceAttr(#[source] SysError),
    #[error("failed to get vfio device's info: {0}")]
    VfioDeviceGetInfo(#[source] SysError),
    #[error("vfio PCI device info doesn't match")]
    VfioDeviceGetInfoPCI,
    #[error("unsupported vfio device type")]
    VfioDeviceGetInfoOther,
    #[error("failed to get vfio device's region info: {0}")]
    VfioDeviceGetRegionInfo(#[source] SysError),
    #[error("invalid file path")]
    InvalidPath,
    #[error("failed to add guest memory map into iommu table: {0}")]
    IommuDmaMap(#[source] SysError),
    #[error("failed to remove guest memory map from iommu table: {0}")]
    IommuDmaUnmap(#[source] SysError),
    #[error("failed to get vfio device irq info")]
    VfioDeviceGetIrqInfo,
    #[error("failed to set vfio device irq")]
    VfioDeviceSetIrq,
    #[error("failed to enable vfio device irq")]
    VfioDeviceEnableIrq,
    #[error("failed to disable vfio device irq")]
    VfioDeviceDisableIrq,
    #[error("failed to unmask vfio device irq")]
    VfioDeviceUnmaskIrq,
    #[error("failed to trigger vfio device irq")]
    VfioDeviceTriggerIrq,
    #[error("failed to set vfio device irq resample fd")]
    VfioDeviceSetIrqResampleFd,
    #[error("failed to duplicate fd")]
    VfioDeviceDupFd,
    #[error("wrong device fd type")]
    VfioDeviceFdWrongType,
    #[error("failed to get host address")]
    GetHostAddress,
    #[error("invalid dma unmap size")]
    InvalidDmaUnmapSize,
}

/// Specialized version of `Result` for VFIO subsystem.
pub type Result<T> = std::result::Result<T, VfioError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_vfio_error_fmt() {
        let e = VfioError::GetGroupStatus;
        let e2 = VfioError::OpenContainer(std::io::Error::from(std::io::ErrorKind::Other));
        let str = format!("{}", e);

        assert_eq!(&str, "failed to get Group Status");
        assert!(e2.source().is_some());
        assert!(e.source().is_none());
    }
}