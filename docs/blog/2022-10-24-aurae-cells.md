# Workload Isolation with Aurae Cells

## Runtime Subsystem

Last week we merged [Pull Request #73](https://github.com/aurae-runtime/aurae/pull/73) which marks the project's formal acceptance of our initial runtime [subsystem](https://aurae.io/stdlib/#what-is-a-subsystem) API.

```protobuf 
service Runtime {

  rpc RunExecutable(Executable) returns (ExecutableStatus) {}
  
  rpc RunCell(Cell) returns (CellStatus) {}

  rpc RunVirtualMachine(VirtualMachine) returns (VirtualMachineStatus) {}

  rpc Spawn(Instance) returns (InstanceStatus) {}
  
  rpc RunPod(Pod) returns (PodStatus) {}

}
```

The runtime subsystem is the most fundamental API for Aurae. The API is synchronous, and is intended to serve as the lowest level building block for future subsystems in the project.

The API introduces 5 workloads types of runtime isolation primitives, as well as a special function known as `Spawn()`. 

The 5 workload types:

 - Executable
 - Cell
 - VirtualMachine
 - Instance
 - Pod

Thank you to the many authors, contributors, and maintainers who helped the project form conviction on the initial API: 

[Dominic Hamon](https://github.com/dominichamon) 	| [@future-highway](https://github.com/future-highway) 	| [Hazel Weakly](https://github.com/hazelweakly) 	| [Josh Grant](https://github.com/j0shgrant) 	| [Malte Janduda](https://github.com/MalteJ) 	| [@taniwha3](https://github.com/taniwha3) 	| [Vincent Riesop](https://github.com/Vincinator) 	|


# Keeping Pods Intuitive

We make the assumption that most Aurae consumers will be interested in "scheduling pods", as this is the primary unit of work for Kubernetes.

Therefore, we knew we wanted to make Pods look and feel as much like Kubernetes as possible, so they would be intuitive for users. 
From a client perspective an Aurae pod should look, feel, and behave just like an OCI compliant Kubernetes pod with only a few small differences.

Aurae pods will run with an extra layer of isolation. This isolation is based on virtualization (when applicable) and resembles [how Kata containers are created](https://github.com/kata-containers/kata-containers/tree/main/docs/design/architecture#container-creation) or [how firecracker creates a jailed isolation zone](https://github.com/firecracker-microvm/firecracker/blob/main/docs/jailer.md#the-firecracker-jailer).

How Aurae manages and builds this isolation zone for pods is what has influenced the runtime API that you see above.

## Back to the Basics: cgroups and namespaces

In order to understand the 5 workload types we need a small lesson in cgroups and namespaces.

### Control Groups (cgroups)

A [control group](https://man7.org/linux/man-pages/man7/cgroups.7.html) or "cgroup" for short is a way of "slicing" a part of a Linux system into smaller units which can be used for whatever you want. For example, you can cordon off 10% of your systems compute and memory resources with a cgroup, and run any process you want inside it. If your workload eats up more than 10% of the allocated resources, the kernel will terminate it. This cgroup behavior is likely the root cause of many of the `OOMKilled` and CPU throttling errors you see in Kubernetes today.

Notably there are [2 types of cgroup implementation](https://man7.org/linux/man-pages/man7/cgroups.7.html): v1 and v2. Aurae will use the v2 standard by default.

### Namespaces 

A [namespace](https://man7.org/linux/man-pages/man7/namespaces.7.html) is a way of sharing or isolating specific parts of a Linux system with a process. If all namespaces are shared a process is as close as possible to the "host" it runs on. If no namespaces are shared a process is as isolated as possible from the "host" it runs on. Exposing namespaces is usually how container escapes are performed, and how lower level networking and storage is managed with Kubernetes.

```bash 
[root@alice]: ls /proc/1/ns
cgroup  ipc  mnt  net  pid  pid_for_children  time  time_for_children  user  uts
```

### Containers

I often say that cgroups are "vertical" resource slices and namespaces are "horizontal" access controls. When a cgroup is run in its own namespaces it's both a slice of resources, and an isolation boundary as well. We call this intersection a "container".

## Systemd Slices

By default, systemd schedules all of its workloads in their own cgroup with access to the same namespaces as PID 1 on the system. These workloads are called [services](https://www.freedesktop.org/software/systemd/man/systemd.service.html) or units.

Interestingly enough, Kubernetes also leverages systemd slices. You can usually see both systemd slices (`system.slice`) and Kubernetes pods (`kubepods.slice`) running side-by-side by exploring [/sys](https://man7.org/linux/man-pages/man5/sysfs.5.html) or `sysfs(5)` on your system. There are usually other cgroups running there as well. 

```bash 
[root@alice]: /sys/fs/cgroup># ls -d */
dev-hugepages.mount//  kubepods.slice//                 sys-kernel-config.mount//   system.slice//
dev-mqueue.mount//     pids//                           sys-kernel-debug.mount//    user.slice//
init.scope//           sys-fs-fuse-connections.mount//  sys-kernel-tracing.mount//
```

## Simplifying the Stack

We know we wanted to simplify how workloads are managed at scale. We believe that standardizing process management and cgroup management is a way to simplify runtime complexity, as well as offer a means to an ends with [the noisy neighbor problem](https://en.wikipedia.org/wiki/Cloud_computing_issues#Performance_interference_and_noisy_neighbors) in multi tenant systems.

Therefore, we knew we wanted Aurae to offer functionality that would allow it to manage cgroups well for a plethora of runtime use cases and not just containers.

In Kubernetes a user needs to understand the nuance of cgroup implementation detail, systemd scheduling semantics, systemd cgroup drivers, 1 of many container runtimes, CNI, CSI, and more in order to cordon off and network a section of their system.

With Aurae a user only needs awareness of a single binary which will safely do all of the above in a secure way by default.

## Introducing Aurae Cells 

An Aurae Cell is just a group of processes running in a unique cgroup with explicit deny-by-default access to host namespaces.

Additionally, the processes running in a cell will share namespaces, which mirrors how Kubernetes runs containers in a pod. This implies that processes will be able to communicate over the Linux loopback interface (localhost), and share storage between them.

These processes can be grouped together and executed beside each other.
Most users will recognize this pattern as the pattern that has enabled [the sidecar pattern](https://www.oreilly.com/library/view/designing-distributed-systems/9781491983638/ch02.html).

![cell](/assets/img/blog-cell.png)

Because Aurae intends to manage every process on a system, Aurae will be able to make trustworthy guarantees and offer expressive controls over how a host is broken into cells.

![cells](/assets/img/blog-cells.png)

### Executables

Aurae will be able to execute regular old shell processes in a cell. We call these each of these basic processes an `Executable`.

```protobuf
  rpc RunExecutable(Executable) returns (ExecutableStatus) {}
```

### Container Cells

Additionally, Aurae will be able to execute OCI compliant container images in a cell which we just call a `Cell`.

```protobuf
  rpc RunCell(Cell) returns (CellStatus) {}
```

Regardless of if an administrator is executing a basic process, or a container: Aurae will manage the underlying cgroup and namespace implementation.

## Introducing Virtualization

Taking a step back from containerization we also understand that many enterprise users will need to execute untrusted code at scale.
Aurae additionally acts as a lightweight virtualization hypervisor and meta-data service in addition to being a cgroup broker.

Each instance of Aurae comes with its own running PID 1 daemon called `auraed`.

![cells](/assets/img/blog-instance.png)

### Understanding Virtualization

Virtualization is a more secure level of isolation that operates closer to the hardware. The boundary between a host and a guest virtualized workload is layer 3 of networking, and block patterns in storage. This more abstract interface creates a much more resilient environment for executing a workload.

```protobuf
  rpc RunVirtualMachine(VirtualMachine) returns (VirtualMachineStatus) {}
```

### MicroVMs with Aurae

Aurae brings the short-lived, destroy on exit (MicroVM) paradigm into scope by embedding the [firecracker](https://github.com/firecracker-microvm/firecracker) Rust crates directly and scheduling workloads with the [KVM](https://www.linux-kvm.org/page/Main_Page) or Kernel-based Virtual Machine.

Aurae is able to `Spawn()` a new `Instance` of itself into a newly created MicroVM which can be used arbitrarily.

### Aurae Spawn 

The name `Spawn()` is taken from the Rust `std::process` crate and resembles a pattern what most Linux users will know as `unshare(2)` or namespace delegation. Basically a spawned instance of Aurae will inherit certain properties from the parent, and will come with a few basic guarantees with regard to security and connectivity.

Aurae is designed to be recursive, which enables nested isolation zones and gives the project the basic building blocks it needs to hold an opinion on how users should run workloads.

Spawned Aurae instances will receive a bridged TAP network device which a nested `auraed` daemon will listen on by default. This allows a parent Aurae instance running with an independent kernel to communicate directly with a child instance over the same mTLS authenticated gRPC API the rest of the project leverages.

```protobuf
  rpc Spawn(Instance) returns (InstanceStatus) {}
```

Aurae will manage creating an ephemeral [SPIFFE](https://github.com/spiffe/spiffe) service identity for each spawned instance and will delegate down kernel images, `initramfs`, and even the `auraed` daemon itself.

Aurae manages the `Spawn()` including the networking bridge, and service identity management transparently at runtime.

![cells](/assets/img/blog-spawn.png)

**Note**: In the case that virtualization is not available on the host (e.g. nested virtualization in the cloud), Aurae will spawn directly into an isolated Cell.

### Virtual Machines with Aurae

Because Aurae will have the capability to `Spawn()` itself using the KVM, it is also possible to expose raw virtual machine functionality for users who wish to leverage Aurae as a long-lived hypervisor as well. Because Aurae maintains its own concept of system state as well as all of the cells on a system it is possible to break up a single host in many ways, with many isolation possibilities. 

```protobuf
  rpc RunVirtualMachine(VirtualMachine) returns (VirtualMachineStatus) {}
```

## Pods

Finally, we have the vocabulary needed to explain how an Aurae pod is unique.

An Aurae pod is a `Cell` running in a spawned Aurae `Instance`. 

```protobuf
  rpc RunPod(Pod) returns (PodStatus) {}
```

First Aurae will spawn a new instance of itself. Next Aurae will bridge to the spawned instance, and establish connectivity as a client to the new instance. The parent will then run a cell in the newly spawned Aurae instance.

Because Aurae is acts as a hypervisor this gives an operator the ability to mount network devices directly into the spawned instance, which can be referenced from the nested cell.

We believe this pattern to be a more flexible, secure, and efficient pattern which can be leveraged in place of traditional sidecar style mesh networking that is often seen with service mesh projects such as [Istio](https://github.com/istio/istio). 

From the original client's perspective scheduling a pod will feel natural, and will still expose basic fields such as OCI image, listen port, etc. Users can run a pod with Aurae, and the extra isolation layer should be transparent and free just by executing the `RunPod` gRPC function.

**Note**: The project has decided not to support the Kubernetes Pod API directly at this layer of the stack.

## What's Next?

The project is under active development, and many of the features described in this blog are currently a work in progress.

If you are interested in helping us work on these features please feel welcome to [join the discord](https://discord.gg/aTe2Rjg5rq) where we discuss our progress.

If you are interested in contributing please see the [getting involved](https://aurae.io/community/#getting-involved) documentation.

If you are interested in finding areas to contribute please see our [good first issues](https://github.com/aurae-runtime/aurae/issues?q=is%3Aopen+is%3Aissue+label%3A%22Good+First+Issue%22) which are designed to be easy for a newcomer to pick up and get started with. 

f you are interested in discussing product opportunities, or venture funding we unfortunately are not taking these discussions at this time. Our intention is to keep Aurae free and community driven.

---

_Author: [Kris Nóva](https://github.com/krisnova)_
