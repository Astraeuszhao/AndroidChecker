use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupItem {
    UserFiles,
    AppList,
    AppData,
    SystemSettings,
}
impl BackupItem {
    pub fn name(&self) -> &str {
        match self {
            Self::UserFiles => "用户文件 (/sdcard/)",
            Self::AppList => "已安装应用列表",
            Self::AppData => "应用数据 (需Root或授权)",
            Self::SystemSettings => "系统设置数据库 (需Root)",
        }
    }
    pub fn requires_root(&self) -> bool {
        matches!(self, Self::SystemSettings)
    }
    pub fn all_items() -> Vec<Self> {
        vec![Self::UserFiles, Self::AppList, Self::AppData, Self::SystemSettings]
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub version: String,
    pub device_serial: String,
    pub device_model: String,
    pub android_version: String,
    pub backup_time: String,
    pub items: Vec<BackupItem>,
    pub has_root: bool,
}
#[derive(Debug, Clone)]
pub enum RestoreMode {
    Full,
    Selective(Vec<BackupItem>),
}