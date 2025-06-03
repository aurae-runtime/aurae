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
    vm_service_server, VirtualMachineSummary, VmServiceAllocateRequest,
    VmServiceAllocateResponse, VmServiceFreeRequest, VmServiceFreeResponse,
    VmServiceListRequest, VmServiceListResponse, VmServiceStartRequest,
    VmServiceStartResponse, VmServiceStopRequest, VmServiceStopResponse,
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use super::{
    error::{Result, VmServiceError},
    virtual_machine::{MountSpec, VmID, VmSpec},
    virtual_machines::VirtualMachines,
};

/// VmService struct manages the lifecycle of virtual machines.
#[derive(Debug, Clone)]
pub struct VmService {
    vms: Arc<Mutex<VirtualMachines>>,
    // TODO: ObserveService
}

impl VmService {
    /// Allocates a new instance of VmService.
    pub fn new() -> Self {
        Self { vms: Default::default() }
    }

    // TODO: validate requestts
    /// Allocates a new VM based on the provided request.
    ///
    /// # Arguments
    /// * `request` - A (currently unvalidated) request to allocate a VM
    ///
    /// # Returns
    /// A result containing the VmServiceAllocateResponse or an error.
    #[tracing::instrument(skip(self))]
    async fn allocate(
        &self,
        request: VmServiceAllocateRequest,
    ) -> Result<VmServiceAllocateResponse> {
        let mut vms = self.vms.lock().await;

        let Some(vm) = request.machine else {
            return Err(VmServiceError::MissingMachineConfig {});
        };

        let id = VmID::new(vm.id);
        let Some(root_drive) = vm.root_drive else {
            return Err(VmServiceError::MissingRootDrive { id: id.clone() });
        };

        let mut mounts = vec![MountSpec {
            host_path: PathBuf::from(root_drive.image_path.as_str()),
            read_only: root_drive.read_only,
        }];
        mounts.extend(vm.drive_mounts.into_iter().map(|m| MountSpec {
            host_path: PathBuf::from(m.image_path.as_str()),
            read_only: m.read_only,
        }));

        let spec = VmSpec {
            memory_size: vm.mem_size_mb,
            vcpu_count: vm.vcpu_count,
            kernel_image_path: PathBuf::from(vm.kernel_img_path.as_str()),
            kernel_args: vm.kernel_args,
            mounts,
            net: vec![],
        };

        let vm = vms.create(id.clone(), spec).map_err(|e| {
            VmServiceError::FailedToAllocateError { id, source: e }
        })?;

        Ok(VmServiceAllocateResponse { vm_id: vm.id.to_string() })
    }

    /// Frees a VM
    ///
    /// # Arguments
    /// * `request` - An (unvalidated) request to free a VM
    ///
    /// # Returns
    /// A result containing VmServiceFreeResponse or an error.
    #[tracing::instrument(skip(self))]
    async fn free(
        &self,
        request: VmServiceFreeRequest,
    ) -> Result<VmServiceFreeResponse> {
        let id = VmID::new(request.vm_id);

        let mut vms = self.vms.lock().await;
        vms.delete(&id)
            .map_err(|e| VmServiceError::FailedToFreeError { id, source: e })?;

        Ok(VmServiceFreeResponse {})
    }

    /// Starts a VM
    ///
    /// # Arguments
    /// * `request` - An (unvalidated) request to start a VM
    ///
    /// # Returns
    /// A result containing VmServiceStartResponse or an error.
    #[tracing::instrument(skip(self))]
    async fn start(
        &self,
        request: VmServiceStartRequest,
    ) -> Result<VmServiceStartResponse> {
        let id = VmID::new(request.vm_id);

        let mut vms = self.vms.lock().await;
        let addr = vms.start(&id).map_err(|e| {
            VmServiceError::FailedToStartError { id, source: e }
        })?;

        Ok(VmServiceStartResponse { auraed_address: addr })
    }

    /// Stops a VM
    ///
    /// # Arguments
    /// * `request` - An (unvalidated) request to stop a VM
    ///
    /// # Returns
    /// A result containing VmServiceStopResponse or an error.
    #[tracing::instrument(skip(self))]
    async fn stop(
        &self,
        request: VmServiceStopRequest,
    ) -> Result<VmServiceStopResponse> {
        let id = VmID::new(request.vm_id);

        let mut vms = self.vms.lock().await;
        vms.stop(&id)
            .map_err(|e| VmServiceError::FailedToStopError { id, source: e })?;

        Ok(VmServiceStopResponse {})
    }

    /// List VMs
    ///
    /// # Returns
    /// A result containing VmServiceListResponse or an error.
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> Result<VmServiceListResponse> {
        let vms = self.vms.lock().await;
        Ok(VmServiceListResponse {
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
        })
    }

    /// Stop all VMs
    #[tracing::instrument(skip(self))]
    pub async fn stop_all(&self) -> Result<()> {
        for vm in self.list().await?.machines {
            let _ = self.stop(VmServiceStopRequest { vm_id: vm.id }).await?;
        }
        Ok(())
    }

    /// Free all VMs
    #[tracing::instrument(skip(self))]
    pub async fn free_all(&self) -> Result<()> {
        for vm in self.list().await?.machines {
            let _ = self.free(VmServiceFreeRequest { vm_id: vm.id }).await?;
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl vm_service_server::VmService for VmService {
    async fn allocate(
        &self,
        request: Request<VmServiceAllocateRequest>,
    ) -> std::result::Result<Response<VmServiceAllocateResponse>, Status> {
        let req = request.into_inner();
        // TODO: validate the request
        Ok(Response::new(self.allocate(req).await?))
    }

    async fn free(
        &self,
        request: Request<VmServiceFreeRequest>,
    ) -> std::result::Result<Response<VmServiceFreeResponse>, Status> {
        let req = request.into_inner();
        // TODO: validate request
        Ok(Response::new(self.free(req).await?))
    }

    async fn start(
        &self,
        request: Request<VmServiceStartRequest>,
    ) -> std::result::Result<Response<VmServiceStartResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(self.start(req).await?))
    }

    async fn stop(
        &self,
        request: Request<VmServiceStopRequest>,
    ) -> std::result::Result<Response<VmServiceStopResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(self.stop(req).await?))
    }

    async fn list(
        &self,
        _request: Request<VmServiceListRequest>,
    ) -> std::result::Result<Response<VmServiceListResponse>, Status> {
        Ok(Response::new(self.list().await?))
    }
}
