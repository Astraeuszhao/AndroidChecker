use crate::adb::AdbClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub pid: Option<u32>,
    pub user: String,
    pub state: ServiceState,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceState {
    Running,
    Stopped,
    Unknown,
}
impl ServiceState {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceState::Running => "Running",
            ServiceState::Stopped => "Stopped",
            ServiceState::Unknown => "Unknown",
        }
    }
}
pub struct ServiceMonitor {
    client: AdbClient,
}
impl ServiceMonitor {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }
    pub async fn list_services(&self, serial: &str) -> Result<Vec<ServiceInfo>> {
        let output = self
            .client
            .invokeShellCommand(serial, &["dumpsys", "activity", "services"])
            .await
            .context("Failed to get service list")?;
        let services = self.parse_services(&output);
        Ok(services)
    }
    pub async fn list_running_services(&self, serial: &str) -> Result<Vec<ServiceInfo>> {
        let all_services = self.list_services(serial).await?;
        Ok(all_services
            .into_iter()
            .filter(|s| s.state == ServiceState::Running)
            .collect())
    }
    pub async fn get_service_details(&self, serial: &str, service_name: &str) -> Result<String> {
        let output = self
            .client
            .invokeShellCommand(serial, &["dumpsys", service_name])
            .await
            .context("Failed to get service details")?;
        Ok(output)
    }
    pub async fn stop_service(&self, serial: &str, package: &str, service_class: &str) -> Result<()> {
        let intent = format!("{}/{}", package, service_class);
        self.client
            .invokeShellCommand(serial, &["am", "stopservice", &intent])
            .await
            .context("Failed to stop service")?;
        Ok(())
    }
    pub async fn start_service(&self, serial: &str, package: &str, service_class: &str) -> Result<()> {
        let intent = format!("{}/{}", package, service_class);
        self.client
            .invokeShellCommand(serial, &["am", "startservice", &intent])
            .await
            .context("Failed to start service")?;
        Ok(())
    }
    pub async fn list_system_services(&self, serial: &str) -> Result<Vec<String>> {
        let output = self
            .client
            .invokeShellCommand(serial, &["service", "list"])
            .await
            .context("Failed to get system services")?;
        let services: Vec<String> = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(parts[1].trim_matches(|c| c == '[' || c == ']').to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(services)
    }
    fn parse_services(&self, output: &str) -> Vec<ServiceInfo> {
        let mut services = Vec::new();
        let mut current_service: Option<ServiceInfo> = None;
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("* ServiceRecord{") {
                if let Some(service) = current_service.take() {
                    services.push(service);
                }
                if let Some(name_part) = trimmed.split_whitespace().last() {
                    let name = name_part.trim_end_matches('}').to_string();
                    current_service = Some(ServiceInfo {
                        name: name.clone(),
                        pid: None,
                        user: String::from("system"),
                        state: ServiceState::Unknown,
                    });
                }
            } else if let Some(ref mut service) = current_service {
                if trimmed.contains("app=ProcessRecord{") {
                    if let Some(pid_str) = trimmed.split(':').nth(1) {
                        if let Some(pid) = pid_str.split_whitespace().next() {
                            service.pid = pid.parse().ok();
                            if service.pid.is_some() {
                                service.state = ServiceState::Running;
                            }
                        }
                    }
                }
            }
        }
        if let Some(service) = current_service {
            services.push(service);
        }
        services
    }
}