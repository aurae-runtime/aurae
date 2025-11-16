# vfio-bindings

## Design

The vfio-bindings crate is designed as rust FFI bindings to vfio
generated using [bindgen](https://crates.io/crates/bindgen).

Multiple Linux versions are supported through rust 'features'. For each
supported Linux version, a feature is introduced.

Currently supported features/Linux versions:
- vfio-v5_0_0 contains the bindings for the Linux kernel version 5.0

## Usage

First, add the following to your Cargo.toml:
```toml
vfio-bindings = "0.3"
```
Next, add this to your crate root:

```rust
extern crate vfio_bindings;
```

By default vfio-bindings will export a wrapper over the latest available kernel
version it supported, but you can select a different version by specifying it in
your Cargo.toml:
```toml
vfio-bindings = { version = "0.3", features = ["vfio-v5_0_0"]}
```

## Examples

To use this bindings, you can do:
```rust
use vfio_bindings::bindings::vfio::*;
```

## License

This code is licensed under Apache-2.0 or BSD-3-Clause.
