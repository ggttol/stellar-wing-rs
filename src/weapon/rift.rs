//! 虚空裂隙：部署型伤害场。在玩家位置放置持续数秒的裂隙，周期性脉冲伤害范围内敌人。
//! 等级提升 → 裂隙数量 + 持续时间 + 脉冲频率 + 范围。

use macroquad::prelude::*;

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::SubWeapon;

struct RiftInstance {
    x: f32,
    y: f32,
    life: f32,
    pulse_timer: f32,
}

pub struct VoidRift {
    level: u8,
    last_place: f32,
    rifts: Vec<RiftInstance>,
}

impl VoidRift {
    pub fn new() -> Self {
        Self {
            level: 1,
            last_place: -10.0,
            rifts: Vec::with_capacity(4),
        }
    }

    fn max_rifts(&self) -> usize {
        match self.level {
            1 => 1,
            2 => 1,
            3 => 2,
            4 => 2,
            _ => 3,
        }
    }

    fn lifetime(&self) -> f32 {
        2.5 + self.level as f32 * 0.4
    }

    fn pulse_interval(&self) -> f32 {
        (1.2 - (self.level as f32 - 1.0) * 0.1).max(0.8)
    }

    fn radius(&self) -> f32 {
        55.0 + self.level as f32 * 7.0
    }

    fn place_interval(&self) -> f32 {
        match self.level {
            1 => 3.5,
            2 => 3.0,
            3 => 2.8,
            4 => 2.5,
            _ => 2.2,
        }
    }
}

impl SubWeapon for VoidRift {
    fn id(&self) -> &'static str {
        "rift"
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
        _bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
    ) {
        // 放置新裂隙
        if t - self.last_place >= self.place_interval() {
            self.last_place = t;
            if self.rifts.len() >= self.max_rifts() {
                self.rifts.remove(0); // 移除最老的
            }
            self.rifts.push(RiftInstance {
                x: player.x,
                y: player.y,
                life: self.lifetime(),
                pulse_timer: 0.0,
            });
        }

        let radius = self.radius();
        let interval = self.pulse_interval();
        let color = Color::from_rgba(160, 100, 255, 255);
        let gravity = player.perks.gravity_well;

        // 更新 & 脉冲
        self.rifts.retain_mut(|r| {
            r.life -= dt;
            r.pulse_timer += dt;

            // Gravity Well：缓慢吸入敌人
            if gravity && r.life > 0.0 {
                for e in enemies.iter_mut() {
                    if e.dead {
                        continue;
                    }
                    let dx = r.x - e.x;
                    let dy = r.y - e.y;
                    let d2 = dx * dx + dy * dy;
                    if d2 < radius * radius && d2 > 1.0 {
                        let d = d2.sqrt();
                        let pull = 40.0 * dt;
                        e.x += dx / d * pull;
                        e.y += dy / d * pull;
                    }
                }
            }

            if r.pulse_timer >= interval {
                r.pulse_timer = 0.0;
                let dmg = 1.3 + self.level as f32 * 0.5; // 基础脉冲伤害
                let hit_count = enemies
                    .iter_mut()
                    .filter(|e| {
                        if e.dead {
                            return false;
                        }
                        let dx = r.x - e.x;
                        let dy = r.y - e.y;
                        dx * dx + dy * dy < radius * radius
                    })
                    .map(|e| {
                        e.hp -= dmg;
                        e.hit_flash = 0.08;
                        e.last_hit = HitSource::Rift;
                    })
                    .count();
                if hit_count > 0 {
                    fx.burst(r.x, r.y, 4, 2.5, color, 60.0);
                }
            }
            r.life > 0.0
        });
    }

    fn draw(&self, player: &Player, t: f32, ox: f32, oy: f32) {
        let color = Color::from_rgba(160, 100, 255, 255);
        for r in &self.rifts {
            let a = (r.life / self.lifetime()).min(1.0).max(0.0);
            let x = r.x + ox;
            let y = r.y + oy;

            // 外圈脉冲
            let pulse_phase = (t * 4.0).sin() * 0.2 + 0.8;
            let mut outer = color;
            outer.a = 0.2 * a * pulse_phase;
            draw_circle(x, y, self.radius(), outer);

            // 边界环
            let mut ring = color;
            ring.a = 0.55 * a;
            draw_circle_lines(x, y, self.radius(), 2.0, ring);

            // 内核
            let mut core = color;
            core.a = 0.7 * a * pulse_phase;
            draw_circle(x, y, 12.0, core);

            // 吸入粒子（Gravity Well 生效时更密）
            let particle_count = if player.perks.gravity_well { 4 } else { 1 };
            for i in 0..particle_count {
                let angle = t * 3.0 + i as f32 * std::f32::consts::TAU / particle_count as f32
                    + r.life * 2.0;
                let dist = self.radius() * 0.6 * (t * 2.0 + i as f32).sin().abs();
                let px = x + angle.cos() * dist;
                let py = y + angle.sin() * dist;
                let mut pc = color;
                pc.a = 0.6 * a;
                draw_circle(px, py, 2.0, pc);
            }
        }
    }
}
