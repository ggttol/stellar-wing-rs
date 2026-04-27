//! 顶层场景状态机。

use crate::upgrade::Card;

#[allow(dead_code)] // Instructions/Settings/Paused 等留给后续里程碑
pub enum Scene {
    Menu,
    Instructions,
    Leaderboard,
    Settings,
    Playing,
    Paused,
    UpgradePick(Vec<Card>),
    GameOver,
}
