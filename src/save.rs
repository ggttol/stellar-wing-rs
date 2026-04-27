//! 简单 JSON 存档：最高分 + Top5 排行榜 + 是否静音 + 是否全屏。
//! 路径：~/Library/Application Support/dev.ggttol.stellar-wing/save.json (mac)

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::lang::Lang;

#[derive(Serialize, Deserialize, Clone)]
pub struct Save {
    pub high: u32,
    pub leaderboard: Vec<Record>,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub lang: Lang,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            high: 0,
            leaderboard: Vec::new(),
            muted: true,
            fullscreen: false,
            lang: Lang::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Record {
    pub score: u32,
    pub level: u32,
    pub date: String,
}

fn path() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("dev", "ggttol", "stellar-wing")?;
    let dir = dirs.config_dir();
    fs::create_dir_all(dir).ok()?;
    Some(dir.join("save.json"))
}

pub fn load() -> Save {
    let Some(p) = path() else {
        return Save::default();
    };
    let Ok(s) = fs::read_to_string(&p) else {
        return Save::default();
    };
    serde_json::from_str(&s).unwrap_or_default()
}

pub fn write(save: &Save) {
    let Some(p) = path() else { return };
    if let Ok(s) = serde_json::to_string_pretty(save) {
        let _ = fs::write(p, s);
    }
}

impl Save {
    /// 插入一条记录，按分数倒序保留前 5。
    pub fn push_record(&mut self, score: u32, level: u32) {
        let date = today();
        self.leaderboard.push(Record { score, level, date });
        self.leaderboard.sort_by(|a, b| b.score.cmp(&a.score));
        self.leaderboard.truncate(5);
        if score > self.high {
            self.high = score;
        }
    }
}

/// 不依赖 chrono：用 SystemTime 算出 YYYY-MM-DD。
fn today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let days = (dur.as_secs() / 86400) as i64;
    let (y, m, d) = epoch_days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// 把"自 1970-01-01 起的天数"换算到 (year, month, day)。
fn epoch_days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    days += 719468; // 转换到从 0000-03-01 起的内部公历
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = (y + if m <= 2 { 1 } else { 0 }) as i32;
    (y, m, d)
}
