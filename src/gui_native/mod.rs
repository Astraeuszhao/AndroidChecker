pub mod monitor_window;`npub use monitor_window::MonitorApp;`nuse crate::adb::AdbClient;
use anyhow::Result;`npub fn launch_monitor_gui(serial: String) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 850.0])
            .with_title("绯荤粺鐩戞帶涓庤繘绋嬬锟?- AndroidChecker")
            .with_resizable(true)
            .with_min_inner_size([1000.0, 600.0]),
        ..Default::default()
    };`n    let client = AdbClient::new()?;`n    eframe::run_native(
        "AndroidChecker Monitor",
        native_options,
        Box::new(move |cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            Ok(Box::new(MonitorApp::new(client, serial)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI 鍚姩澶辫触: {}", e))
}`nfn setup_chinese_fonts(ctx: &egui::Context) {
    use std::fs;`n    let mut fonts = egui::FontDefinitions::default();`n    let font_paths = vec![
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyhbd.ttc", 
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];`n    for (idx, font_path) in font_paths.iter().enumerate() {
        if let Ok(font_data) = fs::read(font_path) {
            fonts.font_data.insert(
                format!("chinese_{}", idx),
                egui::FontData::from_owned(font_data),
            );`n            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(idx, format!("chinese_{}", idx));`n            break;
        }
    }`n    ctx.set_fonts(fonts);
}`n