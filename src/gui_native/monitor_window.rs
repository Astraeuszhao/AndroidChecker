use egui::{CentralPanel, Color32, RichText, ScrollArea, SidePanel, TopBottomPanel, Ui};
use crate::adb::AdbClient;
use crate::monitor::{ProcessMonitor, AppManager, ResourceMonitor};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
#[derive(PartialEq)]
enum Tab {
    Processes,
    Performance,
}
pub struct MonitorApp {
    serial: String,
    current_tab: Tab,
    process_monitor: ProcessMonitor,
    app_manager: AppManager,
    resource_monitor: ResourceMonitor,
    runtime: Runtime,
    processes: Arc<Mutex<Vec<ProcessInfo>>>,
    process_filter: String,
    package_to_name: Arc<Mutex<std::collections::HashMap<String, String>>>,
    cpu_usage: Arc<Mutex<f32>>,
    memory_used: Arc<Mutex<u64>>,
    memory_total: Arc<Mutex<u64>>,
    disk_usage: Arc<Mutex<f32>>,
    network_rx: Arc<Mutex<u64>>,
    network_tx: Arc<Mutex<u64>>,
    last_refresh: f64,
    loading: bool,
    selected_process: Option<u32>,
    show_context_menu: bool,
    context_menu_pos: egui::Pos2,
}
#[derive(Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    process_type: ProcessType,
    cpu: f32,
    memory: u64,
    disk: f32,
    network: f32,
}
#[derive(Clone, PartialEq)]
enum ProcessType {
    App,
    Background,
    System,
}
impl MonitorApp {
    pub fn new(client: AdbClient, serial: String) -> Self {
        let process_monitor = ProcessMonitor::new(client.clone());
        let app_manager = AppManager::new(client.clone());
        let resource_monitor = ResourceMonitor::new(client);
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        Self {
            serial,
            current_tab: Tab::Processes,
            process_monitor,
            app_manager,
            resource_monitor,
            runtime,
            processes: Arc::new(Mutex::new(Vec::new())),
            process_filter: String::new(),
            package_to_name: Arc::new(Mutex::new(std::collections::HashMap::new())),
            cpu_usage: Arc::new(Mutex::new(0.0)),
            memory_used: Arc::new(Mutex::new(0)),
            memory_total: Arc::new(Mutex::new(0)),
            disk_usage: Arc::new(Mutex::new(0.0)),
            network_rx: Arc::new(Mutex::new(0)),
            network_tx: Arc::new(Mutex::new(0)),
            last_refresh: 0.0,
            loading: false,
            selected_process: None,
            show_context_menu: false,
            context_menu_pos: egui::Pos2::ZERO,
        }
    }
    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("浠诲姟绠＄悊锟?).size(18.0).strong().color(Color32::from_rgb(80, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("璁惧: {}", self.serial)).color(Color32::from_rgb(180, 180, 180)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_sized([80.0, 28.0], egui::Button::new(RichText::new("鍒锋柊").size(14.0))).clicked() {
                    self.refresh_data();
                }
            });
        });
    }
    fn render_tabs(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(5.0);
            let tab_height = 40.0;
            if ui.add_sized(
                [ui.available_width(), tab_height],
                egui::SelectableLabel::new(
                    self.current_tab == Tab::Processes,
                    RichText::new("杩涚▼").size(15.0)
                )
            ).clicked() {
                self.current_tab = Tab::Processes;
            }
            if ui.add_sized(
                [ui.available_width(), tab_height],
                egui::SelectableLabel::new(
                    self.current_tab == Tab::Performance,
                    RichText::new("鎬ц兘").size(15.0)
                )
            ).clicked() {
                self.current_tab = Tab::Performance;
            }
        });
    }
    fn render_processes_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("鎼滅储:").size(14.0));
            ui.add(egui::TextEdit::singleline(&mut self.process_filter).desired_width(250.0));
        });
        ui.add_space(10.0);
        let processes = self.processes.lock().unwrap().clone();
        let total_count = processes.len();
        let filtered_count = processes.iter().filter(|p| {
            self.process_filter.is_empty() ||
            p.name.to_lowercase().contains(&self.process_filter.to_lowercase())
        }).count();
        ui.label(RichText::new(format!("杩涚▼: {}", filtered_count))
            .size(12.0).color(Color32::from_rgb(150, 150, 150)));
        ui.add_space(5.0);
        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("process_grid")
                .striped(true)
                .spacing([20.0, 8.0])
                .num_columns(6)
                .show(ui, |ui| {
                    ui.label(RichText::new("鍚嶇О").strong().size(13.0));
                    ui.label(RichText::new("PID").strong().size(13.0));
                    ui.label(RichText::new("CPU").strong().size(13.0));
                    ui.label(RichText::new("鍐呭瓨").strong().size(13.0));
                    ui.label(RichText::new("纾佺洏").strong().size(13.0));
                    ui.label(RichText::new("缃戠粶").strong().size(13.0));
                    ui.end_row();
                    let mut app_procs = Vec::new();
                    let mut bg_procs = Vec::new();
                    let mut sys_procs = Vec::new();
                    for process in processes.iter() {
                        if !self.process_filter.is_empty()
                            && !process.name.to_lowercase().contains(&self.process_filter.to_lowercase()) {
                            continue;
                        }
                        match process.process_type {
                            ProcessType::App => app_procs.push(process),
                            ProcessType::Background => bg_procs.push(process),
                            ProcessType::System => sys_procs.push(process),
                        }
                    }
                    if !app_procs.is_empty() {
                        ui.label(RichText::new("搴旂敤杩涚▼").strong().color(Color32::from_rgb(100, 200, 255)));
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        for p in app_procs {
                            self.render_process_row(ui, p);
                        }
                    }
                    if !bg_procs.is_empty() {
                        ui.label(RichText::new("鍚庡彴杩涚▼").strong().color(Color32::from_rgb(255, 200, 100)));
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        for p in bg_procs {
                            self.render_process_row(ui, p);
                        }
                    }
                    if !sys_procs.is_empty() {
                        ui.label(RichText::new("绯荤粺杩涚▼").strong().color(Color32::from_rgb(200, 200, 200)));
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        for p in sys_procs {
                            self.render_process_row(ui, p);
                        }
                    }
                });
        });
    }
    fn render_process_row(&mut self, ui: &mut Ui, process: &ProcessInfo) {
        let response = ui.label(RichText::new(&process.name).size(12.0));
        response.context_menu(|ui| {
            if ui.button("缁撴潫杩涚▼").clicked() {
                self.kill_process(process.pid);
                ui.close_menu();
            }
            if ui.button("鏌ョ湅璇︾粏淇℃伅").clicked() {
                ui.close_menu();
            }
            if ui.button("灞烇拷?).clicked() {
                ui.close_menu();
            }
        });
        ui.label(RichText::new(process.pid.to_string()).size(12.0).color(Color32::from_rgb(180, 180, 180)));
        ui.label(RichText::new(format!("{:.1}%", process.cpu)).size(12.0)
            .color(if process.cpu > 50.0 { Color32::from_rgb(255, 100, 100) } else { Color32::WHITE }));
        ui.label(RichText::new(format!("{:.1} MB", process.memory as f64 / 1024.0 / 1024.0)).size(12.0));
        ui.label(RichText::new(format!("{:.1} MB/s", process.disk)).size(12.0));
        ui.label(RichText::new(format!("{:.1} Mbps", process.network)).size(12.0));
        ui.end_row();
    }
    fn render_performance_tab(&mut self, ui: &mut Ui) {
        ui.heading(RichText::new("鎬ц兘鐩戞帶").size(16.0).color(Color32::from_rgb(80, 180, 255)));
        ui.add_space(15.0);
        let cpu_usage = *self.cpu_usage.lock().unwrap();
        let memory_used = *self.memory_used.lock().unwrap();
        let memory_total = *self.memory_total.lock().unwrap();
        ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(10.0);
                ui.label(RichText::new("CPU").size(16.0).strong());
                ui.add_space(5.0);
                ui.label(RichText::new(format!("浣跨敤锟? {:.1}%", cpu_usage)).size(14.0));
                ui.add_space(8.0);
                let cpu_bar = egui::ProgressBar::new(cpu_usage / 100.0)
                    .text(RichText::new(format!("{:.1}%", cpu_usage)).size(13.0))
                    .desired_height(30.0);
                ui.add(cpu_bar);
                ui.add_space(10.0);
            });
            ui.add_space(20.0);
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(10.0);
                ui.label(RichText::new("鍐呭瓨").size(16.0).strong());
                ui.add_space(5.0);
                let memory_percent = if memory_total > 0 {
                    memory_used as f32 / memory_total as f32
                } else {
                    0.0
                };
                let mem_text = format!(
                    "{:.1} GB / {:.1} GB ({:.1}%)",
                    memory_used as f64 / 1024.0 / 1024.0 / 1024.0,
                    memory_total as f64 / 1024.0 / 1024.0 / 1024.0,
                    memory_percent * 100.0
                );
                ui.label(RichText::new(&mem_text).size(14.0));
                ui.add_space(8.0);
                let mem_bar = egui::ProgressBar::new(memory_percent)
                    .text(RichText::new(format!("{:.1}%", memory_percent * 100.0)).size(13.0))
                    .desired_height(30.0);
                ui.add(mem_bar);
                ui.add_space(10.0);
            });
            ui.add_space(20.0);
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(10.0);
                ui.label(RichText::new("纾佺洏").size(16.0).strong());
                ui.add_space(5.0);
                let disk_usage = *self.disk_usage.lock().unwrap();
                ui.label(RichText::new(format!("浣跨敤锟? {:.1}%", disk_usage)).size(14.0));
                ui.add_space(8.0);
                let disk_bar = egui::ProgressBar::new(disk_usage / 100.0)
                    .text(RichText::new(format!("{:.1}%", disk_usage)).size(13.0))
                    .desired_height(30.0);
                ui.add(disk_bar);
                ui.add_space(10.0);
            });
            ui.add_space(20.0);
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(10.0);
                ui.label(RichText::new("缃戠粶").size(16.0).strong());
                ui.add_space(5.0);
                let net_rx = *self.network_rx.lock().unwrap();
                let net_tx = *self.network_tx.lock().unwrap();
                let rx_mbps = (net_rx as f64 / 1024.0 / 1024.0 * 8.0);
                let tx_mbps = (net_tx as f64 / 1024.0 / 1024.0 * 8.0);
                ui.label(RichText::new(format!("鍙戯拷? {:.2} Mbps | 鎺ユ敹: {:.2} Mbps", tx_mbps, rx_mbps)).size(14.0));
                ui.add_space(8.0);
                let total_mbps = (rx_mbps + tx_mbps) as f32;
                let net_bar = egui::ProgressBar::new((total_mbps / 100.0).min(1.0))
                    .text(RichText::new(format!("{:.2} Mbps", total_mbps)).size(13.0))
                    .desired_height(30.0);
                ui.add(net_bar);
                ui.add_space(10.0);
            });
        });
    }
    fn refresh_data(&mut self) {
        self.load_package_names();
        self.load_processes();
        self.load_resources();
    }
    fn load_package_names(&mut self) {
        let serial = self.serial.clone();
        let app_manager = self.app_manager.clone();
        let pkg_map = Arc::clone(&self.package_to_name);
        self.runtime.spawn(async move {
            if let Ok(apps) = app_manager.list_apps(&serial).await {
                let mut map = pkg_map.lock().unwrap();
                for app in apps {
                    map.insert(app.package_name.clone(), app.app_name);
                }
            }
        });
    }
    fn load_processes(&mut self) {
        self.loading = true;
        let serial = self.serial.clone();
        let process_monitor = self.process_monitor.clone();
        let processes = Arc::clone(&self.processes);
        let pkg_map = Arc::clone(&self.package_to_name);
        self.runtime.spawn(async move {
            if let Ok(proc_list) = process_monitor.list_processes(&serial).await {
                let map = pkg_map.lock().unwrap();
                let mut procs = processes.lock().unwrap();
                *procs = proc_list.into_iter().map(|p| {
                    let process_type = if p.name.starts_with("com.") || p.name.starts_with("org.") {
                        if p.name.contains("android") || p.name.contains("google") {
                            ProcessType::System
                        } else {
                            ProcessType::App
                        }
                    } else if p.name.starts_with("system") || p.name == "init" || p.name == "zygote" {
                        ProcessType::System
                    } else {
                        ProcessType::Background
                    };
                    let display_name = map.get(&p.name).cloned().unwrap_or_else(|| p.name.clone());
                    ProcessInfo {
                        pid: p.pid,
                        name: display_name,
                        process_type,
                        cpu: p.cpu_percent,
                        memory: p.mem_kb * 1024,
                        disk: 0.0,
                        network: 0.0,
                    }
                }).collect();
            }
        });
    }
    fn load_resources(&mut self) {
        let serial = self.serial.clone();
        let resource_monitor = self.resource_monitor.clone();
        let mem_used = Arc::clone(&self.memory_used);
        let mem_total = Arc::clone(&self.memory_total);
        let cpu = Arc::clone(&self.cpu_usage);
        let disk = Arc::clone(&self.disk_usage);
        let net_rx = Arc::clone(&self.network_rx);
        let net_tx = Arc::clone(&self.network_tx);
        self.runtime.spawn(async move {
            if let Ok(mem_info) = resource_monitor.get_memory_info(&serial).await {
                *mem_used.lock().unwrap() = mem_info.used_kb * 1024;
                *mem_total.lock().unwrap() = mem_info.total_kb * 1024;
            }
            if let Ok(cpu_info) = resource_monitor.get_cpu_info(&serial).await {
                *cpu.lock().unwrap() = cpu_info.total_usage;
            }
            if let Ok(disk_info) = resource_monitor.get_disk_info(&serial).await {
                let usage_percent = if disk_info.total_mb > 0 {
                    (disk_info.used_mb as f32 / disk_info.total_mb as f32) * 100.0
                } else {
                    0.0
                };
                *disk.lock().unwrap() = usage_percent;
            }
            if let Ok(net_info) = resource_monitor.get_network_stats(&serial).await {
                *net_rx.lock().unwrap() = net_info.rx_bytes;
                *net_tx.lock().unwrap() = net_info.tx_bytes;
            }
        });
    }
    fn kill_process(&mut self, pid: u32) {
        let serial = self.serial.clone();
        let process_monitor = self.process_monitor.clone();
        self.runtime.spawn(async move {
            let _ = process_monitor.kill_process(&serial, &pid.to_string()).await;
        });
    }
}
impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        if now - self.last_refresh > 2.0 {
            self.refresh_data();
            self.last_refresh = now;
        }
        ctx.request_repaint();
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(5.0);
            self.render_header(ui);
            ui.add_space(5.0);
        });
        SidePanel::left("sidebar").default_width(180.0).show(ctx, |ui| {
            ui.add_space(10.0);
            self.render_tabs(ui);
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            match self.current_tab {
                Tab::Processes => self.render_processes_tab(ui),
                Tab::Performance => self.render_performance_tab(ui),
            }
        });
    }
}