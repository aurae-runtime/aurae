# Contributing to vfio-bindings

## Dependencies

### Bindgen
The bindings are currently generated using
[bindgen](https://crates.io/crates/bindgen):

```bash
cargo install bindgen
```

### Linux Kernel
Generating bindings depends on the Linux kernel, so you need to have the
repository on your machine:

```bash
git clone https://github.com/torvalds/linux.git
```

## Example for adding a new version

For this example we assume that you have both linux and vfio-bindings
repositories in your root and we will use linux version v5.2 as example.

```bash
# Step 1: Crate a new module using a name with format "bindings_vVERSION" in
# src/
cd vfio-bindings
mkdir src/bindings_v5_2_0
cd ~

# Step 2: Copy the "mod.rs" file from the directory of an already existing
# version module to the one we've just created.
cd vfio-bindings/src
cp bindings_v5_0_0/mod.rs bindings_v5_2_0/mod.rs

# linux is the repository that you cloned at the previous step.
cd linux

# Step 3: Checkout the version you want to generate the bindings for.
git checkout v5.2

# Step 4: Generate the bindings from the kernel headers.
make headers_install INSTALL_HDR_PATH=v5_2_headers
cd v5_2_headers
bindgen include/linux/vfio.h -o vfio.rs \
    --with-derive-default \
    --with-derive-partialeq \
    -- -Iinclude

cd ~

# Step 5: Copy the generated files to the new version module.
cp linux/v5_2_headers/vfio.rs vfio-bindings/src/bindings_v5_2_0
```

Once this is done, you need some modifications to the generated vfio.rs.
First change below line:
```rust
pub const VFIO_TYPE: u8 = 59u8;
```
to
```rust
pub const VFIO_TYPE: u32 = 59;
```

This is required due to that bindgen can not generate VFIO_TYPE correctly
at this moment. You might also want to add the proper license header to
the file.

Finally add the new version module to `vfio-bindings/lib.rs`. If this version
is newer than the others already present, make this version the default one by
getting it imported when there isn't any other version specified as a feature:

```rust
#[cfg(all(not(feature = "vfio-v5_0_0"), not(feature = "vfio-v5_2_0")))]
pub use super::bindings_v5_2_0::*;
```
