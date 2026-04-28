//! 环绕僚机：在玩家周围旋转，定期朝最近敌人射击。
//! 等级提升 → 数量增加 + 射速更高 + 单发伤害略升。

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
        enemies: &mut [Enemy],
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
            let sx = player.x + dx;
            let sy = player.y + dy;
            let (vx, vy) = aim_at_nearest(enemies, sx, sy).unwrap_or((0.0, -1.0));
            let speed = 650.0 + self.level as f32 * 25.0;
            let mut b = Bullet::player_shot(sx, sy, vx * speed, vy * speed);
            let (dmg, crit) = roll_crit(player, 0.86 + self.level as f32 * 0.04);
            b.damage = dmg;
            b.is_crit = crit;
            b.w = 4.0;
            b.h = 11.0;
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

fn aim_at_nearest(enemies: &[Enemy], x: f32, y: f32) -> Option<(f32, f32)> {
    enemies
        .iter()
        .filter(|e| !e.dead)
        .map(|e| {
            let dx = e.x - x;
            let dy = e.y - y;
            (dx, dy, dx * dx + dy * dy)
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(dx, dy, _)| {
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            (dx / len, dy / len)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EnemyKind;
    use crate::ship::ShipType;

    #[test]
    fn drone_aims_at_side_enemy_instead_of_only_up() {
        let player = Player::with_ship(ShipType::Vanguard);
        let mut drone = Drone::new();
        let mut enemies = vec![Enemy::new(EnemyKind::Medium, player.x + 160.0, 0.0)];
        enemies[0].y = player.y - 20.0;
        let mut bullets = Vec::new();
        let mut fx = Fx::default();

        drone.tick(0.0, 1.0, &player, &mut enemies, &mut bullets, &mut fx);

        assert_eq!(bullets.len(), 1);
        assert!(bullets[0].vx.abs() > 100.0);
    }
}
