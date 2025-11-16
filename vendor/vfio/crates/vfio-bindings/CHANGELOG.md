# [v0.3.1]

- Update repository to https://github.com/rust-vmm/vfio

# [v0.3.0]

## Added

- Update vmm-sys-util version to ">=0.8.0"

# [v0.2.0]

## Added

- Add FAM wrappers for vfio\_irq\_set
- Update vmm-sys-util version to ">=0.2.0"

# [v0.1.0]

This is the first `vfio-bindings` crate release.

This crate provides Rust FFI bindings to the
[Virtual Function I/O (VFIO)](https://www.kernel.org/doc/Documentation/vfio.txt)
Linux kernel API. With this first release, the bindings are for the Linux kernel
version 5.0.

The bindings are generated using [bindgen](https://crates.io/crates/bindgen).
