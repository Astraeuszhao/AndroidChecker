use crate::adb::{AdbClient, DeviceManager};
use crate::core::Result;
use crate::ui::ConsoleUi;
use super::models::{BackupItem, BackupMetadata};
use super::root_checker::RootChecker;
use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Builder;`npub struct BackupEngine {
    client: AdbClient,
    device_mgr: DeviceManager,
    root_checker: RootChecker,
}`nimpl BackupEngine {
    pub fn new(client: AdbClient) -> Self {
        let device_mgr = DeviceManager::new(client.clone());
        let root_checker = RootChecker::new(client.clone());
        Self {
            client,
            device_mgr,
            root_checker,
        }
    }`n    pub async fn start_backup(
        &self,
        serial: &str,
        items: Vec<BackupItem>,
    ) -> Result<PathBuf> {`n        let has_root = self.root_checker.has_root(serial).await.unwrap_or(false);`n        if !has_root {
            ConsoleUi::warn("鏈娴嬪埌 Root 鏉冮檺锛屽皢璺宠繃闇€瑕?Root 鐨勯」鐩?);
        } else {
            ConsoleUi::info("宸叉娴嬪埌 Root 鏉冮檺");
        }`n        let items: Vec<_> = items
            .into_iter()
            .filter(|item| !item.requires_root() || has_root)
            .collect();`n        if items.is_empty() {
            ConsoleUi::error("娌℃湁鍙浠界殑椤圭洰");
            return Err(crate::core::AdbError::CommandFailed(
                "No items to backup".to_string(),
            )
            .into());
        }`n        let device_info = self.get_device_info(serial).await?;`n        let backup_dir = self.create_backup_dir()?;
        let temp_dir = backup_dir.join(format!(
            "temp_{}_{}",
            Local::now().format("%Y%m%d_%H%M%S"),
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir)?;`n        ConsoleUi::info(&format!("澶囦唤鐩綍: {}", backup_dir.display()));`n        for item in &items {
            ConsoleUi::info(&format!("姝ｅ湪澶囦唤: {}", item.name()));
            self.backup_item(serial, item, &temp_dir, has_root).await?;
        }`n        let metadata = BackupMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            device_serial: serial.to_string(),
            device_model: device_info.0,
            android_version: device_info.1,
            backup_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            items: items.clone(),
            has_root,
        };`n        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        let metadata_file = temp_dir.join("metadata.json");
        fs::write(&metadata_file, metadata_json)?;`n        ConsoleUi::info("姝ｅ湪鍘嬬缉澶囦唤鏂囦欢...");
        let backup_file = self.create_archive(&temp_dir, &backup_dir, serial)?;`n        fs::remove_dir_all(&temp_dir)?;`n        ConsoleUi::success(&format!("澶囦唤瀹屾垚: {}", backup_file.display()));`n        Ok(backup_file)
    }`n    async fn get_device_info(&self, serial: &str) -> Result<(String, String)> {
        let props = self.device_mgr.get_properties(serial).await?;`n        let model = props
            .get("ro.product.model")
            .or(props.get("ro.product.device"))
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());`n        let android = props
            .get("ro.build.version.release")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());`n        Ok((model, android))
    }`n    fn create_backup_dir(&self) -> Result<PathBuf> {
        let base_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Parent dir not found")
            })?
            .join("backups");`n        fs::create_dir_all(&base_dir)?;
        Ok(base_dir)
    }`n    async fn backup_item(
        &self,
        serial: &str,
        item: &BackupItem,
        temp_dir: &Path,
        has_root: bool,
    ) -> Result<()> {
        match item {
            BackupItem::UserFiles => self.backup_user_files(serial, temp_dir).await,
            BackupItem::AppList => self.backup_app_list(serial, temp_dir).await,
            BackupItem::AppData => self.backup_app_data(serial, temp_dir, has_root).await,
            BackupItem::SystemSettings => self.backup_system_settings(serial, temp_dir).await,
        }
    }`n    async fn backup_user_files(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let target = temp_dir.join("sdcard");
        fs::create_dir_all(&target)?;`n        ConsoleUi::info("  鎷夊彇 /sdcard/ 鐩綍...");`n        let target_str = target.to_str().unwrap();
        let (_stdout, stderr) = self
            .client
            .execute(&["-s", serial, "pull", "/sdcard/", target_str])
            .await?;`n        if !stderr.is_empty() && !stderr.contains("pulled") {
            ConsoleUi::warn(&format!("  璀﹀憡: {}", stderr));
        }`n        ConsoleUi::success("  鐢ㄦ埛鏂囦欢澶囦唤瀹屾垚");
        Ok(())
    }`n    async fn backup_app_list(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let packages = self.device_mgr.get_packages(serial).await?;`n        let list_file = temp_dir.join("app_list.txt");
        let content = packages.join("\n");
        fs::write(list_file, content)?;`n        ConsoleUi::success(&format!("  搴旂敤鍒楄〃澶囦唤瀹屾垚 (鍏?{} 涓簲鐢?", packages.len()));
        Ok(())
    }`n    async fn backup_app_data(&self, serial: &str, temp_dir: &Path, has_root: bool) -> Result<()> {
        if !has_root {
            ConsoleUi::warn("  搴旂敤鏁版嵁澶囦唤闇€瑕?Root 鏉冮檺锛屽凡璺宠繃");
            return Ok(());
        }`n        ConsoleUi::info("  澶囦唤搴旂敤鏁版嵁 (闇€瑕佽緝闀挎椂闂?...");`n        ConsoleUi::warn("  璇峰湪璁惧涓婄‘璁ゅ浠借姹?);`n        let backup_file = temp_dir.join("app_data.ab");
        let backup_file_str = backup_file.to_str().unwrap();`n        let result = self
            .client
            .try_execute(&[
                "-s",
                serial,
                "backup",
                "-apk",
                "-shared",
                "-all",
                "-f",
                backup_file_str,
            ])
            .await;`n        match result {
            Ok((true, _, _)) => {
                ConsoleUi::success("  搴旂敤鏁版嵁澶囦唤瀹屾垚");
            }
            _ => {
                ConsoleUi::warn("  搴旂敤鏁版嵁澶囦唤澶辫触鎴栬鍙栨秷");
            }
        }`n        Ok(())
    }`n    async fn backup_system_settings(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        ConsoleUi::info("  澶囦唤绯荤粺璁剧疆鏁版嵁搴?..");`n        let target = temp_dir.join("system_settings");
        fs::create_dir_all(&target)?;`n        let db_files = vec![
            "/data/system/users/0/settings_system.xml",
            "/data/system/users/0/settings_secure.xml",
            "/data/system/users/0/settings_global.xml",
        ];`n        let mut success_count = 0;
        let total_files = db_files.len();
        for db_file in db_files {
            let filename = Path::new(db_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");`n            let target_file = target.join(filename);
            let target_str = target_file.to_str().unwrap();`n            let result = self
                .client
                .try_execute(&["-s", serial, "pull", db_file, target_str])
                .await;`n            if result.is_ok_and(|(success, _, _)| success) {
                success_count += 1;
            }
        }`n        ConsoleUi::success(&format!("  绯荤粺璁剧疆澶囦唤瀹屾垚 ({}/{} 涓枃浠?", success_count, total_files));
        Ok(())
    }`n    fn create_archive(
        &self,
        temp_dir: &Path,
        backup_dir: &Path,
        serial: &str,
    ) -> Result<PathBuf> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let archive_name = format!("{}_{}.adbbackup", serial, timestamp);
        let archive_path = backup_dir.join(&archive_name);`n        let tar_gz = File::create(&archive_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);`n        tar.append_dir_all(".", temp_dir)?;
        tar.finish()?;`n        Ok(archive_path)
    }
}
