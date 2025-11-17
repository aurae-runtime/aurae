# Building Aurae from Source

Checkout the core [aurae](https://github.com/aurae-runtime/aurae) repository.

**Note**: Aurae currently only targets support for Linux on X86 architecture.

```bash 
https://github.com/aurae-runtime/aurae.git
```

### Dependencies

The Aurae environment has certain dependencies that are expected to be available. Some of them can be installed via
commands provided below.

- [Rust](https://rustup.rs)
- [Protocol Buffer Compiler](https://grpc.io/docs/protoc-installation/)
- [buf](https://docs.buf.build/installation)
- [musl libc](https://musl.libc.org)
- [BPF Linker](https://github.com/aya-rs/bpf-linker)

### Toolchain notes

- **Rust**: `rust-toolchain.toml` pins Rust 1.91.1 with the `x86_64-unknown-linux-musl` target. On an x86_64 host run `rustup target add x86_64-unknown-linux-musl` before `make`. On Apple Silicon or other aarch64 hosts also install/override `aarch64-unknown-linux-musl` (Aurae's `Makefile` derives the target via `$(uname -m)-unknown-linux-musl`). If you must experiment with other compiler versions, prefer `rustup override set <version>` locally rather than editing `rust-toolchain.toml`.
- **Buf**: Buf drives all protobuf/TypeScript generation. Install Buf v1.50.0 (the version referenced in the `Makefile`) and verify `buf lint api` and `buf generate -v api` pass before building other components.
- **System toolchain helpers** used by dependencies include `clang`/`libclang` (for `virtio-bindings`), `llvm` (for compiling eBPF programs), `python3` and `ninja` (for `rusty_v8` when compiling `auraescript`), and musl-capable cross compilers (`aarch64-linux-musl-gcc`, `x86_64-linux-musl-gcc`). On Fedora you can install `musl-gcc`/`musl-clang` and symlink `/usr/bin/musl-gcc` to `aarch64-linux-musl-gcc`, or use distro cross-compilers such as `gcc-aarch64-linux-gnu`/`gcc-x86_64-linux-gnu` and rename/symlink them to the `*-linux-musl-gcc` names Aurae expects.
- **Portable musl toolchains**: If your distribution does not ship musl cross-compilers, fetch them from [musl.cc](https://musl.cc) instead:

```bash
mkdir -p ~/toolchains
cd ~/toolchains
curl -LO https://musl.cc/aarch64-linux-musl-cross.tgz
tar xf aarch64-linux-musl-cross.tgz
```

Add the extracted `bin/` directory to your `PATH`, or symlink the provided `*-linux-musl-gcc` binaries into `~/.local/bin`.


##### Ubuntu

```bash
sudo apt-get install -y protobuf-compiler; # Protocol Buffer Compiler
sudo apt-get install -y musl-tools; # musl libc
sudo apt-get install -y build-essential; # gcc compiler
```

##### Fedora

```bash
sudo dnf install -y protobuf-compiler; # Protocol Buffer Compiler
sudo dnf install -y musl-gcc; # musl libc
sudo dnf install -y '@Development Tools'; # gcc compiler
```

##### Arch

```bash
yay -S protobuf # Protocol Buffer Compiler
yay -S buf # buf
yay -S musl # musl libc 
yay -S gcc # gcc compiler
```

### Prepare the Environment

First you will need to create [authentication certificates](/certs) and create an `~/.aurae/config` file.

```bash 
make pki config # For quick-start only
```

Now you can compile and install the toolchain

```bash 
make all
```

You can optionally compile and install each binary directly. E.g.,:

```bash 
make auraed      # compile and install auraed with cargo
make auraescript # compile and install auraescript with cargo
```

*For more commands, and the dependencies between them, please see the Makefile at the root of the repository.*
