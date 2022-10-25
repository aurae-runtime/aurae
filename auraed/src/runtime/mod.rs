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
/*
 * [Runtime] is a SYNCHRONOUS subsystem.
 */

#![allow(dead_code)]
tonic::include_proto!("runtime");

use crate::runtime::core_server::Core;
use crate::{cell_name_from_string, command_from_string, meta};
use anyhow::Result;
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::*;
use cgroups_rs::{CgroupPid, Controller};
use log::{debug, info};
use std::io::Read;
use std::process::Stdio;
use tonic::{Request, Response, Status};

/// The server side implementation of the core runtime subsystem.
///
/// The Runtime subsystem is synchronous and will operate against
/// the system inline with any requests.
///
/// Warning: Because of the synchronous nature of the subsystem ths
/// part of the daemon is potentially vulnerable to denial of service
/// and flooding attacks.
#[derive(Debug, Default, Clone)]
pub struct CoreService {}

#[tonic::async_trait]
impl Core for CoreService {
    /// Create a cgroup based on the "name" of the executable and spawn a process inside.
    async fn run_executable(
        &self,
        request: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = request.into_inner();
        // let rmeta = r.meta.expect("parsing request meta");

        // Build command to execute.
        let mut cmd =
            command_from_string(&r.command).expect("command string parsing");

        // Calculate the ID for the cell to run the command in
        let cell_id = cell_name_from_string(&r.command)
            .expect("cell name hash calculation");

        info!("Spawning cell: {}", cell_id);

        // Create the cgroup for the cell
        let hierarchy = cgroups_rs::hierarchies::auto(); // v1/v2 cgroup switch automatically
        let cgroup: Cgroup = CgroupBuilder::new(&cell_id)
            .cpu()
            .shares(10) // Use 10% of the CPU relative to other cgroups
            .done()
            .build(hierarchy);
        // Attach the running command to the cgroup
        let controller: &cgroups_rs::cpu::CpuController =
            cgroup.controller_of().expect("cgroup controller");

        // Spawn the command
        let running = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn();
        match running {
            Ok(mut running) => {
                let pid = running.id();
                controller
                    .add_task(&CgroupPid::from(pid as u64))
                    .expect("attaching to cgroup");

                // Wait for the command to terminate
                let cell_exit_status =
                    running.wait().expect("waiting process termination");

                // Destroy the cgroup upon completion
                // Note: https://github.com/kata-containers/cgroups-rs/issues/92
                // Note: The library does not clean up the cgroup, so it needs to be
                // Note: removed manually.
                let controller_resp = controller.delete();
                if controller_resp.is_err() {
                    debug!("{:?}", controller_resp)
                }
                let cgroup_resp = cgroup.delete();
                if cgroup_resp.is_err() {
                    debug!("{:?}", cgroup_resp)
                }

                // Return the result synchronously
                let meta = meta::AuraeMeta {
                    name: r.command,
                    message: "".to_string(),
                };
                let proc = meta::ProcessMeta { pid: pid as i32 };
                let status = meta::Status::Complete as i32;

                // Parse stdout from pipe
                let mut stdout_val = String::new();
                running
                    .stdout
                    .expect("parse stdout")
                    .read_to_string(&mut stdout_val)
                    .expect("reading stdout");

                // Parse stderr from pipe
                let mut stderr_val = String::new();
                running
                    .stderr
                    .expect("parse stderr")
                    .read_to_string(&mut stderr_val)
                    .expect("reading stderr");

                let response = ExecutableStatus {
                    meta: Some(meta),
                    proc: Some(proc),
                    status,
                    stdout: stdout_val,
                    stderr: stderr_val,
                    exit_code: cell_exit_status.to_string(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let meta = meta::AuraeMeta {
                    name: "-".to_string(),
                    message: format!("failed spawning process: {:?}", e),
                };
                let proc = meta::ProcessMeta { pid: -1 };
                let status = meta::Status::Error as i32;
                let response = ExecutableStatus {
                    meta: Some(meta),
                    proc: Some(proc),
                    status,
                    stdout: "-".to_string(),
                    stderr: "-".to_string(),
                    exit_code: "-".to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    async fn run_pod(
        &self,
        _request: Request<Pod>,
    ) -> Result<Response<PodStatus>, Status> {
        todo!()
    }

    async fn spawn(
        &self,
        _request: Request<SpawnRequest>,
    ) -> Result<Response<SpawnResponse>, Status> {
        todo!()
    }

    async fn run_virtual_machine(
        &self,
        _request: Request<VirtualMachine>,
    ) -> Result<Response<VirtualMachineStatus>, Status> {
        todo!()
    }

    async fn run_cell(
        &self,
        _request: Request<Cell>,
    ) -> Result<Response<CellStatus>, Status> {
        todo!();
        // let syscall = create_syscall();
        // let mut container =
        //     ContainerBuilder::new("123".to_string(), syscall.as_ref())
        //         .as_init(PathBuf::new())
        //         .with_systemd(false)
        //         .build()
        //         .expect("building container");
        // // .with_pid_file(args.pid_file.as_ref())?
        // // .with_console_socket(args.console_socket.as_ref())
        // // .with_root_path(root_path)?
        // // .with_preserved_fds(args.preserve_fds)
        // // .as_init(&args.bundle)
        // // .with_systemd(false)
        // // .build()?;
        //
        // let _ = container.start();
        // let meta =
        //     meta::AuraeMeta { name: "-".to_string(), message: "-".to_string() };
        // let status = meta::Status::Complete as i32;
        // let container_statuses = vec![ContainerStatus {
        //     meta: Some(meta::AuraeMeta {
        //         name: "-".to_string(),
        //         message: "-".to_string(),
        //     }),
        //     status: meta::Status::Complete as i32,
        //     proc: Some(meta::ProcessMeta { pid: -1 }),
        // }];
        // let response =
        //     CellStatus { meta: Some(meta), status, container_statuses };
        // Ok(Response::new(response))
    }

    // async fn function_name(
    //     &self,
    //     _request: Request<Container>,
    // ) -> Result<Response<ContainerStatus>, Status> {
    //     todo!()
    // }
}
