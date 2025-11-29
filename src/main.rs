mod adb;
mod audit;
mod backup;
mod checks;
mod core;
mod gui;
mod stress;
mod ui;
use adb::{AdbClient, DeviceManager};
use audit::AuditRunner;
use backup::BackupRunner;
use checks::{BootloaderChecker, RootChecker};
use core::config::Config;
use stress::StressRunner;
use ui::{ConsoleUi, Menu};
use chrono::{Datelike, Local};
use colored::Colorize;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ConsoleUi::write_banner();
    let now = Local::now();
    if now.month() == 10 && now.day() == 24 {
        println!();
        let msg1 = " 1024 ";
        let msg2 = "2^10 = 1024, ";
        let msg3 = "Bug, ";
        println!("{}", msg1.bright_cyan().bold());
        println!("{}", msg2.bright_green());
        println!("{}", msg3.yellow());
        println!();
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
    let config = Config::load()?;
    let client = match AdbClient::new() {
        Ok(c) => c,
        Err(_) => {
            ConsoleUi::error("未检测到 adb。请确保内置 adb 存在或已安装 Android Platform-Tools。");
            ConsoleUi::pause_exit();
            return Ok(());
        }
    };

    if client.ensure_present().await.is_err() {
        ConsoleUi::error("ADB 版本检测失败");
        ConsoleUi::pause_exit();
        return Ok(());
    }

    client.start_server().await?;

    loop {
        let device_mgr = DeviceManager::new(client.clone());
        let devices = match device_mgr.list_devices().await {
            Ok(d) => d,
            Err(e) => {
                ConsoleUi::error(&format!("ADB 调用失败: {}", e));
                ConsoleUi::pause_exit();
                return Ok(());
            }
        };

        if devices.is_empty() {
            ConsoleUi::warn("未发现已授权的设备。请连接设备并开启 USB 调试，按回车重试，或输入 q 退出。");
            let input = ConsoleUi::read_line();
            if input.eq_ignore_ascii_case("q") {
                return Ok(());
            }
            continue;
        }

        println!("\n已连接设备：");
        ConsoleUi::render_device_table(&devices);
        println!("\n输入数字选择设备 (q 退出，可选编号: 1..{}): ", devices.len());
        let input = ConsoleUi::read_line();

        if input.eq_ignore_ascii_case("q") {
            return Ok(());
        }

        let idx = match Menu::parse_index(&input) {
            Some(n) if n >= 1 && n <= devices.len() => n - 1,
            _ => {
                ConsoleUi::warn("无效输入，请重试");
                continue;
            }
        };

        let device = &devices[idx];
        if let Err(e) = device_menu(&device.serial, &device.display_name(), &client, &config).await
        {
            ConsoleUi::error(&format!("发生异常: {}", e));
            ConsoleUi::pause_exit();
            return Ok(());
        }
    }
}

async fn device_menu(
    serial: &str,
    display_name: &str,
    client: &AdbClient,
    config: &Config,
) -> anyhow::Result<()> {
    let menu = Menu::new(config.clone());
    loop {
        menu.render_device_menu(display_name);
        let choice = ConsoleUi::read_line();
        match choice.as_str() {
            "0" => {
                let checker = RootChecker::new(client.clone());
                let report = checker.check(serial).await?;
                println!("{}", report);
            }
            "1" => {
                let device_mgr = DeviceManager::new(client.clone());
                let checker = BootloaderChecker::new(device_mgr);
                let report = checker.check(serial).await?;
                println!("{}", report);
            }
            "2" => {
                let backup = BackupRunner::new(client.clone());
                backup.show_menu(serial).await?;
            }
            "3" => {
                ConsoleUi::info("开始 60 秒 ADB 稳定性测试...");
                let stress = StressRunner::new(client.clone());
                let (ok, fail) = stress.adb_stability_test(serial, 60).await?;
                ConsoleUi::success(&format!("测试完成: 成功 {} 次, 失败 {} 次", ok, fail));
            }
            "4" => {
                ConsoleUi::info("开始安全审计...");
                let audit = AuditRunner::new(client.clone());
                let report = audit.run(serial, None).await?;
                println!("\n审计报告:");
                println!("设备: {} {}", report.device_info.brand, report.device_info.marketing_name);
                println!("Android: {} (SDK {})", report.device_info.android, report.device_info.sdk);
                println!("Root 检测: su={}, 可疑包={}",
                    if report.root.su_in_path.is_empty() { "否" } else { "是" },
                    report.root.suspicious_packages.len());
                println!("Bootloader: verifiedbootstate={}", report.boot.verifiedbootstate);
                println!("SELinux: {}", report.security_env.selinux);
                println!("安全补丁: {}", report.integrity.security_patch);
            }
            "5" => {
                ConsoleUi::info("正在启动图形化系统资源监视器...");
                if let Err(e) = gui::run_monitor(client.clone(), serial.to_string()) {
                    ConsoleUi::error(&format!("GUI 启动失败: {}", e));
                }
            }
            "6" | "7" | "8" | "9" => {
                ConsoleUi::info("功能敬请期待...");
            }
            "q" | "Q" => {
                std::process::exit(0);
            }
            _ => {
                ConsoleUi::warn("无效选择，请重试");
            }
        }
    }
}