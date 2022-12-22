use crate::runtime::cells::validation::ValidatedCell;
use crate::runtime::cells::ExecutableName;
use cgroups_rs::CgroupPid;
use std::io;
use sys_mount::*;
use tracing::info;
use unshare::Command;
use unshare::Error;
use unshare::ExitStatus;
use unshare::{Child, Namespace};

#[derive(Debug)]
pub(crate) struct Executable {
    pub name: ExecutableName,
    pub command: String,
    pub args: Vec<String>,
    pub description: String,
    state: ExecutableState,
}

#[derive(Debug)]
enum ExecutableState {
    Init,
    Started(Child),
    Stopped(ExitStatus),
}

impl Executable {
    pub fn new(
        name: ExecutableName,
        command: String,
        args: Vec<String>,
        description: String,
    ) -> Self {
        Self { name, command, args, description, state: ExecutableState::Init }
    }

    /// Starts the executable and returns the pid.
    /// If the executable is already started, just returns the pid.
    pub fn start(&mut self, spec: ValidatedCell) -> Result<CgroupPid, Error> {
        match &self.state {
            ExecutableState::Started(child) => Ok((child.id() as u64).into()),

            ExecutableState::Init | ExecutableState::Stopped(_) => {
                let mut command = Command::new(&self.command);
                let command = command.args(&self.args);

                // Namespaces
                //
                // TODO We need to track the namespace for all newly
                //      unshared namespaces within a Cell such that
                //      we can call command.set_namespace() for
                //      each of the new namespaces at the cell level!
                //      This will likely require changing how Cells
                //      manage namespaces as we need to cache the namespace
                //      IDs (names?)
                //
                // TODO Basically once a namespace has been created for a Cell
                //      we should put ALL future executables into the same namespace!
                let mut namespaces_to_unshare: Vec<Namespace> = vec![];
                // Note: The logic here is reversed. We define the flags as "share'
                //       and map them to "unshare".
                //       This is by design as the API has a concept of "share".
                if !spec.ns_share_mount {
                    info!("Unshare: mount");
                    namespaces_to_unshare.push(Namespace::Mount)
                }
                if !spec.ns_share_uts {
                    info!("Unshare: uts");
                    namespaces_to_unshare.push(Namespace::Uts)
                }
                if !spec.ns_share_ipc {
                    info!("Unshare: ipc");
                    namespaces_to_unshare.push(Namespace::Ipc)
                }
                if !spec.ns_share_pid {
                    info!("Unshare: pid");
                    namespaces_to_unshare.push(Namespace::Pid)
                }
                if !spec.ns_share_net {
                    info!("Unshare: net");
                    namespaces_to_unshare.push(Namespace::Net)
                }
                if !spec.ns_share_cgroup {
                    info!("Unshare: cgroup");
                    namespaces_to_unshare.push(Namespace::Cgroup)
                }

                let command = command.unshare(&namespaces_to_unshare);

                // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
                let command = {
                    let executable_name = self.name.clone();
                    unsafe {
                        command.pre_exec(move || {
                            pre_exec(&executable_name, spec.clone())
                        })
                    }
                };

                let child = command.spawn()?;
                let pid = (child.id() as u64).into();
                self.state = ExecutableState::Started(child);

                Ok(pid)
            }
        }
    }

    /// Stops the executable and returns the `ExitStatus`.
    /// If the executable has never been started, returns `None`.
    pub fn kill(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        match &mut self.state {
            ExecutableState::Init => Ok(None),
            ExecutableState::Started(child) => {
                let id = child.id();
                child.kill()?;
                let exit_status = child.wait()?;

                info!(
                    "Executable with pid {id} exited with status {exit_status}",
                );

                self.state = ExecutableState::Stopped(exit_status);
                Ok(Some(exit_status))
            }
            ExecutableState::Stopped(exit_status) => Ok(Some(*exit_status)),
        }
    }

    /// Returns the `pid` while `Executable` is running, otherwise returns `None`.
    pub fn pid(&self) -> Option<CgroupPid> {
        match &self.state {
            ExecutableState::Started(child) => Some((child.id() as u64).into()),
            ExecutableState::Init | ExecutableState::Stopped(_) => None,
        }
    }
}

/// Common functionality within the context of the new executable
fn pre_exec(
    executable_name: &ExecutableName,
    spec: ValidatedCell,
) -> io::Result<()> {
    info!("CellService: pre_exec(): {executable_name}");
    // Here we are executing as the new spawned pid.
    // This is a place where we can "hook" into all processes
    // started with Aurae in the future. Similar to kprobe/uprobe
    // in Linux or LD_PRELOAD in libc.

    // In the event we are not sharing the mount namespace or the pid namespace
    // with the host, we will manually mount /proc
    if !spec.ns_share_pid && !spec.ns_share_mount {
        info!("CellService: pre_exec(): mounting procfs /proc");
        // TODO Implement mount proc
        // TODO validate this logic is the correct logic for mounting proc in our new namespace isolation zone
    }

    Ok(())
}
