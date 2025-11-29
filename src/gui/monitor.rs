use crate::adb::AdbClient;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default)]
pub struct ProcessInfo {
    pub pid: String,
    pub user: String,
    pub cpu: f32,
    pub mem: f32,
    pub name: String,
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

#[derive(PartialEq)]
enum Nav {
    Processes,
    Performance,
}

pub struct MonitorApp {
    state: Arc<Mutex<SystemState>>,
    filter: String,
    client: AdbClient,
    serial: String,
    nav: Nav,
    selected_pid: Option<String>,
}

impl MonitorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: Arc<Mutex<SystemState>>, client: AdbClient, serial: String) -> Self {
        Self {
            state,
            filter: String::new(),
            client,
            serial,
            nav: Nav::Processes,
            selected_pid: None,
        }
    }
    
    fn kill_process(&self, pid: &str, force: bool) {
        let client = self.client.clone();
        let serial = self.serial.clone();
        let pid = pid.to_string();
        tokio::spawn(async move {
            if force {
                let _ = client.try_execute(&["-s", &serial, "shell", "kill", "-9", &pid]).await;
            } else {
                 let _ = client.try_execute(&["-s", &serial, "shell", "kill", &pid]).await;
            }
        });
    }
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state_snapshot = {
            let s = self.state.lock().unwrap();

            (
                s.processes.clone(),
                s.cpu_history.clone(),
                s.mem_history.clone(),
                s.net_rx_history.clone(),
                s.net_tx_history.clone(),
                s.total_cpu,
                s.mem_total,
                s.mem_available,
                s.net_stats.clone(),
                s.disk_stats.clone(),
            )
        };

        let (
            processes, 
            cpu_hist, 
            mem_hist, 
            net_rx_hist, 
            net_tx_hist, 
            total_cpu, 
            mem_total, 
            mem_avail, 
            net_stats, 
            disk_stats
        ) = state_snapshot;
        
        let mem_used = mem_total.saturating_sub(mem_avail);
        let mem_percent = if mem_total > 0 { mem_used as f32 / mem_total as f32 * 100.0 } else { 0.0 };

        egui::SidePanel::left("sidebar").resizable(false).default_width(150.0).show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading("Monitor");
            ui.add_space(20.0);
            
            if ui.selectable_label(self.nav == Nav::Processes, "Processes").clicked() {
                self.nav = Nav::Processes;
            }
            if ui.selectable_label(self.nav == Nav::Performance, "Performance").clicked() {
                self.nav = Nav::Performance;
            }
            
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(format!("Device: {}", self.serial));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.nav {
                Nav::Processes => {
                    ui.heading("Processes");
                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.text_edit_singleline(&mut self.filter);
                        if ui.button("Refresh").clicked() {

                        }
                    });
                    ui.separator();
                    
                    use egui_extras::{Column, TableBuilder};
                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    
                    let filtered: Vec<&ProcessInfo> = processes.iter()
                        .filter(|p| self.filter.is_empty() 
                            || p.name.to_lowercase().contains(&self.filter.to_lowercase()) 
                            || p.pid.contains(&self.filter))
                        .collect();

                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(Column::initial(60.0)) 
                        .column(Column::initial(80.0)) 
                        .column(Column::initial(50.0)) 
                        .column(Column::initial(50.0)) 
                        .column(Column::remainder())   
                        .header(20.0, |mut header| {
                            header.col(|ui| { ui.strong("PID"); });
                            header.col(|ui| { ui.strong("User"); });
                            header.col(|ui| { ui.strong("CPU%"); });
                            header.col(|ui| { ui.strong("Mem%"); });
                            header.col(|ui| { ui.strong("Name"); });
                        })
                        .body(|mut body| {
                            for p in filtered {
                                body.row(text_height, |mut row| {
                                    row.col(|ui| { ui.label(&p.pid); });
                                    row.col(|ui| { ui.label(&p.user); });
                                    row.col(|ui| { ui.label(format!("{:.1}", p.cpu)); });
                                    row.col(|ui| { ui.label(format!("{:.1}", p.mem)); });
                                    let response = row.col(|ui| { ui.label(&p.name); }).1;
                                    
                                    if response.clicked() {
                                        self.selected_pid = Some(p.pid.clone());
                                    }
                                    
                                    response.context_menu(|ui| {
                                        if ui.button("Kill Process").clicked() {
                                            self.kill_process(&p.pid, false);
                                            ui.close_menu();
                                        }
                                        if ui.button("Force Stop (Kill -9)").clicked() {
                                            self.kill_process(&p.pid, true);
                                            ui.close_menu();
                                        }
                                    });
                                });
                            }
                        });
                }
                Nav::Performance => {
                    ui.heading("Performance");
                    ui.add_space(10.0);
                    
                    let available_width = ui.available_width();
                    let card_width = (available_width - 20.0) / 2.0;
                    
                    egui::Grid::new("perf_grid").spacing([20.0, 20.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(card_width);
                            ui.group(|ui| {
                                ui.heading(format!("CPU {:.1}%", total_cpu));
                                draw_simple_graph(ui, &cpu_hist, 100.0, egui::Color32::from_rgb(255, 80, 80));
                            });
                        });
                        
                        ui.vertical(|ui| {
                            ui.set_width(card_width);
                            ui.group(|ui| {
                                let used_gb = mem_used as f32 / 1024.0 / 1024.0;
                                let total_gb = mem_total as f32 / 1024.0 / 1024.0;
                                ui.heading(format!("Memory {:.1} / {:.1} GB ({:.0}%)", used_gb, total_gb, mem_percent));
                                draw_simple_graph(ui, &mem_hist, 100.0, egui::Color32::from_rgb(80, 150, 255));
                            });
                        });
                        ui.end_row();
                        
                        ui.vertical(|ui| {
                            ui.set_width(card_width);
                            ui.group(|ui| {
                                ui.heading("Network");
                                ui.label(format!("Download: {:.1} KB/s", net_stats.rx_speed as f32 / 1024.0));
                                ui.label(format!("Upload:   {:.1} KB/s", net_stats.tx_speed as f32 / 1024.0));
                                 
                                let max_rx = net_rx_hist.iter().cloned().fold(1.0f32, f32::max);
                                draw_simple_graph(ui, &net_rx_hist, max_rx, egui::Color32::from_rgb(80, 200, 80));
                            });
                        });
                        
                        ui.vertical(|ui| {
                            ui.set_width(card_width);
                            ui.group(|ui| {
                                ui.heading("Storage (Internal)");
                                let used_gb = disk_stats.used as f32 / 1024.0 / 1024.0;
                                let total_gb = disk_stats.total as f32 / 1024.0 / 1024.0;
                                ui.label(format!("{:.1} GB used / {:.1} GB total", used_gb, total_gb));
                                let usage = if disk_stats.total > 0 { disk_stats.used as f32 / disk_stats.total as f32 } else { 0.0 };
                                ui.add(egui::ProgressBar::new(usage).text(format!("{:.1}%", usage * 100.0)));
                            });
                        });
                        ui.end_row();
                    });
                }
            }
        });

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn draw_simple_graph(ui: &mut egui::Ui, data: &[f32], max_val: f32, color: egui::Color32) {
    let (response, painter) = ui.allocate_painter(egui::Vec2::new(ui.available_width(), 80.0), egui::Sense::hover());
    let rect = response.rect;

    painter.rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::from_gray(40)));

    if data.len() < 2 {
        return;
    }

    let points: Vec<egui::Pos2> = data.iter().enumerate().map(|(i, &val)| {
        let x = rect.min.x + (i as f32 / (data.len() - 1) as f32) * rect.width();
        let y = rect.max.y - (val / max_val).clamp(0.0, 1.0) * rect.height();
        egui::Pos2::new(x, y)
    }).collect();

    let mut shape_points = points.clone();
    shape_points.push(egui::Pos2::new(rect.max.x, rect.max.y));
    shape_points.push(egui::Pos2::new(rect.min.x, rect.max.y));
    
    painter.add(egui::Shape::convex_polygon(
        shape_points,
        color.linear_multiply(0.2),
        egui::Stroke::NONE,
    ));

    let stroke = egui::Stroke::new(2.0, color);
    painter.add(egui::Shape::line(points, stroke));
}

