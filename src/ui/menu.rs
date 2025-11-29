use crate::core::config::Config;
pub struct Menu {
    config: Config,
}
impl Menu {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    pub fn parse_index(input: &str) -> Option<usize> {
        let s = input.trim();
        if let Ok(n) = s.parse::<usize>() {
            if n > 0 {
                return Some(n);
            }
        }
        // 尝试处理全角数字 (简单映射)
        let clean: String = s
            .chars()
            .map(|c| match c {
                '０'..='９' => ((c as u32 - '０' as u32) + '0' as u32) as u8 as char,
                _ => c,
            })
            .filter(|c| c.is_ascii_digit())
            .collect();
        if let Ok(num) = clean.parse::<usize>() {
            if num > 0 {
                return Some(num);
            }
        }
        None
    }
    pub fn render_device_menu(&self, device_name: &str) {
        use colored::*;
        println!("\n{} {} {}",
            "===".bright_cyan(),
            format!("设备 {} 功能菜单", device_name).bright_white().bold(),
            "===".bright_cyan()
        );
        let items = [
            ("0", "检测是否存在 Root 环境"),
            ("1", "查看引导/BL 锁状态"),
            ("2", "手机备份及恢复"),
            ("3", "压力测试"),
            ("4", "系统与硬件安全检查"),
            ("5", "功能敬请期待..."),
            ("6", "功能敬请期待..."),
            ("7", "功能敬请期待..."),
            ("8", "功能敬请期待..."),
            ("9", "功能敬请期待..."),
        ];
        for (k, def_label) in items {
            let lbl = self.config.get_label(k, def_label);
            println!("  {}) {}", k.bright_cyan(), lbl);
        }
        println!("  {}) {}", "q".bright_red(), "退出程序");
        print!("\n选择编号后回车：");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}