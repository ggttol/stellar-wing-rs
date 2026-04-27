//! 粒子 / 浮动文字 / 屏幕震动。M1 只搭壳，M2 起逐步填充。

use macroquad::prelude::*;

#[derive(Default)]
pub struct Fx {
    pub particles: Vec<Particle>,
    pub texts: Vec<FloatText>,
    pub bolts: Vec<Bolt>,
    pub shake: f32,
}

pub struct Bolt {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub color: Color,
    pub life: f32,
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub decay: f32,
    pub size: f32,
    pub color: Color,
}

pub struct FloatText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub size: f32,
    pub life: f32,
}

impl Fx {
    pub fn burst(&mut self, x: f32, y: f32, n: usize, size: f32, color: Color, speed: f32) {
        use ::rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        for _ in 0..n {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let s = speed * rng.gen_range(0.4..1.2);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * s,
                vy: a.sin() * s,
                life: 1.0,
                decay: rng.gen_range(0.9..1.6),
                size: size * rng.gen_range(0.6..1.2),
                color,
            });
        }
    }

    pub fn explode(&mut self, x: f32, y: f32, scale: f32, color: Color) {
        self.burst(
            x,
            y,
            (20.0 * scale) as usize,
            3.0 * scale,
            color,
            220.0 * scale,
        );
        self.burst(
            x,
            y,
            (10.0 * scale) as usize,
            5.0 * scale,
            WHITE,
            140.0 * scale,
        );
        self.shake = self.shake.max(6.0 * scale);
    }

    pub fn bolt(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        self.bolts.push(Bolt {
            x1,
            y1,
            x2,
            y2,
            color,
            life: 1.0,
        });
    }

    pub fn float_text(&mut self, x: f32, y: f32, text: impl Into<String>, color: Color, size: f32) {
        self.texts.push(FloatText {
            x,
            y,
            text: text.into(),
            color,
            size,
            life: 1.0,
        });
    }

    pub fn update(&mut self, dt: f32) {
        self.particles.retain_mut(|p| {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= p.decay * dt;
            p.life > 0.0
        });
        self.texts.retain_mut(|t| {
            t.y -= 30.0 * dt;
            t.life -= 0.6 * dt;
            t.life > 0.0
        });
        self.bolts.retain_mut(|b| {
            b.life -= 6.0 * dt;
            b.life > 0.0
        });
        if self.shake > 0.0 {
            self.shake = (self.shake - 18.0 * dt).max(0.0);
        }
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let mut c = p.color;
            c.a = p.life.clamp(0.0, 1.0);
            draw_circle(p.x, p.y, p.size, c);
        }
        for b in &self.bolts {
            let mut c = b.color;
            c.a = b.life.clamp(0.0, 1.0);
            draw_line(b.x1, b.y1, b.x2, b.y2, 2.5, c);
            // 外侧辉光
            let mut g = c;
            g.a = c.a * 0.4;
            draw_line(b.x1, b.y1, b.x2, b.y2, 6.0, g);
        }
        for t in &self.texts {
            let mut c = t.color;
            c.a = t.life.clamp(0.0, 1.0);
            draw_text(&t.text, t.x, t.y, t.size, c);
        }
    }
}
