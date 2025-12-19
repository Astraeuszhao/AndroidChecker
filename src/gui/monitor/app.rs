use super::state::{Nav, ProcessInfo, SystemState};
use crate::adb::AdbClient;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use egui_extras::{Column, TableBuilder};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct MonitorApp {
    state: Arc<Mutex<SystemState>>,
    filter: String,
    client: AdbClient,
    serial: String,
    nav: Nav,
    selected_pid: Option<String>,
    perf_tab: PerfTab,
}

#[derive(PartialEq, Clone, Copy)]
enum PerfTab {
    Cpu,
    Memory,
    Network,
    Disk,
}

impl MonitorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: Arc<Mutex<SystemState>>, client: AdbClient, serial: String) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        configure_visuals(&cc.egui_ctx);
        
        Self {
            state,
            filter: String::new(),
            client,
            serial,
            nav: Nav::Processes,
            selected_pid: None,
            perf_tab: PerfTab::Cpu,
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

        render_sidebar(ctx, &mut self.nav, &self.serial);

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.nav {
                Nav::Processes => {
                    ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
                    ui.heading("Processes");
                    
                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Search name or PID..."));
                        if ui.button("Refresh").clicked() {
                        }
                    });
                    ui.separator();
                    
                    let filtered: Vec<&ProcessInfo> = processes.iter()
                        .filter(|p| self.filter.is_empty() 
                            || p.name.to_lowercase().contains(&self.filter.to_lowercase()) 
                            || p.pid.contains(&self.filter))
                        .collect();

                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(60.0).resizable(true)) 
                        .column(Column::initial(80.0).resizable(true)) 
                        .column(Column::initial(60.0).resizable(true)) 
                        .column(Column::initial(60.0).resizable(true)) 
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
                                    row.col(|ui| { 
                                        let color = if p.cpu > 20.0 { Color32::from_rgb(255, 100, 100) } else { ui.visuals().text_color() };
                                        ui.colored_label(color, format!("{:.1}", p.cpu)); 
                                    });
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
                    ui.columns(2, |cols| {
                        let perf_tab = self.perf_tab;
                        
                        cols[0].vertical(|ui| {
                            ui.set_width(ui.available_width());
                            
                            let selected_bg = Color32::from_gray(60);
                            
                            if perf_card(ui, "CPU", format!("{:.1}%", total_cpu).as_str(), &cpu_hist, perf_tab == PerfTab::Cpu) {
                                self.perf_tab = PerfTab::Cpu;
                            }
                            if perf_card(ui, "Memory", format!("{:.1} GB", mem_used as f32 / 1024.0 / 1024.0).as_str(), &mem_hist, perf_tab == PerfTab::Memory) {
                                self.perf_tab = PerfTab::Memory;
                            }
                            if perf_card(ui, "Disk", format!("{:.0}%", if disk_stats.total > 0 { disk_stats.used as f32 / disk_stats.total as f32 * 100.0 } else { 0.0 }).as_str(), &vec![], perf_tab == PerfTab::Disk) {
                                self.perf_tab = PerfTab::Disk;
                            }
                            if perf_card(ui, "Network", format!("D: {:.1} K", net_stats.rx_speed as f32 / 1024.0).as_str(), &net_rx_hist, perf_tab == PerfTab::Network) {
                                self.perf_tab = PerfTab::Network;
                            }
                        });
                        
                        cols[1].vertical(|ui| {
                            ui.add_space(10.0);
                            match perf_tab {
                                PerfTab::Cpu => {
                                    ui.heading("CPU");
                                    ui.label(format!("Usage: {:.1}%", total_cpu));
                                    draw_large_graph(ui, &cpu_hist, 100.0, Color32::from_rgb(59, 130, 246));
                                }
                                PerfTab::Memory => {
                                    ui.heading("Memory");
                                    let used_gb = mem_used as f32 / 1024.0 / 1024.0;
                                    let total_gb = mem_total as f32 / 1024.0 / 1024.0;
                                    let avail_gb = mem_avail as f32 / 1024.0 / 1024.0;
                                    
                                    ui.label(format!("Used: {:.1} GB", used_gb));
                                    ui.label(format!("Available: {:.1} GB", avail_gb));
                                    ui.label(format!("Total: {:.1} GB", total_gb));
                                    
                                    draw_large_graph(ui, &mem_hist, 100.0, Color32::from_rgb(168, 85, 247));
                                }
                                PerfTab::Disk => {
                                    ui.heading("Disk (Internal)");
                                    let used_gb = disk_stats.used as f32 / 1024.0 / 1024.0;
                                    let total_gb = disk_stats.total as f32 / 1024.0 / 1024.0;
                                    ui.label(format!("Capacity: {:.1} GB", total_gb));
                                    ui.label(format!("Used: {:.1} GB", used_gb));
                                    
                                    let usage = if disk_stats.total > 0 { disk_stats.used as f32 / disk_stats.total as f32 } else { 0.0 };
                                    ui.add(egui::ProgressBar::new(usage).text(format!("{:.1}%", usage * 100.0)));
                                }
                                PerfTab::Network => {
                                    ui.heading("Network");
                                    ui.label(format!("Download: {:.1} KB/s", net_stats.rx_speed as f32 / 1024.0));
                                    ui.label(format!("Upload:   {:.1} KB/s", net_stats.tx_speed as f32 / 1024.0));
                                    ui.label(format!("Total Rx: {:.1} MB", net_stats.rx_bytes as f32 / 1024.0 / 1024.0));
                                    ui.label(format!("Total Tx: {:.1} MB", net_stats.tx_bytes as f32 / 1024.0 / 1024.0));
                                    
                                    let max_val = net_rx_hist.iter().cloned().fold(1.0f32, f32::max).max(net_tx_hist.iter().cloned().fold(1.0f32, f32::max));
                                    
                                    ui.label("Download");
                                    draw_large_graph(ui, &net_rx_hist, max_val, Color32::from_rgb(34, 197, 94));
                                    ui.add_space(10.0);
                                    ui.label("Upload");
                                    draw_large_graph(ui, &net_tx_hist, max_val, Color32::from_rgb(234, 179, 8));
                                }
                            }
                        });
                    });
                }
            }
        });
        
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn render_sidebar(ctx: &egui::Context, nav: &mut Nav, serial: &str) {
    egui::SidePanel::left("sidebar").resizable(false).default_width(60.0).show(ctx, |ui| {
        ui.add_space(10.0);
        
        if ui.add(egui::Button::new(RichText::new("☰").size(20.0)).frame(false)).clicked() {
            // Toggle expanded sidebar? For now just icon
        }
        ui.add_space(20.0);
        
        let btn_proc = ui.add_sized([40.0, 40.0], egui::SelectableLabel::new(*nav == Nav::Processes, "💻"));
        if btn_proc.clicked() { *nav = Nav::Processes; }
        btn_proc.on_hover_text("Processes");
        
        ui.add_space(10.0);
        
        let btn_perf = ui.add_sized([40.0, 40.0], egui::SelectableLabel::new(*nav == Nav::Performance, "📊"));
        if btn_perf.clicked() { *nav = Nav::Performance; }
        btn_perf.on_hover_text("Performance");
        
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.label("🤖");
            ui.small(serial);
            ui.add_space(10.0);
        });
    });
}

