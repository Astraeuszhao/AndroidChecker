pub mod state;
pub mod data;
pub mod app;

use crate::adb::AdbClient;
use app::MonitorApp;
use data::fetch_loop;
use eframe::egui;
use state::SystemState;
use std::sync::{Arc, Mutex};

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
