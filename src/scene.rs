//! 顶层场景状态机。

use crate::upgrade::Card;

pub enum Scene {
    Menu,
    /// 永久天赋购买页（光标位置）
    Talents(usize),
    Playing,
    Paused,
    UpgradePick(Vec<Card>),
    GameOver,
}
