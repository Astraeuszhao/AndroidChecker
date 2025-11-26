use crate::adb::AdbClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};`n#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub user: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub mem_kb: u64,
    pub threads: u32,
    pub state: String,
    pub cmd: String,
}`nimpl ProcessInfo {
    pub fn display_name(&self) -> String {
        if !self.cmd.is_empty() && self.cmd != self.name {
            format!("{} ({})", self.name, self.cmd)
        } else {
            self.name.clone()
        }
    }
}`n#[derive(Clone)]
pub struct ProcessMonitor {
    client: AdbClient,
}`nimpl ProcessMonitor {
    pub fn new(client: AdbClient) -> Self {
        Self { client }
    }`n    pub async fn list_processes(&self, serial: &str) -> Result<Vec<ProcessInfo>> {
        let output = self
            .client
            .invokeShellCommand(serial, &["ps", "-A", "-o", "USER,PID,PPID,VSZ,RSS,%CPU,%MEM,S,ARGS"])
            .await
            .context("Failed to get processes")?;`n        let processes = self.parse_ps_output(&output)?;
        Ok(processes)
    }`n    pub async fn get_process_details(&self, serial: &str, pid: u32) -> Result<ProcessDetails> {
        let stat_cmd = format!("cat /proc/{}/stat", pid);
        let stat = self
            .client
            .invokeShellCommand(serial, &["sh", "-c", &stat_cmd])
            .await
            .unwrap_or_default();`n        let status_cmd = format!("cat /proc/{}/status", pid);
        let status = self
            .client
            .invokeShellCommand(serial, &["sh", "-c", &status_cmd])
            .await
            .unwrap_or_default();`n        Ok(ProcessDetails {
            pid,
            stat,
            status,
            cmdline: String::new(),
            io_stats: String::new(),
        })
    }`n    pub async fn kill_process(&self, serial: &str, package: &str) -> Result<()> {
        self.client
            .invokeShellCommand(serial, &["am", "force-stop", package])
            .await
            .context("Failed to kill process")?;
        Ok(())
    }`n    pub async fn get_top_cpu_processes(&self, serial: &str, count: usize) -> Result<Vec<ProcessInfo>> {
        let mut processes = self.list_processes(serial).await?;
        processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
        Ok(processes.into_iter().take(count).collect())
    }`n    pub async fn get_top_memory_processes(&self, serial: &str, count: usize) -> Result<Vec<ProcessInfo>> {
        let mut processes = self.list_processes(serial).await?;
        processes.sort_by(|a, b| b.mem_kb.cmp(&a.mem_kb));
        Ok(processes.into_iter().take(count).collect())
    }`n    fn parse_ps_output(&self, output: &str) -> Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();
        let lines: Vec<&str> = output.lines().collect();`n        if lines.is_empty() {
            return Ok(processes);
        }`n        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }`n            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }`n            let process = ProcessInfo {
                user: parts[0].to_string(),
                pid: parts[1].parse().unwrap_or(0),
                ppid: parts[2].parse().unwrap_or(0),
                mem_kb: parts[4].parse().unwrap_or(0),
                cpu_percent: parts[5].parse().unwrap_or(0.0),
                mem_percent: parts[6].parse().unwrap_or(0.0),
                state: parts[7].to_string(),
                name: parts.get(8).unwrap_or(&"").to_string(),
                cmd: parts[8..].join(" "),
                threads: 0,
            };`n            processes.push(process);
        }`n        Ok(processes)
    }
}`n#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub stat: String,
    pub status: String,
    pub cmdline: String,
    pub io_stats: String,
}`nimpl ProcessDetails {
    pub fn format_display(&self) -> String {
        format!(
            "Process Details (PID: {})\n\
            ======================================\n\
            Status:\n{}",
            self.pid,
            self.status
        )
    }
}
