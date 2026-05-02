//! 一局游戏的可变状态。

use crate::entity::{Bullet, Enemy, Pickup, Player};
use crate::ship::ShipType;
use crate::weapon::{SynergyGauge, WeaponSlot};

pub struct SpawnTimers {
    pub small: f32,
    pub medium: f32,
    pub large: f32,
}

impl SpawnTimers {
    pub fn new() -> Self {
        Self {
            small: 0.0,
            medium: 0.0,
            large: 0.0,
        }
    }
}

pub struct World {
    pub player: Player,
    pub weapons: WeaponSlot,
    pub bullets: Vec<Bullet>,
    pub enemies: Vec<Enemy>,
    pub pickups: Vec<Pickup>,
    pub spawn: SpawnTimers,
    pub score: u32,
    pub run_time: f32,
    pub xp: u32,
    pub level: u32,
    pub xp_to_next: u32,

    // —— 章节 / Boss 节奏 ——
    /// 当前章节索引；0..=4 = 故事章节，5+ = 无尽（取 endless 模板）。
    pub chapter_idx: u32,
    /// 章节内累计时间，到 chapter.duration 触发 Boss。
    pub chapter_time: f32,
    /// 当前章节是否已经派出 Boss（避免重复 spawn）
    pub chapter_boss_spawned: bool,
    /// 章节切换时显示标题/副标题的剩余时间
    pub chapter_intro: f32,
    /// 当章节内 Strafer 生成倒计时
    pub strafer_timer: f32,
    /// 已击杀 Boss 总数（结算用）
    pub bosses_killed_run: u32,

    pub boss_alive: bool,
    pub super_charge: f32,
    pub combo: u32,
    pub combo_timer: f32,
    pub combo_flash: f32,
    pub combo_note_idx: usize,
    /// 主武器击中音效冷却（秒），避免一帧打几十发就响成一片。
    pub hit_sfx_cooldown: f32,

    /// 共鸣槽：击杀填充、满槽过载（×1.30 伤害）。
    pub synergy: SynergyGauge,
    /// 进入过载瞬间的视觉脉冲（0..1，倒数到 0）
    pub overload_flash: f32,
    /// 无尽模式伤害加成：每圈 +4%（用于抵消 Boss HP 指数增长）
    pub endless_damage_bonus: f32,
    /// 低血量警告脉冲计数，配合 run_time 做周期性 beep
    pub last_hp_warn_beat: u32,

    // —— 局内统计：暂停页 / 结算页用 ——
    /// 按 HitSource 维度的累计伤害（索引 = HitSource as u8）。9 个变体，
    /// MainGun=0, Missile=1, Drone=2, Laser=3, Chain=4, Rift=5, Wave=6, Reflector=7, Enemy=8。
    pub damage_by_source: [f32; 9],
    /// 总击杀数（Boss 也算 1）
    pub kills: u32,
    /// 历史最高连击（结算页"巅峰连击"）
    pub max_combo: u32,

    // —— 难度 / 模式 ——
    /// 0/1/2 = Normal/Hard/Nightmare
    pub difficulty: u8,
    /// 是否每日挑战（启动时根据日期种子初始化）
    pub daily_mode: bool,
    /// RNG 种子（每日挑战时固定）。0 表示走 thread_rng。
    #[allow(dead_code)] // 将来全 RNG seed 化时使用
    pub run_seed: u64,
    /// 当前章节修饰（章节分叉选项）
    pub chapter_modifier: ChapterMod,
    /// 是否本章未受过伤害（用于 "no-hit" 成就）
    pub chapter_no_hit: bool,
    /// 本局内已解锁的 codex 位（敌人/Boss/武器）。结算时合并入 Save。
    pub codex_enemies_run: u32,
    pub codex_bosses_run: u32,
    pub codex_weapons_run: u32,
}

/// 章节分叉路线选择，应用到当前章节的 spawn / drop。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChapterMod {
    /// 默认平衡
    None,
    /// 猛攻路：敌人 +30% HP，分数 +20%，结算时多掉 buff
    Onslaught,
    /// 速攻路：敌人 -15% HP，章节时长 -25%
    Blitz,
    /// 收成路：buff 掉率 ×1.5、分数 +30%
    Harvest,
}

