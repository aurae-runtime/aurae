/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

use proto::vms::{
    vm_service_server, VmServiceCreateRequest, VmServiceCreateResponse,
    VmServiceFreeRequest, VmServiceFreeResponse, VmServiceStartRequest,
    VmServiceStartResponse, VmServiceStopRequest, VmServiceStopResponse,
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use super::{
    virtual_machine::{MountSpec, VmID, VmSpec},
    virtual_machines::VirtualMachines,
};

#[derive(Debug, Clone)]
pub struct VmService {
    vms: Arc<Mutex<VirtualMachines>>,
}

impl VmService {
    pub fn new() -> Self {
        let vms = Arc::new(Mutex::new(VirtualMachines::new()));
        Self { vms }
    }
}

#[tonic::async_trait]
impl vm_service_server::VmService for VmService {
    async fn create(
        &self,
        request: Request<VmServiceCreateRequest>,
    ) -> Result<Response<VmServiceCreateResponse>, Status> {
        let mut vms = self.vms.lock().await;
        let req = request.into_inner();

        let Some(vm) = req.machine else {
            return Err(Status::invalid_argument("No machine config provided"));
        };

        let Some(root_drive) = vm.root_drive else {
            return Err(Status::invalid_argument("No root drive provided"));
        };

        let id = VmID::new(vm.id);
        let spec = VmSpec {
            memory_size: vm.mem_size_mb,
            vcpu_count: vm.vcpu_count,
            kernel_image_path: PathBuf::from(vm.kernel_img_path.as_str()),
            kernel_args: vm.kernel_args,
            mounts: vec![MountSpec {
                host_path: PathBuf::from(root_drive.host_path.as_str()),
                read_only: false,
            }],
            net: Vec::new(),
        };

        let vm = vms.create(id, spec).map_err(|e| {
            Status::internal(format!("Failed to create VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceCreateResponse { vm_id: vm.id.to_string() }))
    }

    async fn free(
        &self,
        _request: Request<VmServiceFreeRequest>,
    ) -> Result<Response<VmServiceFreeResponse>, Status> {
        todo!()
    }

    async fn start(
        &self,
        request: Request<VmServiceStartRequest>,
    ) -> Result<Response<VmServiceStartResponse>, Status> {
        let vms = self.vms.lock().await;
        let req = request.into_inner();

        let id = VmID::new(req.vm_id);
        let vm =
            vms.get(&id).ok_or_else(|| Status::not_found("VM not found"))?;
        vm.start().map_err(|e| {
            Status::internal(format!("Failed to start VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceStartResponse {}))
    }

    async fn stop(
        &self,
        request: Request<VmServiceStopRequest>,
    ) -> Result<Response<VmServiceStopResponse>, Status> {
        let vms = self.vms.lock().await;
        let req = request.into_inner();

        let id = VmID::new(req.vm_id);
        let vm =
            vms.get(&id).ok_or_else(|| Status::not_found("VM not found"))?;
        vm.stop().map_err(|e| {
            Status::internal(format!("Failed to stop VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceStopResponse {}))
    }
}
