use crate::adb::AdbClient;
use crate::core::Result;
use crate::ui::ConsoleUi;
use super::models::{BackupItem, BackupMetadata, RestoreMode};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;`npub struct RestoreEngine {
    client: AdbClient,
}`nimpl RestoreEngine {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }`n    pub async fn start_restore(
        &self,
        serial: &str,
        backup_file: &Path,
        mode: RestoreMode,
    ) -> Result<()> {
        if !backup_file.exists() {
            return Err(crate::core::AdbError::CommandFailed(
                "澶囦唤鏂囦欢涓嶅瓨鍦?.to_string(),
            )
            .into());
        }`n        if backup_file.extension().and_then(|e| e.to_str()) != Some("adbbackup") {
            ConsoleUi::warn("鏂囦欢鎵╁睍鍚嶄笉鏄?.adbbackup锛屽皢灏濊瘯浣滀负 tar.gz 澶勭悊");
        }`n        ConsoleUi::info(&format!("姝ｅ湪鎭㈠: {}", backup_file.display()));`n        let temp_dir = self.extract_backup(backup_file)?;`n        let metadata = self.read_metadata(&temp_dir)?;`n        ConsoleUi::info(&format!(
            "澶囦唤淇℃伅: {} (Android {}) - {}",
            metadata.device_model, metadata.android_version, metadata.backup_time
        ));`n        let items_to_restore = match mode {
            RestoreMode::Full => metadata.items.clone(),
            RestoreMode::Selective(items) => items,
        };`n        for item in items_to_restore {
            ConsoleUi::info(&format!("姝ｅ湪鎭㈠: {}", item.name()));
            self.restore_item(serial, &item, &temp_dir).await?;
        }`n        fs::remove_dir_all(&temp_dir)?;`n        ConsoleUi::success("鎭㈠瀹屾垚锛?);
        Ok(())
    }`n    fn extract_backup(&self, backup_file: &Path) -> Result<PathBuf> {
        ConsoleUi::info("姝ｅ湪瑙ｅ帇澶囦唤鏂囦欢...");`n        let temp_dir = std::env::temp_dir().join(format!(
            "androidchecker_restore_{}_{}",
            chrono::Local::now().timestamp(),
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir)?;`n        let tar_gz = File::open(backup_file)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);`n        archive.unpack(&temp_dir)?;`n        ConsoleUi::success("瑙ｅ帇瀹屾垚");
        Ok(temp_dir)
    }`n    fn read_metadata(&self, temp_dir: &Path) -> Result<BackupMetadata> {
        let metadata_file = temp_dir.join("metadata.json");
        let mut content = String::new();
        File::open(&metadata_file)?.read_to_string(&mut content)?;`n        let metadata: BackupMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }`n    async fn restore_item(
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
    }`n    async fn restore_user_files(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let source = temp_dir.join("sdcard");
        if !source.exists() {
            ConsoleUi::warn("  鐢ㄦ埛鏂囦欢澶囦唤涓嶅瓨鍦紝璺宠繃");
            return Ok(());
        }`n        ConsoleUi::info("  鎺ㄩ€佹枃浠跺埌 /sdcard/ ...");`n        let source_str = source.to_str().unwrap();
        let (_, stderr) = self
            .client
            .execute(&["-s", serial, "push", source_str, "/sdcard/"])
            .await?;`n        if !stderr.is_empty() && !stderr.contains("pushed") {
            ConsoleUi::warn(&format!("  璀﹀憡: {}", stderr));
        }`n        ConsoleUi::success("  鐢ㄦ埛鏂囦欢鎭㈠瀹屾垚");
        Ok(())
    }`n    async fn restore_app_list(&self, _serial: &str, temp_dir: &Path) -> Result<()> {
        let list_file = temp_dir.join("app_list.txt");
        if !list_file.exists() {
            ConsoleUi::warn("  搴旂敤鍒楄〃澶囦唤涓嶅瓨鍦紝璺宠繃");
            return Ok(());
        }`n        let content = fs::read_to_string(&list_file)?;
        let packages: Vec<&str> = content.lines().collect();`n        ConsoleUi::info(&format!("  澶囦唤涓寘鍚?{} 涓簲鐢?, packages.len()));
        ConsoleUi::warn("  搴旂敤鍒楄〃浠呬緵鍙傝€冿紝闇€鎵嬪姩瀹夎");`n        Ok(())
    }`n    async fn restore_app_data(&self, serial: &str, temp_dir: &Path) -> Result<()> {
        let backup_file = temp_dir.join("app_data.ab");
        if !backup_file.exists() {
            ConsoleUi::warn("  搴旂敤鏁版嵁澶囦唤涓嶅瓨鍦紝璺宠繃");
            return Ok(());
        }`n        ConsoleUi::info("  鎭㈠搴旂敤鏁版嵁...");
        ConsoleUi::warn("  璇峰湪璁惧涓婄‘璁ゆ仮澶嶈姹?);`n        let backup_file_str = backup_file.to_str().unwrap();
        let result = self
            .client
            .try_execute(&["-s", serial, "restore", backup_file_str])
            .await;`n        match result {
            Ok((true, _, _)) => {
                ConsoleUi::success("  搴旂敤鏁版嵁鎭㈠瀹屾垚");
            }
            _ => {
                ConsoleUi::warn("  搴旂敤鏁版嵁鎭㈠澶辫触鎴栬鍙栨秷");
            }
        }`n        Ok(())
    }`n    async fn restore_system_settings(&self, _serial: &str, temp_dir: &Path) -> Result<()> {
        let source = temp_dir.join("system_settings");
        if !source.exists() {
            ConsoleUi::warn("  绯荤粺璁剧疆澶囦唤涓嶅瓨鍦紝璺宠繃");
            return Ok(());
        }`n        ConsoleUi::warn("  绯荤粺璁剧疆鎭㈠闇€瑕?Root 鏉冮檺涓斿彲鑳藉鑷寸郴缁熶笉绋冲畾");
        ConsoleUi::warn("  姝ゅ姛鑳芥殏鏈疄鐜帮紝寤鸿鎵嬪姩鎭㈠");`n        Ok(())
    }`n    pub fn list_backup_info(&self, backup_file: &Path) -> Result<()> {
        if !backup_file.exists() {
            return Err(crate::core::AdbError::CommandFailed(
                "澶囦唤鏂囦欢涓嶅瓨鍦?.to_string(),
            )
            .into());
        }`n        let temp_dir = self.extract_backup(backup_file)?;
        let metadata = self.read_metadata(&temp_dir)?;`n        println!("\n澶囦唤鏂囦欢淇℃伅:");
        println!("  璁惧鍨嬪彿: {}", metadata.device_model);
        println!("  Android 鐗堟湰: {}", metadata.android_version);
        println!("  澶囦唤鏃堕棿: {}", metadata.backup_time);
        println!("  Root 鏉冮檺: {}", if metadata.has_root { "鏄? } else { "鍚? });
        println!("  澶囦唤椤圭洰:");
        for item in metadata.items {
            println!("    - {}", item.name());
        }`n        fs::remove_dir_all(&temp_dir)?;`n        Ok(())
    }
}
