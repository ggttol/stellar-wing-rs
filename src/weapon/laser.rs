//! 持续激光束：从玩家头顶向上发出会轻微追踪目标横坐标的光束。
//! 周期性 ON/OFF（避免无脑碾压）。等级提升 → 宽度 + DPS + 占空比。

use macroquad::prelude::*;

use crate::entity::{Bullet, Enemy, Player};
use crate::fx::Fx;
use crate::weapon::SubWeapon;

pub struct Laser {
    level: u8,
    /// 0..1，> on_duty 表示在冷却。每 cycle 秒回到 0。
    phase: f32,
    beam_x: f32,
}

impl Laser {
    pub fn new() -> Self {
        Self {
            level: 1,
            phase: 0.0,
            beam_x: -1.0,
        }
    }

    fn cycle(&self) -> f32 {
        2.0 // 每 2 秒一个 ON+OFF 循环
    }
    fn on_duty(&self, evo: bool) -> f32 {
        let base = 0.45 + (self.level as f32 - 1.0) * 0.06;
        if evo { (base + 0.15).min(0.95) } else { base }
    }
    fn dps(&self, player: &Player) -> f32 {
        let base = 1.6 + self.level as f32 * 0.55;
        let mul = if player.perks.evo_laser { 1.6 } else { 1.0 };
        base * player.stats.damage_mul * mul
    }
    fn width(&self, evo: bool) -> f32 {
        let w = 14.0 + self.level as f32 * 3.0;
        if evo { w * 1.5 } else { w }
    }
    fn is_on_with(&self, evo: bool) -> bool {
        self.phase < self.on_duty(evo)
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
        if self.beam_x < 0.0 {
            self.beam_x = player.x;
        }
        let target_x = laser_target_x(enemies, player).unwrap_or(player.x);
        let track = (dt * (3.8 + self.level as f32 * 0.35)).min(1.0);
        self.beam_x += (target_x - self.beam_x) * track;

        self.phase = (self.phase + dt / self.cycle()) % 1.0;
        let evo = player.perks.evo_laser;
        if !self.is_on_with(evo) {
            return;
        }
        let half_w = self.width(evo) * 0.5;
        let dps = self.dps(player);
        for e in enemies.iter_mut() {
            if e.dead || e.y > player.y {
                continue;
            }
            if (e.x - self.beam_x).abs() < half_w + e.radius {
                let mut mul = 1.0;
                if player.perks.heat_lock && e.marked_until > t {
                    mul += 0.4;
                }
                let dmg = dps * mul * e.damage_mul() * dt;
                e.hp -= dmg;
                damage_acc[crate::entity::HitSource::Laser as usize] += dmg;
                if dmg > 0.0 {
                    e.hit_flash = 0.06;
                }
                e.last_hit = crate::entity::HitSource::Laser;
                // 偶发命中粒子
                if rand_chance(dt * 25.0) {
                    fx.burst(e.x, e.y, 2, 2.0, Color::from_rgba(125, 249, 255, 255), 80.0);
                }
            }
        }
    }

