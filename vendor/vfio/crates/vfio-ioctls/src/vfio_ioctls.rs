// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CStr;
use std::fs::File;
use std::mem::size_of;
use std::os::unix::io::AsRawFd;

use vfio_bindings::bindings::vfio::*;
use vmm_sys_util::errno::Error as SysError;

use crate::vfio_device::{vfio_region_info_with_cap, VfioDeviceInfo};
use crate::{Result, VfioContainer, VfioDevice, VfioError, VfioGroup};

ioctl_io_nr!(VFIO_GET_API_VERSION, VFIO_TYPE, VFIO_BASE);
ioctl_io_nr!(VFIO_CHECK_EXTENSION, VFIO_TYPE, VFIO_BASE + 1);
ioctl_io_nr!(VFIO_SET_IOMMU, VFIO_TYPE, VFIO_BASE + 2);
ioctl_io_nr!(VFIO_GROUP_GET_STATUS, VFIO_TYPE, VFIO_BASE + 3);
ioctl_io_nr!(VFIO_GROUP_SET_CONTAINER, VFIO_TYPE, VFIO_BASE + 4);
ioctl_io_nr!(VFIO_GROUP_UNSET_CONTAINER, VFIO_TYPE, VFIO_BASE + 5);
ioctl_io_nr!(VFIO_GROUP_GET_DEVICE_FD, VFIO_TYPE, VFIO_BASE + 6);
ioctl_io_nr!(VFIO_DEVICE_GET_INFO, VFIO_TYPE, VFIO_BASE + 7);
ioctl_io_nr!(VFIO_DEVICE_GET_REGION_INFO, VFIO_TYPE, VFIO_BASE + 8);
ioctl_io_nr!(VFIO_DEVICE_GET_IRQ_INFO, VFIO_TYPE, VFIO_BASE + 9);
ioctl_io_nr!(VFIO_DEVICE_SET_IRQS, VFIO_TYPE, VFIO_BASE + 10);
ioctl_io_nr!(VFIO_DEVICE_RESET, VFIO_TYPE, VFIO_BASE + 11);
ioctl_io_nr!(
    VFIO_DEVICE_GET_PCI_HOT_RESET_INFO,
    VFIO_TYPE,
    VFIO_BASE + 12
);
ioctl_io_nr!(VFIO_DEVICE_PCI_HOT_RESET, VFIO_TYPE, VFIO_BASE + 13);
ioctl_io_nr!(VFIO_DEVICE_QUERY_GFX_PLANE, VFIO_TYPE, VFIO_BASE + 14);
ioctl_io_nr!(VFIO_DEVICE_GET_GFX_DMABUF, VFIO_TYPE, VFIO_BASE + 15);
ioctl_io_nr!(VFIO_DEVICE_IOEVENTFD, VFIO_TYPE, VFIO_BASE + 16);
ioctl_io_nr!(VFIO_IOMMU_GET_INFO, VFIO_TYPE, VFIO_BASE + 12);
ioctl_io_nr!(VFIO_IOMMU_MAP_DMA, VFIO_TYPE, VFIO_BASE + 13);
ioctl_io_nr!(VFIO_IOMMU_UNMAP_DMA, VFIO_TYPE, VFIO_BASE + 14);
ioctl_io_nr!(VFIO_IOMMU_ENABLE, VFIO_TYPE, VFIO_BASE + 15);
ioctl_io_nr!(VFIO_IOMMU_DISABLE, VFIO_TYPE, VFIO_BASE + 16);

#[cfg(not(test))]
// Safety:
// - absolutely trust the underlying kernel
// - absolutely trust data returned by the underlying kernel
// - assume kernel will return error if caller passes in invalid file handle, parameter or buffer.
pub(crate) mod vfio_syscall {
    use super::*;
    use std::os::unix::io::FromRawFd;
    use vmm_sys_util::ioctl::{
        ioctl, ioctl_with_mut_ref, ioctl_with_ptr, ioctl_with_ref, ioctl_with_val,
    };

    pub(crate) fn check_api_version(container: &VfioContainer) -> i32 {
        // SAFETY: file is vfio container fd and ioctl is defined by kernel.
        unsafe { ioctl(container, VFIO_GET_API_VERSION()) }
    }

