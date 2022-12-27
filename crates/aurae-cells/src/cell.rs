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

use crate::{CellName, CellSpec, CellsError, Result};
use aurae_client::AuraeConfig;
use aurae_executables::{Executable, ExecutableName, ExecutableSpec};
use cgroups_rs::Cgroup;
use std::process::Command;
use tracing::info;
use validation::ValidatedField;

// We should not be able to change a cell after it has been created.
// You must free the cell and create a new one if you want to change anything about the cell.
// In order to facilitate that immutability:
// NEVER MAKE THE FIELDS PUB (OF ANY KIND)
#[derive(Debug)]
pub struct Cell {
    name: CellName,
    spec: CellSpec,
    state: CellState,
}

// TODO: look into clippy warning
// TODO: remove #[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
#[derive(Debug)]
enum CellState {
    Unallocated,
    Allocated { cgroup: Cgroup, pid1: Pid1 },
    Freed,
}

#[derive(Debug)]
struct Pid1 {
    #[allow(unused)]
    auraed: Executable,
    client_config: AuraeConfig,
}

impl Cell {
    pub fn new(name: CellName, cell_spec: CellSpec) -> Self {
        Self { name, spec: cell_spec, state: CellState::Unallocated }
    }

    /// Creates the underlying cgroup.
    /// Does nothing if [Cell] has been previously allocated.
    // Here is where we define the "default" cgroup parameters for Aurae cells
    pub fn allocate(&mut self) -> Result<()> {
        let CellState::Unallocated = &self.state else {
            return Ok(());
        };

        let cgroup: Cgroup =
            self.spec.cgroup_spec.clone().into_cgroup(&self.name);

        // Launch nested Auraed
        //
        // Here we launch a nested auraed with the --nested flag
        // which is used our way of "hooking" into the newly created
        // aurae isolation zone.
        //
        // TODO: Consider changing "--nested" to "--nested-cell" or similar
        // TODO: handle expect
        // TODO: Pull nested auraed command into a deterministic function EG: nested_cell_command()
        let mut client_config =
            AuraeConfig::try_default().expect("file based config");
        client_config.system.socket =
            format!("/var/run/aurae/aurae-{}.sock", uuid::Uuid::new_v4());

        let mut command = Command::new("auraed");
        let _ = command.args([
            "--socket",
            &client_config.system.socket,
            "--nested",
        ]);

        // We are checking that command has an arg to assure ourselves that `command.arg`
        // mutates command, and is not making a clone to return
        // We have a concern that the "command" API make change/break in the future and this
        // test is intended to help safeguard against that!
        assert_eq!(command.get_args().len(), 3);

        // Create the nested Auraed executable
        let executable_spec = ExecutableSpec {
            // TODO: don't require use of validate
            name: ExecutableName::validate(Some("auraed".into()), "", None)
                .expect("valid executable name"),
            command,
            description: "nested auraed".to_string(),
        };

        // TODO: Its only a "new pid 1" if we unshare the pid namespace, otherwise its just a new process.
        let mut auraed = Executable::new_pid1(
            executable_spec,
            self.spec.shared_namespaces.clone(),
        );

        auraed.start().map_err(|e| CellsError::FailedToAllocateCell {
            cell_name: self.name.clone(),
            source: e,
        })?;

        let pid = auraed.pid().expect("pid");

        println!("auraed pid {}", pid);

        if let Err(e) = cgroup.add_task((pid.as_raw() as u64).into()) {
            // TODO: what if free also fails?
            let _ = self.free();

            return Err(CellsError::AbortedAllocateCell {
                cell_name: self.name.clone(),
                source: e,
            });
        }

        println!("inserted auraed pid {}", pid);

        self.state = CellState::Allocated {
            cgroup,
            pid1: Pid1 { auraed, client_config },
        };

        Ok(())
    }

    /// Deletes the underlying cgroup.
    /// A [Cell] should never be reused after calling [free].
    pub fn free(&mut self) -> Result<()> {
        // TODO send SIGKILL to nested auraed before destroying the cgroup or the cgroup wont delete properly
        // TODO In the future, use SIGINT intstead of SIGKILL once https://github.com/aurae-runtime/aurae/issues/199 is ready
        // TODO nested auraed should proxy (bus) POSIX signals to child executables
        if let CellState::Allocated { cgroup, .. } = &mut self.state {
            cgroup.delete().map_err(|e| CellsError::FailedToFreeCell {
                cell_name: self.name.clone(),
                source: e,
            })?;
        }

        // set cell state to freed, independent of the current state
        self.state = CellState::Freed;

        Ok(())
    }

    // NOTE: Having this function return the AuraeClient means we need to make it async,
    // or we need to make [AuraeClient::new] not async.
    pub fn client_config(&self) -> Result<AuraeConfig> {
        let CellState::Allocated { pid1, .. } = &self.state else {
            return Err(CellsError::CellNotAllocated {
                cell_name: self.name.clone(),
            })
        };

        Ok(pid1.client_config.clone())
    }

