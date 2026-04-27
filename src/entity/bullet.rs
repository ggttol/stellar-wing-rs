use macroquad::prelude::*;

use crate::config::CFG;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HitSource {
    MainGun,
    Missile,
    Drone,
    Laser,
    Chain,
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        if self.x < -20.0 || self.x > CFG.w + 20.0 || self.y < -20.0 || self.y > CFG.h + 20.0 {
            self.dead = true;
        }
    }

    pub fn draw(&self) {
        let c = if self.from_player {
            if self.is_crit {
                Color::from_rgba(255, 230, 90, 255) // 暴击：金黄
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
        draw_circle(self.x, self.y, glow_r, g);
        let w = if self.is_crit { self.w * 1.4 } else { self.w };
        draw_rectangle(self.x - w * 0.5, self.y - self.h * 0.5, w, self.h, c);
    }
}
