use proto::vms::{
    vm_service_server, VirtualMachine, VmServiceAllocateRequest,
    VmServiceAllocateResponse, VmServiceFreeRequest, VmServiceFreeResponse,
    VmServiceStartRequest, VmServiceStartResponse, VmServiceStopRequest,
    VmServiceStopResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct VmService {
    _vms: Arc<Mutex<VirtualMachine>>,
}

#[tonic::async_trait]
impl vm_service_server::VmService for VmService {
    async fn allocate(
        &self,
        _request: Request<VmServiceAllocateRequest>,
    ) -> Result<Response<VmServiceAllocateResponse>, Status> {
        todo!()
    }

    async fn free(
        &self,
        _request: Request<VmServiceFreeRequest>,
    ) -> Result<Response<VmServiceFreeResponse>, Status> {
        todo!()
    }

    async fn start(
        &self,
        _request: Request<VmServiceStartRequest>,
    ) -> Result<Response<VmServiceStartResponse>, Status> {
        todo!()
    }

    async fn stop(
        &self,
        _request: Request<VmServiceStopRequest>,
    ) -> Result<Response<VmServiceStopResponse>, Status> {
        todo!()
    }
}
