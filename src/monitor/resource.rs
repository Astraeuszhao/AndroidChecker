use crate::adb::AdbClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub cores: Vec<CpuCore>,
    pub total_usage: f32,
    pub temperature: Option<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCore {
    pub core_id: usize,
    pub usage_percent: f32,
    pub current_freq: u64,
    pub max_freq: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub available_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub cached_kb: u64,
    pub usage_percent: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub partitions: Vec<DiskPartition>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPartition {
    pub mount_point: String,
    pub filesystem: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub usage_percent: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDiskInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub usage_percent: f32,
}
#[derive(Clone)]
pub struct ResourceMonitor {
    client: AdbClient,
}
impl ResourceMonitor {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }
    pub async fn get_cpu_info(&self, serial: &str) -> Result<CpuInfo> {
        let cpu_count = 4;
        let mut cores = Vec::new();
        for i in 0..cpu_count {
            cores.push(CpuCore {
                core_id: i,
                usage_percent: 0.0,
                current_freq: 0,
                max_freq: 2400000,
            });
        }
        Ok(CpuInfo {
            cores,
            total_usage: 0.0,
            temperature: None,
        })
    }
    pub async fn get_memory_info(&self, serial: &str) -> Result<MemoryInfo> {
        let output = self
            .client
            .invokeShellCommand(serial, &["cat", "/proc/meminfo"])
            .await
            .context("Failed to read memory info")?;
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        let mut free_kb = 0u64;
        let mut cached_kb = 0u64;
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let value: u64 = parts[1].parse().unwrap_or(0);
            match parts[0] {
                "MemTotal:" => total_kb = value,
                "MemAvailable:" => available_kb = value,
                "MemFree:" => free_kb = value,
                "Cached:" => cached_kb = value,
                _ => {}
            }
        }
        let used_kb = total_kb.saturating_sub(available_kb);
        let usage_percent = if total_kb > 0 {
            (used_kb as f32 / total_kb as f32) * 100.0
        } else {
            0.0
        };
        Ok(MemoryInfo {
            total_kb,
            available_kb,
            used_kb,
            free_kb,
            cached_kb,
            usage_percent,
        })
    }
    pub async fn get_disk_info(&self, serial: &str) -> Result<SimpleDiskInfo> {
        let output = self
            .client
            .invokeShellCommand(serial, &["df", "/data"])
            .await
            .context("Failed to read disk info")?;
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                continue;
            }
            let total = parts[1].parse::<u64>().unwrap_or(0) / 1024;
            let used = parts[2].parse::<u64>().unwrap_or(0) / 1024;
            let available = parts[3].parse::<u64>().unwrap_or(0) / 1024;
            let usage_percent = parts[4].trim_end_matches('%').parse().unwrap_or(0.0);
            return Ok(SimpleDiskInfo {
                total_mb: total,
                used_mb: used,
                available_mb: available,
                usage_percent,
            });
        }
        Ok(SimpleDiskInfo {
            total_mb: 0,
            used_mb: 0,
            available_mb: 0,
            usage_percent: 0.0,
        })
    }
    pub async fn get_all_disk_info(&self, serial: &str) -> Result<DiskInfo> {
        let output = self
            .client
            .invokeShellCommand(serial, &["df", "-h"])
            .await
            .context("Failed to read disk info")?;
        let mut partitions = Vec::new();
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                continue;
            }
            let filesystem = parts[0].to_string();
            let total = Self::parse_size(parts[1]);
            let used = Self::parse_size(parts[2]);
            let available = Self::parse_size(parts[3]);
            let usage_percent = parts[4].trim_end_matches('%').parse().unwrap_or(0.0);
            let mount_point = parts[5].to_string();
            partitions.push(DiskPartition {
                mount_point,
                filesystem,
                total_mb: total,
                used_mb: used,
                available_mb: available,
                usage_percent,
            });
        }
        Ok(DiskInfo { partitions })
    }
    pub async fn get_network_stats(&self, serial: &str) -> Result<NetworkStats> {
        let output = self
            .client
            .invokeShellCommand(serial, &["cat", "/proc/net/dev"])
            .await
            .context("Failed to read network info")?;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        for line in output.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }
            let name = parts[0].trim_end_matches(':').to_string();
            if name == "lo" {
                continue;
            }
            let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
            let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
            total_rx += rx_bytes;
            total_tx += tx_bytes;
        }
        Ok(NetworkStats {
            rx_bytes: total_rx,
            tx_bytes: total_tx,
        })
    }
    pub async fn get_network_info(&self, serial: &str) -> Result<NetworkInfo> {
        let output = self
            .client
            .invokeShellCommand(serial, &["cat", "/proc/net/dev"])
            .await
            .context("Failed to read network info")?;
        let mut interfaces = Vec::new();
        for line in output.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }
            let name = parts[0].trim_end_matches(':').to_string();
            if name == "lo" {
                continue;
            }
            let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
            let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
            interfaces.push(NetworkInterface {
                name,
                rx_bytes,
                tx_bytes,
            });
        }
        Ok(NetworkInfo { interfaces })
    }
    fn parse_size(size_str: &str) -> u64 {
        let size_str = size_str.trim();
        let (num_str, unit) = size_str.split_at(size_str.len().saturating_sub(1));
        let num: f64 = num_str.parse().unwrap_or(0.0);
        let multiplier = match unit.to_uppercase().as_str() {
            "G" => 1024.0,
            "M" => 1.0,
            "K" => 1.0 / 1024.0,
            _ => 1.0,
        };
        (num * multiplier) as u64
    }
}