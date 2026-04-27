//! 顶层场景状态机。

use crate::upgrade::Card;

#[allow(dead_code)] // Instructions/Settings/Paused 等留给后续里程碑
pub enum Scene {
    Menu,
    Instructions,
    Leaderboard,
    Settings,
    /// 永久天赋购买页（光标位置）
    Talents(usize),
    Playing,
    Paused,
    UpgradePick(Vec<Card>),
    GameOver,
}
