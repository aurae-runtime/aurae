# vfio-ioctls

## Design

The [VFIO driver framework](https://www.kernel.org/doc/Documentation/vfio.txt)
provides unified APIs for direct device access. It is an IOMMU/device-agnostic framework for
exposing direct device access to user space in a secure, IOMMU-protected environment.
This framework is used for multiple devices, such as GPUs, network adapters, and compute
accelerators. With direct device access, virtual machines or user space applications have
direct access to the physical device.

The VFIO framework is originally developed on Linux system, and later Microsoft HyperVisor
technology provides a compatible implementation. Therefore the VFIO framework is supported
by both Linux and Microsoft HyperVisor.

The `vfio-ioctls` crate is a safe wrapper over the VFIO APIs. It provides three classes of structs:
- `VfioContainer`: a safe wrapper over a VFIO container object, and acts a container object
  to associate `VfioDevice` objects with IOMMU domains.
- `VfioDevice`: a wrapper over a VFIO device object, provide methods to access the underlying
  hardware device.
- `VfioIrq/VfioRegion`: describes capabilities/resources about a `VfioDevice` object. 

## Usage

The `vfio-ioctls` crate may be used to support following usage scenarios:
- Direct device assignment to virtual machine based on Linux KVM, with default features.
- Direct device assignment to virtual machine based on Microsoft HyperVisor, with `--no-default-features --features=mshv`.
- User mode device drivers, with `--no-default-features`.

First, add the following to your Cargo.toml:
```toml
vfio-ioctls = "0.1"
```
Next, add this to your crate root:

```rust
extern crate vfio_ioctls;
```

By default vfio-ioctls has the `kvm` feature enabled. You may turn off the default features by
`default-features = false`. To enable feature `mshv`,
```toml
vfio-ioctls = { version = "0.1", default-features = false, features = ["mshv"]}
```


## Examples

To create VFIO device object for user mode drivers,

```rust
use std::sync::Arc;
use vfio_ioctls::{VfioContainer, VfioDevice};

fn create_vfio_device() {
  // TODO: change to your device's path
  let device_path = "/sys/bus/pci/devices/00:03.0";
  let vfio_container = Arc::new(VfioContainer::new(()).unwrap());
  let vfio_dev = VfioDevice::new(&Path::new(device_path), vfio_container.clone()).unwrap();
  let irqs = vfio_dev.max_interrupts();

  assert!(irqs > 0);
}
```

## License

This code is licensed under Apache-2.0 or BSD-3-Clause.
