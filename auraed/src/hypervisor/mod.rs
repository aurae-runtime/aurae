// #[async_trait]
// pub trait Hypervisor: Send + Sync {
//     async fn create_vm(&self, id: &str, netns: Option<String>) -> Result<()>;
//     async fn start_vm(&self, timeout: i32) -> Result<()>;
//     async fn stop_vm(&self) -> Result<()>;
//     async fn add_device(&self, device: device::Device) -> Result<()>;
//     async fn remove_device(&self, device: device::Device) -> Result<()>;
// }
