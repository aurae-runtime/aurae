# The Saga Language

Saga is a turing complete interpreted language for platform teams built on [Rhai](https://rhai.rs/book/) and is similar to TypeScript and Rust.

```typescript

let connect = true;

if connect {
    let aurae = connect();
    aurae.info().json();
}

```

Saga is a lightweight client that wraps the [Aurae Standard Library](https://github.com/aurae-runtime/api/blob/main/README.md#the-aurae-standard-library). 

Saga is a quick way to access the core Aurae APIs and follows normal UNIX parlance. Sage should feel simple and intuitive for any Go, C, Python, or Rust programmer.

```typescript
#!/usr/bin/env aurae

let aurae = connect(); // Connect and authenticate with mTLS stored in a ~/.aurae/config
aurae.info().json();   // Print the connection details as JSON


let observe = aurae.observe() // Initialize the observe subsystem
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

Alternatively it is possible to build `saga` by itself. Check out this repository and use the Makefile.

```bash
make
```

...or manually using Cargo. 

```bash
cargo build 
cargo install --path .
```

### Architecture 

The Saga language follows a similar client paradigm to Kubernetes `kubectl` command. However, unlike Kubernetes this is not a command line tool like `kubectl`. Saga is a fully supported programing language complete with an infrastructure standard library. The Aurae runtime projects supports many clients, and the easiest client to get started building with is Saga.

Download the static binary directly to your system, and you can begin writing saga programs directly against a running Aurae server.


