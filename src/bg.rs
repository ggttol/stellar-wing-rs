//! 多层视差星空。等价于原 shooter.html 的 BG 模块。

use crate::config::CFG;
use ::rand::{thread_rng, Rng};
use macroquad::prelude::*;

pub struct Star {
    pub x: f32,
    pub y: f32,
    /// 深度系数 [0.3, 1.0)，影响下落速度与亮度
    pub z: f32,
    pub r: f32,
    pub tw: f32,
}

pub struct StarField {
    pub stars: Vec<Star>,
}

impl StarField {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let stars = (0..120)
            .map(|_| Star {
                x: rng.gen_range(0.0..CFG.w),
                y: rng.gen_range(0.0..CFG.h),
                z: rng.gen_range(0.3..1.0),
                r: rng.gen_range(0.4..1.9),
                tw: rng.gen_range(0.0..std::f32::consts::TAU),
            })
            .collect();
        Self { stars }
    }

    pub fn update(&mut self, dt: f32) {
        let mut rng = thread_rng();
        for s in &mut self.stars {
            s.y += s.z * 60.0 * dt; // 60px/s @ z=1
            s.tw += 1.8 * dt;
            if s.y > CFG.h {
                s.y = -2.0;
                s.x = rng.gen_range(0.0..CFG.w);
            }
        }
    }

    pub fn draw(&self) {
        // 顶部偏蓝、底部更暗的渐变
        draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::from_rgba(2, 3, 10, 255));
        draw_rectangle(
            0.0,
            CFG.h * 0.25,
            CFG.w,
            CFG.h * 0.5,
            Color::from_rgba(6, 9, 26, 255),
        );

        for s in &self.stars {
            let a = (0.4 + s.tw.sin() * 0.4) * s.z;
            let c = Color::new(0.70, 0.86, 1.0, a.clamp(0.0, 1.0));
            draw_circle(s.x, s.y, s.r, c);
        }
    }
}
