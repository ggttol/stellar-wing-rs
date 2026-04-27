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
            xp_to_next: 6,
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
