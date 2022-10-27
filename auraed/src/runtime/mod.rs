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
use crate::command_from_string;
use crate::meta;
use crate::runtime::core_server::Core;
use anyhow::Result;
pub use api::runtime::*;
// use libcontainer::{
//     container::builder::ContainerBuilder, syscall::syscall::create_syscall,
// };
// use std::path::PathBuf;
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
    async fn run_executable(
        &self,
        request: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = request.into_inner();
        let cmd = command_from_string(&r.command);
        match cmd {
            Ok(mut cmd) => {
                let output = cmd.output();
                match output {
                    Ok(output) => {
                        let meta = meta::AuraeMeta {
                            name: r.command,
                            message: "-".to_string(),
                        };
                        let proc = meta::ProcessMeta { pid: -1 }; // todo @kris-nova get pid, we will probably want to spawn() and wait and remember the pid
                        let status = meta::Status::Complete as i32;
                        let response = ExecutableStatus {
                            meta: Some(meta),
                            proc: Some(proc),
                            status,
                            stdout: String::from_utf8(output.stdout)
                                .expect("reading stdout"),
                            stderr: String::from_utf8(output.stderr)
                                .expect("reading stderr"),
                            exit_code: output.status.to_string(),
                        };
                        Ok(Response::new(response))
                    }
                    Err(e) => {
                        let meta = meta::AuraeMeta {
                            name: "-".to_string(),
                            message: format!("{:?}", e),
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
            Err(e) => {
                let meta = meta::AuraeMeta {
                    name: "-".to_string(),
                    message: format!("{:?}", e),
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
