use crate::adb::{AdbClient, DeviceManager};
use crate::core::Result;
pub struct RootChecker {
    client: AdbClient,
    device_mgr: DeviceManager,
}
impl RootChecker {
    pub fn new(client: AdbClient) -> Self {
        let dm = DeviceManager::new(client.clone());
        Self {
            client,
            device_mgr: dm,
        }
    }
    pub async fn check(&self, serial: &str) -> Result<String> {
        let mut results = Vec::new();
        let packages = self.device_mgr.get_packages(serial).await?;
        let suspect_list = vec![
            "com.topjohnwu.magisk",
            "org.lsposed.manager",
            "eu.chainfire.supersu",
            "stericson.busybox",
            "com.koushikdutta.superuser",
            "com.noshufou.android.su",
        ];
        let found_apps: Vec<_> = packages
            .iter()
            .filter(|pkg| suspect_list.iter().any(|s| pkg.contains(s)))
            .collect();
        if !found_apps.is_empty() {
            results.push(format!("发现可疑包: {:?}", found_apps));
        } else {
            results.push("未发现可疑 Root 管理包".to_string());
        }
        let (ok, path, _) = self.client.try_shell(serial, &["which", "su"]).await?;
        if ok && !path.trim().is_empty() {
            results.push(format!("发现 su 命令: {}", path.trim()));
        } else {
            results.push("未发现 su 命令".to_string());
        }
        let paths_to_check = vec![
            "/system/bin/su",
            "/system/xbin/su",
            "/sbin/su",
            "/system/app/Superuser.apk",
            "/system/app/SuperSU.apk",
        ];
        let mut files_found = Vec::new();
        for p in paths_to_check {
            if self.device_mgr.file_exists(serial, p).await {
                files_found.push(p);
            }
        }
        if !files_found.is_empty() {
            results.push(format!("发现 Root 相关文件: {:?}", files_found));
        } else {
            results.push("未发现 Root 相关文件".to_string());
        }
        let (works1, out1, _) = self.client.try_shell(serial, &["su", "-c", "id"]).await?;
        let (works2, out2, _) = self.client.try_shell(serial, &["su", "0", "id"]).await?;
        let (works3, out3, _) = self.client.try_shell(serial, &["id"]).await?;
        let root_works = (works1 && out1.contains("uid=0"))
            || (works2 && out2.contains("uid=0"))
            || (works3 && out3.contains("uid=0"));
        if root_works {
            results.push("su 命令可执行 (已获取 Root)".to_string());
        } else {
            results.push("su 命令不可执行".to_string());
        }
        let detected = results.iter().any(|r| r.contains("发现 su") || r.contains("可执行"));
        let mut out = String::new();
        out.push_str("\n[Root 环境检测]\n");
        out.push_str(&format!("总体判断: {}\n\n",
            if detected { "检测到 Root 痕迹" } else { "未检测到 Root" }
        ));
        for r in results {
            out.push_str(&format!("  {}\n", r));
        }
        Ok(out)
    }
}