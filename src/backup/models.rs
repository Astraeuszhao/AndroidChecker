use serde::{Deserialize, Serialize};`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupItem {
    UserFiles,
    AppList,
    AppData,
    SystemSettings,
}`nimpl BackupItem {
    pub fn name(&self) -> &str {
        match self {
            Self::UserFiles => "鐢ㄦ埛鏂囦欢 (/sdcard/)",
            Self::AppList => "宸插畨瑁呭簲鐢ㄥ垪琛?,
            Self::AppData => "搴旂敤鏁版嵁 (闇€Root鎴栨巿鏉?",
            Self::SystemSettings => "绯荤粺璁剧疆鏁版嵁搴?(闇€Root)",
        }
    }`n    pub fn requires_root(&self) -> bool {
        matches!(self, Self::SystemSettings)
    }`n    pub fn all_items() -> Vec<Self> {
        vec![Self::UserFiles, Self::AppList, Self::AppData, Self::SystemSettings]
    }
}`n#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub version: String,
    pub device_serial: String,
    pub device_model: String,
    pub android_version: String,
    pub backup_time: String,
    pub items: Vec<BackupItem>,
    pub has_root: bool,
}`n#[derive(Debug, Clone)]
pub enum RestoreMode {
    Full,
    Selective(Vec<BackupItem>),
}
