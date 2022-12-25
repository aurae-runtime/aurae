pub use executable::Executable;
pub use executable_name::ExecutableName;
use nix::sched::CloneFlags;
use std::ffi::CString;

mod executable;
mod executable_name;

pub struct ExecutableSpec {
    pub name: ExecutableName,
    pub command: CString,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SharedNamespaces {
    pub mount: bool,
    pub uts: bool,
    pub ipc: bool,
    pub pid: bool,
    pub net: bool,
    pub cgroup: bool,
}

impl From<&SharedNamespaces> for CloneFlags {
    fn from(shared_namespaces: &SharedNamespaces) -> Self {
        // Docs: https://man7.org/linux/man-pages/man2/clone.2.html
        let mut flags: CloneFlags = CloneFlags::empty();

        // If CLONE_NEWNS is set, the cloned child is started in a
        // new mount namespace, initialized with a copy of the
        // namespace of the parent.  If CLONE_NEWNS is not set, the
        // child lives in the same mount namespace as the parent.
        if !shared_namespaces.mount {
            flags.set(CloneFlags::CLONE_NEWNS, true);
        }

        //If CLONE_NEWUTS is set, then create the process in a new
        // UTS namespace, whose identifiers are initialized by
        // duplicating the identifiers from the UTS namespace of the
        // calling process.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same UTS namespace
        // as the calling process.
        if !shared_namespaces.uts {
            flags.set(CloneFlags::CLONE_NEWUTS, true);
        }

        // If CLONE_NEWIPC is set, then create the process in a new
        // IPC namespace.  If this flag is not set, then (as with
        // fork(2)), the process is created in the same IPC namespace
        // as the calling process.
        if !shared_namespaces.ipc {
            flags.set(CloneFlags::CLONE_NEWIPC, true);
        }

        // If CLONE_NEWPID is set, then create the process in a new
        // PID namespace.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same PID namespace
        // as the calling process.
        if !shared_namespaces.pid {
            flags.set(CloneFlags::CLONE_NEWPID, true);
        }

        // If CLONE_NEWNET is set, then create the process in a new
        // network namespace.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same network
        // namespace as the calling process.
        if !shared_namespaces.net {
            flags.set(CloneFlags::CLONE_NEWNET, true);
        }

        // If this flag is not set, then (as with fork(2)) the process is
        // created in the same cgroup namespaces as the calling
        // process.
        if !shared_namespaces.cgroup {
            flags.set(CloneFlags::CLONE_NEWCGROUP, true);
        }

        flags
    }
}
