# AuraeScript

AuraeScript is a turing complete language for platform teams built on [Deno](https://deno.land).

AuraeScript is a lightweight client that wraps the [Aurae Standard Library](https://aurae.io/stdlib/).

AuraeScript is a quick way to access the core Aurae APIs and follows normal UNIX parlance. AuraeScript should feel simple and intuitive for any Go, C, Python, or Rust programmer.

```typescript
// @ts-ignore
import {AllocateCellRequest, Cell, CellServiceClient} from "../lib/runtime.ts";

// @ts-ignore
Deno.core.initializeAsyncOps();

let cells = new CellServiceClient();

cells.Allocate(<AllocateCellRequest>{
    cell: Cell.fromPartial({
        name: "test",
        cpus: "2"
    })
}).then(r => {
    // @ts-ignore
    Deno.core.print("done")
});
```

## Build From Source

⚠️ Early Active Development ⚠️

```
git clone git@github.com:aurae-runtime/aurae.git
cd aurae
make pki config all
```

Alternatively it is possible to build `auraescript` by itself. Check out this repository and use the Makefile.

```bash
make auraescript
```

...or manually using Cargo.

```bash
cd auraescript
cargo build 
cargo install --path .
```

### Architecture

AuraeScript follows a similar client paradigm to Kubernetes `kubectl` command. However, unlike Kubernetes this is not a command line tool like `kubectl`. AuraeScript is a fully supported programing language complete with a systems standard library. The Aurae runtime projects supports many clients, and the easiest client to get started building with is AuraeScript.

Download the static binary directly to your system, and you can begin writing AuraeScript programs directly against a running Aurae server.

