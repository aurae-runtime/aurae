# Aurae

The Aurae Project is a simplified distributed systems toolchain aimed at providing composable and multi tenant distributed systems.
This repository is the 'aurae' interpreter for executing scripts against the daemon.

The project aims at targetting the same space as the Kubernetes Kubelet, and Systemd.

Scripts can be optionally used for quick and easy access to the core APIs.

---

⚠️ Early Active Development ⚠️

## Build

We suggest building the project from the higher order [environment](https://github.com/aurae-runtime/environment) repository.

```
git clone git@github.com:aurae-runtime/environment.git
cd environment
make submodules pki config all
```

Alternatively it is possible to build `aurae` by itself check out this repository and use the Makefile.

```bash
make # Will compile and install Aurae using Cargo.
```

Or manually using Cargo. 

```bash
cargo build 
cargo install --path .
```

## Aurae Scripts

Aurae has a TypeScript-like programming language that interfaces directly with the rest of the system.

Start and run [auraed](https://github.com/aurae-runtime/auraed) and you can begin writing scripts.

```typescript
#!/usr/bin/env aurae

// Connect and authenticate with a local Daemon
let aurae = connect();
aurae.info();

// Get the status of the daemon
let observe = aurae.observe()
observe.status()
```
### Architecture 

See the [whitepaper](https://docs.google.com/document/d/1dA591eipsgWeAlaSwbYNQtAQaES243IIqXPAfKhJSjU/edit#heading=h.vknhjb3d4yfc).

