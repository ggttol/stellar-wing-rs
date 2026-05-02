//! 简单 JSON 存档：最高分 + Top5 排行榜 + 是否静音 + 是否全屏。
//! 路径：~/Library/Application Support/dev.ggttol.stellar-wing/save.json (mac)

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::lang::Lang;
use crate::ship::ShipType;

/// 飞船解锁位掩码：bit0 = Vanguard（永久解锁，固定占位）。
/// 当前 3 艘船全部默认开放；位掩码保留给将来追加的第 4+ 艘船。
const SHIP_BIT_VANGUARD: u32 = 1 << 0;

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
    #[serde(default = "default_master_vol")]
    pub master_vol: f32,
    #[serde(default = "default_bgm_vol")]
    pub bgm_vol: f32,
    #[serde(default = "default_sfx_vol")]
    pub sfx_vol: f32,
    /// 屏幕震动强度倍率（无障碍 / 偏好），0.0..1.5
    #[serde(default = "default_shake")]
    pub screen_shake: f32,

    // —— 元进度（跨局） ————————————————————————————
    #[serde(default)]
    pub stardust: u64,
    #[serde(default)]
    pub lifetime_score: u64,
    #[serde(default)]
    pub bosses_killed: u32,
    #[serde(default)]
    pub runs: u32,
    /// 解锁飞船位掩码；Vanguard 总是有效。
    #[serde(default = "default_ship_mask")]
    pub unlocked_ships: u32,
    /// 历史最远到达的章节 (0..5) +1（人类可读）；endless 不计入。
    #[serde(default)]
    pub furthest_chapter: u32,

    // —— 永久天赋等级 ——
    #[serde(default)]
    pub talent_dmg: u32,
    #[serde(default)]
    pub talent_hp: u32,
    #[serde(default)]
    pub talent_speed: u32,
    #[serde(default)]
    pub talent_xp: u32,
    #[serde(default)]
    pub talent_stardust: u32,
    #[serde(default)]
    pub talent_super: u32,

    // —— 成就 / 图鉴：位掩码持久化 ——
    /// 已解锁成就的位掩码（key = AchievementId 数值索引）。
    #[serde(default)]
    pub achievements: u64,
    /// 已遭遇过的 Boss 修饰（BossMod 索引位）
    #[serde(default)]
    pub codex_bosses: u32,
    /// 已击杀过的敌人种类（EnemyKind 索引位）
    #[serde(default)]
    pub codex_enemies: u32,
    /// 已使用过 / 解锁过的副武器（id 哈希到 0..16）
    #[serde(default)]
    pub codex_weapons: u32,
    /// 已经看过的 Perk（perk 序号）
    #[serde(default)]
    pub codex_perks: u32,

    // —— 难度 / 每日挑战 ——
    /// 0 = Normal，1 = Hard，2 = Nightmare（按通关解锁）。仅作"上一次选择"持久化。
    #[serde(default)]
    pub difficulty: u8,
    /// 通关 Normal 后变 true
    #[serde(default)]
    pub hard_unlocked: bool,
    /// 通关 Hard 后变 true
    #[serde(default)]
    pub nightmare_unlocked: bool,
    /// 已挑战过的最近一天（YYYY-MM-DD）
    #[serde(default)]
    pub daily_date: String,
    /// 当日最高分
    #[serde(default)]
    pub daily_high: u32,

    // —— 涂装 / 外观 ——
    /// 飞船涂装位掩码（每艘 3 套：默认/A/B，索引 = ship_idx*3 + variant）
    #[serde(default = "default_ship_skin_mask")]
    pub ship_skins_unlocked: u32,
    /// 各 ship 当前选中涂装索引（0..3）
    #[serde(default)]
    pub ship_skin_choice: [u8; 3],
}

fn default_ship_mask() -> u32 {
    SHIP_BIT_VANGUARD
}

fn default_master_vol() -> f32 {
    0.5
}
fn default_bgm_vol() -> f32 {
    0.55
}
fn default_sfx_vol() -> f32 {
    1.0
}
fn default_shake() -> f32 {
    1.0
}

/// 默认每艘飞船第 0 套涂装永久解锁。
fn default_ship_skin_mask() -> u32 {
    // bit0 / bit3 / bit6 — 三艘飞船各自 base 涂装
    (1 << 0) | (1 << 3) | (1 << 6)
}

impl Default for Save {
    fn default() -> Self {
        Self {
            high: 0,
            leaderboard: Vec::new(),
            muted: true,
            fullscreen: false,
            lang: Lang::default(),
            master_vol: default_master_vol(),
            bgm_vol: default_bgm_vol(),
            sfx_vol: default_sfx_vol(),
            screen_shake: default_shake(),
            stardust: 0,
            lifetime_score: 0,
            bosses_killed: 0,
            runs: 0,
            unlocked_ships: default_ship_mask(),
            furthest_chapter: 0,
            talent_dmg: 0,
            talent_hp: 0,
            talent_speed: 0,
            talent_xp: 0,
            talent_stardust: 0,
            talent_super: 0,
            achievements: 0,
            codex_bosses: 0,
            codex_enemies: 0,
            codex_weapons: 0,
            codex_perks: 0,
            difficulty: 0,
            hard_unlocked: false,
            nightmare_unlocked: false,
            daily_date: String::new(),
            daily_high: 0,
            ship_skins_unlocked: default_ship_skin_mask(),
            ship_skin_choice: [0; 3],
        }
    }
}

