//! 章节定义：每章一段固定时长 + 主题色 + Boss 池 + 章节专属敌人。
//!
//! 1..=5 为故事章节，超出后进入循环增强的 [`ENDLESS`] 模板。
//! `chapter::get(chapter_idx)` 屏蔽这一差异，直接给出当前生效的章节配置。

use crate::entity::BossMod;

#[derive(Clone, Copy)]
pub struct Chapter {
    pub id: u32,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub tagline_en: &'static str,
    pub tagline_zh: &'static str,
    /// 章节内"普通推进"时长（秒），到点 spawn boss。
    pub duration: f32,
    /// 星空着色（RGB 倍率，会乘到默认蓝白星色上）。
    pub star_tint: (f32, f32, f32),
    /// 背景顶部 / 中部渐变颜色。
    pub bg_top: [u8; 3],
    pub bg_mid: [u8; 3],
    /// 该章节可以掉落的 Boss 词缀池。
    pub boss_pool: &'static [BossMod],
    /// 普通敌人生成密度倍率（×刷怪间隔的倒数）。
    pub spawn_intensity: f32,
    /// 把 Small/Medium 替换成 Kamikaze 的概率。
    pub kamikaze_chance: f32,
    /// Strafer 生成间隔；0 = 不出。
    pub strafer_interval: f32,
    /// Endless 模式标志：取消数值上限 + 双 boss。
    pub endless: bool,
}

pub static CHAPTERS: &[Chapter] = &[
    Chapter {
        id: 1,
        name_en: "OUTER BELT",
        name_zh: "外环带",
        tagline_en: "Routine patrol — clear the asteroid lane",
        tagline_zh: "例行巡航 · 清扫小行星带",
        duration: 60.0,
        star_tint: (0.85, 0.95, 1.0),
        bg_top: [2, 3, 10],
        bg_mid: [6, 9, 26],
        boss_pool: &[BossMod::Frenzied],
        spawn_intensity: 1.0,
        kamikaze_chance: 0.0,
        strafer_interval: 0.0,
        endless: false,
    },
    Chapter {
        id: 2,
        name_en: "CRIMSON DRIFT",
        name_zh: "赤色漂流",
        tagline_en: "Suicide raiders inbound",
        tagline_zh: "自爆袭击者正在逼近",
        duration: 75.0,
        star_tint: (1.10, 0.55, 0.55),
        bg_top: [12, 4, 8],
        bg_mid: [22, 6, 12],
        boss_pool: &[BossMod::Frenzied, BossMod::Bulwark],
        spawn_intensity: 1.10,
        kamikaze_chance: 0.30,
        strafer_interval: 0.0,
        endless: false,
    },
    Chapter {
        id: 3,
        name_en: "ION STORM",
        name_zh: "离子风暴",
        tagline_en: "Strafing runs cut the sky",
        tagline_zh: "扫射机划破天幕",
        duration: 90.0,
        star_tint: (0.55, 0.85, 1.20),
        bg_top: [3, 6, 22],
        bg_mid: [4, 12, 36],
        boss_pool: &[BossMod::Summoner, BossMod::StormCore],
        spawn_intensity: 1.18,
        kamikaze_chance: 0.10,
        strafer_interval: 6.0,
        endless: false,
    },
    Chapter {
        id: 4,
        name_en: "GHOST TIDE",
        name_zh: "幽灵潮汐",
        tagline_en: "What you can't see still kills",
        tagline_zh: "看不见的东西也会杀人",
        duration: 100.0,
        star_tint: (0.95, 0.70, 1.10),
        bg_top: [8, 4, 14],
        bg_mid: [16, 8, 26],
        boss_pool: &[BossMod::Phantom, BossMod::Summoner],
        spawn_intensity: 1.30,
        kamikaze_chance: 0.20,
        strafer_interval: 8.0,
        endless: false,
    },
    Chapter {
        id: 5,
        name_en: "DREADNOUGHT CORE",
        name_zh: "无畏舰核心",
        tagline_en: "Tear the heart from the fleet",
        tagline_zh: "撕开舰队的心脏",
        duration: 110.0,
        star_tint: (1.10, 0.80, 0.45),
        bg_top: [14, 8, 4],
        bg_mid: [24, 14, 6],
        boss_pool: &[BossMod::Hydra, BossMod::StormCore, BossMod::Bulwark],
        spawn_intensity: 1.45,
        kamikaze_chance: 0.25,
        strafer_interval: 5.0,
        endless: false,
    },
];

pub static ENDLESS: Chapter = Chapter {
    id: 99,
    name_en: "ENDLESS",
    name_zh: "无尽",
    tagline_en: "No boundaries. No mercy.",
    tagline_zh: "无界 · 无情",
    duration: 60.0,
    star_tint: (1.20, 0.65, 0.95),
    bg_top: [16, 4, 12],
    bg_mid: [28, 8, 22],
    boss_pool: &[
        BossMod::Frenzied,
        BossMod::Bulwark,
        BossMod::Summoner,
        BossMod::StormCore,
        BossMod::Phantom,
        BossMod::Hydra,
    ],
    spawn_intensity: 1.80,
    kamikaze_chance: 0.30,
    strafer_interval: 4.0,
    endless: true,
};

pub fn get(chapter_idx: u32) -> &'static Chapter {
    let i = chapter_idx as usize;
    if i < CHAPTERS.len() {
        &CHAPTERS[i]
    } else {
        &ENDLESS
    }
}

pub fn total() -> u32 {
    CHAPTERS.len() as u32
}
