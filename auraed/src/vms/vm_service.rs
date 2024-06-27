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
    vm_service_server, VirtualMachineSummary, VmServiceCreateRequest,
    VmServiceCreateResponse, VmServiceFreeRequest, VmServiceFreeResponse,
    VmServiceListResponse, VmServiceStartRequest, VmServiceStartResponse,
    VmServiceStopRequest, VmServiceStopResponse,
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
        Self { vms: Default::default() }
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

        let mut mounts = vec![MountSpec {
            host_path: PathBuf::from(root_drive.image_path.as_str()),
            read_only: root_drive.read_only,
        }];
        mounts.extend(vm.drive_mounts.into_iter().map(|m| MountSpec {
            host_path: PathBuf::from(m.image_path.as_str()),
            read_only: m.read_only,
        }));

        let id = VmID::new(vm.id);
        let spec = VmSpec {
            memory_size: vm.mem_size_mb,
            vcpu_count: vm.vcpu_count,
            kernel_image_path: PathBuf::from(vm.kernel_img_path.as_str()),
            kernel_args: vm.kernel_args,
            mounts,
            net: vec![],
        };

        let vm = vms.create(id, spec).map_err(|e| {
            Status::internal(format!("Failed to create VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceCreateResponse { vm_id: vm.id.to_string() }))
    }

    async fn free(
        &self,
        request: Request<VmServiceFreeRequest>,
    ) -> Result<Response<VmServiceFreeResponse>, Status> {
        let req = request.into_inner();
        let id = VmID::new(req.vm_id);

        let mut vms = self.vms.lock().await;
        vms.delete(&id).map_err(|e| {
            Status::internal(format!("Failed to start VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceFreeResponse {}))
    }

    async fn start(
        &self,
        request: Request<VmServiceStartRequest>,
    ) -> Result<Response<VmServiceStartResponse>, Status> {
        let req = request.into_inner();
        let id = VmID::new(req.vm_id);

        let mut vms = self.vms.lock().await;
        let addr = vms.start(&id).map_err(|e| {
            Status::internal(format!("Failed to start VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceStartResponse { auraed_address: addr }))
    }

    async fn stop(
        &self,
        request: Request<VmServiceStopRequest>,
    ) -> Result<Response<VmServiceStopResponse>, Status> {
        let req = request.into_inner();
        let id = VmID::new(req.vm_id);

        let mut vms = self.vms.lock().await;
        vms.stop(&id).map_err(|e| {
            Status::internal(format!("Failed to stop VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceStopResponse {}))
    }

    async fn list(
        &self,
        _request: Request<proto::vms::VmServiceListRequest>,
    ) -> Result<Response<proto::vms::VmServiceListResponse>, Status> {
        let vms = self.vms.lock().await;
        Ok(Response::new(VmServiceListResponse {
            machines: vms
                .list()
                .iter()
                .map(|m| VirtualMachineSummary {
                    id: m.id.to_string(),
                    mem_size_mb: m.vm.memory_size,
                    vcpu_count: m.vm.vcpu_count,
                    kernel_img_path: m
                        .vm
                        .kernel_image_path
                        .to_string_lossy()
                        .to_string(),
                    root_dir_path: m.vm.mounts[0]
                        .host_path
                        .to_string_lossy()
                        .to_string(),
                    auraed_address: m
                        .tap()
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                    status: m.status.to_string(),
                })
                .collect(),
        }))
    }
}