fn perf_card(ui: &mut egui::Ui, title: &str, value: &str, history: &[f32], selected: bool) -> bool {
    let mut clicked = false;
    let bg_color = if selected { Color32::from_gray(50) } else { Color32::TRANSPARENT };
    
    egui::Frame::none().fill(bg_color).inner_margin(10.0).rounding(5.0).show(ui, |ui| {
        let resp = ui.allocate_response(Vec2::new(ui.available_width(), 60.0), egui::Sense::click());
        if resp.clicked() { clicked = true; }
        
        let rect = resp.rect;
        
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).strong());
                    ui.label(RichText::new(value).size(18.0));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    draw_mini_graph(ui, history, 100.0, Color32::from_rgb(59, 130, 246));
                });
            });
        });
    });
    
    clicked
}

fn draw_mini_graph(ui: &mut egui::Ui, data: &[f32], max_val: f32, color: Color32) {
    let (response, painter) = ui.allocate_painter(Vec2::new(60.0, 40.0), egui::Sense::hover());
    let rect = response.rect;
    
    if data.len() < 2 { return; }
    
    let points: Vec<egui::Pos2> = data.iter().enumerate().map(|(i, &val)| {
        let x = rect.min.x + (i as f32 / (data.len() - 1) as f32) * rect.width();
        let y = rect.max.y - (val / max_val).clamp(0.0, 1.0) * rect.height();
        egui::Pos2::new(x, y)
    }).collect();
    
    painter.add(egui::Shape::line(points, Stroke::new(1.5, color)));
}

fn draw_large_graph(ui: &mut egui::Ui, data: &[f32], max_val: f32, color: Color32) {
    let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 200.0), egui::Sense::hover());
    let rect = response.rect;
    
    painter.rect_filled(rect, 5.0, Color32::from_gray(25));
    painter.rect_stroke(rect, 5.0, Stroke::new(1.0, Color32::from_gray(40)));

    if data.len() < 2 { return; }
    
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
        Stroke::NONE,
    ));

    painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // You can load fonts here if needed
    ctx.set_fonts(fonts);
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = egui::Rounding::same(8.0);
    visuals.panel_fill = Color32::from_rgb(32, 32, 32); 
    ctx.set_visuals(visuals);
}
