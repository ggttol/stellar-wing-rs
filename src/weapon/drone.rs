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
    /// 缓存上一帧每个僚机锁定的目标方向，用于绘制锁敌指示
    aim_cache: Vec<(f32, f32)>,
}

impl Drone {
    pub fn new() -> Self {
        Self {
            level: 1,
            angle: 0.0,
            last_shot: -10.0,
            aim_cache: Vec::new(),
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
        _damage_acc: &mut [f32; 9],
    ) {
        self.angle += dt * 1.8;
        let evo = player.perks.evo_drone;
        let n = self.count() + if evo { 1 } else { 0 };

        // 每帧刷新瞄准缓存（用于绘制锁敌方向）
        self.aim_cache.clear();
        for i in 0..n {
            let a = self.angle + i as f32 * std::f32::consts::TAU / n as f32;
            let sx = player.x + a.cos() * self.radius();
            let sy = player.y + a.sin() * self.radius();
            let dir = aim_at_nearest(enemies, sx, sy).unwrap_or((0.0, -1.0));
            self.aim_cache.push(dir);
        }

        let fr = self.fire_rate() * if evo { 0.85 } else { 1.0 };
        if t - self.last_shot < fr {
            return;
        }
        self.last_shot = t;
        for i in 0..n {
            let a = self.angle + i as f32 * std::f32::consts::TAU / n as f32;
            let dx = a.cos() * self.radius();
            let dy = a.sin() * self.radius();
            let sx = player.x + dx;
            let sy = player.y + dy;
            let (vx, vy) = self.aim_cache[i];
            let speed = 650.0 + self.level as f32 * 25.0;
            let mut b = Bullet::player_shot(sx, sy, vx * speed, vy * speed);
            let (dmg, crit) = roll_crit(player, 1.00 + self.level as f32 * 0.05);
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
        let cyan_soft = Color::from_rgba(125, 249, 255, 255);
        let cyan_core = Color::from_rgba(0, 212, 255, 255);
        for i in 0..n {
            let a = self.angle + i as f32 * std::f32::consts::TAU / n as f32;
            let dx = a.cos() * self.radius();
            let dy = a.sin() * self.radius();
            let x = player.x + dx + ox;
            let y = player.y + dy + oy;

            // 锁敌指示线：从僚机向当前瞄准方向画一段渐隐虚线
            if let Some((vx, vy)) = self.aim_cache.get(i).copied() {
                let pulse = 0.55 + (t * 6.0 + i as f32 * 0.7).sin() * 0.25;
                let mut lc = cyan_soft;
                lc.a = 0.30 * pulse;
                let len = 38.0;
                let ex = x + vx * len;
                let ey = y + vy * len;
                draw_line(x, y, ex, ey, 1.0, lc);
                // 末端小三角，强化"锁定"暗示
                let perp = (-vy, vx);
                let tip = (x + vx * len, y + vy * len);
                let base = (x + vx * (len - 5.0), y + vy * (len - 5.0));
                let p1 = (base.0 + perp.0 * 2.5, base.1 + perp.1 * 2.5);
                let p2 = (base.0 - perp.0 * 2.5, base.1 - perp.1 * 2.5);
                let mut tc = cyan_soft;
                tc.a = 0.55 * pulse;
                draw_triangle(
                    Vec2::new(tip.0, tip.1),
                    Vec2::new(p1.0, p1.1),
                    Vec2::new(p2.0, p2.1),
                    tc,
                );
            }

            // 僚机本体：双层 halo + 内核 + 白点
            let pulse = 0.7 + (t * 8.0 + i as f32).sin() * 0.3;
            let mut g = cyan_soft;
            g.a = 0.18 * pulse;
            draw_circle(x, y, 14.0, g);
            g.a = 0.45 * pulse;
            draw_circle(x, y, 9.0, g);
            draw_circle(x, y, 4.5, cyan_core);
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

        let mut acc = [0.0_f32; 9];
        drone.tick(0.0, 1.0, &player, &mut enemies, &mut bullets, &mut fx, &mut acc);

        assert_eq!(bullets.len(), 1);
        assert!(bullets[0].vx.abs() > 100.0);
    }
}
