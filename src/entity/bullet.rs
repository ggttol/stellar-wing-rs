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
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Wave Cannon 正弦摆动
        if self.wave_amp > 0.0 {
            self.wave_phase += dt * self.wave_freq;
            self.x = self.spawn_x + self.wave_amp * self.wave_phase.sin();
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

    pub fn draw(&self, ox: f32, oy: f32) {
        let c = if self.from_player {
            if self.is_crit {
                Color::from_rgba(255, 230, 90, 255)
            } else if matches!(self.source, HitSource::Wave) {
                Color::from_rgba(120, 255, 200, 255) // Wave: 青绿
            } else if matches!(self.source, HitSource::Reflector) {
                Color::from_rgba(255, 255, 255, 255) // Reflector: 亮白
            } else if self.homing {
                Color::from_rgba(255, 200, 120, 255)
            } else {
                Color::from_rgba(155, 240, 255, 255)
            }
        } else {
            Color::from_rgba(255, 85, 119, 255)
        };
        let mut g = c;
        g.a = if self.is_crit { 0.6 } else { 0.35 };
        let glow_r = if self.is_crit {
            self.h * 0.85
        } else {
            self.h * 0.55
        };
        let sx = self.x + ox;
        let sy = self.y + oy;
        draw_circle(sx, sy, glow_r, g);
        let w = if self.is_crit { self.w * 1.4 } else { self.w };
        draw_rectangle(sx - w * 0.5, sy - self.h * 0.5, w, self.h, c);
    }
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
        let mut bullet = Bullet::player_shot(120.0, 200.0, 0.0, -100.0);
        bullet.wave_amp = 40.0;
        bullet.wave_freq = 2.0;

        bullet.update(0.25);

        assert_eq!(bullet.spawn_x, 120.0);
        assert_ne!(bullet.x, 120.0);
        assert!(bullet.y < 200.0);
    }
}
