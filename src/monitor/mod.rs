pub mod process;
pub mod resource;
pub mod service;
pub mod app;`npub use process::ProcessMonitor;
pub use resource::ResourceMonitor;
pub use service::ServiceMonitor;
pub use app::AppManager;`nuse crate::adb::AdbClient;`npub struct MonitorRunner {
    client: AdbClient,
}`nimpl MonitorRunner {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }
}`n