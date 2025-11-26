use crate::adb::{AdbClient, DeviceManager};
use crate::core::Result;`npub struct RootChecker {
    client: AdbClient,
    device_mgr: DeviceManager,
}`nimpl RootChecker {
    pub fn new(client: AdbClient) -> Self {
        let dm = DeviceManager::new(client.clone());
        Self {
            client,
            device_mgr: dm,
        }
    }`n    pub async fn check(&self, serial: &str) -> Result<String> {
        let mut results = Vec::new();`n        let packages = self.device_mgr.get_packages(serial).await?;
        let suspect_list = vec![
            "com.topjohnwu.magisk",
            "org.lsposed.manager",
            "eu.chainfire.supersu",
            "stericson.busybox",
            "com.koushikdutta.superuser",
            "com.noshufou.android.su",
        ];`n        let found_apps: Vec<_> = packages
            .iter()
            .filter(|pkg| suspect_list.iter().any(|s| pkg.contains(s)))
            .collect();`n        if !found_apps.is_empty() {
            results.push(format!("鍙戠幇鍙枒鍖? {:?}", found_apps));
        } else {
            results.push("鏈彂鐜板彲鐤?Root 绠＄悊鍖?.to_string());
        }`n        let (ok, path, _) = self.client.try_shell(serial, &["which", "su"]).await?;
        if ok && !path.trim().is_empty() {
            results.push(format!("鍙戠幇 su 鍛戒护: {}", path.trim()));
        } else {
            results.push("鏈彂鐜?su 鍛戒护".to_string());
        }`n        let paths_to_check = vec![
            "/system/bin/su",
            "/system/xbin/su",
            "/sbin/su",
            "/system/app/Superuser.apk",
            "/system/app/SuperSU.apk",
        ];`n        let mut files_found = Vec::new();
        for p in paths_to_check {
            if self.device_mgr.file_exists(serial, p).await {
                files_found.push(p);
            }
        }`n        if !files_found.is_empty() {
            results.push(format!("鍙戠幇 Root 鐩稿叧鏂囦欢: {:?}", files_found));
        } else {
            results.push("鏈彂鐜?Root 鐩稿叧鏂囦欢".to_string());
        }`n        let (works1, out1, _) = self.client.try_shell(serial, &["su", "-c", "id"]).await?;
        let (works2, out2, _) = self.client.try_shell(serial, &["su", "0", "id"]).await?;
        let (works3, out3, _) = self.client.try_shell(serial, &["id"]).await?;`n        let root_works = (works1 && out1.contains("uid=0"))
            || (works2 && out2.contains("uid=0"))
            || (works3 && out3.contains("uid=0"));`n        if root_works {
            results.push("su 鍛戒护鍙墽琛?(宸茶幏鍙?Root)".to_string());
        } else {
            results.push("su 鍛戒护涓嶅彲鎵ц".to_string());
        }`n        let detected = results.iter().any(|r| r.contains("鍙戠幇 su") || r.contains("鍙墽琛?));`n        let mut out = String::new();
        out.push_str("\n[Root 鐜妫€娴媇\n");
        out.push_str(&format!("鎬讳綋鍒ゆ柇: {}\n\n", 
            if detected { "妫€娴嬪埌 Root 鐥曡抗" } else { "鏈娴嬪埌 Root" }
        ));`n        for r in results {
            out.push_str(&format!("  {}\n", r));
        }
        Ok(out)
    }
}`n