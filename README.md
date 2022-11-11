# Aurae

![Main Build](https://github.com/aurae-runtime/aurae/actions/workflows/build-compile-test.yml/badge.svg)


Aurae is a free and open source Rust project which houses a memory-safe systems runtime daemon built specifically for enterprise distributed systems called `auraed`.

The `auraed` daemon can be ran as a pid 1 on a Linux kernel and manages containers, virtual machines, and spawning short-lived nested virtual instances of itself for an additional layer of isolation.

### Mission

Aurae is on a mission to be the most loved and effective way of managing workloads on a single piece of hardware. Our hope is that by bringing a better set of controls to a node, we can unlock brilliant higher order distributed systems in the future.

Aurae takes ownership of all runtime processes on a single piece of hardware, and provides mTLS encrypted gRPC APIs ([Aurae Standard Library](https://aurae.io/stdlib/)) to manage the processes. With Aurae [Cells](https://aurae.io/blog/24-10-2022-aurae-cells/) the project offers a way to slice up a system using various isolation strategies for enterprise workloads.

### Project Status

The project is very young and under active development. The APIs are subject to change without notice until further notice.
As we continue to develop the project the APIs will stabilize and eventually a long term stable release will be offered.

At this time the project should not be run in production.

Please read [getting involved](https://github.com/aurae-runtime/community#getting-involved) if you are interested in joining the project in its early phases. Contribution types of all types and ranges are welcome. You do not have to know Rust to join the project.

### Runtime Workloads

Aurae offers a runtime API which is capable of managing:

 - [Executables](https://aurae.io/stdlib/v0/#executable) (Basic runtime processes)
 - [Cells](https://aurae.io/stdlib/v0/#cell) (Processes running in a shared cgroup namespace)
 - [Spawned Aurae Instances](https://aurae.io/stdlib/v0/#instance) (Short lived nested virtual instances of Aurae)
 - [Pods](https://aurae.io/stdlib/v0/#pod) (Cells running in spawned instances)
 - [Virtual Machines](https://aurae.io/stdlib/v0/#virtualmachine) (Long-lived arbitrary virtual machines)

### Auraed

Think of [auraed](https://github.com/aurae-runtime/aurae/tree/main/auraed) as a pid 1 init machine daemon with a scope similar to [systemd](https://www.freedesktop.org/wiki/Software/systemd/) and functionality similar to [containerd](https://github.com/containerd/containerd) and [firecracker](https://github.com/firecracker-microvm/firecracker).

### Authentication

Aurae brings [SPIFFE](https://github.com/spiffe)/[SPIRE](https://github.com/spiffe/spire) (x509 mTLS) backed identity, authentication (authn) and authorization (authz) as low as the Unix domain socket layer in a distributed system.

### Standard Library

Aurae exposes its functionality over a gRPC API which is referred to as the [Aurae Standard Library](https://aurae.io/stdlib/).

### Principle of Least Awareness

A single Aurae instance has no awareness of higher order scheduling mechanisms such as the Kubernetes control plane. Aurae is designed to take ownership of a single node, and expose the standard library a generic and meaningful way for higher order consumers.

Aurae is a low level building block and is designed to work well with any higher order system by offering a thoughtful set of APIs and controls for managing workloads on a node.

### Motivation 

Read [Why fix Kubernetes and Systemd](https://medium.com/@kris-nova/why-fix-kubernetes-and-systemd-782840e50104) by [Kris Nóva](https://github.com/krisnova). 

Aurae attempts to simplify and improve the stack in enterprise distributed systems by carving out a small portion of responsibility while offering a few basic guarantees with regard to state, synchronicity, awareness, and security.

Aurae brings enterprise identity as low as the socket layer in a system, which unlocks multi tenant workloads that run below tools like Kubernetes.

## AuraeScript 

Aurae offers a Turing complete scripting language written in Rust and similar to TypeScript called [AuraeScript](https://github.com/aurae-runtime/aurae/tree/main/auraescript).

```typescript
let execute = true;    // Toggle execution

let aurae = connect(); // Connect to the daemon
aurae.info().json();   // Show identity

if execute {
    // Execute "cat /etc/resolv.conf"
    let runtime = aurae.runtime();
    exec("cat /etc/resolv.conf").json();
}
```

AuraeScript serves as one of many clients to the system and can be used to express workload manifests instead of YAML.
AuraeScript can be used to control and gain visibility to the system.

## The Aurae Standard Library

See the [V0 API Reference](https://aurae.io/stdlib/v0/) for the current library definition.
