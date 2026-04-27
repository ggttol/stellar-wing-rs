pub mod chain;
pub mod drone;
pub mod laser;
pub mod main_gun;
pub mod missile;

pub use chain::Chain;
pub use drone::Drone;
pub use laser::Laser;
pub use main_gun::MainGun;
pub use missile::Missile;

use crate::entity::{Bullet, Enemy, Player};
use crate::fx::Fx;

pub struct DecayGauge {
    timer: f32,
}

impl DecayGauge {
    pub fn new() -> Self {
        Self { timer: 0.0 }
    }

    pub fn refill(&mut self, level: u8) {
        self.timer = Self::duration_for(level);
    }

    pub fn tick_dt(&mut self, dt: f32, level: &mut u8, floor: u8) {
        if *level <= floor {
            self.timer = 0.0;
            return;
        }

        self.timer -= dt;
        if self.timer > 0.0 {
            return;
        }

        *level = level.saturating_sub(1).max(floor);
        if *level > floor {
            self.timer = Self::duration_for(*level);
        } else {
            self.timer = 0.0;
        }
    }

    pub fn ratio(&self, level: u8, floor: u8) -> Option<f32> {
        if level <= floor {
            None
        } else {
            Some((self.timer / Self::duration_for(level)).clamp(0.0, 1.0))
        }
    }

    fn duration_for(level: u8) -> f32 {
        match level {
            0 | 1 => 0.0,
            2 => 28.0,
            3 => 22.0,
            4 => 17.0,
            _ => 13.0,
        }
    }
}

/// 副武器接口。主武器有自己的具体类型，不走 trait（性能更稳）。
pub trait SubWeapon {
    fn id(&self) -> &'static str;
    fn level(&self) -> u8;
    fn level_up(&mut self);
    fn decay_tick(&mut self, dt: f32);
    fn decay_ratio(&self) -> Option<f32>;
    fn max_level(&self) -> u8 {
        5
    }
    fn tick(
        &mut self,
        dt: f32,
        t: f32,
        player: &Player,
        enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
    );
    fn draw(&self, player: &Player, t: f32);
}

pub struct WeaponSlot {
    pub main: MainGun,
    pub subs: Vec<Box<dyn SubWeapon>>,
}

impl WeaponSlot {
    pub fn new() -> Self {
        Self {
            main: MainGun::new(),
            subs: Vec::with_capacity(4),
        }
    }

    pub fn tick(
        &mut self,
        dt: f32,
        t: f32,
        player: &Player,
        enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
    ) -> bool {
        self.main.decay_tick(dt);
        let fired_main = self.main.tick(t, player, bullets);
        for s in &mut self.subs {
            s.decay_tick(dt);
            s.tick(dt, t, player, enemies, bullets, fx);
        }

        fired_main
    }

    pub fn draw(&self, player: &Player, t: f32) {
        for s in &self.subs {
            s.draw(player, t);
        }
    }

    pub fn has(&self, id: &str) -> bool {
        self.subs.iter().any(|s| s.id() == id)
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Box<dyn SubWeapon>> {
        self.subs.iter_mut().find(|s| s.id() == id)
    }
}

/// 武器统一的暴击滚动：返回 (damage, is_crit)。`base_mul` 是该武器对玩家伤害的基础倍率。
pub fn roll_crit(player: &Player, base_mul: f32) -> (f32, bool) {
    use ::rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    let base = player.stats.damage_mul * base_mul;
    if player.stats.crit_chance > 0.0 && rng.gen::<f32>() < player.stats.crit_chance {
        (base * player.stats.crit_mul, true)
    } else {
        (base, false)
    }
}
