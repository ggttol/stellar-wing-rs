//! 虚空裂隙：追猎型伤害场。裂隙会从玩家附近生成，主动漂向最近敌人，
//! 持续灼烧范围内目标，并周期性释放更强脉冲。
//! 等级提升 → 裂隙数量 + 持续时间 + 脉冲频率 + 范围 + 追猎速度。

use macroquad::prelude::*;

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

struct RiftInstance {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    pulse_timer: f32,
    burn_timer: f32,
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
        3.8 + self.level as f32 * 0.45
    }

    fn pulse_interval(&self) -> f32 {
        (1.0 - (self.level as f32 - 1.0) * 0.08).max(0.68)
    }

    fn radius(&self) -> f32 {
        66.0 + self.level as f32 * 8.0
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

    fn base_damage(&self) -> f32 {
        1.2 + self.level as f32 * 0.28
    }

    fn chase_speed(&self) -> f32 {
        190.0 + self.level as f32 * 22.0
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
        damage_acc: &mut [f32; 9],
    ) {
        // 放置新裂隙
        if t - self.last_place >= self.place_interval() {
            self.last_place = t;
            if self.rifts.len() >= self.max_rifts() {
                self.rifts.remove(0); // 移除最老的
            }
            self.rifts.push(RiftInstance {
                x: player.x,
                y: player.y - 24.0,
                vx: 0.0,
                vy: -self.chase_speed() * 0.45,
                life: self.lifetime(),
                pulse_timer: self.pulse_interval(),
                burn_timer: 0.0,
            });
        }

        let radius = self.radius();
        let interval = self.pulse_interval();
        let base = self.base_damage();
        let chase_speed = self.chase_speed();
        let color = Color::from_rgba(160, 100, 255, 255);
        let gravity = player.perks.gravity_well;

        // 更新 & 脉冲
        self.rifts.retain_mut(|r| {
            r.life -= dt;
            r.pulse_timer += dt;
            r.burn_timer += dt;

            if let Some((dx, dy, dist)) = nearest_enemy_delta(enemies, r.x, r.y) {
                let desired_vx = dx / dist * chase_speed;
                let desired_vy = dy / dist * chase_speed;
                let steer = (6.0 * dt).min(1.0);
                r.vx += (desired_vx - r.vx) * steer;
                r.vy += (desired_vy - r.vy) * steer;
            } else {
                let steer = (2.0 * dt).min(1.0);
                r.vx += (0.0 - r.vx) * steer;
                r.vy += (-90.0 - r.vy) * steer;
            }
            r.x += r.vx * dt;
            r.y += r.vy * dt;

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

            if r.burn_timer >= 0.22 {
                r.burn_timer = 0.0;
                let mut hit_count = 0u32;
                for e in enemies.iter_mut() {
                    if e.dead {
                        continue;
                    }
                    let dx = r.x - e.x;
                    let dy = r.y - e.y;
                    if dx * dx + dy * dy < radius * radius {
                        let (dmg, _) = roll_crit(player, base * 0.34);
                        let applied = dmg * e.damage_mul();
                        e.hp -= applied;
                        damage_acc[HitSource::Rift as usize] += applied;
                        e.hit_flash = 0.05;
                        e.last_hit = HitSource::Rift;
                        hit_count += 1;
                    }
                }
                if hit_count > 0 {
                    fx.burst(r.x, r.y, 2, 1.6, color, 36.0);
                }
            }

            if r.pulse_timer >= interval {
                r.pulse_timer = 0.0;
                let mut hit_count = 0u32;
                for e in enemies.iter_mut() {
                    if e.dead {
                        continue;
                    }
                    let dx = r.x - e.x;
                    let dy = r.y - e.y;
                    if dx * dx + dy * dy < radius * radius {
                        let (dmg, _) = roll_crit(player, base);
                        let applied = dmg * e.damage_mul();
                        e.hp -= applied;
                        damage_acc[HitSource::Rift as usize] += applied;
                        e.hit_flash = 0.08;
                        e.last_hit = HitSource::Rift;
                        hit_count += 1;
                    }
                }
                if hit_count > 0 {
                    fx.burst(r.x, r.y, 7, 3.0, color, 90.0);
                }
            }
            r.life > 0.0
        });
    }

    fn draw(&self, player: &Player, t: f32, ox: f32, oy: f32) {
        let color = Color::from_rgba(160, 100, 255, 255);
        let core_color = Color::from_rgba(220, 180, 255, 255);
        let radius = self.radius();
        for r in &self.rifts {
            let a = (r.life / self.lifetime()).clamp(0.0, 1.0);
            let x = r.x + ox;
            let y = r.y + oy;

            // 外晕 — 多层 soft halo 模拟"重力扭曲"的边缘渐变
            let pulse = 0.85 + (t * 4.0).sin() * 0.15;
            let halo_layers: [(f32, f32); 3] = [(1.18, 0.10), (0.95, 0.18), (0.72, 0.30)];
            for (rmul, amul) in halo_layers {
                let mut c = color;
                c.a = amul * a * pulse;
                draw_circle(x, y, radius * rmul, c);
            }

            // 双层边界环：外细虚线感（描两遍），内厚一层
            let mut ring = color;
            ring.a = 0.30 * a;
            draw_circle_lines(x, y, radius * 1.02, 1.0, ring);
            ring.a = 0.65 * a;
            draw_circle_lines(x, y, radius, 2.5, ring);

            // 旋转中的能量弧线（3 道，错相位）
            let arcs = 3;
            let segs = 18;
            let r_orbit = radius * 0.78;
            for k in 0..arcs {
                let phase = t * 1.6 + k as f32 * std::f32::consts::TAU / arcs as f32;
                // 每弧只画一段（0.6 弧度 ≈ 35°），然后随 phase 旋转
                let arc_span = 0.55_f32;
                for s in 0..segs {
                    let u = s as f32 / segs as f32;
                    let theta = phase + u * arc_span;
                    let theta_n = phase + (u + 1.0 / segs as f32) * arc_span;
                    // 弧线"飘动"半径，让它有起伏
                    let wob = (t * 3.0 + u * std::f32::consts::TAU + k as f32).sin() * 4.0;
                    let r1 = r_orbit + wob;
                    let r2 = r_orbit + wob;
                    let x1 = x + theta.cos() * r1;
                    let y1 = y + theta.sin() * r1;
                    let x2 = x + theta_n.cos() * r2;
                    let y2 = y + theta_n.sin() * r2;
                    let mut c = core_color;
                    c.a = a * 0.85 * (1.0 - u * 0.5);
                    draw_line(x1, y1, x2, y2, 2.0, c);
                }
            }

            // 内核：双层 + 偏白热点
            let core_pulse = 0.80 + (t * 8.0).sin() * 0.20;
            let mut hot = core_color;
            hot.a = 0.70 * a * core_pulse;
            draw_circle(x, y, 14.0, hot);
            hot.a = a * core_pulse;
            draw_circle(x, y, 7.0, hot);
            let mut white = WHITE;
            white.a = 0.85 * a * core_pulse;
            draw_circle(x, y, 3.0, white);

            // Gravity Well 生效：从外缘朝内的 4 个吸入粒子
            if player.perks.gravity_well {
                for i in 0..4 {
                    let phase = (t * 1.5 + i as f32 * 0.25) % 1.0;
                    let theta =
                        i as f32 * std::f32::consts::TAU / 4.0 + t * 0.7;
                    let inward = radius * (1.0 - phase);
                    let px = x + theta.cos() * inward;
                    let py = y + theta.sin() * inward;
                    let mut pc = core_color;
                    pc.a = a * (1.0 - phase) * 0.85;
                    draw_circle(px, py, 2.6, pc);
                }
            }
        }
    }
}

