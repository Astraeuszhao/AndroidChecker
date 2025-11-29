use crate::adb::AdbClient;
use crate::core::Result;
pub struct RootChecker {
    client: AdbClient,
}
impl RootChecker {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }
    pub async fn has_root(&self, serial: &str) -> Result<bool> {
        let (success1, output1, _) = self
            .client
            .try_shell(serial, &["su", "-c", "id"])
            .await?;
        if success1 && output1.contains("uid=0") {
            return Ok(true);
        }
        let (success2, output2, _) = self
            .client
            .try_shell(serial, &["su", "0", "id"])
            .await?;
        if success2 && output2.contains("uid=0") {
            return Ok(true);
        }
        let (success3, output3, _) = self
            .client
            .try_shell(serial, &["id"])
            .await?;
        Ok(success3 && output3.contains("uid=0"))
    }
    pub async fn request_root(&self, serial: &str) -> Result<bool> {
        let (success, _, _) = self
            .client
            .try_shell(serial, &["su", "-c", "echo", "test"])
            .await?;
        Ok(success)
    }
}