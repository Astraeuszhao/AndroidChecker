use serde::{Deserialize, Serialize};
use std::collections::HashMap;`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub menu_labels: HashMap<String, String>,
}`nimpl Default for Config {
    fn default() -> Self {
        let mut labels = HashMap::new();
        labels.insert("0".to_string(), "妫€娴嬫槸鍚﹀瓨鍦?Root 鐜".to_string());
        labels.insert("1".to_string(), "鏌ョ湅寮曞/BL 閿佺姸鎬?.to_string());
        labels.insert("2".to_string(), "鎵嬫満澶囦唤鍙婃仮澶?猸?.to_string());
        labels.insert("3".to_string(), "绯荤粺鐩戞帶涓庤繘绋嬬鐞?.to_string());
        labels.insert("4".to_string(), "鏁鏈熷緟涓?.to_string());
        labels.insert("5".to_string(), "鏁鏈熷緟涓?.to_string());
        labels.insert("6".to_string(), "鏁鏈熷緟涓?.to_string());
        labels.insert("7".to_string(), "鏁鏈熷緟涓?.to_string());
        labels.insert("8".to_string(), "鏁鏈熷緟涓?.to_string());
        labels.insert("9".to_string(), "鏁鏈熷緟涓?.to_string());
        Self { menu_labels: labels }
    }
}`nimpl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self::default())
    }`n    pub fn get_label(&self, key: &str, fallback: &str) -> String {
        self.menu_labels
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }
}
