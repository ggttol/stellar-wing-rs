use macroquad::prelude::*;

use crate::config::CFG;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HitSource {
    MainGun,
    Missile,
    Drone,
    Laser,
    Chain,
    Rift,
    Wave,
    Reflector,
    Enemy,
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub w: f32,
    pub h: f32,
    pub damage: f32,
    pub pierce: u8,
    pub from_player: bool,
    pub dead: bool,
    pub homing: bool,
    pub is_crit: bool,
    pub source: HitSource,
    // Wave Cannon：正弦摆动
    pub spawn_x: f32,
    pub wave_amp: f32,
    pub wave_freq: f32,
    pub wave_phase: f32,
    // Reflector：反弹
    pub bounces: u8,
    // Prism：反射弹穿过激光束后的增益只触发一次
    pub prism_boosted: bool,
}

impl Bullet {
    pub fn player_shot(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            w: 4.0,
            h: 14.0,
            damage: 1.0,
            pierce: 0,
            from_player: true,
            dead: false,
            homing: false,
            is_crit: false,
            source: HitSource::MainGun,
            spawn_x: x,
            wave_amp: 0.0,
            wave_freq: 0.0,
            wave_phase: 0.0,
            bounces: 0,
            prism_boosted: false,
        }
    }

    pub fn enemy_shot(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            w: 5.0,
            h: 11.0,
            damage: 1.0,
            pierce: 0,
            from_player: false,
            dead: false,
            homing: false,
            is_crit: false,
            source: HitSource::Enemy,
            spawn_x: x,
            wave_amp: 0.0,
            wave_freq: 0.0,
            wave_phase: 0.0,
            bounces: 0,
            prism_boosted: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.y += self.vy * dt;

        // Wave Cannon / Weaver：摆动中心也要随 vx 推进，否则横向速度会被正弦覆盖。
        if self.wave_amp > 0.0 {
            self.spawn_x += self.vx * dt;
            self.wave_phase += dt * self.wave_freq;
            self.x = self.spawn_x + self.wave_amp * self.wave_phase.sin();
        } else {
            self.x += self.vx * dt;
        }

        // Reflector 屏幕边缘反弹
        if self.bounces > 0 {
            let mut bounced = false;
            if self.x <= 0.0 {
                self.x = 0.0;
                self.vx = self.vx.abs();
                bounced = true;
            }
            if self.x >= CFG.w {
                self.x = CFG.w;
                self.vx = -self.vx.abs();
                bounced = true;
            }
            if self.y <= 0.0 {
                self.y = 0.0;
                self.vy = self.vy.abs();
                bounced = true;
            }
            if self.y >= CFG.h {
                self.y = CFG.h;
                self.vy = -self.vy.abs();
                bounced = true;
            }
            if bounced {
                self.bounces -= 1;
                self.spawn_x = self.x; // 重设摆动中心
            }
            return; // 反弹弹丸不因越界销毁
        }

        // 普通弹丸越界销毁
        if self.x < -20.0 || self.x > CFG.w + 20.0 || self.y < -20.0 || self.y > CFG.h + 20.0 {
            self.dead = true;
        }
    }

    /// 子弹的“代表色”，外部（如 fx.trail）可以借这个值来生成同色拖尾。
    pub fn tint(&self) -> Color {
        if self.from_player {
            if self.is_crit {
                Color::from_rgba(255, 230, 90, 255)
            } else if matches!(self.source, HitSource::Wave) {
                Color::from_rgba(120, 255, 200, 255)
            } else if matches!(self.source, HitSource::Reflector) {
                Color::from_rgba(255, 255, 255, 255)
            } else if self.homing {
                Color::from_rgba(255, 200, 120, 255)
            } else {
                Color::from_rgba(155, 240, 255, 255)
            }
        } else if self.wave_amp > 0.0 {
            Color::from_rgba(92, 240, 210, 255)
        } else if self.h >= 18.0 {
            Color::from_rgba(255, 206, 96, 255)
        } else if self.w >= 12.0 {
            Color::from_rgba(255, 158, 76, 255)
        } else {
            Color::from_rgba(255, 85, 119, 255)
        }
    }

    pub fn draw(&self, ox: f32, oy: f32) {
        let c = self.tint();
        let sx = self.x + ox;
        let sy = self.y + oy;
        let enemy_wave = !self.from_player && self.wave_amp > 0.0;

        // 多层柔光：从外到内三层，模拟 emissive halo
        let glow_r = if self.is_crit {
            self.h * 0.95
        } else {
            self.h * 0.65
        };
        draw_circle(
            sx,
            sy,
            if enemy_wave { glow_r * 1.6 } else { glow_r * 1.5 },
            with_alpha(c, if self.is_crit { 0.18 } else { 0.10 }),
        );
        draw_circle(
            sx,
            sy,
            if enemy_wave { glow_r * 1.15 } else { glow_r },
            with_alpha(c, if self.is_crit { 0.45 } else { 0.30 }),
        );

        let w = if self.is_crit { self.w * 1.4 } else { self.w };
        if enemy_wave {
            draw_circle(sx, sy, self.w * 0.85, c);
            draw_circle(sx, sy - self.h * 0.36, self.w * 0.45, with_alpha(c, 0.65));
            draw_circle(sx, sy + self.h * 0.36, self.w * 0.45, with_alpha(c, 0.65));
        } else if !self.from_player && self.w >= 12.0 {
            draw_circle(sx, sy, self.w * 0.5, c);
            draw_circle_lines(sx, sy, self.w * 0.72, 2.0, with_alpha(c, 0.45));
            draw_circle_lines(sx, sy, self.w * 1.05, 1.5, with_alpha(c, 0.28));
        } else {
            // 实心条 + 顶端高亮，强化“激光柱”观感
            draw_rectangle(sx - w * 0.5, sy - self.h * 0.5, w, self.h, c);
            let tip_y = if self.from_player {
                sy - self.h * 0.5
            } else {
                sy + self.h * 0.5
            };
            draw_circle(sx, tip_y, w * 0.9, with_alpha(WHITE, 0.7));
        }
    }
}

fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflector_bounces_then_expires_after_leaving_bounds() {
        let mut bullet = Bullet::player_shot(1.0, 100.0, -100.0, 0.0);
        bullet.bounces = 1;

        bullet.update(0.05);
        assert_eq!(bullet.bounces, 0);
        assert!(bullet.vx > 0.0);
        assert!(!bullet.dead);

        bullet.x = -30.0;
        bullet.update(0.01);
        assert!(bullet.dead);
    }

    #[test]
    fn wave_bullet_tracks_spawn_center_with_sine_offset() {
        let mut bullet = Bullet::player_shot(120.0, 200.0, 20.0, -100.0);
        bullet.wave_amp = 40.0;
        bullet.wave_freq = 2.0;

        bullet.update(0.25);

        assert_eq!(bullet.spawn_x, 125.0);
        assert_ne!(bullet.x, bullet.spawn_x);
        assert!(bullet.y < 200.0);
    }
}
