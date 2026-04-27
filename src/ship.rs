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
                player.stats.damage_mul *= 0.92;
                if !weapons.has("drone") {
                    weapons.subs.push(Box::new(Drone::new()));
                } else if !weapons.has("missile") {
                    weapons.subs.push(Box::new(Missile::new()));
                }
            }
        }
    }
}
