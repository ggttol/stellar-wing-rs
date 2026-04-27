//! 跨局永久天赋。用 stardust 购买，开局自动叠加到 Player.stats 上。
//!
//! 设计取舍：每条天赋 3-5 级，递增成本，单条投入到顶大致需要 10-20 局；
//! 全部点满需要数十局——既给了"再来一把"的目标感，又避免一周就刷完。

use crate::save::Save;
use crate::world::World;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TalentId {
    Damage,
    Health,
    Speed,
    Xp,
    Stardust,
    SuperStart,
}

pub struct TalentDef {
    pub id: TalentId,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub desc_en: &'static str,
    pub desc_zh: &'static str,
    /// costs[i] 表示从 i 级升到 i+1 级的花费（i 从 0 开始）。len = max_level。
    pub costs: &'static [u64],
}

impl TalentDef {
    pub fn max_level(&self) -> u32 {
        self.costs.len() as u32
    }
    pub fn next_cost(&self, current_level: u32) -> Option<u64> {
        self.costs.get(current_level as usize).copied()
    }
}

pub static TALENTS: &[TalentDef] = &[
    TalentDef {
        id: TalentId::Damage,
        name_en: "PIERCING ROUNDS",
        name_zh: "穿甲弹",
        desc_en: "+6% damage per level",
        desc_zh: "每级 +6% 伤害",
        costs: &[120, 280, 600, 1200, 2400],
    },
    TalentDef {
        id: TalentId::Health,
        name_en: "REINFORCED HULL",
        name_zh: "强化机壳",
        desc_en: "+1 max HP per level",
        desc_zh: "每级 +1 HP 上限",
        costs: &[260, 700, 1700],
    },
    TalentDef {
        id: TalentId::Speed,
        name_en: "AGILE THRUSTERS",
        name_zh: "敏捷推进器",
        desc_en: "+5% move speed per level",
        desc_zh: "每级 +5% 移速",
        costs: &[180, 420, 950],
    },
    TalentDef {
        id: TalentId::Xp,
        name_en: "DATA HARVEST",
        name_zh: "数据收割",
        desc_en: "+10% XP from gems per level",
        desc_zh: "每级 +10% XP 收益",
        costs: &[200, 480, 1100],
    },
    TalentDef {
        id: TalentId::Stardust,
        name_en: "DUST REFINERY",
        name_zh: "星尘精炼",
        desc_en: "+15% stardust earned per level",
        desc_zh: "每级 +15% 局后星尘",
        costs: &[300, 750, 1700],
    },
    TalentDef {
        id: TalentId::SuperStart,
        name_en: "PRELOADED CORE",
        name_zh: "预充能核心",
        desc_en: "Start each run with +20% SUPER per level",
        desc_zh: "每级 开局多 20% SUPER",
        costs: &[220, 520, 1200],
    },
];

pub fn level_of(save: &Save, id: TalentId) -> u32 {
    match id {
        TalentId::Damage => save.talent_dmg,
        TalentId::Health => save.talent_hp,
        TalentId::Speed => save.talent_speed,
        TalentId::Xp => save.talent_xp,
        TalentId::Stardust => save.talent_stardust,
        TalentId::SuperStart => save.talent_super,
    }
}

pub fn set_level(save: &mut Save, id: TalentId, level: u32) {
    match id {
        TalentId::Damage => save.talent_dmg = level,
        TalentId::Health => save.talent_hp = level,
        TalentId::Speed => save.talent_speed = level,
        TalentId::Xp => save.talent_xp = level,
        TalentId::Stardust => save.talent_stardust = level,
        TalentId::SuperStart => save.talent_super = level,
    }
}

/// 尝试买入下一级。成功返回 true。
pub fn try_buy(save: &mut Save, id: TalentId) -> bool {
    let def = TALENTS
        .iter()
        .find(|d| d.id == id)
        .expect("known talent id");
    let cur = level_of(save, id);
    let Some(cost) = def.next_cost(cur) else {
        return false;
    };
    if save.stardust < cost {
        return false;
    }
    save.stardust -= cost;
    set_level(save, id, cur + 1);
    true
}

/// 局开始时把天赋叠到 World/Player 上。
pub fn apply_to_world(world: &mut World, save: &Save) {
    let dmg_lv = save.talent_dmg as f32;
    let hp_lv = save.talent_hp as u8;
    let spd_lv = save.talent_speed as f32;
    let xp_lv = save.talent_xp as f32;
    let super_lv = save.talent_super as f32;

    world.player.stats.damage_mul *= 1.0 + dmg_lv * 0.06;
    if hp_lv > 0 {
        world.player.stats.max_lives = world.player.stats.max_lives.saturating_add(hp_lv);
        world.player.lives = world.player.stats.max_lives;
    }
    world.player.stats.speed *= 1.0 + spd_lv * 0.05;
    world.player.stats.xp_mul *= 1.0 + xp_lv * 0.10;
    world.super_charge = (world.super_charge + super_lv * 0.20).min(1.0);
}

/// 计算本局结算时星尘的最终倍率（精炼天赋）。
pub fn stardust_multiplier(save: &Save) -> f32 {
    1.0 + (save.talent_stardust as f32) * 0.15
}
