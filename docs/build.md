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

##### Ubuntu

```bash
sudo apt-get update;
sudo apt-get install -y protobuf-compiler; # Protocol Buffer Compiler
sudo apt-get install -y musl-tools; # musl libc
```

### Prepare the Environment

First you will need to create [authentication certificates](/certs) and create an `~/.aurae/config` file.

```bash 
make pki config # For quick-start only
```

Now you can compile and install the toolchain

```bash 
make
```

You can optionally compile each directly:

```bash 
make auraed      # compile and install auraed with cargo
make auraescript # compile and install auraescript with cargo
```
