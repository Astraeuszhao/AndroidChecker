use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub menu_labels: HashMap<String, String>,
}
impl Default for Config {
    fn default() -> Self {
        let mut labels = HashMap::new();
        labels.insert("0".to_string(), "检测是否存在 Root 环境".to_string());
        labels.insert("1".to_string(), "查看引导/BL 锁状态".to_string());
        labels.insert("2".to_string(), "手机备份及恢复".to_string());
        labels.insert("3".to_string(), "ADB 稳定性/压力测试".to_string());
        labels.insert("4".to_string(), "系统与硬件安全检查".to_string());
        labels.insert("5".to_string(), "敬请期待...".to_string());
        labels.insert("6".to_string(), "敬请期待...".to_string());
        labels.insert("7".to_string(), "敬请期待...".to_string());
        labels.insert("8".to_string(), "敬请期待...".to_string());
        labels.insert("9".to_string(), "敬请期待...".to_string());
        Self { menu_labels: labels }
    }
}
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self::default())
    }
    pub fn get_label(&self, key: &str, fallback: &str) -> String {
        self.menu_labels
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }
}