mod backup_engine;
mod models;
mod restore_engine;
mod root_checker;`npub use backup_engine::BackupEngine;
pub use models::{BackupItem, RestoreMode};
pub use restore_engine::RestoreEngine;`nuse crate::adb::AdbClient;
use crate::core::Result;
use crate::ui::ConsoleUi;`npub struct BackupRunner {
    bak_eng: BackupEngine,
    rst_eng: RestoreEngine,
}`nimpl BackupRunner {
    pub fn new(c: AdbClient) -> Self {
        let be = BackupEngine::new(c.clone());
        let re = RestoreEngine::new(c);
        Self {
            bak_eng: be,
            rst_eng: re,
        }
    }`n    pub async fn show_menu(&self, serial: &str) -> Result<()> {
        loop {
            println!("\n=== 澶囦唤涓庢仮澶?===");
            println!("1) 鍒涘缓澶囦唤");
            println!("2) 鎭㈠澶囦唤");
            println!("3) 鏌ョ湅澶囦唤淇℃伅");
            println!("0) 杩斿洖");
            print!("\n璇烽€夋嫨: ");
            std::io::Write::flush(&mut std::io::stdout())?;`n            let inp = ConsoleUi::read_line();`n            match inp.as_str() {
                "1" => {
                    if let Err(e) = self.do_backup(serial).await {
                        ConsoleUi::error(&format!("澶囦唤澶辫触: {}", e));
                    }
                }
                "2" => {
                    if let Err(e) = self.do_restore(serial).await {
                        ConsoleUi::error(&format!("鎭㈠澶辫触: {}", e));
                    }
                }
                "3" => {
                    if let Err(e) = self.do_view() {
                        ConsoleUi::error(&format!("鏌ョ湅澶辫触: {}", e));
                    }
                }
                "0" => break,
                _ => ConsoleUi::warn("鏃犳晥閫夋嫨"),
            }
        }`n        Ok(())
    }`n    async fn do_backup(&self, serial: &str) -> Result<()> {
        println!("\n璇烽€夋嫨瑕佸浠界殑鍐呭锛堝閫夛紝鐢ㄧ┖鏍煎垎闅旓紝濡? 1 2 3锛?");`n        let all = BackupItem::all_items();
        for (idx, it) in all.iter().enumerate() {
            println!("  {}) {}", idx + 1, it.name());
        }
        println!("  0) 鍏ㄩ儴澶囦唤");`n        print!("\n璇疯緭鍏? ");
        std::io::Write::flush(&mut std::io::stdout())?;`n        let inp = ConsoleUi::read_line();`n        let selected = if inp.trim() == "0" {
            all
        } else {
            let nums: Vec<usize> = inp
                .split_whitespace()
                .filter_map(|s| s.parse::<usize>().ok())
                .filter(|&i| i > 0 && i <= all.len())
                .map(|i| i - 1)
                .collect();`n            if nums.is_empty() {
                ConsoleUi::error("鏈€夋嫨浠讳綍椤圭洰");
                return Ok(());
            }`n            nums.iter().map(|&i| all[i].clone()).collect()
        };`n        println!("\n灏嗗浠戒互涓嬮」鐩?");
        for it in &selected {
            println!("  - {}", it.name());
        }`n        print!("\n纭寮€濮嬪浠? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout())?;`n        let conf = ConsoleUi::read_line();
        if !conf.eq_ignore_ascii_case("y") {
            ConsoleUi::info("宸插彇娑?);
            return Ok(());
        }`n        self.bak_eng.start_backup(serial, selected).await?;
        Ok(())
    }`n    async fn do_restore(&self, serial: &str) -> Result<()> {
        print!("\n璇疯緭鍏ュ浠芥枃浠惰矾寰? ");
        std::io::Write::flush(&mut std::io::stdout())?;`n        let p = ConsoleUi::read_line();
        let bak_file = std::path::PathBuf::from(p.trim());`n        if !bak_file.exists() {
            ConsoleUi::error("鏂囦欢涓嶅瓨鍦?);
            return Ok(());
        }`n        self.rst_eng.list_backup_info(&bak_file)?;`n        print!("\n纭鎭㈠? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout())?;`n        let conf = ConsoleUi::read_line();
        if !conf.eq_ignore_ascii_case("y") {
            ConsoleUi::info("宸插彇娑?);
            return Ok(());
        }`n        self.rst_eng.start_restore(serial, &bak_file, RestoreMode::Full).await?;
        Ok(())
    }`n    fn do_view(&self) -> Result<()> {
        print!("\n璇疯緭鍏ュ浠芥枃浠惰矾寰? ");
        std::io::Write::flush(&mut std::io::stdout())?;`n        let p = ConsoleUi::read_line();
        let bak_file = std::path::PathBuf::from(p.trim());`n        self.rst_eng.list_backup_info(&bak_file)?;
        Ok(())
    }
}