pub fn run_monitor(client: AdbClient, serial: String) -> anyhow::Result<()> {
    let state = Arc::new(Mutex::new(SystemState::default()));
    
    let state_clone = state.clone();
    let client_clone = client.clone();
    let serial_clone = serial.clone();
    
    tokio::spawn(async move {
        fetch_loop(client_clone, serial_clone, state_clone).await;
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Android System Monitor")
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()), 
        ..Default::default()
    };

    eframe::run_native(
        "System Monitor",
        options,
        Box::new(move |cc| Ok(Box::new(MonitorApp::new(cc, state, client, serial)) as Box<dyn eframe::App>)),
    ).map_err(|e| anyhow::anyhow!("GUI Error: {}", e))?;
    
    Ok(())
}

async fn fetch_loop(client: AdbClient, serial: String, state: Arc<Mutex<SystemState>>) {
    loop {
        if let Ok((_, output, _)) = client.try_execute(&["-s", &serial, "shell", "cat", "/proc/stat"]).await {
            if let Some((total_cpu, new_stat)) = parse_proc_stat(&output, &state) {
                 let mut s = state.lock().unwrap();
                 s.total_cpu = total_cpu;
                 s.last_cpu_stat = Some(new_stat);
                 s.cpu_history.push(total_cpu);
                 if s.cpu_history.len() > 60 { s.cpu_history.remove(0); }
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
            let busy = total - idle;
            
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
    if s.mem_history.len() > 60 { s.mem_history.remove(0); }
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
    if s.net_rx_history.len() > 60 { s.net_rx_history.remove(0); }
    
    s.net_tx_history.push(tx_speed as f32); 
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
                });
            }
        }
    }
    
    processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut s = state.lock().unwrap();
    s.processes = processes;
}
