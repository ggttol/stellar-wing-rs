//! 环绕僚机：在玩家周围旋转，定期向前发射子弹。
//! 等级提升 → 数量增加 + 射速更高。

use macroquad::prelude::*;

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

pub struct Drone {
    level: u8,
    angle: f32,
    last_shot: f32,
}

impl Drone {
    pub fn new() -> Self {
        Self {
            level: 1,
            angle: 0.0,
            last_shot: -10.0,
        }
    }

    fn count(&self) -> usize {
        match self.level {
            1 => 1,
            2 => 2,
            3 => 2,
            4 => 3,
            _ => 3,
        }
    }
    fn radius(&self) -> f32 {
        52.0
    }
    fn fire_rate(&self) -> f32 {
        match self.level {
            1 => 0.62,
            2 => 0.56,
            3 => 0.52,
            4 => 0.48,
            _ => 0.44,
        }
    }
}

impl SubWeapon for Drone {
    fn id(&self) -> &'static str {
        "drone"
    }
    fn level(&self) -> u8 {
        self.level
    }
    fn level_up(&mut self) {
        if self.level < 5 {
            self.level += 1;
        }
    }

    fn tick(
        &mut self,
        dt: f32,
        t: f32,
        player: &Player,
        _enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        _fx: &mut Fx,
    ) {
        self.angle += dt * 1.8;
        let n = self.count();
        if t - self.last_shot < self.fire_rate() {
            return;
        }
        self.last_shot = t;
        for i in 0..n {
            let a = self.angle + i as f32 * std::f32::consts::TAU / n as f32;
            let dx = a.cos() * self.radius();
            let dy = a.sin() * self.radius();
            let mut b = Bullet::player_shot(player.x + dx, player.y + dy, 0.0, -700.0);
            let (dmg, crit) = roll_crit(player, 0.80);
            b.damage = dmg;
            b.is_crit = crit;
            b.w = 3.0;
            b.h = 10.0;
            b.source = HitSource::Drone;
            bullets.push(b);
        }
    }

    fn draw(&self, player: &Player, t: f32, ox: f32, oy: f32) {
        let n = self.count();
        for i in 0..n {
            let a = self.angle + i as f32 * std::f32::consts::TAU / n as f32;
            let dx = a.cos() * self.radius();
            let dy = a.sin() * self.radius();
            let x = player.x + dx + ox;
            let y = player.y + dy + oy;
            let pulse = 0.7 + (t * 8.0 + i as f32).sin() * 0.3;
            let mut g = Color::from_rgba(125, 249, 255, 255);
            g.a = 0.4 * pulse;
            draw_circle(x, y, 9.0, g);
            draw_circle(x, y, 4.5, Color::from_rgba(0, 212, 255, 255));
            draw_circle(x, y, 1.8, WHITE);
        }
    }
}
