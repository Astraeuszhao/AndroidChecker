use colored::*;
use std::io::{self, Write};
pub struct ConsoleUi;
impl ConsoleUi {
    pub fn write_banner() {
        println!("AndroidChecker\n");
    }
    pub fn read_line() -> String {
        let mut buf = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buf).unwrap();
        buf.trim().to_string()
    }
    pub fn info(msg: &str) {
        println!("{} {}", "[INFO]".bright_blue(), msg);
    }
    pub fn warn(msg: &str) {
        println!("{} {}", "[WARN]".bright_yellow(), msg);
    }
    pub fn error(msg: &str) {
        eprintln!("{} {}", "[ERROR]".bright_red().bold(), msg);
    }
    pub fn success(msg: &str) {
        println!("{}", msg);
    }
    pub fn pause_exit() {
        println!("\n按 Enter 退出...");
        let mut tmp = String::new();
        io::stdin().read_line(&mut tmp).unwrap();
    }
    pub fn render_device_table(devices: &[crate::adb::Device]) {
        if devices.is_empty() {
            Self::warn("没有找到设备");
            return;
        }
        let sep = "-".repeat(80);
        println!("{}", sep.bright_black());
        println!(
            "{:<4} {:<20} {:<30} {:<15}",
            "#".bright_cyan(),
            "Serial".bright_cyan(),
            "Device".bright_cyan(),
            "Android".bright_cyan()
        );
        println!("{}", sep.bright_black());
        for (idx, d) in devices.iter().enumerate() {
            let n = format!("{}", idx + 1).bright_white().bold();
            let s = d.serial.bright_white();
            let name = d.display_name().bright_green();
            let ver = d.android_version.as_deref().unwrap_or("Unknown");
            let v = ver.bright_yellow();
            println!("{:<4} {:<20} {:<30} {:<15}", n, s, name, v);
        }
        println!("{}", sep.bright_black());
        println!();
    }
}