    fn draw(&self, player: &Player, t: f32, ox: f32, oy: f32) {
        let evo = player.perks.evo_laser;
        let x_center = if self.beam_x >= 0.0 {
            self.beam_x
        } else {
            player.x
        };
        let muzzle_y = player.y - player.h * 0.5;
        let top_y = 0.0;
        let beam_h = (muzzle_y - top_y).max(1.0);
        let x = x_center + ox;

        // —— OFF 充能态：渐次密集的下行能量点，临开火前最亮 ——
        if !self.is_on_with(evo) {
            let on_d = self.on_duty(evo);
            let off_phase =
                ((self.phase - on_d) / (1.0 - on_d)).clamp(0.0, 1.0);
            // charge: 0 = 刚 OFF, 1 = 即将开火
            let charge = off_phase;

            let mut guide = Color::from_rgba(125, 249, 255, 255);
            guide.a = 0.10 + 0.30 * charge;
            draw_line(x, top_y + oy, x, muzzle_y + oy, 1.0, guide);

            let n = 2 + (charge * 6.0) as usize;
            for i in 0..n {
                let prog = ((t * (1.4 + charge * 2.5) + i as f32 / n as f32) % 1.0).clamp(0.0, 1.0);
                let py = top_y + prog * beam_h;
                let mut pc = Color::from_rgba(200, 240, 255, 255);
                pc.a = 0.55 * charge;
                let r = 1.5 + charge * 2.0;
                draw_circle(x, py + oy, r * 1.8, pc);
                pc.a = 0.85 * charge;
                draw_circle(x, py + oy, r, pc);
            }
            return;
        }

        // —— ON 态：多层光柱 + 顶端冲击 + 流动能量 + muzzle ——
        let half_w = self.width(evo) * 0.5;
        let pulse = 0.88 + (t * 24.0).sin() * 0.12;
        // 整束随时间轻微"呼吸"——比纯 alpha 闪更接近真实激光
        let breath = 1.0 + (t * 6.0).sin() * 0.06;

        // 顶端冲击：光束打到屏幕外的发散圆斑（双层）
        let mut splash = Color::from_rgba(220, 250, 255, 255);
        splash.a = 0.30 * pulse;
        draw_circle(x, top_y + oy + 4.0, half_w * 3.0, splash);
        splash.a = 0.55 * pulse;
        draw_circle(x, top_y + oy + 4.0, half_w * 1.6, splash);
        splash.a = pulse;
        draw_circle(x, top_y + oy + 2.0, half_w * 0.6, splash);

        // 多层光柱：从外软到内硬，五层叠出 soft falloff
        // 元组：(宽度倍率, 颜色, alpha 倍率)
        let cyan = Color::from_rgba(125, 249, 255, 255);
        let bright = Color::from_rgba(210, 248, 255, 255);
        let bands: [(f32, Color, f32); 5] = [
            (3.4 * breath, cyan, 0.10),
            (2.3 * breath, cyan, 0.22),
            (1.55, cyan, 0.45),
            (1.0, bright, 0.85),
            (0.32, WHITE, 1.0),
        ];
        for (wmul, mut c, amul) in bands {
            c.a = amul * pulse;
            draw_rectangle(x - half_w * wmul, top_y + oy, half_w * wmul * 2.0, beam_h, c);
        }

        // 流动能量：从 muzzle 向 top 滚动的亮节，越接近顶端越大
        let pulses = 4 + self.level as usize;
        for i in 0..pulses {
            let prog = ((t * 1.6 + i as f32 / pulses as f32) % 1.0).clamp(0.0, 1.0);
            let py = muzzle_y - prog * beam_h;
            let envelope = (prog * std::f32::consts::PI).sin();
            let size = 2.5 + envelope * (3.0 + self.level as f32 * 0.4);
            let mut pc = WHITE;
            pc.a = 0.55 * pulse * envelope;
            draw_circle(x, py + oy, size * 1.8, pc);
            pc.a = 0.95 * pulse * envelope;
            draw_circle(x, py + oy, size, pc);
        }

        // muzzle 发射口：脉冲圆 + 纯白核
        let muzzle_pulse = 1.0 + (t * 30.0).sin() * 0.20;
        let mut m = Color::from_rgba(220, 250, 255, 255);
        m.a = 0.45 * pulse;
        draw_circle(x, muzzle_y + oy, half_w * 2.4 * muzzle_pulse, m);
        m.a = 0.85 * pulse;
        draw_circle(x, muzzle_y + oy, half_w * 1.3 * muzzle_pulse, m);
        m = WHITE;
        m.a = pulse;
        draw_circle(x, muzzle_y + oy, half_w * 0.55, m);
    }
}

fn laser_target_x(enemies: &[Enemy], player: &Player) -> Option<f32> {
    enemies
        .iter()
        .filter(|e| !e.dead && e.y <= player.y)
        .min_by(|a, b| {
            let ay = (player.y - a.y).abs();
            let by = (player.y - b.y).abs();
            ay.total_cmp(&by)
        })
        .map(|e| e.x)
}

fn rand_chance(p: f32) -> bool {
    use ::rand::{thread_rng, Rng};
    thread_rng().gen::<f32>() < p
}
