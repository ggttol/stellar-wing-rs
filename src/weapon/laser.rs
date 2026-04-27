//! 持续激光束：从玩家头顶向上发出垂直光束，对其覆盖范围内的敌机持续掉血。
//! 周期性 ON/OFF（避免无脑碾压）。等级提升 → 宽度 + DPS + 占空比。

use macroquad::prelude::*;

use crate::entity::{Bullet, Enemy, Player};
use crate::fx::Fx;
use crate::weapon::{DecayGauge, SubWeapon};

pub struct Laser {
    level: u8,
    /// 0..1，> on_duty 表示在冷却。每 cycle 秒回到 0。
    phase: f32,
    decay: DecayGauge,
}

impl Laser {
    pub fn new() -> Self {
        Self {
            level: 1,
            phase: 0.0,
            decay: DecayGauge::new(),
        }
    }

    fn cycle(&self) -> f32 {
        2.0 // 每 2 秒一个 ON+OFF 循环
    }
    fn on_duty(&self) -> f32 {
        0.45 + (self.level as f32 - 1.0) * 0.06
    }
    fn dps(&self, player: &Player) -> f32 {
        let base = 1.1 + self.level as f32 * 0.45;
        base * player.stats.damage_mul
    }
    fn width(&self) -> f32 {
        8.0 + self.level as f32 * 2.0
    }
    fn is_on(&self) -> bool {
        self.phase < self.on_duty()
    }
}

impl SubWeapon for Laser {
    fn id(&self) -> &'static str {
        "laser"
    }
    fn level(&self) -> u8 {
        self.level
    }
    fn level_up(&mut self) {
        if self.level < 5 {
            self.level += 1;
            self.decay.refill(self.level);
        }
    }
    fn decay_tick(&mut self, dt: f32) {
        self.decay.tick_dt(dt, &mut self.level, 1);
    }
    fn decay_ratio(&self) -> Option<f32> {
        self.decay.ratio(self.level, 1)
    }

    fn tick(
        &mut self,
        dt: f32,
        t: f32,
        player: &Player,
        enemies: &mut [Enemy],
        _bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
    ) {
        self.phase = (self.phase + dt / self.cycle()) % 1.0;
        if !self.is_on() {
            return;
        }
        let half_w = self.width() * 0.5;
        let dps = self.dps(player);
        for e in enemies.iter_mut() {
            if e.dead || e.y > player.y {
                continue;
            }
            if (e.x - player.x).abs() < half_w + e.radius {
                let mut mul = 1.0;
                if player.perks.heat_lock && e.marked_until > t {
                    mul += 0.4;
                }
                e.hp -= dps * mul * dt;
                e.hit_flash = 0.06;
                e.last_hit = crate::entity::HitSource::Laser;
                // 偶发命中粒子
                if rand_chance(dt * 25.0) {
                    fx.burst(e.x, e.y, 2, 2.0, Color::from_rgba(125, 249, 255, 255), 80.0);
                }
            }
        }
    }

    fn draw(&self, player: &Player, t: f32) {
        if !self.is_on() {
            // OFF 期间画一个微弱的瞄准虚线
            let mut c = Color::from_rgba(125, 249, 255, 255);
            c.a = 0.15;
            draw_line(player.x, player.y - player.h * 0.5, player.x, 0.0, 1.0, c);
            return;
        }
        let half_w = self.width() * 0.5;
        let pulse = 0.85 + (t * 18.0).sin() * 0.15;
        // 外辉
        let mut outer = Color::from_rgba(125, 249, 255, 255);
        outer.a = 0.25 * pulse;
        draw_rectangle(
            player.x - half_w * 1.6,
            0.0,
            half_w * 3.2,
            player.y - player.h * 0.5,
            outer,
        );
        // 主束
        let mut core = Color::from_rgba(220, 250, 255, 255);
        core.a = 0.85 * pulse;
        draw_rectangle(
            player.x - half_w,
            0.0,
            half_w * 2.0,
            player.y - player.h * 0.5,
            core,
        );
        // 中心高亮
        let mut hot = WHITE;
        hot.a = pulse;
        draw_rectangle(player.x - 1.5, 0.0, 3.0, player.y - player.h * 0.5, hot);
    }
}

fn rand_chance(p: f32) -> bool {
    use ::rand::{thread_rng, Rng};
    thread_rng().gen::<f32>() < p
}
