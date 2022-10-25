# Isolation with Aurae Cells

Last week we merged [Pull Request #73](https://github.com/aurae-runtime/aurae/pull/73) which marks the project's decision to pursue building a new kind of isolation strategy with a concept we are calling **Cells**.

The concept of what Aurae will call a cell isn't anything new.

## What is an Aurae Cell?

An Aurae Cell is just a group of processes running in a unique cgroup.

However, Aurae needed a new name for cells as the scope of Aurae is more than just containers.

Additionally, Aurae also has a concept of "Pods" which we will explain later. First we need to understand what an Aurae Cell is, in order to understand how an "Aurae Pod" is unique.

To most container users, an Aurae Cell is nothing more than what Kubernetes calls a [Pod](https://kubernetes.io/docs/concepts/workloads/pods/#what-is-a-pod) or what Systemd has called a [service](https://www.freedesktop.org/software/systemd/man/systemd.service.html). 

We chose the name "Cell" because it starts with a "C" for "cgroup".

### Kubernetes Pods vs Systemd services

For most practical applications, the runtime semantics between a systemd service and a pod are relatively benign. A pod comes with some additional storage and networking primitives and makes a few assumptions about the intentions of the user, where as a systemd service is a little more lightweight and typically shares the host namespaces. The point is that at the end of the day they are still both implementations of processes running inside of a [control group](https://man7.org/linux/man-pages/man7/cgroups.7.html) or "cgroup" for short. What makes a "container" a "container" is the fact that the pods get a unique cgroup namespace, and often times a systemd service does not. 

However, they **both** run in a cgroup. In fact, you can see Kubernetes pods and systemd services side-by-side in the hierarchical cgroup directory on modern Linux distributions in [/sys](https://man7.org/linux/man-pages/man5/sysfs.5.html) or `sysfs(5)`

```bash 
[root@alice]: /sys/fs/cgroup># ls -d */
dev-hugepages.mount//  kubepods.slice//                 sys-kernel-config.mount//   system.slice//
dev-mqueue.mount//     pids//                           sys-kernel-debug.mount//    user.slice//
init.scope//           sys-fs-fuse-connections.mount//  sys-kernel-tracing.mount//
```
