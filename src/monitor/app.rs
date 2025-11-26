use crate::adb::AdbClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub package_name: String,
    pub app_name: String,
    pub version_name: String,
    pub is_system: bool,
    pub is_enabled: bool,
}`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPermission {
    pub name: String,
    pub granted: bool,
    pub dangerous: bool,
}`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSignature {
    pub algorithm: String,
    pub fingerprint: String,
    pub valid_from: String,
    pub valid_to: String,
}`n#[derive(Clone)]
pub struct AppManager {
    client: AdbClient,
}`nimpl AppManager {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }`n    pub async fn list_apps(&self, serial: &str) -> Result<Vec<AppInfo>> {
        let output = self
            .client
            .invokeShellCommand(serial, &["pm", "list", "packages", "-f"])
            .await
            .context("Failed to get app list")?;`n        let mut apps = Vec::new();
        for line in output.lines() {
            if let Some(pkg_line) = line.strip_prefix("package:") {
                let parts: Vec<&str> = pkg_line.split('=').collect();
                if parts.len() == 2 {
                    let apk_path = parts[0].to_string();
                    let package_name = parts[1].to_string();
                    let is_system = apk_path.starts_with("/system/");`n                    apps.push(AppInfo {
                        package_name: package_name.clone(),
                        app_name: package_name.clone(),
                        version_name: String::from("Unknown"),
                        is_system,
                        is_enabled: true,
                    });
                }
            }
        }
        Ok(apps)
    }`n    pub async fn list_user_apps(&self, serial: &str) -> Result<Vec<AppInfo>> {
        let all_apps = self.list_apps(serial).await?;
        Ok(all_apps.into_iter().filter(|app| !app.is_system).collect())
    }`n    pub async fn list_system_apps(&self, serial: &str) -> Result<Vec<AppInfo>> {
        let all_apps = self.list_apps(serial).await?;
        Ok(all_apps.into_iter().filter(|app| app.is_system).collect())
    }`n    pub async fn launch_app(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["monkey", "-p", package, "-c", "android.intent.category.LAUNCHER", "1"])
            .await
            .context("Failed to launch app")?;
        Ok(())
    }`n    pub async fn stop_app(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["am", "force-stop", package])
            .await
            .context("Failed to stop app")?;
        Ok(())
    }`n    pub async fn clear_app_data(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["pm", "clear", package])
            .await
            .context("Failed to clear app data")?;
        Ok(())
    }`n    pub async fn uninstall_app(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["pm", "uninstall", package])
            .await
            .context("Failed to uninstall app")?;
        Ok(())
    }`n    pub async fn enable_app(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["pm", "enable", package])
            .await
            .context("鍚敤搴旂敤澶辫触")?;
        Ok(())
    }`n    pub async fn disable_app(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["pm", "disable-user", package])
            .await
            .context("绂佺敤搴旂敤澶辫触")?;
        Ok(())
    }`n    pub async fn get_app_details(&self, serial: &str, package: &str) -> Result<String> {
        let output = self
            .client
            .invokeShellCommand(serial, &["dumpsys", "package", package])
            .await
            .context("Failed to get app details")?;
        Ok(output)
    }`n    pub async fn get_app_permissions(&self, serial: &str, package: &str) -> Result<Vec<AppPermission>> {
        let grep_cmd = format!("dumpsys package {} | grep permission", package);
        let output = self
            .client
            .invokeShellCommand(serial, &["sh", "-c", &grep_cmd])
            .await
            .context("鑾峰彇搴旂敤鏉冮檺澶辫触")?;`n        let mut permissions = Vec::new();`n        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("android.permission.") || trimmed.starts_with("com.") {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if let Some(perm_name) = parts.first() {
                    let granted = trimmed.contains("granted=true");
                    permissions.push(AppPermission {
                        name: perm_name.to_string(),
                        granted,
                        dangerous: Self::is_dangerous_permission(perm_name),
                    });
                }
            }
        }`n        Ok(permissions)
    }`n    pub async fn get_app_signature(&self, serial: &str, package: &str) -> Result<Vec<String>> {
        let grep_cmd = format!("dumpsys package {} | grep signatures", package);
        let output = self
            .client
            .invokeShellCommand(serial, &["sh", "-c", &grep_cmd])
            .await
            .context("鑾峰彇搴旂敤绛惧悕澶辫触")?;`n        let signatures: Vec<String> = output
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("signatures=") {
                    Some(trimmed.trim_start_matches("signatures=").to_string())
                } else {
                    None
                }
            })
            .collect();`n        Ok(signatures)
    }`n    pub async fn get_app_storage(&self, serial: &str, package: &str) -> Result<AppStorageInfo> {
        let grep_cmd = format!("dumpsys package {} | grep -A 5 'User 0:'", package);
        let output = self
            .client
            .invokeShellCommand(serial, &["sh", "-c", &grep_cmd])
            .await
            .unwrap_or_default();`n        let mut code_size = 0u64;
        let mut data_size = 0u64;
        let mut cache_size = 0u64;`n        for line in output.lines() {
            if line.contains("codeSize=") {
                if let Some(size) = line.split('=').nth(1) {
                    code_size = size.trim().parse().unwrap_or(0);
                }
            } else if line.contains("dataSize=") {
                if let Some(size) = line.split('=').nth(1) {
                    data_size = size.trim().parse().unwrap_or(0);
                }
            } else if line.contains("cacheSize=") {
                if let Some(size) = line.split('=').nth(1) {
                    cache_size = size.trim().parse().unwrap_or(0);
                }
            }
        }`n        Ok(AppStorageInfo {
            code_size,
            data_size,
            cache_size,
            total_size: code_size + data_size + cache_size,
        })
    }`n    async fn get_app_basic_info(
        &self,
        serial: &str,
        package: &str,
    ) -> Result<(String, String, String, bool, u32, String, String)> {
        let output = self
            .client
            .invokeShellCommand(serial, &["dumpsys", "package", package])
            .await?;`n        let app_name = package.to_string();
        let mut version_name = String::from("Unknown");
        let mut version_code = String::from("0");
        let mut is_enabled = true;
        let mut uid = 0u32;
        let mut install_time = String::from("Unknown");
        let mut update_time = String::from("Unknown");`n        for line in output.lines() {
            let trimmed = line.trim();`n            if trimmed.starts_with("versionName=") {
                version_name = trimmed.trim_start_matches("versionName=").to_string();
            } else if trimmed.starts_with("versionCode=") {
                version_code = trimmed
                    .trim_start_matches("versionCode=")
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .to_string();
            } else if trimmed.starts_with("userId=") {
                uid = trimmed
                    .trim_start_matches("userId=")
                    .parse()
                    .unwrap_or(0);
            } else if trimmed.contains("enabled=") {
                is_enabled = !trimmed.contains("enabled=3"); 
            } else if trimmed.starts_with("firstInstallTime=") {
                install_time = trimmed.trim_start_matches("firstInstallTime=").to_string();
            } else if trimmed.starts_with("lastUpdateTime=") {
                update_time = trimmed.trim_start_matches("lastUpdateTime=").to_string();
            }
        }`n        Ok((app_name, version_name, version_code, is_enabled, uid, install_time, update_time))
    }`n    fn is_dangerous_permission(perm: &str) -> bool {
        let dangerous_perms = vec![
            "READ_CONTACTS",
            "WRITE_CONTACTS",
            "READ_CALENDAR",
            "WRITE_CALENDAR",
            "SEND_SMS",
            "RECEIVE_SMS",
            "READ_SMS",
            "CAMERA",
            "RECORD_AUDIO",
            "ACCESS_FINE_LOCATION",
            "ACCESS_COARSE_LOCATION",
            "READ_PHONE_STATE",
            "CALL_PHONE",
            "READ_CALL_LOG",
            "WRITE_CALL_LOG",
            "READ_EXTERNAL_STORAGE",
            "WRITE_EXTERNAL_STORAGE",
        ];`n        dangerous_perms.iter().any(|dp| perm.contains(dp))
    }
}`n#[derive(Debug, Clone)]
pub struct AppStorageInfo {
    pub code_size: u64,
    pub data_size: u64,
    pub cache_size: u64,
    pub total_size: u64,
}`nimpl AppStorageInfo {
    pub fn format_display(&self) -> String {
        format!(
            "搴旂敤瀛樺偍鍗犵敤:\n\
            - APK澶у皬: {} MB\n\
            - 鏁版嵁澶у皬: {} MB\n\
            - 缂撳瓨澶у皬: {} MB\n\
            - 鎬昏: {} MB",
            self.code_size / 1024 / 1024,
            self.data_size / 1024 / 1024,
            self.cache_size / 1024 / 1024,
            self.total_size / 1024 / 1024
        )
    }
}`n