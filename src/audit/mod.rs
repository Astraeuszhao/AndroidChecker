use crate::adb::{AdbClient, DeviceManager};
use crate::core::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditReport {
    pub device_info: DeviceInfo,
    pub root: RootInfo,
    pub boot: BootInfo,
    pub security_env: SecurityEnv,
    pub hardware: HardwareInfo,
    pub integrity: IntegrityInfo,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub brand: String,
    pub marketing_name: String,
    pub model: String,
    pub android: String,
    pub sdk: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RootInfo {
    pub su_in_path: String,
    pub suspicious_packages: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BootInfo {
    pub verifiedbootstate: String,
    pub vbmeta_device_state: String,
    pub flash_locked: String,
    pub veritymode: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEnv {
    pub selinux: String,
    pub crypto_state: String,
    pub crypto_type: String,
    pub debuggable: String,
    pub secure: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub battery: String,
    pub thermal: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityInfo {
    pub security_patch: String,
    pub build_tags: String,
    pub build_type: String,
}
pub struct AuditRunner {
    client: AdbClient,
    device_mgr: DeviceManager,
}
impl AuditRunner {
    pub fn new(c: AdbClient) -> Self {
        let dm = DeviceManager::new(c.clone());
        Self {
            client: c,
            device_mgr: dm,
        }
    }
    pub async fn run(&self, serial: &str, save_dir: Option<&str>) -> Result<AuditReport> {
        let props = self.device_mgr.get_properties(serial).await?;
        let pkgs = self.device_mgr.get_packages(serial).await?;
        let hw_cmds = vec![
            vec!["dumpsys".to_string(), "battery".to_string()],
            vec!["dumpsys".to_string(), "thermalservice".to_string()],
        ];
        let hw_out = self.client.parallel_shell(serial, hw_cmds).await;
        let report = AuditReport {
            device_info: DeviceInfo {
                brand: props.get("ro.product.brand").cloned().unwrap_or_default(),
                marketing_name: props.get("ro.product.marketname").cloned().unwrap_or_default(),
                model: props.get("ro.product.model").cloned().unwrap_or_default(),
                android: props.get("ro.build.version.release").cloned().unwrap_or_default(),
                sdk: props.get("ro.build.version.sdk").cloned().unwrap_or_default(),
            },
            root: RootInfo {
                su_in_path: self
                    .client
                    .try_shell(serial, &["which", "su"])
                    .await
                    .ok()
                    .and_then(|(ok, out, _)| if ok { Some(out.trim().to_string()) } else { None })
                    .unwrap_or_default(),
                suspicious_packages: pkgs
                    .iter()
                    .filter(|p| {
                        p.contains("magisk") || p.contains("lsposed") || p.contains("supersu") || p.contains("busybox")
                    })
                    .cloned()
                    .collect(),
            },
            boot: BootInfo {
                verifiedbootstate: props.get("ro.boot.verifiedbootstate").cloned().unwrap_or_default(),
                vbmeta_device_state: props.get("ro.boot.vbmeta.device_state").cloned().unwrap_or_default(),
                flash_locked: props.get("ro.boot.flash.locked").cloned().unwrap_or_default(),
                veritymode: props.get("ro.boot.veritymode").cloned().unwrap_or_default(),
            },
            security_env: SecurityEnv {
                selinux: self
                    .client
                    .try_shell(serial, &["getenforce"])
                    .await
                    .ok()
                    .and_then(|(ok, out, _)| {
                        if ok {
                            Some(out.trim().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "unknown".to_string()),
                crypto_state: props.get("ro.crypto.state").cloned().unwrap_or_default(),
                crypto_type: props.get("ro.crypto.type").cloned().unwrap_or_default(),
                debuggable: props.get("ro.debuggable").cloned().unwrap_or_default(),
                secure: props.get("ro.secure").cloned().unwrap_or_default(),
            },
            hardware: HardwareInfo {
                battery: hw_out.get(0).and_then(|r| r.as_ref().ok().cloned()).unwrap_or_default(),
                thermal: hw_out.get(1).and_then(|r| r.as_ref().ok().cloned()).unwrap_or_default(),
            },
            integrity: IntegrityInfo {
                security_patch: props.get("ro.build.version.security_patch").cloned().unwrap_or_default(),
                build_tags: props.get("ro.build.tags").cloned().unwrap_or_default(),
                build_type: props.get("ro.build.type").cloned().unwrap_or_default(),
            },
        };
        if let Some(dir) = save_dir {
            let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
            let path = format!("{}/audit-{}.json", dir, ts);
            let json = serde_json::to_string_pretty(&report).unwrap();
            fs::write(&path, json).await?;
        }
        Ok(report)
    }
}