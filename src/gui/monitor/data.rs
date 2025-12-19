use super::state::{DiskStats, NetworkStats, ProcessInfo, SystemState};
use crate::adb::AdbClient;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub async fn fetch_loop(client: AdbClient, serial: String, state: Arc<Mutex<SystemState>>) {
    loop {
        if let Ok((_, output, _)) = client.try_execute(&["-s", &serial, "shell", "cat", "/proc/stat"]).await {
            if let Some((total_cpu, new_stat)) = parse_proc_stat(&output, &state) {
                 let mut s = state.lock().unwrap();
                 s.total_cpu = total_cpu;
                 s.last_cpu_stat = Some(new_stat);
                 s.cpu_history.push(total_cpu);
                 if s.cpu_history.len() > 120 { s.cpu_history.remove(0); }
            }
        }
        
        if let Ok((_, output, _)) = client.try_execute(&["-s", &serial, "shell", "cat", "/proc/meminfo"]).await {
            parse_meminfo(&output, &state);
        }

        if let Ok((_, output, _)) = client.try_execute(&["-s", &serial, "shell", "cat", "/proc/net/dev"]).await {
            parse_net_dev(&output, &state);
        }

        if let Ok((_, output, _)) = client.try_execute(&["-s", &serial, "shell", "df", "-h", "/data"]).await {
            parse_disk(&output, &state);
        }

        if let Ok((_, output, _)) = client.try_execute(&["-s", &serial, "shell", "top", "-b", "-n", "1"]).await {
            parse_top_output(&output, &state);
        }

        {
             let mut s = state.lock().unwrap();
             s.last_update = Some(Instant::now());
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn parse_proc_stat(output: &str, state: &Arc<Mutex<SystemState>>) -> Option<(f32, (u64, u64))> {
    if let Some(line) = output.lines().find(|l| l.starts_with("cpu ")) {
        let parts: Vec<u64> = line.split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
            
        if parts.len() >= 4 {
            let user = parts[0];
            let nice = parts[1];
            let system = parts[2];
            let idle = parts[3];
            let iowait = if parts.len() > 4 { parts[4] } else { 0 };
            let irq = if parts.len() > 5 { parts[5] } else { 0 };
            let softirq = if parts.len() > 6 { parts[6] } else { 0 };
            
            let total = user + nice + system + idle + iowait + irq + softirq;
            
            let last_stat = {
                let s = state.lock().unwrap();
                s.last_cpu_stat
            };
            
            if let Some((last_total, last_idle)) = last_stat {
                let d_total = total.saturating_sub(last_total);
                let d_idle = idle.saturating_sub(last_idle);
                
                if d_total > 0 {
                    let usage = 1.0 - (d_idle as f32 / d_total as f32);
                    return Some((usage * 100.0, (total, idle)));
                }
            } else {
                return Some((0.0, (total, idle)));
            }
        }
    }
    None
}

fn parse_meminfo(output: &str, state: &Arc<Mutex<SystemState>>) {
    let mut total = 0;
    let mut available = 0;
    
    for line in output.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_kb(line);
        } else if line.starts_with("MemAvailable:") {
            available = parse_kb(line);
        }
    }
    
    let mut s = state.lock().unwrap();
    s.mem_total = total;
    s.mem_available = available;
    
    let used = total.saturating_sub(available);
    let percent = if total > 0 { used as f32 / total as f32 * 100.0 } else { 0.0 };
    
    s.mem_history.push(percent);
    if s.mem_history.len() > 120 { s.mem_history.remove(0); }
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

fn parse_net_dev(output: &str, state: &Arc<Mutex<SystemState>>) {
    let mut rx_total = 0;
    let mut tx_total = 0;
    
    for line in output.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 9 {
             let iface = parts[0].trim_end_matches(':');
             if iface != "lo" {
                 let rx = parts[1].parse::<u64>().unwrap_or(0);
                 let tx = if parts.len() >= 10 { parts[9].parse::<u64>().unwrap_or(0) } else { 0 };
                 rx_total += rx;
                 tx_total += tx;
             }
        }
    }
    
    let now = Instant::now();
    let last_stat = {
        let s = state.lock().unwrap();
        s.last_net_stat
    };
    
    let mut rx_speed = 0;
    let mut tx_speed = 0;
    
    if let Some((last_time, last_rx, last_tx)) = last_stat {
        let dur = now.duration_since(last_time).as_secs_f32();
        if dur > 0.0 {
            rx_speed = ((rx_total.saturating_sub(last_rx)) as f32 / dur) as u64;
            tx_speed = ((tx_total.saturating_sub(last_tx)) as f32 / dur) as u64;
        }
    }
    
    let mut s = state.lock().unwrap();
    s.net_stats = NetworkStats {
        rx_bytes: rx_total,
        tx_bytes: tx_total,
        rx_speed,
        tx_speed,
    };
    s.last_net_stat = Some((now, rx_total, tx_total));
    
    s.net_rx_history.push(rx_speed as f32);
    if s.net_rx_history.len() > 120 { s.net_rx_history.remove(0); }
    
    s.net_tx_history.push(tx_speed as f32);
    if s.net_tx_history.len() > 120 { s.net_tx_history.remove(0); }
}

fn parse_disk(output: &str, state: &Arc<Mutex<SystemState>>) {
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let size_str = parts[1];
            let used_str = parts[2];
            
            let size_kb = parse_size_str(size_str);
            let used_kb = parse_size_str(used_str);
            
            let mut s = state.lock().unwrap();
            s.disk_stats = DiskStats {
                total: size_kb,
                used: used_kb,
                path: "/data".to_string(),
            };
            break;
        }
    }
}

fn parse_size_str(s: &str) -> u64 {
    let len = s.len();
    if len == 0 { return 0; }
    let unit = &s[len-1..];
    let val_str = &s[..len-1];
    let val = val_str.parse::<f32>().unwrap_or(0.0);
    
    match unit {
        "G" => (val * 1024.0 * 1024.0) as u64,
        "M" => (val * 1024.0) as u64,
        "K" => val as u64,
        _ => val as u64,
    }
}

fn parse_top_output(output: &str, state: &Arc<Mutex<SystemState>>) {
    let lines = output.lines();
    let mut processes = Vec::new();
    
    let mut headers_found = false;
    for line in lines {
        if line.trim().is_empty() { continue; }
        if line.contains("PID") && line.contains("USER") {
            headers_found = true;
            continue;
        }
        
        if headers_found {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let pid = parts.get(0).unwrap_or(&"?").to_string();
                let user = parts.get(1).unwrap_or(&"?").to_string();
                
                let cpu_val = parts.get(8).and_then(|s| s.trim_end_matches('%').parse::<f32>().ok()).unwrap_or(0.0);
                let mem_val = parts.get(9).and_then(|s| s.trim_end_matches('%').parse::<f32>().ok()).unwrap_or(0.0);
                let name = parts.last().unwrap_or(&"?").to_string();
                
                processes.push(ProcessInfo {
                    pid,
                    user,
                    cpu: cpu_val,
                    mem: mem_val,
                    name,
                    cmdline: String::new(),
                });
            }
        }
    }
    
    processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut s = state.lock().unwrap();
    s.processes = processes;
}
