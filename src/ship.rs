use crate::entity::Player;
use crate::weapon::{Drone, Missile, WeaponSlot};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShipType {
    Vanguard,
    Striker,
    Engineer,
}

impl ShipType {
    pub const ALL: [ShipType; 3] = [ShipType::Vanguard, ShipType::Striker, ShipType::Engineer];

    pub fn name(self) -> &'static str {
        match self {
            ShipType::Vanguard => "Vanguard",
            ShipType::Striker => "Striker",
            ShipType::Engineer => "Engineer",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            ShipType::Vanguard => "Main Gun Lv2, stronger frontal burst",
            ShipType::Striker => "Move speed +18%, tighter evasion",
            ShipType::Engineer => "Starts with a support weapon, weaker gun",
        }
    }

    /// 给菜单 UI 用的属性条预览（值在 0..1 内）。
    pub fn stats_preview(self) -> [(&'static str, f32); 3] {
        match self {
            ShipType::Vanguard => [("DMG", 0.85), ("SPD", 0.50), ("TECH", 0.40)],
            ShipType::Striker => [("DMG", 0.55), ("SPD", 0.90), ("TECH", 0.40)],
            ShipType::Engineer => [("DMG", 0.45), ("SPD", 0.60), ("TECH", 0.90)],
        }
    }

    pub fn apply(self, player: &mut Player, weapons: &mut WeaponSlot) {
        match self {
            ShipType::Vanguard => {
                weapons.main.level_up();
                player.stats.damage_mul *= 1.10;
            }
            ShipType::Striker => {
                player.stats.speed *= 1.18;
                player.stats.invincible *= 0.92;
            }
            ShipType::Engineer => {
                player.stats.fire_rate *= 1.08;
                player.stats.damage_mul *= 0.96; // 仅 -4% 伤害，副武器收益可抵消
                if !weapons.has("drone") {
                    weapons.subs.push(Box::new(Drone::new()));
                } else if !weapons.has("missile") {
                    weapons.subs.push(Box::new(Missile::new()));
                }
            }
        }
    }
}
