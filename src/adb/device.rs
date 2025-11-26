use crate::adb::client::AdbClient;
use crate::core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub serial: String,
    pub state: String,
    pub model: Option<String>,
    pub brand: Option<String>,
    pub marketing_name: Option<String>,
    pub android_version: Option<String>,
    pub sdk_version: Option<String>,
}`nimpl Device {
    pub fn display_name(&self) -> String {
        if let Some(brand) = &self.brand {
            if let Some(name) = &self.marketing_name {
                return format!("{} {}", brand, name).trim().to_string();
            }
        }`n        if let Some(model) = &self.model {
            return model.clone();
        }`n        self.serial.clone()
    }
}`npub struct DeviceManager {
    client: AdbClient,
}`nimpl DeviceManager {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }`n    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        let (output, _) = self.client.execute(&["devices", "-l"]).await?;`n        let mut devices = Vec::new();`n        for line in output.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                continue;
            }`n            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }`n            let serial = parts[0].to_string();
            let state = parts[1].to_string();`n            if state != "device" {
                continue;
            }`n            let props = self.get_properties(&serial).await.unwrap_or_default();`n            let model = props.get("ro.product.model").cloned();
            let brand = props.get("ro.product.brand").cloned();
            let marketing_name = Self::first_non_empty(&[
                props.get("ro.product.marketname").cloned(),
                props.get("ro.config.marketing_name").cloned(),
                props.get("ro.product.vendor.model").cloned(),
            ]);
            let android_version = props.get("ro.build.version.release").cloned();
            let sdk_version = props.get("ro.build.version.sdk").cloned();`n            devices.push(Device {
                serial,
                state,
                model,
                brand,
                marketing_name,
                android_version,
                sdk_version,
            });
        }`n        Ok(devices)
    }`n    pub async fn get_properties(&self, serial: &str) -> Result<HashMap<String, String>> {
        let output = self.client.shell(serial, &["getprop"]).await?;`n        let mut props = HashMap::new();`n        for line in output.lines() {
            let line = line.trim();
            if !line.starts_with('[') {
                continue;
            }`n            if let Some(idx) = line.find("]: [") {
                let key = &line[1..idx];
                let rest = &line[idx + 4..];
                if let Some(end) = rest.rfind(']') {
                    let value = &rest[..end];
                    props.insert(key.to_string(), value.to_string());
                }
            }
        }`n        Ok(props)
    }`n    pub async fn get_packages(&self, serial: &str) -> Result<Vec<String>> {
        let output = self.client.shell(serial, &["pm", "list", "packages"]).await?;`n        let packages: Vec<String> = output
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with("package:") {
                    Some(line[8..].trim().to_string())
                } else {
                    None
                }
            })
            .collect();`n        Ok(packages)
    }`n    pub async fn file_exists(&self, serial: &str, path: &str) -> bool {
        let (ok, _, stderr) = self
            .client
            .try_shell(serial, &["ls", "-l", path])
            .await
            .unwrap_or((false, String::new(), String::new()));`n        if !ok {
            let err_lower = stderr.to_lowercase();
            return !err_lower.contains("no such file") && !err_lower.contains("not found");
        }`n        true
    }`n    fn first_non_empty(values: &[Option<String>]) -> Option<String> {
        values
            .iter()
            .find(|v| v.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
            .and_then(|v| v.clone())
    }
}
