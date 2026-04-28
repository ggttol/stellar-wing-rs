//! 反射镜：优先朝最近敌人发射可反弹弹丸，兼具精准首击和复杂反弹路径。
//! 等级提升 → 弹数 + 反弹次数 + 射速。

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

pub struct Reflector {
    level: u8,
    last_shot: f32,
}

impl Reflector {
    pub fn new() -> Self {
        Self {
            level: 1,
            last_shot: -10.0,
        }
    }

    fn count(&self) -> usize {
        match self.level {
            1 => 1,
            2 => 2,
            3 => 2,
            4 => 2,
            _ => 3,
        }
    }

    fn interval(&self) -> f32 {
        (1.10 - (self.level as f32 - 1.0) * 0.09).max(0.75)
    }

    fn bounces(&self) -> u8 {
        match self.level {
            1 => 2,
            2 => 3,
            3 => 3,
            4 => 4,
            _ => 4,
        }
    }
}

impl SubWeapon for Reflector {
    fn id(&self) -> &'static str {
        "reflector"
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
        _dt: f32,
        t: f32,
        player: &Player,
        enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        _fx: &mut Fx,
    ) {
        if t - self.last_shot < self.interval() {
            return;
        }
        self.last_shot = t;
        let n = self.count();
        let speed = 430.0;
        let bounces = self.bounces();
        let base_dir =
            aim_at_nearest(enemies, player.x, player.y - player.h * 0.5).unwrap_or((0.0, -1.0));

        for i in 0..n {
            let offsets: [f32; 3] = [0.0, -0.24, 0.24];
            let dir = rotate(base_dir, offsets[i.min(2)]);
            let vx = dir.0 * speed;
            let vy = dir.1 * speed;

            let mut b = Bullet::player_shot(player.x, player.y - player.h * 0.5, vx, vy);
            let (dmg, crit) = roll_crit(player, 1.0 + self.level as f32 * 0.14);
            b.damage = dmg;
            b.is_crit = crit;
            b.w = 7.0;
            b.h = 7.0;
            b.source = HitSource::Reflector;
            b.bounces = bounces;
            if self.level >= 4 {
                b.pierce = 1;
            }
            bullets.push(b);
        }
    }

    fn draw(&self, _player: &Player, _t: f32, _ox: f32, _oy: f32) {}
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

fn rotate((x, y): (f32, f32), angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (x * c - y * s, x * s + y * c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EnemyKind;
    use crate::ship::ShipType;

    #[test]
    fn reflector_aims_first_shot_toward_enemy() {
        let player = Player::with_ship(ShipType::Vanguard);
        let mut reflector = Reflector::new();
        let mut enemies = vec![Enemy::new(EnemyKind::Medium, player.x + 120.0, 0.0)];
        enemies[0].y = player.y - 160.0;
        let mut bullets = Vec::new();
        let mut fx = Fx::default();

        reflector.tick(0.0, 1.2, &player, &mut enemies, &mut bullets, &mut fx);

        assert_eq!(bullets.len(), 1);
        assert!(bullets[0].vx > 0.0);
        assert!(bullets[0].vy < 0.0);
    }
}
