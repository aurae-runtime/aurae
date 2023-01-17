<!-- THE DOCUMENT -->

![Deploy (aurae.io)] ![Documentation] ![Main Build (lint, compile, test)]

<p align="center">
  <img src="https://raw.githubusercontent.com/aurae-runtime/aurae/main/docs/assets/logo-1024x1024.png" width="450">
</p>

# Mission

Aurae is on a mission to be the most loved and effective way of managing
workloads on a node. Our hope is that by bringing a better set of controls to a
node, we can unlock brilliant higher order distributed systems in the future.

# Introduction

[Aurae] is a memory-safe[^memory-safe] runtime daemon, process manager, and
PID-1 initialization system designed to remotely schedule processes, containers,
and virtual machines as well as set node configurations like networking storage.

Through system proportioning and enterprise workload isolation techniques, the
Aurae [open-source] project can complement Kubernetes, as Aurae supports the
usage of multi-tenant workloads and enterprise identities all the way down to
the socket layer.

## Project Status

> **STILL IN EARLY DEVELOPMENT!**<br>
> **The Aurae project and API can change without notice.**<br> 
> **Do not run the project in production until further notice!**
> <br>

- The Aurae project welcomes contributions of all kinds and sizes.
- Please read the "[getting involved]" documentation before contributing to the
  project.
- You do not have to know [Rust] to join the project.

By joining the project in its early phases, you'll participate in the
development of a milestone candidate for enterprise distributed systems and
automation.

## **In more detail**

Aurae enables the control of each internal runtime process on a piece of
hardware or node as its [PID]-1 instance on a [Linux kernel] and offers
mTLS-encrypted gRPC APIs for managing processes through
[workload isolation with Aurae cells] [^cells].

Furthermore, Aurae also enables the management of [virtual machines] and
containers by adding additional features comparable to those of [Firecracker]
and [containerd] and combining effective node management with additional
controls while offering a scope comparable to that of [systemd]. In doing so,
Aurae takes ownership of all runtime processes on a single piece of hardware and
provides [mTLS]-encrypted [gRPC] APIs (the [Aurae standard library]) to manage
these processes in [Aurae cells].

The inherent compatibility with Kubernetes and the capacity to function as a
systemd [substitute](#project-status) enable commendable performance in
enterprise settings to meet the demands of distributed systems both now and in
the future [^medium].

---

Many parts of the Aurae runtime system and the Aurae standard library use the
core definitions in predefined [.proto] files from this repository for their
automatic generation. While TypeScript files can be leveraged to replace static
manifests, such as YAML, as well as interact directly with a running system.

|||
| :--- | :--- |
| **Auraed**      | To ensure memory safety, Aurae serves the generic system's runtime daemon ([auraed]).|
| **AuraeScript** | The [AuraeScript] (a Turing-complete scripting language built on TypeScript) library automatically generates itself from the pre-defined [.proto] files defined in the Aurae standard library.<br>It also directly embeds [Deno] source code to provide an SDK and the functionality to attach remote clients for the direct remote communication with Aurae. |
|||

```typescript
#!/usr/bin/env auraescript
let cells = new runtime.CellServiceClient();

let allocated = await cells.allocate(<runtime.AllocateCellRequest>{
  cell: runtime.Cell.fromPartial({
    name: "my-cell",
    cpus: "2",
  }),
});

let started = await cells.start(<runtime.StartExecutableRequest>{
  executable: runtime.Executable.fromPartial({
    cellName: "my-cell",
    command: "sleep 4000",
    description: "Sleep for 4000 seconds",
    name: "sleep-4000",
  }),
});
```

|||
| :--- | :--- |
| **Authentication**               | Aurae extends [SPIFFE]/[SPIRE] (x509 mTLS)-backed identity, authentication (authn), and authorization (authz) in a distributed system down to the Unix domain socket layer. |
| **Principle of Least Awareness** | A single Aurae instance has no awareness of higher order scheduling mechanisms such as the Kubernetes control plane.                                                        |
| **Runtime Workloads**            | The Aurae runtime API can manage [virtual machines], [executables], [cells], [pods], and other [spawned Aurae instances].                                                   |
| **The Aurae Standard Library**   | The Aurae project exposes its functionality as a gRPC API through the [Aurae standard library]. The [V0 API reference] contains the current library definition.             |
|||

<!-- All the links!! -->
<!-- +Footnotes -->

[^cells]:
    Additionally, with Aurae cells, the project provides various ways to partition
    and slice up systems allowing for isolation strategies in enterprise workloads.

[^compare]:
    As a low-level building block, the Aurae Project works well with any
    higher-order system by offering a thoughtful set of API calls and controls for
    managing workloads on a node or single piece of hardware.

[^medium]:
    Learn more from the [Medium Blog: Why fix Kubernetes and Systemd?] by
    [Kris Nóva]).

