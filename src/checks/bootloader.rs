use crate::adb::DeviceManager;
use crate::core::Result;
use std::collections::HashMap;
pub struct BootloaderChecker {
    device_mgr: DeviceManager,
}
impl BootloaderChecker {
    pub fn new(dm: DeviceManager) -> Self {
        Self { device_mgr: dm }
    }
    fn analyze(&self, props: &HashMap<String, String>) -> String {
        let vb = props.get("ro.boot.verifiedbootstate").map(|s| s.as_str());
        let locked = props.get("ro.boot.flash.locked").map(|s| s.as_str());
        let state = props.get("ro.boot.vbmeta.device_state").map(|s| s.as_str());
        let verify = props.get("ro.boot.veritymode").map(|s| s.as_str());
        let dbg = props.get("ro.debuggable").map(|s| s.as_str());
        let qemu = props.get("ro.boot.qemu").map(|s| s.as_str());
        if qemu == Some("1") {
            if dbg == Some("1") {
                return "模拟器环境，默认可调试模式（相当于已解锁）".to_string();
            } else {
                return "模拟器环境".to_string();
            }
        }
        match (vb, locked, state) {
            (Some("green"), Some("1"), Some("locked")) => {
                "Bootloader 已锁定，系统完整性良好".to_string()
            }
            (Some("orange"), _, _) | (_, Some("0"), _) | (_, _, Some("unlocked")) => {
                "Bootloader 已解锁，系统可能被修改".to_string()
            }
            (Some("red"), _, _) => "系统验证失败，存在严重安全风险".to_string(),
            _ => {
                if verify == Some("enforcing") && dbg == Some("0") {
                    "Bootloader 可能已锁定（基于 veritymode）".to_string()
                } else if dbg == Some("1") {
                    "设备处于调试模式，可能已解锁".to_string()
                } else {
                    "无法确定 Bootloader 状态（设备不提供标准属性）".to_string()
                }
            }
        }
    }
    fn extract_info(&self, props: &HashMap<String, String>) -> Vec<(String, String)> {
        let items = vec![
            ("ro.boot.verifiedbootstate", "verifiedbootstate"),
            ("ro.boot.vbmeta.device_state", "vbmeta.device_state"),
            ("ro.boot.flash.locked", "flash.locked"),
            ("ro.boot.veritymode", "veritymode"),
            ("ro.boot.warranty_bit", "warranty_bit"),
            ("ro.boot.bootloader", "bootloader"),
            ("ro.debuggable", "debuggable"),
            ("ro.boot.qemu", "qemu"),
        ];
        items.into_iter()
            .filter_map(|(k, label)| {
                props.get(k).map(|v| {
                    (label.to_string(), if v.is_empty() { "-".to_string() } else { v.clone() })
                })
            })
            .collect()
    }
    pub async fn check(&self, serial: &str) -> Result<String> {
        let props = self.device_mgr.get_properties(serial).await?;
        let info = self.extract_info(&props);
        let mut out = String::new();
        out.push_str("\n[BL 锁 / Verified Boot]\n");
        for (k, v) in info {
            out.push_str(&format!("  {}={}\n", k, v));
        }
        let analysis = self.analyze(&props);
        out.push_str(&format!("\n分析: {}\n", analysis));
        Ok(out)
    }
}