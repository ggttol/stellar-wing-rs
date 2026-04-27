use ::rand::{thread_rng, Rng};
use macroquad::prelude::*;

use crate::art::draw_player_ship;
use crate::config::CFG;
use crate::fx::{Fx, Particle};
use crate::ship::ShipType;

#[allow(dead_code)] // damage_mul/invincible 在 M3 接入
pub struct PlayerStats {
    pub speed: f32,        // 像素/秒²的加速度（与 friction 配合）
    pub friction: f32,     // 0..1，每帧速度衰减系数（按 dt 归一化处理）
    pub fire_rate: f32,    // 射击间隔（秒）
    pub bullet_speed: f32, // 子弹速度（像素/秒）
    pub damage_mul: f32,
    pub max_lives: u8,
    pub invincible: f32,     // 受击无敌时长（秒）
    pub pickup_radius: f32,  // 实际拾取半径（碰到才吃）
    pub attract_radius: f32, // 吸附半径（进入后 gem 朝玩家加速）
    pub crit_chance: f32,    // [0,1]
    pub crit_mul: f32,       // 暴击倍率
    pub score_mul: f32,
    pub xp_mul: f32,
    pub regen_per_min: f32, // 每分钟恢复多少 HP
}

#[derive(Default)]
pub struct CombatPerks {
    pub heat_lock: bool,
    pub static_mark: bool,
    pub drone_relay: bool,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            speed: 1500.0,
            friction: 0.86,
            fire_rate: 0.30,
            bullet_speed: 800.0,
            damage_mul: 1.0,
            max_lives: 3,
            invincible: 1.5,
            pickup_radius: 22.0,
            attract_radius: 90.0,
            crit_chance: 0.0,
            crit_mul: 2.0,
            score_mul: 1.0,
            xp_mul: 1.0,
            regen_per_min: 0.0,
        }
    }
}

#[allow(dead_code)] // dead 字段在 M3 接入死亡判定
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub lives: u8,
    pub shield: bool,
    pub invincible_until: f32,
    pub regen_acc: f32,
    pub magnet_until: f32,
    pub perks: CombatPerks,
    pub ship: ShipType,
    pub stats: PlayerStats,
    pub dead: bool,
}

impl Player {
    pub fn with_ship(ship: ShipType) -> Self {
        let stats = PlayerStats::default();
        Self {
            x: CFG.w * 0.5,
            y: CFG.h - 100.0,
            vx: 0.0,
            vy: 0.0,
            w: 42.0,
            h: 48.0,
            radius: 18.0,
            lives: stats.max_lives,
            shield: false,
            invincible_until: 0.0,
            regen_acc: 0.0,
            magnet_until: 0.0,
            perks: CombatPerks::default(),
            ship,
            stats,
            dead: false,
        }
    }

    pub fn attract_radius_at(&self, t: f32) -> f32 {
        if t < self.magnet_until {
            self.stats.attract_radius * 2.4
        } else {
            self.stats.attract_radius
        }
    }

    pub fn update(&mut self, dt: f32, _t: f32, fx: &mut Fx) {
        // HP 缓回（每分钟）
        if self.stats.regen_per_min > 0.0 && self.lives < self.stats.max_lives {
            self.regen_acc += dt * self.stats.regen_per_min / 60.0;
            if self.regen_acc >= 1.0 {
                self.regen_acc -= 1.0;
                self.lives = (self.lives + 1).min(self.stats.max_lives);
            }
        }
        // 输入
        let mut ax = 0.0_f32;
        let mut ay = 0.0_f32;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            ax -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            ax += 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            ay -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            ay += 1.0;
        }
        if ax != 0.0 || ay != 0.0 {
            let len = (ax * ax + ay * ay).sqrt();
            self.vx += ax / len * self.stats.speed * dt;
            self.vy += ay / len * self.stats.speed * dt;
        }

        // 摩擦：把每帧基于 60FPS 的衰减系数转成连续衰减
        let frame_decay = self.stats.friction.powf(dt * 60.0);
        self.vx *= frame_decay;
        self.vy *= frame_decay;

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // 边界
        let m = self.w * 0.5;
        if self.x < m {
            self.x = m;
            self.vx = 0.0;
        }
        if self.x > CFG.w - m {
            self.x = CFG.w - m;
            self.vx = 0.0;
        }
        if self.y < self.h * 0.5 {
            self.y = self.h * 0.5;
            self.vy = 0.0;
        }
        if self.y > CFG.h - self.h * 0.5 {
            self.y = CFG.h - self.h * 0.5;
            self.vy = 0.0;
        }

        // 推进器粒子
        let mut rng = thread_rng();
        for off in [-6.0, 6.0] {
            fx.particles.push(Particle {
                x: self.x + off,
                y: self.y + self.h * 0.5 - 4.0,
                vx: rng.gen_range(-12.0..12.0),
                vy: rng.gen_range(60.0..120.0),
                life: 1.0,
                decay: 4.0,
                size: 2.5,
                color: if off < 0.0 {
                    Color::from_rgba(125, 249, 255, 255)
                } else {
                    Color::from_rgba(0, 212, 255, 255)
                },
            });
        }
    }

    /// 受击。返回是否真正掉血（false 表示无敌或被护盾挡住）。
    pub fn hit(&mut self, t: f32) -> bool {
        if t < self.invincible_until {
            return false;
        }
        if self.shield {
            self.shield = false;
            self.invincible_until = t + 0.5;
            return false;
        }
        self.lives = self.lives.saturating_sub(1);
        self.invincible_until = t + self.stats.invincible;
        if self.lives == 0 {
            self.dead = true;
        }
        true
    }

    pub fn draw(&self, t: f32) {
        // 受击闪烁
        if t < self.invincible_until && ((t * 20.0) as i32) % 2 == 0 {
            return;
        }

        let cx = self.x;
        let cy = self.y;
        let w = self.w;
        let h = self.h;

        // 推进器尾焰
        let flame_color = Color::from_rgba(125, 249, 255, 255);
        let mut glow = flame_color;
        glow.a = 0.3;
        draw_circle(cx, cy + h * 0.5, 14.0, glow);
        let flicker = (t * 30.0).sin() * 2.0;
        draw_triangle(
            vec2(cx - 7.0, cy + h * 0.5 - 6.0),
            vec2(cx + 7.0, cy + h * 0.5 - 6.0),
            vec2(cx, cy + h * 0.5 + 8.0 + flicker),
            flame_color,
        );

        draw_player_ship(self.ship, cx, cy, w, h, t);

        // 护盾
        if self.shield {
            let pulse = 0.6 + (t * 6.0).sin() * 0.2;
            let mut sc = Color::from_rgba(77, 210, 255, 255);
            sc.a = pulse;
            draw_circle_lines(cx, cy, self.radius + 8.0, 2.0, sc);
        }
    }
}
