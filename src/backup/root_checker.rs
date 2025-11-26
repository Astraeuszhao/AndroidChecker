use crate::adb::AdbClient;
use crate::core::Result;`npub struct RootChecker {
    client: AdbClient,
}`nimpl RootChecker {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }`n    pub async fn has_root(&self, serial: &str) -> Result<bool> {
        let (success1, output1, _) = self
            .client
            .try_shell(serial, &["su", "-c", "id"])
            .await?;`n        if success1 && output1.contains("uid=0") {
            return Ok(true);
        }`n        let (success2, output2, _) = self
            .client
            .try_shell(serial, &["su", "0", "id"])
            .await?;`n        if success2 && output2.contains("uid=0") {
            return Ok(true);
        }`n        let (success3, output3, _) = self
            .client
            .try_shell(serial, &["id"])
            .await?;`n        Ok(success3 && output3.contains("uid=0"))
    }`n    pub async fn request_root(&self, serial: &str) -> Result<bool> {
        let (success, _, _) = self
            .client
            .try_shell(serial, &["su", "-c", "echo", "test"])
            .await?;`n        Ok(success)
    }
}
