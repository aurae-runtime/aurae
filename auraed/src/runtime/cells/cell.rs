/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use super::{
    validation::ValidatedCell, CellsError, Executable, ExecutableName, Result,
};
use crate::runtime::cells::cell_name::CellName;
use cgroups_rs::{
    cgroup_builder::CgroupBuilder, hierarchies, Cgroup, CgroupPid, Hierarchy,
};
// use log::error;
// use std::borrow::Borrow;
// use std::fs::File;
use std::ops::Deref;
// use std::os::fd::AsRawFd;
use std::{collections::HashMap, io, process::ExitStatus};
use tracing::info;

// We should not be able to change a cell after it has been created.
// You must free the cell and create a new one if you want to change anything about the cell.
// In order to facilitate that immutability:
// NEVER MAKE THE FIELDS PUB (OF ANY KIND)
#[derive(Debug)]
pub(crate) struct Cell {
    spec: ValidatedCell,
    state: CellState,
}

#[derive(Debug)]
enum CellState {
    Unallocated,
    Allocated {
        cgroup: Cgroup,
        executables: HashMap<ExecutableName, Executable>,
    },
    Freed,
}

impl Cell {
    pub fn new(cell_spec: ValidatedCell) -> Self {
        Self { spec: cell_spec, state: CellState::Unallocated }
    }

    /// Creates the underlying cgroup.
    /// Does nothing if [Cell] has been previously allocated.
    // Here is where we define the "default" cgroup parameters for Aurae cells
    pub fn allocate(&mut self) {
        let CellState::Unallocated = &self.state else {
            return;
        };

        let ValidatedCell {
            name,
            cpu_cpus,
            cpu_shares,
            cpu_mems,
            cpu_quota,
            ns_share_mount: _ns_share_mount,
            ns_share_uts: _ns_share_uts,
            ns_share_ipc: _ns_share_ipc,
            ns_share_pid: _ns_share_pid,
            ns_share_net: _ns_share_net,
            ns_share_cgroup: _ns_share_cgroup,
        } = self.spec.clone();

        let hierarchy = hierarchy();
        let cgroup = CgroupBuilder::new(&name)
            // CPU Controller
            .cpu()
            .shares(cpu_shares.into_inner())
            .mems(cpu_mems.into_inner())
            .period(1000000) // microseconds in a second
            .quota(cpu_quota.into_inner())
            .cpus(cpu_cpus.into_inner())
            .done()
            // Final Build
            .build(hierarchy);

        self.state =
            CellState::Allocated { cgroup, executables: Default::default() }
    }

    /// Deletes the underlying cgroup.
    /// A [Cell] should never be reused after calling [free].
    pub fn free(&mut self) -> Result<()> {
        if let CellState::Allocated { cgroup, executables: _ } = &mut self.state
        {
            cgroup.delete().map_err(|e| CellsError::FailedToFreeCell {
                cell_name: self.spec.name.clone(),
                source: e,
            })?;
        }

        // set cell state to freed, independent of the current state
        self.state = CellState::Freed;

        Ok(())
    }

    pub fn start_executable<T: Into<Executable>>(
        &mut self,
        executable: T,
    ) -> Result<i32> {
        let CellState::Allocated { cgroup, executables } = &mut self.state else {
            return Err(CellsError::CellNotAllocated {
                cell_name: self.spec.name.clone(),
            });
        };

        let executable = executable.into();

        // TODO: replace with try_insert when it becomes stable
        // Check if there was already an executable with the same name.
        if executables.contains_key(&executable.name) {
            return Err(CellsError::ExecutableExists {
                cell_name: self.spec.name.clone(),
                executable_name: executable.name,
            });
        }

        let executable_name = executable.name.clone();
        let other_executable_pid = executables
            .deref()
            .iter()
            .filter_map(|(other_executable_name, other_executable)| {
                if *other_executable_name != executable_name {
                    other_executable.pid()
                } else {
                    None
                }
            })
            .next();

        // `or_insert` will always insert as we've already assured ourselves that the key does not exist.
        let executable =
            executables.entry(executable.name.clone()).or_insert(executable);

        // Start the child process
        //
        // Here is where we launch an executable within the context of a parent Cell.
        // Aurae makes the assumption that all Executables within a cell share the
        // same namespace isolation rules set up upon creation of the cell.
        let spec = self.spec.clone();
        let pre_exec = move || {
            pre_exec(&executable_name, &spec, other_executable_pid.as_ref())
        };

        let pid = executable.start(Some(pre_exec)).map_err(|e| {
            CellsError::FailedToStartExecutable {
                cell_name: self.spec.name.clone(),
                executable_name: executable.name.clone(),
                command: executable.command.clone(),
                args: executable.args.clone(),
                source: e,
            }
        })?;

        // TODO: We've inserted the executable into our in-memory cache, and started it,
        //   but we've failed to move it to the Cell...bad...solution?
        if let Err(e) = cgroup.add_task(pid.pid.into()) {
            return Err(CellsError::FailedToAddExecutableToCell {
                cell_name: self.spec.name.clone(),
                executable_name: executable.name.clone(),
                source: e,
            });
        }

        info!(
            "Cells: cell_name={} executable_name={} spawn() -> pid={pid:?}",
            self.spec.name, executable.name
        );

        Ok(pid.pid as i32)
    }

