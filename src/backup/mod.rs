mod backup_engine;
mod models;
mod restore_engine;
mod root_checker;
pub use backup_engine::BackupEngine;
pub use models::{BackupItem, RestoreMode};
pub use restore_engine::RestoreEngine;
use crate::adb::AdbClient;
use crate::core::Result;
use crate::ui::ConsoleUi;
pub struct BackupRunner {
    bak_eng: BackupEngine,
    rst_eng: RestoreEngine,
}
impl BackupRunner {
    pub fn new(c: AdbClient) -> Self {
        let be = BackupEngine::new(c.clone());
        let re = RestoreEngine::new(c);
        Self {
            bak_eng: be,
            rst_eng: re,
        }
    }
    pub async fn show_menu(&self, serial: &str) -> Result<()> {
        loop {
            println!("\n=== 备份与恢复 ===");
            println!("1) 创建备份");
            println!("2) 恢复备份");
            println!("3) 查看备份信息");
            println!("0) 返回");
            print!("\n请选择: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let inp = ConsoleUi::read_line();
            match inp.as_str() {
                "1" => {
                    if let Err(e) = self.do_backup(serial).await {
                        ConsoleUi::error(&format!("备份失败: {}", e));
                    }
                }
                "2" => {
                    if let Err(e) = self.do_restore(serial).await {
                        ConsoleUi::error(&format!("恢复失败: {}", e));
                    }
                }
                "3" => {
                    if let Err(e) = self.do_view() {
                        ConsoleUi::error(&format!("查看失败: {}", e));
                    }
                }
                "0" => break,
                _ => ConsoleUi::warn("无效选择"),
            }
        }
        Ok(())
    }
    async fn do_backup(&self, serial: &str) -> Result<()> {
        println!("\n请选择要备份的内容（多选，用空格分隔，如: 1 2 3）：");
        let all = BackupItem::all_items();
        for (idx, it) in all.iter().enumerate() {
            println!("  {}) {}", idx + 1, it.name());
        }
        println!("  0) 全部备份");
        print!("\n请输入: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let inp = ConsoleUi::read_line();
        let selected = if inp.trim() == "0" {
            all
        } else {
            let nums: Vec<usize> = inp
                .split_whitespace()
                .filter_map(|s| s.parse::<usize>().ok())
                .filter(|&i| i > 0 && i <= all.len())
                .map(|i| i - 1)
                .collect();
            if nums.is_empty() {
                ConsoleUi::error("未选择任何项目");
                return Ok(());
            }
            nums.iter().map(|&i| all[i].clone()).collect()
        };
        println!("\n将备份以下项目：");
        for it in &selected {
            println!("  - {}", it.name());
        }
        print!("\n确认开始备份? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let conf = ConsoleUi::read_line();
        if !conf.eq_ignore_ascii_case("y") {
            ConsoleUi::info("已取消");
            return Ok(());
        }
        self.bak_eng.start_backup(serial, selected).await?;
        Ok(())
    }
    async fn do_restore(&self, serial: &str) -> Result<()> {
        print!("\n请输入备份文件路径: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let p = ConsoleUi::read_line();
        let bak_file = std::path::PathBuf::from(p.trim());
        if !bak_file.exists() {
            ConsoleUi::error("文件不存在");
            return Ok(());
        }
        self.rst_eng.list_backup_info(&bak_file)?;
        print!("\n确认恢复? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let conf = ConsoleUi::read_line();
        if !conf.eq_ignore_ascii_case("y") {
            ConsoleUi::info("已取消");
            return Ok(());
        }
        self.rst_eng.start_restore(serial, &bak_file, RestoreMode::Full).await?;
        Ok(())
    }
    fn do_view(&self) -> Result<()> {
        print!("\n请输入备份文件路径: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let p = ConsoleUi::read_line();
        let bak_file = std::path::PathBuf::from(p.trim());
        self.rst_eng.list_backup_info(&bak_file)?;
        Ok(())
    }
}