use crate::adb::AdbClient;
use crate::core::Result;
use crate::ui::ConsoleUi;
use super::models::{BackupItem, BackupMetadata, RestoreMode};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;
pub struct RestoreEngine {
    client: AdbClient,
}
impl RestoreEngine {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }
    pub async fn start_restore(
        &self,
        serial: &str,
        backup_file: &Path,
        mode: RestoreMode,
    ) -> Result<()> {
        if !backup_file.exists() {
            return Err(crate::core::AdbError::CommandFailed(
                "备份文件不存在".to_string(),
            )
            .into());
        }
        if backup_file.extension().and_then(|e| e.to_str()) != Some("adbbackup") {
            ConsoleUi::warn("文件扩展名不是 .adbbackup，将尝试作为 tar.gz 处理");
        }
        ConsoleUi::info(&format!("正在恢复: {}", backup_file.display()));
        let temp_dir = self.extract_backup(backup_file)?;
        let metadata = self.read_metadata(&temp_dir)?;
        ConsoleUi::info(&format!(
            "备份信息: {} (Android {}) - {}",
            metadata.device_model, metadata.android_version, metadata.backup_time
        ));
        let items_to_restore = match mode {
            RestoreMode::Full => metadata.items.clone(),
            RestoreMode::Selective(items) => items,
        };
        for item in items_to_restore {
            ConsoleUi::info(&format!("正在恢复: {}", item.name()));
            self.restore_item(serial, &item, &temp_dir).await?;
        }
        fs::remove_dir_all(&temp_dir)?;
        ConsoleUi::success("恢复完成！");
        Ok(())
    }
    fn extract_backup(&self, backup_file: &Path) -> Result<PathBuf> {
        ConsoleUi::info("正在解压备份文件...");
        let temp_dir = std::env::temp_dir().join(format!(
            "androidchecker_restore_{}_{}",
            chrono::Local::now().timestamp(),
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir)?;
        let tar_gz = File::open(backup_file)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(&temp_dir)?;
        ConsoleUi::success("解压完成");
        Ok(temp_dir)
    }
    fn read_metadata(&self, temp_dir: &Path) -> Result<BackupMetadata> {
        let metadata_file = temp_dir.join("metadata.json");
        let mut content = String::new();
        File::open(&metadata_file)?.read_to_string(&mut content)?;
        let metadata: BackupMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }
    async fn restore_item(
        &self,
        serial: &str,
        item: &BackupItem,
        temp_dir: &Path,
    ) -> Result<()> {
        match item {
            BackupItem::UserFiles => self.restore_user_files(serial, temp_dir).await,
            BackupItem::AppList => self.restore_app_list(serial, temp_dir).await,
            BackupItem::AppData => self.restore_app_data(serial, temp_dir).await,
            BackupItem::SystemSettings => self.restore_system_settings(serial, temp_dir).await,
        }
    }
    async fn restore_user_files(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let source = temp_dir.join("sdcard");
        if !source.exists() {
            ConsoleUi::warn("  用户文件备份不存在，跳过");
            return Ok(());
        }
        ConsoleUi::info("  推送文件到 /sdcard/ ...");
        let source_str = source.to_str().unwrap();
        let (_, stderr) = self
            .client
            .execute(&["-s", serial, "push", source_str, "/sdcard/"])
            .await?;
        if !stderr.is_empty() && !stderr.contains("pushed") {
            ConsoleUi::warn(&format!("  警告: {}", stderr));
        }
        ConsoleUi::success("  用户文件恢复完成");
        Ok(())
    }
    async fn restore_app_list(&self, _serial: &str, temp_dir: &Path) -> Result<()> {
        let list_file = temp_dir.join("app_list.txt");
        if !list_file.exists() {
            ConsoleUi::warn("  应用列表备份不存在，跳过");
            return Ok(());
        }
        let content = fs::read_to_string(&list_file)?;
        let packages: Vec<&str> = content.lines().collect();
        ConsoleUi::info(&format!("  备份中包含 {} 个应用", packages.len()));
        ConsoleUi::warn("  应用列表仅供参考，需手动安装");
        Ok(())
    }
    async fn restore_app_data(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let backup_file = temp_dir.join("app_data.ab");
        if !backup_file.exists() {
            ConsoleUi::warn("  应用数据备份不存在，跳过");
            return Ok(());
        }
        ConsoleUi::info("  恢复应用数据...");
        ConsoleUi::warn("  请在设备上确认恢复请求");
        let backup_file_str = backup_file.to_str().unwrap();
        let result = self
            .client
            .try_execute(&["-s", serial, "restore", backup_file_str])
            .await;
        match result {
            Ok((true, _, _)) => {
                ConsoleUi::success("  应用数据恢复完成");
            }
            _ => {
                ConsoleUi::warn("  应用数据恢复失败或被取消");
            }
        }
        Ok(())
    }
    async fn restore_system_settings(&self, _serial: &str, temp_dir: &Path) -> Result<()> {
        let source = temp_dir.join("system_settings");
        if !source.exists() {
            ConsoleUi::warn("  系统设置备份不存在，跳过");
            return Ok(());
        }
        ConsoleUi::warn("  系统设置恢复需要 Root 权限且可能导致系统不稳定");
        ConsoleUi::warn("  此功能暂未实现，建议手动恢复");
        Ok(())
    }
    pub fn list_backup_info(&self, backup_file: &Path) -> Result<()> {
        if !backup_file.exists() {
            return Err(crate::core::AdbError::CommandFailed(
                "备份文件不存在".to_string(),
            )
            .into());
        }
        let temp_dir = self.extract_backup(backup_file)?;
        let metadata = self.read_metadata(&temp_dir)?;
        println!("\n备份文件信息:");
        println!("  设备型号: {}", metadata.device_model);
        println!("  Android 版本: {}", metadata.android_version);
        println!("  备份时间: {}", metadata.backup_time);
        println!("  Root 权限: {}", if metadata.has_root { "是" } else { "否" });
        println!("  备份项目:");
        for item in metadata.items {
            println!("    - {}", item.name());
        }
        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}