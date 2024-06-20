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

use net_util::MacAddr;
use proto::vms::{
    vm_service_server, VmServiceCreateRequest, VmServiceCreateResponse,
    VmServiceFreeRequest, VmServiceFreeResponse, VmServiceStartRequest,
    VmServiceStartResponse, VmServiceStopRequest, VmServiceStopResponse,
};
use std::{net::Ipv4Addr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use vmm_sys_util::rand;

use super::{
    virtual_machine::{MountSpec, NetSpec, VmID, VmSpec},
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

        let mut mounts = vec![MountSpec {
            host_path: PathBuf::from(root_drive.host_path.as_str()),
            read_only: !root_drive.is_writeable,
        }];
        mounts.extend(vm.drive_mounts.into_iter().map(|m| MountSpec {
            host_path: PathBuf::from(m.host_path.as_str()),
            read_only: !m.is_writeable,
        }));

        let net = vec![NetSpec {
            tap: Some(format!(
                "aurae0-{}",
                rand::rand_alphanumerics(8).into_string().map_err(|_| {
                    Status::internal("Failed to generate tap device name")
                })?
            )),
            ip: Ipv4Addr::new(192, 168, 122, 1),
            mask: Ipv4Addr::new(255, 255, 255, 255),
            mac: MacAddr::local_random(),
            host_mac: Some(MacAddr::local_random()),
        }];

        let id = VmID::new(vm.id);
        let spec = VmSpec {
            memory_size: vm.mem_size_mb,
            vcpu_count: vm.vcpu_count,
            kernel_image_path: PathBuf::from(vm.kernel_img_path.as_str()),
            kernel_args: vm.kernel_args,
            mounts,
            net,
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

        Ok(Response::new(VmServiceFreeResponse { vm_id: req.vm_id }))
    }

    async fn start(
        &self,
        request: Request<VmServiceStartRequest>,
    ) -> Result<Response<VmServiceStartResponse>, Status> {
        let req = request.into_inner();
        let id = VmID::new(req.vm_id);

        let mut vms = self.vms.lock().await;
        let scope_id = vms.start(&id).map_err(|e| {
            Status::internal(format!("Failed to start VM: {:?}", e))
        })?;

        Ok(Response::new(VmServiceStartResponse { socket_scope_id: scope_id }))
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
}
