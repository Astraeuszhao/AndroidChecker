use crate::adb::AdbClient;
use crate::core::Result;`npub struct StressRunner {
    client: AdbClient,
}`nimpl StressRunner {
    pub fn new(c: AdbClient) -> Self {
        Self { client: c }
    }`n    pub async fn adb_stability_test(&self, serial: &str, secs: u64) -> Result<(u64, u64)> {
        let t0 = std::time::Instant::now();
        let mut success = 0u64;
        let mut failed = 0u64;`n        while t0.elapsed().as_secs() < secs {
            let r = self.client.try_shell(serial, &["getprop", "ro.product.model"]).await;
            match r {
                Ok((true, _, _)) => success += 1,
                _ => failed += 1,
            }
        }`n        Ok((success, failed))
    }
}
