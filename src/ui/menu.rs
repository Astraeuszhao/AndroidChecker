use crate::core::config::Config;`npub struct Menu {
    config: Config,
}`nimpl Menu {
    pub fn new(config: Config) -> Self {
        Self { config }
    }`n    pub fn parse_index(input: &str) -> Option<usize> {
        let s = input.trim();`n        if let Ok(n) = s.parse::<usize>() {
            if n > 0 {
                return Some(n);
            }
        }`n        let clean: String = s
            .chars()
            .map(|c| match c {
                '锛?..='锛? => ((c as u32 - '锛? as u32) + '0' as u32) as u8 as char,
                '郯'..='酃' => ((c as u32 - '郯' as u32) + '0' as u32) as u8 as char,
                '贍'..='侃' => ((c as u32 - '贍' as u32) + '0' as u32) as u8 as char,
                _ => c,
            })
            .filter(|c| c.is_ascii_digit())
            .collect();`n        if let Ok(num) = clean.parse::<usize>() {
            if num > 0 {
                return Some(num);
            }
        }`n        Self::try_chinese(s)
    }`n    fn try_chinese(input: &str) -> Option<usize> {
        let s = input.replace("闆?, "0").replace("銆?, "0");`n        if s == "鍗? {
            return Some(10);
        }`n        if s.len() == 2 {
            let ch: Vec<char> = s.chars().collect();
            if ch[0] == '鍗? {
                if let Some(u) = Self::ch_digit(ch[1]) {
                    return Some(10 + u);
                }
            }
            if ch[1] == '鍗? {
                if let Some(t) = Self::ch_digit(ch[0]) {
                    return Some(t * 10);
                }
            }
        }`n        if s.len() == 1 {
            return Self::ch_digit(s.chars().next()?);
        }`n        None
    }`n    fn ch_digit(c: char) -> Option<usize> {
        match c {
            '闆? | '銆? => Some(0),
            '涓€' => Some(1),
            '浜? | '涓? => Some(2),
            '涓? => Some(3),
            '鍥? => Some(4),
            '浜? => Some(5),
            '鍏? => Some(6),
            '涓? | '鏌? => Some(7),
            '鍏? => Some(8),
            '涔? => Some(9),
            _ => None,
        }
    }`n    pub fn render_device_menu(&self, device_name: &str) {
        use colored::*;`n        println!("\n{} {} {}", 
            "===".bright_cyan(), 
            format!("璁惧 {} 鍔熻兘鑿滃崟", device_name).bright_white().bold(), 
            "===".bright_cyan()
        );`n        let items = [
            ("0", "妫€娴嬫槸鍚﹀瓨鍦?Root 鐜"),
            ("1", "鏌ョ湅寮曞/BL 閿佺姸鎬?),
            ("2", "鎵嬫満澶囦唤鍙婃仮澶?),
            ("3", "鍘嬪姏娴嬭瘯"),
            ("4", "绯荤粺涓庣‖浠跺畨鍏ㄦ鏌?),
            ("5", "鏁鏈熷緟涓?),
            ("6", "鏁鏈熷緟涓?),
            ("7", "鏁鏈熷緟涓?),
            ("8", "鏁鏈熷緟涓?),
            ("9", "鏁鏈熷緟涓?),
        ];`n        for (k, def_label) in items {
            let lbl = self.config.get_label(k, def_label);
            println!("  {}) {}", k.bright_cyan(), lbl);
        }`n        println!("  {}) {}", "q".bright_red(), "閫€鍑虹▼搴?);
        print!("\n閫夋嫨缂栧彿鍚庡洖杞︼細");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}
