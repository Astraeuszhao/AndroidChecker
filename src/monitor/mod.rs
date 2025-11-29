pub mod process;
pub mod resource;
pub mod service;
pub mod app;
pub use process::ProcessMonitor;
pub use resource::ResourceMonitor;
pub use service::ServiceMonitor;
pub use app::AppManager;
use crate::adb::AdbClient;
pub struct MonitorRunner {
    client: AdbClient,
}
impl MonitorRunner {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }
}