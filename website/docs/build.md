# Building Aurae from Source

Checkout the core [aurae](https://github.com/aurae-runtime/aurae) repository.

**Note**: Aurae currently only has support for Linux on X86 architecture.

```bash 
https://github.com/aurae-runtime/aurae.git
```

### Dependencies

The Aurae environment depends on the `protoc` protocol buffer compiler being available within the path.
Install `protoc` using your operating system's package manager (Or from source if you want to :) )

##### Ubuntu 

```bash
sudo apt install -y protobuf-compiler
```

##### Arch Linux

```bash 
pacman -S protobuf
```

### Prepare the Environment

To install Aurae first build the submodules for the project.

```bash
make submodules # Please do not forget to do this!
```

Next you will need to create [authentication certificates](/certs) and create an `~/.aurae/config` file.

```bash 
make pki config # For quick-start only
```

Now you can compile and install the toolchain

```bash 
make
```

You can optionally compile each submodule directly.

```bash 
make auraed      # compile and install auraed with cargo
make auraescript # compile and install auraescript with cargo
```