    pub fn stop_executable(
        &mut self,
        executable_name: &ExecutableName,
    ) -> Result<Option<ExitStatus>> {
        let CellState::Allocated { executables, .. } = &mut self.state else {
            // TODO: Do we want to check the system to confirm?
            return Err(CellsError::CellNotAllocated {
                cell_name: self.spec.name.clone(),
            });
        };

        let Some(executable) = executables.get_mut(executable_name) else {
            return Err(CellsError::ExecutableNotFound {
                cell_name: self.spec.name.clone(),
                executable_name: executable_name.clone(),
            });
        };

        match executable.kill() {
            Ok(exit_status) => {
                let _ = executables
                    .remove(executable_name)
                    .expect("asserted above");

                Ok(exit_status)
            }
            Err(e) => Err(CellsError::FailedToStopExecutable {
                cell_name: self.spec.name.clone(),
                executable_name: executable.name.clone(),
                executable_pid: executable.pid().expect("pid"),
                source: e,
            }),
        }
    }

    /// Returns the [CellName] of the [Cell]
    pub fn name(&self) -> &CellName {
        &self.spec.name
    }

    /// Returns [None] if the [Cell] is not allocated.
    pub fn v2(&self) -> Option<bool> {
        match &self.state {
            CellState::Allocated { cgroup, .. } => Some(cgroup.v2()),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn new_for_tests(name: Option<CellName>) -> Self {
        use validation::ValidatedType;

        let cell_name = name.unwrap_or_else(|| CellName::random_for_tests());

        let cell = aurae_proto::runtime::Cell {
            name: cell_name.into_inner(),
            cpu_cpus: "".to_string(),
            cpu_shares: 0,
            cpu_mems: "".to_string(),
            cpu_quota: 0,
            ns_share_mount: false,
            ns_share_uts: false,
            ns_share_ipc: false,
            ns_share_pid: false,
            ns_share_net: false,
            ns_share_cgroup: false,
        };
        let cell = ValidatedCell::validate(cell, None).expect("invalid cell");
        cell.into()
    }
}

#[cfg(test)]
impl Drop for Cell {
    /// A [Cell] leaves a cgroup behind so we call [free] on drop
    fn drop(&mut self) {
        let _best_effort = self.free();
    }
}

fn hierarchy() -> Box<dyn Hierarchy> {
    // Auraed will assume the V2 cgroup hierarchy by default.
    // For now we do not change this, albeit in theory we could
    // likely create backwards compatability for V1 hierarchy.
    //
    // For now, we simply... don't.
    // hierarchies::auto() // Uncomment to auto detect Cgroup hierarchy
    // hierarchies::V2
    Box::new(hierarchies::V2::new())
}

/// Common functionality within the context of the new executable
fn pre_exec(
    executable_name: &ExecutableName,
    spec: &ValidatedCell,
    other_executable_pid: Option<&CgroupPid>,
) -> io::Result<()> {
    match other_executable_pid {
        None => pre_exec_first_exe(executable_name, spec),
        Some(other_executable_pid) => {
            pre_exec_other_exes(executable_name, other_executable_pid)
        }
    }
}

fn pre_exec_first_exe(
    executable_name: &ExecutableName,
    spec: &ValidatedCell,
) -> io::Result<()> {
    info!("CellService: pre_exec_first_exe(): {executable_name}");
    // Here we are executing as the new spawned pid.
    // This is a place where we can "hook" into all processes
    // started with Aurae in the future. Similar to kprobe/uprobe
    // in Linux or LD_PRELOAD in libc.

    pre_exec_unshare(executable_name, spec)?;

    if !spec.ns_share_pid && !spec.ns_share_mount {
        pre_exec_mount_proc(executable_name)?;
    }

    Ok(())
}

/// Common functionality within the context of the new executable
fn pre_exec_other_exes(
    executable_name: &ExecutableName,
    _other_executable_pid: &CgroupPid,
) -> io::Result<()> {
    info!("CellService: pre_exec_other_exes(): {executable_name}");

    // Does not work yet
    // pre_exec_set_ns(executable_name, other_executable_pid)?;

    // TODO: mount?

    Ok(())
}

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
fn pre_exec_unshare(
    executable_name: &ExecutableName,
    spec: &ValidatedCell,
) -> io::Result<()> {
    info!("CellService: pre_exec_unshare(): {executable_name}");

    // Note: The logic here is reversed. We define the flags as "share'
    //       and map them to "unshare".
    //       This is by design as the API has a concept of "share".
    if !spec.ns_share_mount {
        info!("Unshare: mount");
        if let Err(err_no) =
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNS)
        {
            return Err(io::Error::from_raw_os_error(err_no as i32));
        }
    }
    if !spec.ns_share_uts {
        info!("Unshare: uts");
        if let Err(err_no) =
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWUTS)
        {
            return Err(io::Error::from_raw_os_error(err_no as i32));
        }
    }
    if !spec.ns_share_ipc {
        info!("Unshare: ipc");
        if let Err(err_no) =
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWIPC)
        {
            return Err(io::Error::from_raw_os_error(err_no as i32));
        }
    }
    if !spec.ns_share_pid {
        info!("Unshare: pid");
        if let Err(err_no) =
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWPID)
        {
            return Err(io::Error::from_raw_os_error(err_no as i32));
        }
    }
    if !spec.ns_share_net {
        info!("Unshare: net");
        if let Err(err_no) =
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNET)
        {
            return Err(io::Error::from_raw_os_error(err_no as i32));
        }
    }

    if !spec.ns_share_cgroup {
        info!("Unshare: cgroup");
        if let Err(err_no) =
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWCGROUP)
        {
            return Err(io::Error::from_raw_os_error(err_no as i32));
        }
    }

    Ok(())
}

