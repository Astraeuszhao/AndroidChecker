use std::time::Instant;

#[derive(Clone, Debug, Default)]
pub struct ProcessInfo {
    pub pid: String,
    pub user: String,
    pub cpu: f32,
    pub mem: f32,
    pub name: String,
    pub cmdline: String,
}

#[derive(Clone, Debug, Default)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_speed: u64,
    pub tx_speed: u64,
}

#[derive(Clone, Debug, Default)]
pub struct DiskStats {
    pub total: u64,
    pub used: u64,
    pub path: String,
}

#[derive(Default)]
pub struct SystemState {
    pub cpu_history: Vec<f32>,
    pub mem_history: Vec<f32>,
    pub net_rx_history: Vec<f32>,
    pub net_tx_history: Vec<f32>,
    
    pub processes: Vec<ProcessInfo>,
    pub device_info: String,
    pub last_update: Option<Instant>,
    
    pub total_cpu: f32,
    pub mem_total: u64,
    pub mem_available: u64,
    pub net_stats: NetworkStats,
    pub disk_stats: DiskStats,
    
    pub last_cpu_stat: Option<(u64, u64)>,
    pub last_net_stat: Option<(Instant, u64, u64)>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Nav {
    Processes,
    Performance,
}
