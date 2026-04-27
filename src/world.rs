//! 一局游戏的可变状态。

use crate::entity::{Bullet, Enemy, Pickup, Player};
use crate::ship::ShipType;
use crate::weapon::WeaponSlot;

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
    pub next_boss_at: f32,
    pub boss_alive: bool,
    pub super_charge: f32,
    pub combo: u32,
    pub combo_timer: f32,
    pub combo_flash: f32,
    pub combo_note_idx: usize,
    /// 主武器击中音效冷却（秒），避免一帧打几十发就响成一片。
    pub hit_sfx_cooldown: f32,
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
            next_boss_at: 60.0,
            boss_alive: false,
            super_charge: 0.2,
            combo: 0,
            combo_timer: 0.0,
            combo_flash: 0.0,
            combo_note_idx: 0,
            hit_sfx_cooldown: 0.0,
        }
    }

    pub fn diff_mul(&self) -> f32 {
        (0.85 + self.run_time / 200.0).min(1.8)
    }
}