impl ChapterMod {
    /// 修饰名（英文 key，走 lang::t）
    pub fn name(self) -> &'static str {
        match self {
            ChapterMod::None => "Standard",
            ChapterMod::Onslaught => "Onslaught",
            ChapterMod::Blitz => "Blitz",
            ChapterMod::Harvest => "Harvest",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            ChapterMod::None => "Balanced encounter",
            ChapterMod::Onslaught => "+30% HP · +20% score · extra buffs",
            ChapterMod::Blitz => "-15% HP · -25% chapter time",
            ChapterMod::Harvest => "Buff drops ×1.5 · +30% score",
        }
    }
    /// 三选二路线池（None 不出现在选项里）
    pub fn options() -> [ChapterMod; 3] {
        [ChapterMod::Onslaught, ChapterMod::Blitz, ChapterMod::Harvest]
    }
    /// 该路线下敌人 HP 倍率
    pub fn hp_mul(self) -> f32 {
        match self {
            ChapterMod::Onslaught => 1.30,
            ChapterMod::Blitz => 0.85,
            _ => 1.0,
        }
    }
    /// 章节时长倍率
    pub fn duration_mul(self) -> f32 {
        match self {
            ChapterMod::Blitz => 0.75,
            _ => 1.0,
        }
    }
    /// 分数倍率
    pub fn score_mul(self) -> f32 {
        match self {
            ChapterMod::Onslaught => 1.20,
            ChapterMod::Harvest => 1.30,
            _ => 1.0,
        }
    }
    /// buff 掉率倍率
    pub fn buff_drop_mul(self) -> f32 {
        match self {
            ChapterMod::Harvest => 1.5,
            ChapterMod::Onslaught => 1.25,
            _ => 1.0,
        }
    }
}

impl World {
    pub fn new(ship: ShipType) -> Self {
        let mut player = Player::with_ship(ship);
        let mut weapons = WeaponSlot::new();
        ship.apply(&mut player, &mut weapons);
        Self {
            player,
            weapons,
            bullets: Vec::with_capacity(512),
            enemies: Vec::with_capacity(64),
            pickups: Vec::with_capacity(64),
            spawn: SpawnTimers::new(),
            score: 0,
            run_time: 0.0,
            xp: 0,
            level: 1,
            xp_to_next: World::xp_required_for(2),
            chapter_idx: 0,
            chapter_time: 0.0,
            chapter_boss_spawned: false,
            chapter_intro: 2.5,
            strafer_timer: 0.0,
            bosses_killed_run: 0,
            boss_alive: false,
            super_charge: 0.2,
            combo: 0,
            combo_timer: 0.0,
            combo_flash: 0.0,
            combo_note_idx: 0,
            hit_sfx_cooldown: 0.0,
            synergy: SynergyGauge::new(),
            overload_flash: 0.0,
            endless_damage_bonus: 0.0,
            last_hp_warn_beat: 0,
            damage_by_source: [0.0; 9],
            kills: 0,
            max_combo: 0,
            difficulty: 0,
            daily_mode: false,
            run_seed: 0,
            chapter_modifier: ChapterMod::None,
            chapter_no_hit: true,
            codex_enemies_run: 0,
            codex_bosses_run: 0,
            codex_weapons_run: 0,
        }
    }

    /// 升到 `level` 需要从上一级累计的 XP。
    /// 公式：6 + (L-1)*4 + (L-1)^2 * 4。
    /// 早期保留陡度（前 3 级 6/14/30），中期开始指数化（L8=202、L12=490），
    /// 让中后期不再每隔几秒就强制选卡。
    pub fn xp_required_for(level: u32) -> u32 {
        let l = level.saturating_sub(1) as f32;
        (6.0 + l * 4.0 + l * l * 4.0) as u32
    }

    /// 难度对应的（敌人 HP 倍率, 敌方子弹速度倍率, XP 倍率, 分数倍率）。
    pub fn difficulty_mods(d: u8) -> (f32, f32, f32, f32) {
        match d {
            0 => (1.00, 1.00, 1.00, 1.00),
            1 => (1.25, 1.15, 1.10, 1.20), // Hard
            _ => (1.60, 1.25, 1.25, 1.50), // Nightmare
        }
    }
    /// 副武器 id 哈希到 codex_weapons_run 的 bit 索引（0..7）
    pub fn weapon_codex_bit(id: &str) -> Option<u32> {
        match id {
            "missile" => Some(0),
            "drone" => Some(1),
            "laser" => Some(2),
            "chain" => Some(3),
            "rift" => Some(4),
            "wave" => Some(5),
            "reflector" => Some(6),
            _ => None,
        }
    }

    pub fn difficulty_label(d: u8) -> &'static str {
        match d {
            0 => "NORMAL",
            1 => "HARD",
            _ => "NIGHTMARE",
        }
    }

    /// 当前章节是否进入 Endless（数值无封顶 + 双 Boss）。
    pub fn is_endless(&self) -> bool {
        self.chapter_idx as usize >= crate::chapter::CHAPTERS.len()
    }

    /// 敌人移速倍率：故事章节软封顶到 1.8；无尽不再封顶。
    pub fn diff_mul(&self) -> f32 {
        let raw = 0.85 + self.run_time / 200.0;
        if self.is_endless() {
            raw.min(3.5)
        } else {
            raw.min(1.8)
        }
    }
}
