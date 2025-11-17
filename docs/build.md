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
- [buf](https://docs.buf.build/installation) (Aurae pins the CLI to **1.60.0**; run `hack/install-build-deps.sh` or `buf --version` if you’re unsure what you have installed)
- [musl libc](https://musl.libc.org)
- [BPF Linker](https://github.com/aya-rs/bpf-linker)


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