[^memory-safe]: 
    The reliability and effectiveness of the Rust systems language make it an excellent choice for the development of the Aurae project. [Learn more about Rust]

<!-- +Status Badges -->

[deploy (aurae.io)]: https://github.com/aurae-runtime/aurae/actions/workflows/091-deploy-website-documentation-ubuntu-make-docs.yml/badge.svg?branch=main "https://github.com/aurae-runtime/aurae/actions/workflows/091-deploy-website-documentation-ubuntu-make-docs.yml"
[documentation]: https://github.com/aurae-runtime/aurae/actions/workflows/036-check-website-documentation-aurae-builder-make-check-docs.yml/badge.svg "https://github.com/aurae-runtime/aurae/actions/workflows/036-check-website-documentation-aurae-builder-make-check-docs.yml"
[main build (lint, compile, test)]: https://github.com/aurae-runtime/aurae/actions/workflows/001-cargo-install-ubuntu-make-build.yml/badge.svg?branch=main "https://github.com/aurae-runtime/aurae/actions/workflows/001-cargo-install-ubuntu-make-build.yml"

<!-- +aurae.io/ -->

[aurae cells]: https://aurae.io/blog/24-10-2022-aurae-cells/ "Learn more about Aurae cells"
[aurae standard library]: https://aurae.io/stdlib/ "Learn more about Auraes standard library"
[aurae]: https://aurae.io/ "Visit aurae.io"
[cells]: https://aurae.io/stdlib/v0/#cell "Processes running in a shared cgroup namespace"
[executables]: https://aurae.io/stdlib/v0/#executable "Basic runtime processes"
[getting involved]: https://aurae.io/community/#getting-involved "Participate and contribute!"
[pods]: https://aurae.io/stdlib/v0/#pod "Cells running in spawned instances"
[spawned aurae instances]: https://aurae.io/stdlib/v0/#instance "Short lived nested virtual instances of Aurae"
[v0 api reference]: https://aurae.io/stdlib/v0/ "Learn more about the current Aurae library definitions"
[virtual machines]: https://aurae.io/stdlib/v0/#virtualmachine "Long-lived arbitrary virtual machines"
[workload isolation with aurae cells]: https://aurae.io/blog/2022-10-24-aurae-cells/#IntroducingAuraeCells "Aurae Blog: 2022-10-24"

<!-- +Wiki -->

[grpc]: https://en.wikipedia.org/wiki/GRPC "Read about gRPC"
[mtls]: https://en.wikipedia.org/wiki/Mutual_authentication#mTLS "Read about mTLS"
[pid]: https://en.wikipedia.org/wiki/Process_identifier "Read about PID"

<!-- +Github -->

[auraed]: https://github.com/aurae-runtime/auraed "Check out the Aurae runtime deamon on Github 🌟"
[auraescript]: https://github.com/aurae-runtime/aurae/tree/main/auraescript "Check out the Auraescript on Github 🌟"
[containerd]: https://github.com/containerd/containerd "Read about containerd on GH"
[firecracker]: https://github.com/firecracker-microvm/firecracker "Read about firecracker on Github"
[kris nóva]: https://github.com/krisnova "Check out Kris Nóva on Github 🌟"
[open-source]: https://github.com/aurae-runtime/aurae/blob/main/LICENSE "Apache License 2.0"
[spiffe]: https://github.com/spiffe "Read about SPIFFE"
[spire]: https://github.com/spiffe/spire "Read about SPIRE"

<!-- +External links -->

[.proto]: https://protobuf.dev/ "Read more about Protocol Buffers"
[deno]: https://deno.land "Read more about Deno"
[learn more about rust]: https://doc.rust-lang.org/book/ "The book about the Rust programming language"
[linux kernel]: https://git.kernel.org/ "Learn about the Linux kernels"
[medium blog: why fix kubernetes and systemd?]: https://medium.com/@kris-nova/why-fix-kubernetes-and-systemd-782840e50104 "Learn more about the possibilies of Aurae"
[rust]: https://www.rust-lang.org/ "Read and learn more about the Rust language"
[systemd]: https://www.freedesktop.org/wiki/Software/systemd/ "Read about Systemd"
[yaml]: https://yaml.org/ "Read more about YAML"
