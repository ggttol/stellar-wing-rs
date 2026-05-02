//! 顶层场景状态机。

use crate::upgrade::Card;

pub enum Scene {
    Menu,
    /// 永久天赋购买页（光标位置）
    Talents(usize),
    /// 设置页（光标位置）
    Settings(usize),
    /// 成就页（光标位置）
    Achievements(usize),
    /// 图鉴页（tab, 光标）
    Codex(u8, usize),
    /// 章节分叉选择（候选两条路线，光标）
    ChapterChoice([crate::world::ChapterMod; 2], usize),
    Playing,
    Paused,
    UpgradePick(Vec<Card>),
    GameOver,
}
