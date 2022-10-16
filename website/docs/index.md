# Aurae

Aurae is a free and open source Rust project which houses a generic systems runtime daemon built specifically for enterprise distributed systems. Aurae is designed to work well with Kubernetes, and serves as an enterprise focused alternative to systemd.

Think of [auraed](https://github.com/aurae-runtime/auraed) as a pid 1 init machine daemon with a scope similar to [systemd](https://www.freedesktop.org/wiki/Software/systemd/) and functionality similar to [containerd](https://github.com/containerd/containerd) and [firecracker](https://github.com/firecracker-microvm/firecracker).

Aurae brings [SPIFFE](https://github.com/spiffe)/[SPIRE](https://github.com/spiffe/spire) (x509 mTLS) backed identity, authentication (authn) and authorization (authz) as low as the Unix domain socket layer in a distributed system.

Aurae exposes its functionality over a gRPC API which is referred to as the [Aurae Standard Library](https://github.com/aurae-runtime/auraed/tree/main/stdlib#the-aurae-standard-library).

A single Aurae instance has no awareness of higher order scheduling mechanisms such as the Kubernetes control plane. Aurae is designed to take ownership of a single machine, and expose the standard library a generic and meaningful way for higher order consumers.


## Motivation 

Read [Why fix Kubernetes and Systemd](https://medium.com/@kris-nova/why-fix-kubernetes-and-systemd-782840e50104) by [Kris Nóva](https://github.com/krisnova). 

Aurae attempts to simplify and improve the stack in enterprise distributed systems by carving out a small portion of responsibility while offering a few basic guarantees with regard to state, synchronicity, awareness, and security.

Aurae brings enterprise identity as low as the socket layer in a system, which unlocks multi tenant workloads that run below tools like Kubernetes.

## Workloads 

Aurae supports 3 types of workloads which can be started and scheduled alongside each other on a single machine.

 - Executables (Regular executable processes on a host system. Similar to [Systemd Units with ExecStart](https://www.freedesktop.org/software/systemd/man/systemd.service.html)).
 - Containers (A secure and opinionated container runtime is baked directly into the Aurae binary).
 - Virtualization (A secure and opinionated hypervisor is baked directly into the Aurae binary).

## AuraeScript 

Aurae offers a Turing complete scripting language written in Rust and similar to TypeScript called [AuraeScript](https://github.com/aurae-runtime/auraescript).

```typescript
let execute = true;    // Toggle execution

let aurae = connect(); // Connect to the daemon
aurae.info().json();   // Show identity

if execute {
    // Execute "cat /etc/resolv.conf"
    let runtime = aurae.runtime();
    let example = exec("cat /etc/resolv.conf");
    runtime.start(example).json();
}
```

AuraeScript servers as one of many clients to the system and can be used to express workload manifests instead of YAML.
AuraeScript can be used to control and gain visibility to the system.
