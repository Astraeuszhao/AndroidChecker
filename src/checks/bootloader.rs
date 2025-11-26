use crate::adb::DeviceManager;
use crate::core::Result;
use std::collections::HashMap;`npub struct BootloaderChecker {
    device_mgr: DeviceManager,
}`nimpl BootloaderChecker {
    pub fn new(dm: DeviceManager) -> Self {
        Self { device_mgr: dm }
    }`n    fn analyze(&self, props: &HashMap<String, String>) -> String {
        let vb = props.get("ro.boot.verifiedbootstate").map(|s| s.as_str());
        let locked = props.get("ro.boot.flash.locked").map(|s| s.as_str());
        let state = props.get("ro.boot.vbmeta.device_state").map(|s| s.as_str());
        let verify = props.get("ro.boot.veritymode").map(|s| s.as_str());
        let dbg = props.get("ro.debuggable").map(|s| s.as_str());
        let qemu = props.get("ro.boot.qemu").map(|s| s.as_str());`n        if qemu == Some("1") {
            if dbg == Some("1") {
                return "妯℃嫙鍣ㄧ幆澧冿紝榛樿鍙皟璇曟ā寮忥紙鐩稿綋浜庡凡瑙ｉ攣锛?.to_string();
            } else {
                return "妯℃嫙鍣ㄧ幆澧?.to_string();
            }
        }`n        match (vb, locked, state) {
            (Some("green"), Some("1"), Some("locked")) => {
                "Bootloader 宸查攣瀹氾紝绯荤粺瀹屾暣鎬ц壇濂?.to_string()
            }
            (Some("orange"), _, _) | (_, Some("0"), _) | (_, _, Some("unlocked")) => {
                "Bootloader 宸茶В閿侊紝绯荤粺鍙兘琚慨鏀?.to_string()
            }
            (Some("red"), _, _) => "绯荤粺楠岃瘉澶辫触锛屽瓨鍦ㄤ弗閲嶅畨鍏ㄩ闄?.to_string(),
            _ => {
                if verify == Some("enforcing") && dbg == Some("0") {
                    "Bootloader 鍙兘宸查攣瀹氾紙鍩轰簬 veritymode锛?.to_string()
                } else if dbg == Some("1") {
                    "璁惧澶勪簬璋冭瘯妯″紡锛屽彲鑳藉凡瑙ｉ攣".to_string()
                } else {
                    "鏃犳硶纭畾 Bootloader 鐘舵€侊紙璁惧涓嶆彁渚涙爣鍑嗗睘鎬э級".to_string()
                }
            }
        }
    }`n    fn extract_info(&self, props: &HashMap<String, String>) -> Vec<(String, String)> {
        let items = vec![
            ("ro.boot.verifiedbootstate", "verifiedbootstate"),
            ("ro.boot.vbmeta.device_state", "vbmeta.device_state"),
            ("ro.boot.flash.locked", "flash.locked"),
            ("ro.boot.veritymode", "veritymode"),
            ("ro.boot.warranty_bit", "warranty_bit"),
            ("ro.boot.bootloader", "bootloader"),
            ("ro.debuggable", "debuggable"),
            ("ro.boot.qemu", "qemu"),
        ];`n        items.into_iter()
            .filter_map(|(k, label)| {
                props.get(k).map(|v| {
                    (label.to_string(), if v.is_empty() { "-".to_string() } else { v.clone() })
                })
            })
            .collect()
    }`n    pub async fn check(&self, serial: &str) -> Result<String> {
        let props = self.device_mgr.get_properties(serial).await?;`n        let info = self.extract_info(&props);`n        let mut out = String::new();
        out.push_str("\n[BL 閿?/ Verified Boot]\n");`n        for (k, v) in info {
            out.push_str(&format!("  {}={}\n", k, v));
        }`n        let analysis = self.analyze(&props);
        out.push_str(&format!("\n鍒嗘瀽: {}\n", analysis));`n        Ok(out)
    }
}
