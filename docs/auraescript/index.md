# AuraeScript

AuraeScript is a turing complete language for platform teams built on [Rhai](https://rhai.rs/book/) and is similar to TypeScript and Rust.

```typescript

let connect = true;

if connect {
    let aurae = connect();
    aurae.info().json();
}

```

AuraeScript is a lightweight client that wraps the [Aurae Standard Library](https://github.com/aurae-runtime/api/blob/main/README.md#the-aurae-standard-library). 

AuraeScript is a quick way to access the core Aurae APIs and follows normal UNIX parlance. AuraeScript should feel simple and intuitive for any Go, C, Python, or Rust programmer.

```typescript
#!/usr/bin/env auraescript

let client = connect(); // Connect and authenticate with mTLS stored in a ~/.aurae/config
client.info().json();   // Print the connection details as JSON


let observe = client.observe() // Initialize the observe subsystem
observe.status().json();      // Print the status of an Aurae system to JSON
```

## Build From Source

⚠️ Early Active Development ⚠️

We suggest building the project from the higher order [environment](https://github.com/aurae-runtime/environment) repository.

```
git clone git@github.com:aurae-runtime/environment.git
cd environment
make submodules pki config all
```

Alternatively it is possible to build `aurascript` by itself. Check out this repository and use the Makefile.

```bash
make
```

...or manually using Cargo. 

```bash
cargo build 
cargo install --path .
```

### Architecture 

AuraeScript follows a similar client paradigm to Kubernetes `kubectl` command. However, unlike Kubernetes this is not a command line tool like `kubectl`. AuraeScript is a fully supported programing language complete with a systems standard library. The Aurae runtime projects supports many clients, and the easiest client to get started building with is AuraeScript.

Download the static binary directly to your system, and you can begin writing AuraeScript programs directly against a running Aurae server.