    pub(crate) fn check_extension(container: &VfioContainer, val: u32) -> Result<u32> {
        // SAFETY: file is vfio container and make sure val is valid.
        let ret = unsafe { ioctl_with_val(container, VFIO_CHECK_EXTENSION(), val.into()) };
        if ret < 0 {
            Err(VfioError::VfioExtension)
        } else {
            Ok(ret as u32)
        }
    }

    pub(crate) fn set_iommu(container: &VfioContainer, val: u32) -> Result<()> {
        // SAFETY: file is vfio container and make sure val is valid.
        let ret = unsafe { ioctl_with_val(container, VFIO_SET_IOMMU(), val.into()) };
        if ret < 0 {
            Err(VfioError::ContainerSetIOMMU(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn map_dma(
        container: &VfioContainer,
        dma_map: &vfio_iommu_type1_dma_map,
    ) -> Result<()> {
        // SAFETY: file is vfio container, dma_map is constructed by us, and
        // we check the return value
        let ret = unsafe { ioctl_with_ref(container, VFIO_IOMMU_MAP_DMA(), dma_map) };
        if ret != 0 {
            Err(VfioError::IommuDmaMap(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn unmap_dma(
        container: &VfioContainer,
        dma_map: &mut vfio_iommu_type1_dma_unmap,
    ) -> Result<()> {
        // SAFETY: file is vfio container, dma_unmap is constructed by us, and
        // we check the return value
        let ret = unsafe { ioctl_with_ref(container, VFIO_IOMMU_UNMAP_DMA(), dma_map) };
        if ret != 0 {
            Err(VfioError::IommuDmaUnmap(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_group_status(
        file: &File,
        group_status: &mut vfio_group_status,
    ) -> Result<()> {
        // SAFETY: we are the owner of group and group_status which are valid value.
        let ret = unsafe { ioctl_with_mut_ref(file, VFIO_GROUP_GET_STATUS(), group_status) };
        if ret < 0 {
            Err(VfioError::GetGroupStatus)
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_group_device_fd(group: &VfioGroup, path: &CStr) -> Result<File> {
        // SAFETY: we are the owner of self and path_ptr which are valid value.
        let fd = unsafe { ioctl_with_ptr(group, VFIO_GROUP_GET_DEVICE_FD(), path.as_ptr()) };
        if fd < 0 {
            Err(VfioError::GroupGetDeviceFD(SysError::last()))
        } else {
            // SAFETY: fd is valid FD
            Ok(unsafe { File::from_raw_fd(fd) })
        }
    }

    pub(crate) fn set_group_container(group: &VfioGroup, container: &VfioContainer) -> Result<()> {
        let container_raw_fd = container.as_raw_fd();
        // SAFETY: we are the owner of group and container_raw_fd which are valid value,
        // and we verify the ret value
        let ret = unsafe { ioctl_with_ref(group, VFIO_GROUP_SET_CONTAINER(), &container_raw_fd) };
        if ret < 0 {
            Err(VfioError::GroupSetContainer)
        } else {
            Ok(())
        }
    }

    pub(crate) fn unset_group_container(
        group: &VfioGroup,
        container: &VfioContainer,
    ) -> Result<()> {
        let container_raw_fd = container.as_raw_fd();
        // SAFETY: we are the owner of self and container_raw_fd which are valid value.
        let ret = unsafe { ioctl_with_ref(group, VFIO_GROUP_UNSET_CONTAINER(), &container_raw_fd) };
        if ret < 0 {
            Err(VfioError::GroupSetContainer)
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_device_info(file: &File, dev_info: &mut vfio_device_info) -> Result<()> {
        // SAFETY: we are the owner of dev and dev_info which are valid value,
        // and we verify the return value.
        let ret = unsafe { ioctl_with_mut_ref(file, VFIO_DEVICE_GET_INFO(), dev_info) };
        if ret < 0 {
            Err(VfioError::VfioDeviceGetInfo(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn set_device_irqs(device: &VfioDevice, irq_set: &[vfio_irq_set]) -> Result<()> {
        if irq_set.is_empty() || irq_set[0].argsz as usize > std::mem::size_of_val(irq_set) {
            Err(VfioError::VfioDeviceSetIrq)
        } else {
            // SAFETY: we are the owner of self and irq_set which are valid value
            let ret = unsafe { ioctl_with_ref(device, VFIO_DEVICE_SET_IRQS(), &irq_set[0]) };
            if ret < 0 {
                Err(VfioError::VfioDeviceSetIrq)
            } else {
                Ok(())
            }
        }
    }

    pub(crate) fn reset(device: &VfioDevice) -> i32 {
        // SAFETY: file is vfio device
        unsafe { ioctl(device, VFIO_DEVICE_RESET()) }
    }

    pub(crate) fn get_device_irq_info(
        dev_info: &VfioDeviceInfo,
        irq_info: &mut vfio_irq_info,
    ) -> Result<()> {
        // SAFETY: we are the owner of dev and irq_info which are valid value
        let ret = unsafe { ioctl_with_mut_ref(dev_info, VFIO_DEVICE_GET_IRQ_INFO(), irq_info) };
        if ret < 0 {
            Err(VfioError::VfioDeviceGetRegionInfo(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_device_region_info(
        dev_info: &VfioDeviceInfo,
        reg_info: &mut vfio_region_info,
    ) -> Result<()> {
        // SAFETY: we are the owner of dev and region_info which are valid value
        // and we verify the return value.
        let ret = unsafe { ioctl_with_mut_ref(dev_info, VFIO_DEVICE_GET_REGION_INFO(), reg_info) };
        if ret < 0 {
            Err(VfioError::VfioDeviceGetRegionInfo(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_device_region_info_cap(
        dev_info: &VfioDeviceInfo,
        reg_infos: &mut [vfio_region_info_with_cap],
    ) -> Result<()> {
        if reg_infos.is_empty()
            || reg_infos[0].region_info.argsz as usize
                > reg_infos.len() * size_of::<vfio_region_info>()
        {
            Err(VfioError::VfioDeviceGetRegionInfo(SysError::new(
                libc::EINVAL,
            )))
        } else {
            // SAFETY: we are the owner of dev and region_info which are valid value,
            // and we verify the return value.
            let ret = unsafe {
                ioctl_with_mut_ref(dev_info, VFIO_DEVICE_GET_REGION_INFO(), &mut reg_infos[0])
            };
            if ret < 0 {
                Err(VfioError::VfioDeviceGetRegionInfo(SysError::last()))
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod vfio_syscall {
    use super::*;
    use vfio_bindings::bindings::vfio::{vfio_device_info, VFIO_IRQ_INFO_EVENTFD};
    use vmm_sys_util::tempfile::TempFile;

    pub(crate) fn check_api_version(_container: &VfioContainer) -> i32 {
        VFIO_API_VERSION as i32
    }

    pub(crate) fn check_extension(_container: &VfioContainer, val: u32) -> Result<u32> {
        if val == VFIO_TYPE1v2_IOMMU {
            Ok(1)
        } else {
            Err(VfioError::VfioExtension)
        }
    }

    pub(crate) fn set_iommu(_container: &VfioContainer, _val: u32) -> Result<()> {
        Ok(())
    }

    pub(crate) fn map_dma(
        _container: &VfioContainer,
        dma_map: &vfio_iommu_type1_dma_map,
    ) -> Result<()> {
        if dma_map.iova == 0x1000 {
            Ok(())
        } else {
            Err(VfioError::IommuDmaMap(SysError::last()))
        }
    }

    pub(crate) fn unmap_dma(
        _container: &VfioContainer,
        dma_map: &mut vfio_iommu_type1_dma_unmap,
    ) -> Result<()> {
        if dma_map.iova == 0x1000 {
            if dma_map.size == 0x2000 {
                dma_map.size = 0x1000;
            }
            Ok(())
        } else {
            Err(VfioError::IommuDmaUnmap(SysError::last()))
        }
    }

    pub(crate) fn get_group_status(
        _file: &File,
        group_status: &mut vfio_group_status,
    ) -> Result<()> {
        group_status.flags = VFIO_GROUP_FLAGS_VIABLE;
        Ok(())
    }

    pub(crate) fn get_group_device_fd(_group: &VfioGroup, _path: &CStr) -> Result<File> {
        let tmp_file = TempFile::new().unwrap();
        let device = File::open(tmp_file.as_path()).unwrap();

        Ok(device)
    }

    pub(crate) fn set_group_container(group: &VfioGroup, container: &VfioContainer) -> Result<()> {
        if group.as_raw_fd() >= 0 && container.as_raw_fd() >= 0 {
            Ok(())
        } else {
            Err(VfioError::GroupSetContainer)
        }
    }

    pub(crate) fn unset_group_container(
        group: &VfioGroup,
        container: &VfioContainer,
    ) -> Result<()> {
        if group.as_raw_fd() >= 0 && container.as_raw_fd() >= 0 {
            Ok(())
        } else {
            Err(VfioError::GroupSetContainer)
        }
    }

    pub(crate) fn get_device_info(_file: &File, dev_info: &mut vfio_device_info) -> Result<()> {
        dev_info.flags = VFIO_DEVICE_FLAGS_PCI;
        dev_info.num_regions = VFIO_PCI_NUM_REGIONS;
        dev_info.num_irqs = VFIO_PCI_MSIX_IRQ_INDEX + 1;
        Ok(())
    }

    #[allow(clippy::if_same_then_else)]
    pub(crate) fn set_device_irqs(_device: &VfioDevice, irq_sets: &[vfio_irq_set]) -> Result<()> {
        if irq_sets.is_empty() || irq_sets[0].argsz as usize > std::mem::size_of_val(irq_sets) {
            Err(VfioError::VfioDeviceSetIrq)
        } else {
            let irq_set = &irq_sets[0];
            if irq_set.flags == VFIO_IRQ_SET_DATA_EVENTFD | VFIO_IRQ_SET_ACTION_TRIGGER
                && irq_set.index == 0
                && irq_set.count == 0
            {
                Err(VfioError::VfioDeviceSetIrq)
            } else if irq_set.flags == VFIO_IRQ_SET_DATA_NONE | VFIO_IRQ_SET_ACTION_TRIGGER
                && irq_set.index == 0
                && irq_set.count == 0
            {
                Err(VfioError::VfioDeviceSetIrq)
            } else if irq_set.flags == VFIO_IRQ_SET_DATA_NONE | VFIO_IRQ_SET_ACTION_UNMASK
                && irq_set.index == 1
                && irq_set.count == 1
            {
                Err(VfioError::VfioDeviceSetIrq)
            } else {
                Ok(())
            }
        }
    }

    pub(crate) fn reset(_device: &VfioDevice) -> i32 {
        0
    }

    pub(crate) fn get_device_region_info(
        _dev_info: &VfioDeviceInfo,
        reg_info: &mut vfio_region_info,
    ) -> Result<()> {
        match reg_info.index {
            0 => {
                reg_info.flags = 0;
                reg_info.size = 0x1000;
                reg_info.offset = 0x10000;
            }
            1 => {
                reg_info.argsz = 88;
                reg_info.flags = VFIO_REGION_INFO_FLAG_CAPS;
                reg_info.size = 0x2000;
                reg_info.offset = 0x20000;
            }
            idx if idx == VFIO_PCI_VGA_REGION_INDEX => {
                return Err(VfioError::VfioDeviceGetRegionInfo(SysError::new(
                    libc::EINVAL,
                )))
            }
            idx if (2..VFIO_PCI_NUM_REGIONS).contains(&idx) => {
                reg_info.flags = 0;
                reg_info.size = (idx as u64 + 1) * 0x1000;
                reg_info.offset = (idx as u64 + 1) * 0x10000;
            }
            idx if idx == VFIO_PCI_NUM_REGIONS => {
                return Err(VfioError::VfioDeviceGetRegionInfo(SysError::new(
                    libc::EINVAL,
                )))
            }
            _ => panic!("invalid device region index"),
        }

        Ok(())
    }

    pub(crate) fn get_device_region_info_cap(
        _dev_info: &VfioDeviceInfo,
        reg_infos: &mut [vfio_region_info_with_cap],
    ) -> Result<()> {
        if reg_infos.is_empty()
            || reg_infos[0].region_info.argsz as usize
                > reg_infos.len() * size_of::<vfio_region_info>()
        {
            return Err(VfioError::VfioDeviceGetRegionInfo(SysError::new(
                libc::EINVAL,
            )));
        }

        let reg_info = &mut reg_infos[0];
        match reg_info.region_info.index {
            1 => {
                reg_info.region_info.cap_offset = 32;
                // SAFETY: data structure returned by kernel is trusted.
                let header = unsafe {
                    &mut *((reg_info as *mut vfio_region_info_with_cap as *mut u8).add(32)
                        as *mut vfio_info_cap_header)
                };
                header.id = VFIO_REGION_INFO_CAP_MSIX_MAPPABLE as u16;
                header.next = 40;

                // SAFETY: data structure returned by kernel is trusted.
                let header = unsafe {
                    &mut *((header as *mut vfio_info_cap_header as *mut u8).add(8)
                        as *mut vfio_region_info_cap_type)
                };
                header.header.id = VFIO_REGION_INFO_CAP_TYPE as u16;
                header.header.next = 56;
                header.type_ = 0x5;
                header.subtype = 0x6;

                // SAFETY: data structure returned by kernel is trusted.
                let header = unsafe {
                    &mut *((header as *mut vfio_region_info_cap_type as *mut u8).add(16)
                        as *mut vfio_region_info_cap_sparse_mmap)
                };
                header.header.id = VFIO_REGION_INFO_CAP_SPARSE_MMAP as u16;
                header.header.next = 4;
                header.nr_areas = 1;

                // SAFETY: data structure returned by kernel is trusted.
                let mmap = unsafe {
                    &mut *((header as *mut vfio_region_info_cap_sparse_mmap as *mut u8).add(16)
                        as *mut vfio_region_sparse_mmap_area)
                };
                mmap.size = 0x3;
                mmap.offset = 0x4;
            }
            _ => panic!("invalid device region index"),
        }

        Ok(())
    }

    pub(crate) fn get_device_irq_info(
        _dev_info: &VfioDeviceInfo,
        irq_info: &mut vfio_irq_info,
    ) -> Result<()> {
        match irq_info.index {
            0 => {
                irq_info.flags = VFIO_IRQ_INFO_MASKABLE;
                irq_info.count = 1;
            }
            1 => {
                irq_info.flags = VFIO_IRQ_INFO_EVENTFD;
                irq_info.count = 32;
            }
            2 => {
                irq_info.flags = VFIO_IRQ_INFO_EVENTFD;
                irq_info.count = 2048;
            }
            3 => {
                return Err(VfioError::VfioDeviceGetRegionInfo(SysError::new(
                    libc::EINVAL,
                )))
            }
            _ => panic!("invalid device irq index"),
        }

        Ok(())
    }

    pub(crate) fn create_dev_info_for_test() -> vfio_device_info {
        vfio_device_info {
            argsz: 0,
            flags: 0,
            num_regions: 2,
            num_irqs: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfio_ioctl_code() {
        assert_eq!(VFIO_GET_API_VERSION(), 15204);
        assert_eq!(VFIO_CHECK_EXTENSION(), 15205);
        assert_eq!(VFIO_SET_IOMMU(), 15206);
        assert_eq!(VFIO_GROUP_GET_STATUS(), 15207);
        assert_eq!(VFIO_GROUP_SET_CONTAINER(), 15208);
        assert_eq!(VFIO_GROUP_UNSET_CONTAINER(), 15209);
        assert_eq!(VFIO_GROUP_GET_DEVICE_FD(), 15210);
        assert_eq!(VFIO_DEVICE_GET_INFO(), 15211);
        assert_eq!(VFIO_DEVICE_GET_REGION_INFO(), 15212);
        assert_eq!(VFIO_DEVICE_GET_IRQ_INFO(), 15213);
        assert_eq!(VFIO_DEVICE_SET_IRQS(), 15214);
        assert_eq!(VFIO_DEVICE_RESET(), 15215);
        assert_eq!(VFIO_DEVICE_IOEVENTFD(), 15220);
        assert_eq!(VFIO_IOMMU_DISABLE(), 15220);
    }
}