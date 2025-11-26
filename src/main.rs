mod adb;
mod audit;
mod backup;
mod checks;
mod core;
mod stress;
mod ui;`nuse adb::{AdbClient, DeviceManager};
use audit::AuditRunner;
use backup::BackupRunner;
use checks::{BootloaderChecker, RootChecker};
use core::config::Config;
use stress::StressRunner;
use ui::{ConsoleUi, Menu};
use chrono::{Datelike, Local};
use colored::Colorize;`n#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ConsoleUi::write_banner();`n    let now = Local::now();
    if now.month() == 10 && now.day() == 24 {
        println!();
        let msg1 = "馃帀 1024 绋嬪簭鍛樿妭蹇箰锛?;
        let msg2 = "2^10 = 1024, 鑷存暚姣忎竴浣嶆敼鍙樹笘鐣岀殑浠ｇ爜宸ュ尃";
        let msg3 = "Bug灏戜竴鐐癸紝澶村彂澶氫竴鐐?鉁?;
        println!("{}", msg1.bright_cyan().bold());
        println!("{}", msg2.bright_green());
        println!("{}", msg3.yellow());
        println!();
        std::thread::sleep(std::time::Duration::from_millis(800));
    }`n    let config = Config::load()?;`n    let client = match AdbClient::new() {
        Ok(c) => c,
        Err(_) => {
            ConsoleUi::error("鏈娴嬪埌 adb銆傝瀹夎 Android Platform-Tools 骞跺皢 adb 鍔犲叆 PATH銆?);
            ConsoleUi::pause_exit();
            return Ok(());
        }
    };`n    if client.ensure_present().await.is_err() {
        ConsoleUi::error("ADB 鐗堟湰妫€娴嬪け璐?);
        ConsoleUi::pause_exit();
        return Ok(());
    }`n    client.start_server().await?;`n    loop {
        let device_mgr = DeviceManager::new(client.clone());
        let devices = match device_mgr.list_devices().await {
            Ok(d) => d,
            Err(e) => {
                ConsoleUi::error(&format!("ADB 璋冪敤澶辫触: {}", e));
                ConsoleUi::pause_exit();
                return Ok(());
            }
        };`n        if devices.is_empty() {
            ConsoleUi::warn("鏈彂鐜板凡鎺堟潈鐨勮澶囥€傝杩炴帴璁惧骞跺紑鍚?USB 璋冭瘯锛屾寜鍥炶溅閲嶈瘯锛屾垨杈撳叆 q 閫€鍑恒€?);
            let input = ConsoleUi::read_line();
            if input.eq_ignore_ascii_case("q") {
                return Ok(());
            }
            continue;
        }`n        println!("\n宸茶繛鎺ヨ澶囷細");
        ConsoleUi::render_device_table(&devices);`n        println!("\n杈撳叆鏁板瓧閫夋嫨璁惧 (q 閫€鍑猴紝鍙€夌紪鍙? 1..{})锛?, devices.len());
        let input = ConsoleUi::read_line();`n        if input.eq_ignore_ascii_case("q") {
            return Ok(());
        }`n        let idx = match Menu::parse_index(&input) {
            Some(n) if n >= 1 && n <= devices.len() => n - 1,
            _ => {
                ConsoleUi::warn("鏃犳晥杈撳叆锛岃閲嶈瘯");
                continue;
            }
        };`n        let device = &devices[idx];`n        if let Err(e) = device_menu(&device.serial, &device.display_name(), &client, &config).await
        {
            ConsoleUi::error(&format!("鍙戠敓寮傚父: {}", e));
            ConsoleUi::pause_exit();
            return Ok(());
        }
    }
}`nasync fn device_menu(
    serial: &str,
    display_name: &str,
    client: &AdbClient,
    config: &Config,
) -> anyhow::Result<()> {
    let menu = Menu::new(config.clone());`n    loop {
        menu.render_device_menu(display_name);
        let choice = ConsoleUi::read_line();`n        match choice.as_str() {
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
                ConsoleUi::info("寮€濮?60 绉?ADB 绋冲畾鎬ф祴璇?..");
                let stress = StressRunner::new(client.clone());
                let (ok, fail) = stress.adb_stability_test(serial, 60).await?;
                ConsoleUi::success(&format!("娴嬭瘯瀹屾垚: 鎴愬姛 {} 娆? 澶辫触 {} 娆?, ok, fail));
            }
            "4" => {
                ConsoleUi::info("寮€濮嬪畨鍏ㄥ璁?..");
                let audit = AuditRunner::new(client.clone());
                let report = audit.run(serial, None).await?;
                println!("\n瀹¤鎶ュ憡:");
                println!("璁惧: {} {}", report.device_info.brand, report.device_info.marketing_name);
                println!("Android: {} (SDK {})", report.device_info.android, report.device_info.sdk);
                println!("Root 妫€娴? su={}, 鍙枒鍖?{}",
                    if report.root.su_in_path.is_empty() { "鍚? } else { "鏄? },
                    report.root.suspicious_packages.len());
                println!("Bootloader: verifiedbootstate={}", report.boot.verifiedbootstate);
                println!("SELinux: {}", report.security_env.selinux);
                println!("瀹夊叏琛ヤ竵: {}", report.integrity.security_patch);
            }
            "5" | "6" | "7" | "8" | "9" => {
                ConsoleUi::info("鍔熻兘鏁鏈熷緟涓?);
            }
            "q" | "Q" => {
                std::process::exit(0);
            }
            _ => {
                ConsoleUi::warn("鏃犳晥閫夋嫨锛岃閲嶈瘯");
            }
        }
    }
}
