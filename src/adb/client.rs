use crate::core::{AdbError, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
#[derive(Debug, Clone)]
pub struct AdbClient {
    adb_path: PathBuf,
}
impl AdbClient {
    pub fn new() -> Result<Self> {
        let adb_path = Self::resolve_adb()?;
        Ok(Self { adb_path })
    }
    fn resolve_adb() -> Result<PathBuf> {
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let bundled = parent.join("platform-tools").join("adb.exe");
                if bundled.exists() {
                    return Ok(bundled);
                }
                let vendor_bundled = parent.join("vendor").join("platform-tools").join("adb.exe");
                if vendor_bundled.exists() {
                    return Ok(vendor_bundled);
                }
                if let Some(project_root) = parent.parent().and_then(|p| p.parent()) {
                     let dev_bundled = project_root.join("vendor").join("platform-tools").join("adb.exe");
                     if dev_bundled.exists() {
                         return Ok(dev_bundled);
                     }
                }
            }
        }
        let local_vendor = PathBuf::from("vendor/platform-tools/adb.exe");
        if local_vendor.exists() {
             if let Ok(abs) = std::fs::canonicalize(local_vendor) {
                 return Ok(abs);
             }
        }
        if let Ok(output) = std::process::Command::new("where").arg("adb").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = path_str.lines().next() {
                    return Ok(PathBuf::from(line.trim()));
                }
            }
        }
        Err(AdbError::AdbNotFound)
    }
    pub async fn ensure_present(&self) -> Result<()> {
        let output = timeout(
            Duration::from_secs(10),
            Command::new(&self.adb_path).arg("version").output(),
        )
        .await
        .map_err(|_| AdbError::Timeout(10))??;
        if output.status.success() {
            Ok(())
        } else {
            Err(AdbError::CommandFailed("adb version failed".to_string()))
        }
    }
    pub async fn start_server(&self) -> Result<()> {
        timeout(
            Duration::from_secs(10),
            Command::new(&self.adb_path).arg("start-server").output(),
        )
        .await
        .map_err(|_| AdbError::Timeout(10))??;
        Ok(())
    }
    pub async fn execute(&self, args: &[&str]) -> Result<(String, String)> {
        let output = Command::new(&self.adb_path).args(args).output().await?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            return Err(AdbError::CommandFailed(stderr.clone()));
        }
        Ok((stdout, stderr))
    }
    pub async fn try_execute(&self, args: &[&str]) -> Result<(bool, String, String)> {
        let output = Command::new(&self.adb_path).args(args).output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((output.status.success(), stdout, stderr))
    }
    pub async fn shell(&self, serial: &str, cmd: &[&str]) -> Result<String> {
        let mut args = vec!["-s", serial, "shell"];
        args.extend_from_slice(cmd);
        let (stdout, stderr) = self.execute(&args).await?;
        if !stderr.is_empty() && stderr.to_lowercase().contains("error") {
            return Err(AdbError::CommandFailed(stderr));
        }
        Ok(stdout)
    }
    pub async fn try_shell(&self, serial: &str, cmd: &[&str]) -> Result<(bool, String, String)> {
        let mut args = vec!["-s", serial, "shell"];
        args.extend_from_slice(cmd);
        self.try_execute(&args).await
    }
    pub async fn logcat_stream(
        &self,
        serial: &str,
        mut callback: impl FnMut(String) + Send + 'static,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let adb_path = self.adb_path.clone();
        let serial = serial.to_string();
        let handle = tokio::spawn(async move {
            let mut child = match Command::new(&adb_path)
                .args(&["-s", &serial, "logcat"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to spawn logcat: {}", e);
                    return;
                }
            };
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    callback(line);
                }
            }
            let _ = child.wait().await;
        });
        Ok(handle)
    }
    pub async fn parallel_shell(&self, serial: &str, commands: Vec<Vec<String>>) -> Vec<Result<String>> {
        let tasks: Vec<_> = commands
            .into_iter()
            .map(|cmd| {
                let client = self.clone();
                let serial = serial.to_string();
                tokio::spawn(async move {
                    let cmd_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
                    client.shell(&serial, &cmd_refs).await
                })
            })
            .collect();
        let mut results = Vec::new();
        for task in tasks {
            results.push(match task.await {
                Ok(r) => r,
                Err(e) => Err(AdbError::CommandFailed(e.to_string())),
            });
        }
        results
    }
}
impl Default for AdbClient {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ADB client")
    }
}