// // This errors
// fn pre_exec_set_ns(
//     executable_name: &ExecutableName,
//     other_executable_pid: &CgroupPid,
// ) -> io::Result<()> {
//     info!("CellService: pre_exec_setns(): {executable_name}");
//
//     let process = match other_executable_pid.pid.try_into() {
//         Ok(pid) => procfs::process::Process::new(pid),
//         Err(e) => {
//             error!("{}", e);
//             return Err(io::Error::new(io::ErrorKind::Other, e));
//         }
//     }
//     .map_err(|e| {
//         error!("{}", e);
//         io::Error::new(io::ErrorKind::Other, e)
//     })?;
//
//     for (name, ns) in process
//         .namespaces()
//         .map_err(|e| {
//             error!("{}", e);
//             io::Error::new(io::ErrorKind::Other, e)
//         })?
//         .into_iter()
//     {
//         let file = File::open(ns.path)?;
//         let fd = file.as_raw_fd();
//
//         match name.to_string_lossy().borrow() {
//             "ns" => {
//                 if let Err(err_no) =
//                     nix::sched::setns(fd, nix::sched::CloneFlags::CLONE_NEWNS)
//                 {
//                     return Err(io::Error::from_raw_os_error(err_no as i32));
//                 }
//             }
//             "uts" => {
//                 if let Err(err_no) =
//                     nix::sched::setns(fd, nix::sched::CloneFlags::CLONE_NEWUTS)
//                 {
//                     return Err(io::Error::from_raw_os_error(err_no as i32));
//                 }
//             }
//             "ipc" => {
//                 if let Err(err_no) =
//                     nix::sched::setns(fd, nix::sched::CloneFlags::CLONE_NEWIPC)
//                 {
//                     return Err(io::Error::from_raw_os_error(err_no as i32));
//                 }
//             }
//             "pid" => {
//                 if let Err(err_no) =
//                     nix::sched::setns(fd, nix::sched::CloneFlags::CLONE_NEWPID)
//                 {
//                     return Err(io::Error::from_raw_os_error(err_no as i32));
//                 }
//             }
//             "net" => {
//                 if let Err(err_no) =
//                     nix::sched::setns(fd, nix::sched::CloneFlags::CLONE_NEWNET)
//                 {
//                     return Err(io::Error::from_raw_os_error(err_no as i32));
//                 }
//             }
//             "cgroup" => {
//                 if let Err(err_no) = nix::sched::setns(
//                     fd,
//                     nix::sched::CloneFlags::CLONE_NEWCGROUP,
//                 ) {
//                     return Err(io::Error::from_raw_os_error(err_no as i32));
//                 }
//             }
//             other => {
//                 info!("Other namespace: {other}")
//             }
//         }
//     }
//
//     Ok(())
// }

fn pre_exec_mount_proc(executable_name: &ExecutableName) -> io::Result<()> {
    info!("CellService: pre_exec_mount_proc(): {executable_name}");

    if let Err(err_no) = nix::mount::mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        nix::mount::MsFlags::empty(),
        None::<&[u8]>,
    ) {
        return Err(io::Error::from_raw_os_error(err_no as i32));
    }

    // TODO validate this logic is the correct logic for mounting proc in our new namespace isolation zone

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_cant_unfree() {
        let mut cell = Cell::new_for_tests(None);
        assert!(matches!(cell.state, CellState::Unallocated));

        cell.allocate();
        assert!(matches!(cell.state, CellState::Allocated { .. }));

        cell.free().expect("failed to free");
        assert!(matches!(cell.state, CellState::Freed));

        // Calling allocate again should do nothing
        cell.allocate();
        assert!(matches!(cell.state, CellState::Freed));
    }
}
