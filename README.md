# Aurae Language

The Aurae Runtime project is a simplified distributed systems toolchain aimed at providing composable and multi tenant distributed systems.

This repository houses the `aurae` interpreter for executing Aurae scripts against the core gRPC server that runs on a single node.

This repository is the primary client to the [Aurae Standard Library](https://github.com/aurae-runtime/api/blob/main/README.md#the-aurae-standard-library).

The Aurae language is based on [Rhai](https://rhai.rs/book/) and can optionally be used for quick and easy access to the core APIs.

```typescript
#!/usr/bin/env aurae


let aurae = connect(); // Connect and authenticate with mTLS stored in a ~/.aurae/config
aurae.info().json();   // Print the connection details as JSON


let observe = aurae.observe() // Initialize the observe subsystem
observe.status().json();      // Print the status of an Aurae system to JSON
```


## Build

⚠️ Early Active Development ⚠️

We suggest building the project from the higher order [environment](https://github.com/aurae-runtime/environment) repository.

```
git clone git@github.com:aurae-runtime/environment.git
cd environment
make submodules pki config all
```

Alternatively it is possible to build `aurae` by itself check out this repository and use the Makefile.

```bash
make
```

Or manually using Cargo. 

```bash
cargo build 
cargo install --path .
```

### Architecture 

Aurae language follows a similar paradigm to Kubernetes `kubectl` command. However unlike Kubernetes there is no "main command line tool" like `kubectl`. Instead we have clients, and the easiest client to get started building with is this language client. 

Download the static binary directly to your system, and you can begin writing scripts directly against a running Aurae server.


