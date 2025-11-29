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
use tar::Builder;
pub struct BackupEngine {
    client: AdbClient,
    device_mgr: DeviceManager,
    root_checker: RootChecker,
}
impl BackupEngine {
    pub fn new(client: AdbClient) -> Self {
        let device_mgr = DeviceManager::new(client.clone());
        let root_checker = RootChecker::new(client.clone());
        Self {
            client,
            device_mgr,
            root_checker,
        }
    }
    pub async fn start_backup(
        &self,
        serial: &str,
        items: Vec<BackupItem>,
    ) -> Result<PathBuf> {
        let has_root = self.root_checker.has_root(serial).await.unwrap_or(false);
        if !has_root {
            ConsoleUi::warn("未检测到 Root 权限，将跳过需要 Root 的项目");
        } else {
            ConsoleUi::info("已检测到 Root 权限");
        }
        let items: Vec<_> = items
            .into_iter()
            .filter(|item| !item.requires_root() || has_root)
            .collect();
        if items.is_empty() {
            ConsoleUi::error("没有可备份的项目");
            return Err(crate::core::AdbError::CommandFailed(
                "No items to backup".to_string(),
            )
            .into());
        }
        let device_info = self.get_device_info(serial).await?;
        let backup_dir = self.create_backup_dir()?;
        let temp_dir = backup_dir.join(format!(
            "temp_{}_{}",
            Local::now().format("%Y%m%d_%H%M%S"),
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir)?;
        ConsoleUi::info(&format!("备份目录: {}", backup_dir.display()));
        for item in &items {
            ConsoleUi::info(&format!("正在备份: {}", item.name()));
            self.backup_item(serial, item, &temp_dir, has_root).await?;
        }
        let metadata = BackupMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            device_serial: serial.to_string(),
            device_model: device_info.0,
            android_version: device_info.1,
            backup_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            items: items.clone(),
            has_root,
        };
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        let metadata_file = temp_dir.join("metadata.json");
        fs::write(&metadata_file, metadata_json)?;
        ConsoleUi::info("正在压缩备份文件...");
        let backup_file = self.create_archive(&temp_dir, &backup_dir, serial)?;
        fs::remove_dir_all(&temp_dir)?;
        ConsoleUi::success(&format!("备份完成: {}", backup_file.display()));
        Ok(backup_file)
    }
    async fn get_device_info(&self, serial: &str) -> Result<(String, String)> {
        let props = self.device_mgr.get_properties(serial).await?;
        let model = props
            .get("ro.product.model")
            .or(props.get("ro.product.device"))
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let android = props
            .get("ro.build.version.release")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        Ok((model, android))
    }
    fn create_backup_dir(&self) -> Result<PathBuf> {
        let base_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Parent dir not found")
            })?
            .join("backups");
        fs::create_dir_all(&base_dir)?;
        Ok(base_dir)
    }
    async fn backup_item(
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
    }
    async fn backup_user_files(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let target = temp_dir.join("sdcard");
        fs::create_dir_all(&target)?;
        ConsoleUi::info("  拉取 /sdcard/ 目录...");
        let target_str = target.to_str().unwrap();
        let (_stdout, stderr) = self
            .client
            .execute(&["-s", serial, "pull", "/sdcard/", target_str])
            .await?;
        if !stderr.is_empty() && !stderr.contains("pulled") {
            ConsoleUi::warn(&format!("  警告: {}", stderr));
        }
        ConsoleUi::success("  用户文件备份完成");
        Ok(())
    }
    async fn backup_app_list(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let packages = self.device_mgr.get_packages(serial).await?;
        let list_file = temp_dir.join("app_list.txt");
        let content = packages.join("\n");
        fs::write(list_file, content)?;
        ConsoleUi::success(&format!("  应用列表备份完成 (共 {} 个应用)", packages.len()));
        Ok(())
    }
    async fn backup_app_data(&self, serial: &str, temp_dir: &Path, has_root: bool) -> Result<()> {
        if !has_root {
            ConsoleUi::warn("  应用数据备份需要 Root 权限，已跳过");
            return Ok(());
        }
        ConsoleUi::info("  备份应用数据 (需要较长时间)...");
        ConsoleUi::warn("  请在设备上确认备份请求");
        let backup_file = temp_dir.join("app_data.ab");
        let backup_file_str = backup_file.to_str().unwrap();
        let result = self
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
            .await;
        match result {
            Ok((true, _, _)) => {
                ConsoleUi::success("  应用数据备份完成");
            }
            _ => {
                ConsoleUi::warn("  应用数据备份失败或被取消");
            }
        }
        Ok(())
    }
    async fn backup_system_settings(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        ConsoleUi::info("  备份系统设置数据库...");
        let target = temp_dir.join("system_settings");
        fs::create_dir_all(&target)?;
        let db_files = vec![
            "/data/system/users/0/settings_system.xml",
            "/data/system/users/0/settings_secure.xml",
            "/data/system/users/0/settings_global.xml",
        ];
        let mut success_count = 0;
        let total_files = db_files.len();
        for db_file in db_files {
            let filename = Path::new(db_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let target_file = target.join(filename);
            let target_str = target_file.to_str().unwrap();
            let result = self
                .client
                .try_execute(&["-s", serial, "pull", db_file, target_str])
                .await;
            if result.is_ok_and(|(success, _, _)| success) {
                success_count += 1;
            }
        }
        ConsoleUi::success(&format!("  系统设置备份完成 ({}/{} 个文件)", success_count, total_files));
        Ok(())
    }
    fn create_archive(
        &self,
        temp_dir: &Path,
        backup_dir: &Path,
        serial: &str,
    ) -> Result<PathBuf> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let archive_name = format!("{}_{}.adbbackup", serial, timestamp);
        let archive_path = backup_dir.join(&archive_name);
        let tar_gz = File::create(&archive_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);
        tar.append_dir_all(".", temp_dir)?;
        tar.finish()?;
        Ok(archive_path)
    }
}