fn nearest_enemy_delta(enemies: &[Enemy], x: f32, y: f32) -> Option<(f32, f32, f32)> {
    enemies
        .iter()
        .filter(|e| !e.dead)
        .map(|e| {
            let dx = e.x - x;
            let dy = e.y - y;
            let d2 = dx * dx + dy * dy;
            (dx, dy, d2)
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(dx, dy, d2)| (dx, dy, d2.sqrt().max(1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EnemyKind;
    use crate::ship::ShipType;

    #[test]
    fn rift_chases_nearest_enemy_and_deals_damage() {
        let player = Player::with_ship(ShipType::Vanguard);
        let mut rift = VoidRift::new();
        let mut enemies = vec![Enemy::new(EnemyKind::Medium, player.x, 360.0)];
        enemies[0].y = player.y - 120.0;
        let hp = enemies[0].hp;
        let mut bullets = Vec::new();
        let mut fx = Fx::default();

        let mut acc = [0.0_f32; 9];
        rift.tick(0.1, 10.0, &player, &mut enemies, &mut bullets, &mut fx, &mut acc);
        let start_y = rift.rifts[0].y;
        rift.tick(0.3, 10.3, &player, &mut enemies, &mut bullets, &mut fx, &mut acc);

        assert!(rift.rifts[0].y < start_y);
        assert!(enemies[0].hp < hp);
    }
}