/// 一局结算的奖励，给 UI 直接显示用。
pub struct RunReward {
    pub stardust_gained: u64,
    #[allow(dead_code)] // 保留给未来"+X / total"过渡动画
    pub lifetime_before: u64,
    pub lifetime_after: u64,
    /// 本局新解锁的飞船（在 game over 上做 toast）。
    pub newly_unlocked: Vec<ShipType>,
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
        self.leaderboard.sort_by_key(|r| std::cmp::Reverse(r.score));
        self.leaderboard.truncate(5);
        if score > self.high {
            self.high = score;
        }
    }

    pub fn ship_unlocked(&self, _ship: ShipType) -> bool {
        // 当前 3 艘船全部默认解锁。锁定框架（unlocked_ships 位掩码 +
        // ship_unlock_cost）保留给将来追加的第 4+ 艘船 / 涂装。
        true
    }

    pub fn ship_unlock_cost(_ship: ShipType) -> Option<u64> {
        None
    }

    /// 一局结束的奖励登记。返回前端要展示的奖励包。
    pub fn record_run(
        &mut self,
        score: u32,
        level: u32,
        bosses_in_run: u32,
        chapter_reached: u32,
        difficulty: u8,
        cleared_story: bool,
    ) -> RunReward {
        // Stardust：分数主，Boss 加成；再叠精炼天赋倍率，再叠难度倍率。
        let stardust_mul = crate::talents::stardust_multiplier(self);
        let diff_mul: f32 = match difficulty {
            0 => 1.0,
            1 => 1.5,
            _ => 2.0,
        };
        let raw = (score as u64) / 100 + (bosses_in_run as u64) * 50;
        let stardust = ((raw as f32) * stardust_mul * diff_mul) as u64;
        let lifetime_before = self.lifetime_score;
        self.stardust = self.stardust.saturating_add(stardust);
        self.lifetime_score = self.lifetime_score.saturating_add(score as u64);
        self.bosses_killed = self.bosses_killed.saturating_add(bosses_in_run);
        self.runs = self.runs.saturating_add(1);
        if chapter_reached > self.furthest_chapter && chapter_reached < 99 {
            self.furthest_chapter = chapter_reached;
        }
        // 难度解锁：通关故事章节 → 解锁更高难度 + 涂装
        if cleared_story {
            if difficulty == 0 {
                self.hard_unlocked = true;
                // Normal 通关解锁所有飞船的涂装 1（颜色变体）
                self.ship_skins_unlocked |= (1 << 1) | (1 << 4) | (1 << 7);
            } else if difficulty == 1 {
                self.nightmare_unlocked = true;
                // Hard 通关解锁涂装 2
                self.ship_skins_unlocked |= (1 << 2) | (1 << 5) | (1 << 8);
            }
        }
        self.push_record(score, level);

        // 当前没有需要按 lifetime 解锁的飞船；future-proof 用列表占位。
        let newly_unlocked: Vec<ShipType> = Vec::new();

        RunReward {
            stardust_gained: stardust,
            lifetime_before,
            lifetime_after: self.lifetime_score,
            newly_unlocked,
        }
    }
}

/// 不依赖 chrono：用 SystemTime 算出 YYYY-MM-DD。
pub fn today() -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_run_updates_progress_and_leaderboard() {
        let mut save = Save::default();

        let reward = save.record_run(12_500, 8, 2, 3, 0, false);

        assert_eq!(save.high, 12_500);
        assert_eq!(save.leaderboard.len(), 1);
        assert_eq!(save.runs, 1);
        assert_eq!(save.bosses_killed, 2);
        assert_eq!(save.furthest_chapter, 3);
        assert!(save.stardust > 0);
        assert_eq!(reward.lifetime_before, 0);
        assert_eq!(reward.lifetime_after, 12_500);
    }

    #[test]
    fn old_save_json_missing_new_fields_deserializes_with_defaults() {
        let json = r#"{"high":42,"leaderboard":[]}"#;

        let save: Save = serde_json::from_str(json).expect("old save should deserialize");

        assert_eq!(save.high, 42);
        assert_eq!(save.stardust, 0);
        assert_eq!(save.unlocked_ships, default_ship_mask());
    }

    #[test]
    fn epoch_zero_is_unix_epoch_date() {
        assert_eq!(epoch_days_to_ymd(0), (1970, 1, 1));
    }
}
