# `vmm-reference`

:exclamation: `vmm-reference` is for experimental purposes and should *NOT* be
used in production. :exclamation:

## Design

The purpose of the reference VMM is twofold:

1. To validate the `rust-vmm` crates that compose it and demonstrate their
   functionality in a use-case-agnostic, end-to-end VMM.
1. To serve as a starting point in the creation of tailor-made VMMs that users
   build according to their needs. Users can fork the reference VMM, mix and
   match its components and UI to create a functional VMM with a minimal attack
   surface and resource footprint, custom-made to suit their isolation
   requirements.

The reference VMM consists of `rust-vmm` crates and minimal glue code that
sticks them together. The end result is a binary, roughly split between a
simple CLI and a `vmm` crate, which ingests all the available `rust-vmm`
building blocks compiled with all their available features. As crate
development progresses, in the future, we may have feature `X` in crate `A`
mutually incompatible with feature `Y` in crate `B` - therefore the reference
VMM, which depends on both crates `A` and `B`, will no longer support features
`X` and `Y` simultaneously. If and when this situation occurs, multiple
binaries for the reference VMM will be supplied.

The `vmm` crate allows for pluggable UIs via a `VMMConfig` structure. A
basic command line parser demonstrates how a frontend can be stitched to the
VMM.

For more details, see [`DESIGN.md`](docs/DESIGN.md).

## Usage

The reference VMM can be used out of the box as a `hello-world` example of a
fully functional VMM built with `rust-vmm` crates.

To start a basic VM with one vCPU and 256 MiB memory, you can use the following
command:

```bash
vmm-reference                      \
    --kernel path=/path/to/vmlinux \
    [--block <blkdev_config> - TBD]
    [--net <netdev_config> - TBD]
```

The default configuration can be updated through the
[command line](#cli-reference).

The crate's [`Cargo.toml`](Cargo.toml) controls which VMM functionalities are
available. By default, all rust-vmm crates are listed as dependencies and
therefore included. Users can play freely with the building blocks by modifying
the TOML, and the prepackaged CLI can quickly validate the altered
configurations. Advanced users can, of course, plug in their own front-end.

## CLI reference

* `memory` - guest memory configurations
  * `size_mib` - `u32`, guest memory size in MiB (decimal)
    * default: 256 MiB
* `kernel` - guest kernel configurations
  * `path` - `String`, path to the guest kernel image
  * `cmdline` - `String`, kernel command line
    * default: "console=ttyS0 i8042.nokbd reboot=t panic=1 pci=off"
  * `kernel_load_addr` - `u64`, start address for high memory (decimal)
    * default: 0x100000
* `vcpus` - vCPU configurations
  * `num` - `u8`, number of vCPUs (decimal)
    * default: 1
* `block` - block device configuration
    * `path` - `String`, path to the root filesystem
* `net` - network device configuration
    * `tap` - `String`, tap name, only the API support is added for now,
                        an actual network device configuration is done in the
                        [following PR under review](https://github.com/rust-vmm/vmm-reference/pull/49).

*Note*: For now, only the path to the root block device can be configured
via command line. The block device will implicitly be read-write and with
`cache flush` command supported. Passing the `block` argument is optional,
if you want to skip it, make sure you pass to the `path` argument of the
`kernel` configuration, a suitable image (for example a Busybox one).
We plan on extending the API to be able to configure more block devices and
more parameters for those (not just the `path`).
We also want to offer the same support in the near future for network and
vsock devices.

### Example: Override the kernel command line

```bash
vmm-reference \
    --kernel path=/path/to/kernel/image,cmdline="reboot=t panic=1 pci=off"
```

### Example: VM with 2 vCPUs and 1 GiB memory

```bash
vmm-reference                           \
    --memory size_mib=1024          \
    --vcpu num=2                        \
    --kernel path=/path/to/kernel/image
```

## Testing

The reference VMM is, first and foremost, a vehicle for end-to-end testing of
`rust-vmm` crates. Each crate must contain individual functional and
performance tests that exercise as wide a range of use cases as possible; the
reference VMM is not meant to reiterate on that, but to validate all the pieces
put together.
The Rust unit tests are testing modules in isolation and private interfaces,
while the two Rust integration tests (one for each supported kernel image
format, i.e. ELF and bzImage) exercise the only public function of the `Vmm`
object, `run()`.
The Python integration tests make use of the VMM in varied configurations that
aren’t overly complex and illustrate realistic use cases (e.g. one test runs a
VMM with a virtio block device, one test will run a VMM with PCI, etc.).

To be able to successfully run all the tests in this repo, pre-created
resources are stored in S3. The resources are downloaded locally inside the
`resources` directory. This is handled transparently by the test cases.
Note: The resources once downloaded are cached, so they are not downloaded on
every run.

### Running Tests

Recommended way is to run the tests inside a container using the `rustvmm/dev`
docker image as below. (Note: You may need to run `docker` as `sudo`.)

```shell
docker run --device=/dev/kvm -it \
    --security-opt seccomp=unconfined \
    --volume $(pwd):/vmm-reference rustvmm/dev:v11

```

Inside the container, to run the tests, first `cd vmm-reference` and then follow
the instructions as follows.

`vmm-reference` is a
[workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html), so to
run all the Rust tests, the following command should be used:

```bash
cargo test --workspace
```

There is no single command yet for running all the Python integration tests in
one shot. To run the tests from a single file, you can use:

```bash
pytest <path_to_the_file>
```
For example:

```bash
pytest tests/test_run_reference_vmm.py
```

A single Python test can be run as well, as shown below:

```bash
pytest <path_to_the_file>::<test_name>
```
For example:

```bash
pytest tests/test_run_reference_vmm.py::test_reference_vmm_with_disk
```

## License

This project is licensed under either of:

* [Apache License](LICENSE-APACHE), Version 2.0
* [BSD-3-Clause License](LICENSE-BSD-3-CLAUSE)
