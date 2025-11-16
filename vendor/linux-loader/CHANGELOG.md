# Upcoming Release

# [v0.13.0]

## Added

- [[#190](https://github.com/rust-vmm/linux-loader/pull/190)] Introduce RISC-V64
  architecture support.

## Changed

- [[#194](https://github.com/rust-vmm/linux-loader/pull/194)] Updated vm-memory to 0.16.0.
- [[#197](https://github.com/rust-vmm/linux-loader/pull/197)] Re-organize
  `loader`, `configurator` and `benches` module layout, leaving original interface
  intact.

# [v0.12.0]

## Changed

- [[#187](https://github.com/rust-vmm/linux-loader/pull/187)] Updated vm-memory to 0.15.0.
- [[#179](https://github.com/rust-vmm/linux-loader/pull/179)] Load hvm_modlist_entry into guest memory when requested.
- [[#177](https://github.com/rust-vmm/linux-loader/pull/176)] Added loading
  of PVH module blobs into guest memory. This enables booting with `initrd`
  via PVH boot.

# [v0.11.0]

## Changed

- [[#173](https://github.com/rust-vmm/linux-loader/pull/173)] Updated vm-memory to 0.14.0.
- [[#170](https://github.com/rust-vmm/linux-loader/pull/170)] Added all features to the generated docs.rs documentation.

# [v0.10.0]

## Changed

- [[#162](https://github.com/rust-vmm/linux-loader/pull/162)] Updated vm-memory to 0.13.0.
  This introduces a `ReadVolatile` bound on `KernelLoader::load`.

# [v0.9.1]

## Fixed
- [[#130]](https://github.com/rust-vmm/linux-loader/issues/130) Generate bindings
  to fix unaligned references in unit tests. 
- [[#160]](https://github.com/rust-vmm/linux-loader/pulls/160) Update vm-memory to 0.12.2

# [v0.9.0]

## Fixed
- [[#71]](https://github.com/rust-vmm/linux-loader/issues/71) Fix incorrect
  alignment for ELF notes, starting address of name field and descriptor
  field have a 4-byte alignment.

# [v0.8.1]

## Fixed

- [[#125]](https://github.com/rust-vmm/linux-loader/pull/125) The ELF
header contains offsets that the loader uses to find other
structures. If those offsets are beyond the end of the file (or would go
past the end of the file) it is essential to error out when attempting
to read those.

## Added
- Add a new criterion advisory to ignore list [`2580d4`](https://github.com/rust-vmm/linux-loader/commit/2580d45f741988468e9b086adbcadae7cc7433a5)

# [v0.8.0]

## Changed

- Updated vm-memory from 0.9.0 to 0.10.0

# [v0.7.0]

## Added
- Added `insert_init_args` method allowing insertion of init arguments into `Cmdline`.

## Changed
- Removed `InvalidDevice` error type (it wasn't used anywhere).
- Replaced `From` with `TryFrom<Cmdline>` for `Vec<u8>` to be able
  to propagate errors returned by `as_cstring` when converting a `Cmdline` to `Vec<u8>`.
- Support added for both boot and init arguments in `try_from`.
- Changed `new` to return `Result` for invalid command line capacity handling.

# [v0.6.0]

## Changed
- Crate is now using edition 2021.

## Added
- Derived `Eq` for `Error` types and the `PvhBootCapability` enum.

## Fixed
- Fixed a bug in `load_cmdline` due to which the command line was not null
  terminated. This resulted in a change in the `Cmdline` API where instead of
  returning the cmdline as a String, we're now returning it as a `CString` as
  the latter has support for converting it to a null terminated bytes array.
- Fixed an off-by-one error in load_cmdline, where we were doing validations
  on the first address after the command line memory region, instead of the
  last inclusive one of it.

# [v0.5.0]

## Fixed
- [[#104]](https://github.com/rust-vmm/linux-loader/issues/104) Fixed
  the `--no-default-features` not working.

## Changed
- [[#111]](https://github.com/rust-vmm/linux-loader/pull/111) Use
  caret requirements for dependencies.

## Added
- [[#99]](https://github.com/rust-vmm/linux-loader/pull/99) Implement
   `Debug` and `PartialEq` for `CmdLine`.
- [[#100]](https://github.com/rust-vmm/linux-loader/pull/100) Added
   `Clone` derive for `CmdLine`.

# [v0.4.0]

## Fixed

- [[#66]](https://github.com/rust-vmm/linux-loader/issues/66) Fixed potential
  overflow in calls to `align_up`.

## Changed

- [[#62]](https://github.com/rust-vmm/linux-loader/issues/62) The
  `load_cmdline` function now takes as a parameter the crate defined
  `Cmdline` object instead of `Cstr`. This means that customers don't need to
  convert the object before calling into `load_cmdline`.
- [[#83]](https://github.com/rust-vmm/linux-loader/issues/83) Updated the
  vm-memory dependency requirement to the latest version (0.6.0).

## Added

- [[#79]](https://github.com/rust-vmm/linux-loader/pull/79) Implemented
  `From<Cmdline>` for `Vec<u8>`. This replaces the obsolete `Into`
  implementation.

# [v0.3.0]

## Fixed

- Replaced panic condition in `align_up` with returning an Error.
- Fixed potential hang condition in Elf::load caused by arithmetic overflow.
- Disallow overflow when computing the kernel load address when loading ELF.
- Fix unchecked arithmetic in BzImage::load that could lead to undefined
  behavior.


## Added

- Added functions for specifying virtio MMIO devices when building the kernel
  command line.
- Added a function to specify multiple values in `key=values` pairs when
  building the kernel command line.

# [v0.2.0]

## Added

- Added traits and structs for loading ELF (`vmlinux`), big zImage (`bzImage`)
  and PE (`Image`) kernels into guest memory.
- Added traits and structs for writing boot parameters to guest memory.