    /// Returns the [CellName] of the [Cell]
    pub fn name(&self) -> &CellName {
        &self.name
    }

    /// Returns [None] if the [Cell] is not allocated.
    pub fn v2(&self) -> Option<bool> {
        info!("{:?}", self);
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
//
// /// Common functionality within the context of the new executable
// fn pre_exec(
//     executable_name: &ExecutableName,
//     spec: &ValidatedCell,
//     other_executable_pid: Option<&CgroupPid>,
// ) -> io::Result<()> {
//     match other_executable_pid {
//         None => pre_exec_first_exe(executable_name, spec),
//         Some(other_executable_pid) => {
//             pre_exec_other_exes(executable_name, other_executable_pid)
//         }
//     }
// }
//
// fn pre_exec_first_exe(
//     executable_name: &ExecutableName,
//     spec: &ValidatedCell,
// ) -> io::Result<()> {
//     info!("CellService: pre_exec_first_exe(): {executable_name}");
//     // Here we are executing as the new spawned pid.
//     // This is a place where we can "hook" into all processes
//     // started with Aurae in the future. Similar to kprobe/uprobe
//     // in Linux or LD_PRELOAD in libc.
//
//     pre_exec_unshare(executable_name, spec)?;
//
//     if !spec.ns_share_pid && !spec.ns_share_mount {
//         pre_exec_mount_proc(executable_name)?;
//     }
//
//     Ok(())
// }
//
// /// Common functionality within the context of the new executable
// fn pre_exec_other_exes(
//     executable_name: &ExecutableName,
//     _other_executable_pid: &CgroupPid,
// ) -> io::Result<()> {
//     info!("CellService: pre_exec_other_exes(): {executable_name}");
//
//     // Does not work yet
//     // pre_exec_set_ns(executable_name, other_executable_pid)?;
//
//     // TODO: mount?
//
//     Ok(())
// }
//
// // Namespaces
// //
// // TODO We need to track the namespace for all newly
// //      unshared namespaces within a Cell such that
// //      we can call command.set_namespace() for
// //      each of the new namespaces at the cell level!
// //      This will likely require changing how Cells
// //      manage namespaces as we need to cache the namespace
// //      IDs (names?)
// //
// // TODO Basically once a namespace has been created for a Cell
// //      we should put ALL future executables into the same namespace!
// fn pre_exec_unshare(
//     executable_name: &ExecutableName,
//     spec: &ValidatedCell,
// ) -> io::Result<()> {
//     info!("CellService: pre_exec_unshare(): {executable_name}");
//
//     // Note: The logic here is reversed. We define the flags as "share'
//     //       and map them to "unshare".
//     //       This is by design as the API has a concept of "share".
//     if !spec.ns_share_mount {
//         info!("Unshare: mount");
//         if let Err(err_no) =
//             nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNS)
//         {
//             return Err(io::Error::from_raw_os_error(err_no as i32));
//         }
//     }
//     if !spec.ns_share_uts {
//         info!("Unshare: uts");
//         if let Err(err_no) =
//             nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWUTS)
//         {
//             return Err(io::Error::from_raw_os_error(err_no as i32));
//         }
//     }
//     if !spec.ns_share_ipc {
//         info!("Unshare: ipc");
//         if let Err(err_no) =
//             nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWIPC)
//         {
//             return Err(io::Error::from_raw_os_error(err_no as i32));
//         }
//     }
//     if !spec.ns_share_pid {
//         info!("Unshare: pid");
//         if let Err(err_no) =
//             nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWPID)
//         {
//             return Err(io::Error::from_raw_os_error(err_no as i32));
//         }
//     }
//     if !spec.ns_share_net {
//         info!("Unshare: net");
//         if let Err(err_no) =
//             nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNET)
//         {
//             return Err(io::Error::from_raw_os_error(err_no as i32));
//         }
//     }
//
//     if !spec.ns_share_cgroup {
//         info!("Unshare: cgroup");
//         if let Err(err_no) =
//             nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWCGROUP)
//         {
//             return Err(io::Error::from_raw_os_error(err_no as i32));
//         }
//     }
//
//     Ok(())
// }

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
//
// fn pre_exec_mount_proc(executable_name: &ExecutableName) -> io::Result<()> {
//     info!("CellService: pre_exec_mount_proc(): {executable_name}");
//
//     if let Err(err_no) = nix::mount::mount(
//         Some("proc"),
//         "/proc",
//         Some("proc"),
//         nix::mount::MsFlags::MS_BIND,
//         None::<&[u8]>,
//     ) {
//         return Err(io::Error::from_raw_os_error(err_no as i32));
//     }
//     Ok(())
// }

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
