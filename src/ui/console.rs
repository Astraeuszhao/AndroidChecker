use colored::*;
use std::io::{self, Write};`npub struct ConsoleUi;`nimpl ConsoleUi {
    pub fn write_banner() {
        println!("AndroidChecker\n");
    }`n    pub fn read_line() -> String {
        let mut buf = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buf).unwrap();
        buf.trim().to_string()
    }`n    pub fn info(msg: &str) {
        println!("{} {}", "[INFO]".bright_blue(), msg);
    }`n    pub fn warn(msg: &str) {
        println!("{} {}", "[WARN]".bright_yellow(), msg);
    }`n    pub fn error(msg: &str) {
        eprintln!("{} {}", "[ERROR]".bright_red().bold(), msg);
    }`n    pub fn success(msg: &str) {
        println!("{}", msg);
    }`n    pub fn pause_exit() {
        println!("\n鎸?Enter 閫€鍑?..");
        let mut tmp = String::new();
        io::stdin().read_line(&mut tmp).unwrap();
    }`n    pub fn render_device_table(devices: &[crate::adb::Device]) {
        if devices.is_empty() {
            Self::warn("娌℃湁鎵惧埌璁惧");
            return;
        }`n        let sep = "-".repeat(80);
        println!("{}", sep.bright_black());`n        println!(
            "{:<4} {:<20} {:<30} {:<15}",
            "#".bright_cyan(),
            "Serial".bright_cyan(),
            "Device".bright_cyan(),
            "Android".bright_cyan()
        );`n        println!("{}", sep.bright_black());`n        for (idx, d) in devices.iter().enumerate() {
            let n = format!("{}", idx + 1).bright_white().bold();
            let s = d.serial.bright_white();
            let name = d.display_name().bright_green();`n            let ver = d.android_version.as_deref().unwrap_or("Unknown");
            let v = ver.bright_yellow();`n            println!("{:<4} {:<20} {:<30} {:<15}", n, s, name, v);
        }`n        println!("{}", sep.bright_black());
        println!();
    }
}
