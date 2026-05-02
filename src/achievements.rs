//! 成就系统：固定一批可解锁的"目标"，玩家在玩中触发后解锁。
//! 用 64-bit 位掩码持久化到 save.achievements 中。
//!
//! 解锁逻辑：每帧（或在关键事件后）调用 `check_run(save, world)`。
//! 已解锁的成就会跳过检查；新解锁的会返回 stardust 奖励。

use crate::save::Save;
use crate::world::World;

/// 一条成就的静态定义。`check` 接受当前 World 和 save 状态，返回是否已满足。
pub struct Achievement {
    pub id: u8, // 位编号，进 save.achievements bitmask
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub desc_en: &'static str,
    pub desc_zh: &'static str,
    /// 解锁奖励
    pub stardust: u64,
    /// 局内可触发的检查；返回 true 表示满足。`save` 给跨局成就（runs / lifetime）使用。
    pub check: fn(&World, &Save) -> bool,
}

pub const ACHIEVEMENTS: &[Achievement] = &[
    // —— 入门类 ——
    Achievement {
        id: 0,
        name_en: "First Boss Down",
        name_zh: "首杀 Boss",
        desc_en: "Defeat any boss",
        desc_zh: "击败任意一个 Boss",
        stardust: 30,
        check: |w, _| w.bosses_killed_run >= 1,
    },
    Achievement {
        id: 1,
        name_en: "Climber",
        name_zh: "登天梯",
        desc_en: "Reach Level 10 in a run",
        desc_zh: "单局达到 10 级",
        stardust: 30,
        check: |w, _| w.level >= 10,
    },
    Achievement {
        id: 2,
        name_en: "Combo Streak",
        name_zh: "连击大师",
        desc_en: "Reach a 50-kill combo",
        desc_zh: "达成 50 连击",
        stardust: 50,
        check: |w, _| w.combo >= 50 || w.max_combo >= 50,
    },
    Achievement {
        id: 3,
        name_en: "Combo Maniac",
        name_zh: "连击狂魔",
        desc_en: "Reach a 100-kill combo",
        desc_zh: "达成 100 连击",
        stardust: 100,
        check: |w, _| w.combo >= 100 || w.max_combo >= 100,
    },
    // —— 通关类 ——
    Achievement {
        id: 4,
        name_en: "Story Cleared",
        name_zh: "通关故事",
        desc_en: "Complete all story chapters",
        desc_zh: "通关所有故事章节",
        stardust: 200,
        check: |w, _| w.chapter_idx as usize >= crate::chapter::CHAPTERS.len(),
    },
    Achievement {
        id: 5,
        name_en: "Hard Done",
        name_zh: "困难通关",
        desc_en: "Clear story on Hard",
        desc_zh: "在困难难度通关故事",
        stardust: 400,
        check: |w, _| {
            w.difficulty >= 1 && w.chapter_idx as usize >= crate::chapter::CHAPTERS.len()
        },
    },
    Achievement {
        id: 6,
        name_en: "Nightmare Done",
        name_zh: "噩梦通关",
        desc_en: "Clear story on Nightmare",
        desc_zh: "在噩梦难度通关故事",
        stardust: 800,
        check: |w, _| {
            w.difficulty >= 2 && w.chapter_idx as usize >= crate::chapter::CHAPTERS.len()
        },
    },
    Achievement {
        id: 7,
        name_en: "Endless Lap",
        name_zh: "无尽一圈",
        desc_en: "Survive one endless lap",
        desc_zh: "在无尽模式撑过一圈",
        stardust: 200,
        check: |w, _| w.chapter_idx as usize > crate::chapter::CHAPTERS.len(),
    },
    // —— 武器使用 ——
    Achievement {
        id: 8,
        name_en: "Missile Master",
        name_zh: "导弹专家",
        desc_en: "Deal 5000 damage with missiles",
        desc_zh: "用导弹累计造成 5000 伤害",
        stardust: 80,
        check: |w, _| w.damage_by_source[1] >= 5000.0,
    },
    Achievement {
        id: 9,
        name_en: "Beam Wielder",
        name_zh: "光束行者",
        desc_en: "Deal 8000 damage with the laser",
        desc_zh: "用激光累计造成 8000 伤害",
        stardust: 80,
        check: |w, _| w.damage_by_source[3] >= 8000.0,
    },
    Achievement {
        id: 10,
        name_en: "Stormbringer",
        name_zh: "唤雷者",
        desc_en: "Deal 6000 damage with the chain",
        desc_zh: "用闪电链累计造成 6000 伤害",
        stardust: 80,
        check: |w, _| w.damage_by_source[4] >= 6000.0,
    },
    Achievement {
        id: 11,
        name_en: "Riftwalker",
        name_zh: "裂隙行者",
        desc_en: "Deal 6000 damage with rifts",
        desc_zh: "用裂隙累计造成 6000 伤害",
        stardust: 80,
        check: |w, _| w.damage_by_source[5] >= 6000.0,
    },
    Achievement {
        id: 12,
        name_en: "Quartet",
        name_zh: "四重奏",
        desc_en: "Carry 4 sub-weapons in a run",
        desc_zh: "单局同时携带 4 件副武器",
        stardust: 100,
        check: |w, _| w.weapons.subs.len() >= 4,
    },
    // —— 操作 ——
    Achievement {
        id: 13,
        name_en: "Untouched",
        name_zh: "全身而退",
        desc_en: "Clear a chapter without taking damage",
        desc_zh: "完成一个章节且未受伤",
        stardust: 150,
        // 章节切换时由 main.rs 检查，所以这里只校验 chapter_no_hit 和已经过 1 章
        check: |w, _| w.chapter_no_hit && w.chapter_idx >= 1,
    },
    Achievement {
        id: 14,
        name_en: "Pacifist Boss",
        name_zh: "和平 Boss",
        desc_en: "Kill a boss without dying once",
        desc_zh: "击败 Boss 时还满血",
        stardust: 100,
        check: |w, _| w.bosses_killed_run >= 1 && w.player.lives == w.player.stats.max_lives,
    },
    Achievement {
        id: 15,
        name_en: "Six-figure",
        name_zh: "六位数",
        desc_en: "Score 100,000 in one run",
        desc_zh: "单局达到 10 万分",
        stardust: 200,
        check: |w, _| w.score >= 100_000,
    },
    // —— 跨局 ——
    Achievement {
        id: 16,
        name_en: "Hundred Runs",
        name_zh: "百战之身",
        desc_en: "Play 100 runs",
        desc_zh: "累计完成 100 局",
        stardust: 500,
        check: |_, s| s.runs >= 100,
    },
    Achievement {
        id: 17,
        name_en: "Boss Slayer",
        name_zh: "首领猎手",
        desc_en: "Kill 50 bosses lifetime",
        desc_zh: "累计击杀 50 个 Boss",
        stardust: 300,
        check: |_, s| s.bosses_killed >= 50,
    },
    Achievement {
        id: 18,
        name_en: "Star Saver",
        name_zh: "群星之主",
        desc_en: "Hold 10,000 stardust",
        desc_zh: "拥有 10000 星尘",
        stardust: 0, // 已经显式追星尘了，这条只是徽章
        check: |_, s| s.stardust >= 10_000,
    },
    Achievement {
        id: 19,
        name_en: "Synergist",
        name_zh: "协同大师",
        desc_en: "Activate 3 perks in a single run",
        desc_zh: "单局同时激活 3 个 Perk",
        stardust: 200,
        check: |w, _| {
            let p = &w.player.perks;
            let n = [
                p.heat_lock,
                p.static_mark,
                p.drone_relay,
                p.gravity_well,
                p.resonance,
                p.prism,
            ]
            .iter()
            .filter(|x| **x)
            .count();
            n >= 3
        },
    },
];

/// 在 save 里查询第 idx 条是否已解锁。
pub fn is_unlocked(save: &Save, idx: u8) -> bool {
    (save.achievements & (1u64 << idx)) != 0
}

/// 标记某条成就为已解锁（不写盘）。
pub fn mark_unlocked(save: &mut Save, idx: u8) {
    save.achievements |= 1u64 << idx;
}

/// 检查所有成就。返回本次新解锁的 (索引, stardust 奖励) 列表。
/// 调用方负责把奖励加到 save.stardust 并写盘。
pub fn check_all(world: &World, save: &Save) -> Vec<(u8, u64)> {
    let mut new_unlocks = Vec::new();
    for a in ACHIEVEMENTS {
        if is_unlocked(save, a.id) {
            continue;
        }
        if (a.check)(world, save) {
            new_unlocks.push((a.id, a.stardust));
        }
    }
    new_unlocks
}
