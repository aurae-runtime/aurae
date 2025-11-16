# Safe wrappers for VFIO

## Design

This repository provides safe wrappers over the
[VFIO driver framework](https://www.kernel.org/doc/Documentation/vfio.txt).

Many modern systems now provide DMA and interrupt remapping facilities to help ensure I/O devices behave within the boundaries they’ve been allotted. This includes x86 hardware with AMD-Vi and Intel VT-d, POWER systems with Partitionable Endpoints (PEs) and embedded PowerPC systems such as Freescale PAMU. The VFIO driver is an IOMMU/device agnostic framework for exposing direct device access to userspace, in a secure, IOMMU protected environment. In other words, the VFIO framework allows safe, non-privileged, userspace drivers.

Why do we want that? Virtual machines often make use of direct device access (“device assignment”) when configured for the highest possible I/O performance. From a device and host perspective, this simply turns the VM into a userspace driver, with the benefits of significantly reduced latency, higher bandwidth, and direct use of bare-metal device drivers.

Devices are the main target of any I/O driver. Devices typically create a programming interface made up of I/O accesses, interrupts, and DMA. Without going into the details of each of these, DMA is by far the most critical aspect for maintaining a secure environment as allowing a device read-write access to system memory imposes the greatest risk to the overall system integrity.

To help mitigate this risk, many modern IOMMUs now incorporate isolation properties into what was, in many cases, an interface only meant for translation (ie. solving the addressing problems of devices with limited address spaces). With this, devices can now be isolated from each other and from arbitrary memory access, thus allowing things like secure direct assignment of devices into virtual machines.

While for the most part an IOMMU may have device level granularity, any system is susceptible to expose a reduced granularity. The IOMMU API therefore supports a notion of IOMMU groups. A group is a set of devices which is isolated from all other devices in the system. Groups are therefore the unit of ownership used by VFIO.

While the group is the minimum granularity that must be used to ensure secure user access, it’s not necessarily the preferred granularity. In IOMMUs which make use of page tables, it may be possible to share a set of page tables between different groups, reducing the overhead both to the platform (reduced TLB thrashing, reduced duplicate page tables), and to the user (programming only a single set of translations). For this reason, VFIO makes use of a container class, which may hold one or more groups. A container is created by simply opening the /dev/vfio/vfio character device.

## Usage
This repository provides two crates to use the VFIO framework, please refer to crate documentations for detail information.
- [vfio-bindings](https://github.com/rust-vmm/vfio/tree/main/crates/vfio-bindings): a rust FFI bindings to VFIO generated using [bindgen](https://crates.io/crates/bindgen).
- [vfio-ioctls](https://github.com/rust-vmm/vfio/tree/main/crates/vfio-ioctls): a group of safe wrappers over the [VFIO APIs](https://github.com/torvalds/linux/blob/master/include/uapi/linux/vfio.h).

## License

This code is licensed under Apache-2.0 or BSD-3-Clause.
