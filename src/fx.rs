//! 粒子 / 浮动文字 / 拖尾 / 冲击波 / 屏幕震动 / 闪屏 / 时间冻结。
//!
//! 这里既存“当前帧的视觉数据”（particles / shockwaves / texts / bolts），
//! 也存“当前帧的全局视觉状态”（shake / damage_flash / overload_flash / time_freeze /
//! slow_mo），主循环按这些状态去缩放 dt、加屏闪、做 vignette 等。

use macroquad::prelude::*;

#[derive(Default)]
pub struct Fx {
    pub particles: Vec<Particle>,
    pub trails: Vec<TrailDot>,
    pub shocks: Vec<Shock>,
    pub texts: Vec<FloatText>,
    pub bolts: Vec<Bolt>,
    pub shake: f32,
    /// 受击屏闪：剩余强度（0..1）
    pub damage_flash: f32,
    /// 命中冻帧剩余秒数：>0 时主循环对玩法 dt 设为 0
    pub time_freeze: f32,
    /// 慢动作剩余秒数：>0 时主循环对玩法 dt 乘 slow_mo_scale
    pub slow_mo: f32,
    /// 慢动作时的速度比例（0.25 = 25% 速度）
    pub slow_mo_scale: f32,
}

pub struct Bolt {
    /// 锯齿折线点列；至少 2 点，绘制成多段直线
    pub points: Vec<(f32, f32)>,
    pub color: Color,
    pub life: f32,
    /// 主线宽
    pub width: f32,
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

/// 拖尾点：固定位置、按 life 收缩 + 淡出。
pub struct TrailDot {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: Color,
    pub life: f32,
    pub decay: f32,
}

/// 冲击波环：一个圆环从 r0 扩张到 r1，同时淡出。
pub struct Shock {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub r_growth: f32,
    pub thickness: f32,
    pub color: Color,
    pub life: f32,
    pub decay: f32,
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

    /// 多层冲击波 + 粒子 + 余烬 + 屏幕震动。
    pub fn explode(&mut self, x: f32, y: f32, scale: f32, color: Color) {
        // 高速彩色粒子
        self.burst(
            x,
            y,
            (22.0 * scale) as usize,
            3.0 * scale,
            color,
            240.0 * scale,
        );
        // 中速白色芯
        self.burst(
            x,
            y,
            (10.0 * scale) as usize,
            5.0 * scale,
            WHITE,
            150.0 * scale,
        );
        // 慢速余烬（同色暖色调，慢慢衰减给“烟”的感觉）
        self.burst(
            x,
            y,
            (8.0 * scale) as usize,
            2.0 * scale,
            with_alpha(color, 0.5),
            70.0 * scale,
        );
        // 双层冲击波环：外大快、内小慢
        self.shocks.push(Shock {
            x,
            y,
            r: 8.0 * scale,
            r_growth: 360.0 * scale,
            thickness: 2.5,
            color: WHITE,
            life: 1.0,
            decay: 2.4,
        });
        self.shocks.push(Shock {
            x,
            y,
            r: 4.0 * scale,
            r_growth: 220.0 * scale,
            thickness: 4.0,
            color,
            life: 1.0,
            decay: 1.6,
        });
        self.shake = self.shake.max(7.0 * scale);
    }

    /// 大爆炸（Boss 死亡 / 大型敌人）：更夸张的冲击波 + 慢动作 + 长震动。
    pub fn explode_big(&mut self, x: f32, y: f32, scale: f32, color: Color) {
        self.explode(x, y, scale, color);
        self.shocks.push(Shock {
            x,
            y,
            r: 12.0 * scale,
            r_growth: 540.0 * scale,
            thickness: 3.0,
            color: with_alpha(color, 0.85),
            life: 1.0,
            decay: 1.0,
        });
        self.shake = self.shake.max(14.0 * scale);
    }

    /// 命中冻帧：短暂停顿玩法时间，让重击有“咬合感”。
    pub fn hit_pause(&mut self, secs: f32) {
        if secs > self.time_freeze {
            self.time_freeze = secs;
        }
    }

    /// 慢动作：在 secs 时间内把玩法时间缩到 scale。
    pub fn request_slow_mo(&mut self, secs: f32, scale: f32) {
        if secs > self.slow_mo {
            self.slow_mo = secs;
            self.slow_mo_scale = scale;
        }
    }

    pub fn damage_flash(&mut self, intensity: f32) {
        if intensity > self.damage_flash {
            self.damage_flash = intensity.clamp(0.0, 1.0);
        }
    }

    pub fn shock_ring(&mut self, x: f32, y: f32, color: Color, scale: f32) {
        self.shocks.push(Shock {
            x,
            y,
            r: 4.0 * scale,
            r_growth: 240.0 * scale,
            thickness: 2.0,
            color,
            life: 1.0,
            decay: 2.2,
        });
    }

    pub fn trail(&mut self, x: f32, y: f32, size: f32, color: Color, decay: f32) {
        self.trails.push(TrailDot {
            x,
            y,
            size,
            color,
            life: 1.0,
            decay,
        });
    }

    pub fn bolt(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        // 生成锯齿折线：长度越大段数越多，中段抖动幅度最大（正弦包络）
        use ::rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let segs = (4 + (len / 28.0) as usize).clamp(4, 14);
        // 单位法线（垂直方向）
        let nx = -dy / len;
        let ny = dx / len;
        let amp = (len * 0.08).clamp(6.0, 22.0);
        let mut points = Vec::with_capacity(segs + 1);
        for i in 0..=segs {
            let t = i as f32 / segs as f32;
            let cx = x1 + dx * t;
            let cy = y1 + dy * t;
            let envelope = (t * std::f32::consts::PI).sin();
            let jitter = if i == 0 || i == segs {
                0.0
            } else {
                rng.gen_range(-amp..amp) * envelope
            };
            points.push((cx + nx * jitter, cy + ny * jitter));
        }
        self.bolts.push(Bolt {
            points,
            color,
            life: 1.0,
            width: 2.5,
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
            // 轻微减速，模拟阻力
            p.vx *= (1.0 - 0.6 * dt).max(0.0);
            p.vy *= (1.0 - 0.6 * dt).max(0.0);
            p.life -= p.decay * dt;
            p.life > 0.0
        });
        self.trails.retain_mut(|t| {
            t.life -= t.decay * dt;
            t.life > 0.0
        });
        self.shocks.retain_mut(|s| {
            s.r += s.r_growth * dt;
            s.life -= s.decay * dt;
            s.life > 0.0
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
        if self.damage_flash > 0.0 {
            self.damage_flash = (self.damage_flash - 2.4 * dt).max(0.0);
        }
        // time_freeze / slow_mo 的递减由 main.rs 在缩放完 dt 之后调用 tick_time_modifiers 处理，
        // 因为它们影响的是 *玩法* dt，不是自身寿命。
    }

    /// 主循环每帧调用，按 *真实* dt 减少时间修饰符。
    /// 返回这一帧应作用于玩法的 dt。
    pub fn tick_time_modifiers(&mut self, real_dt: f32) -> f32 {
        if self.time_freeze > 0.0 {
            self.time_freeze = (self.time_freeze - real_dt).max(0.0);
            return 0.0;
        }
        if self.slow_mo > 0.0 {
            self.slow_mo = (self.slow_mo - real_dt).max(0.0);
            return real_dt * self.slow_mo_scale;
        }
        real_dt
    }

    pub fn draw(&self) {
        // 拖尾在最底下：双层（外圈大软、内圈小亮）
        for t in &self.trails {
            let life = t.life.clamp(0.0, 1.0);
            let mut outer = t.color;
            outer.a = life * 0.30;
            draw_circle(t.x, t.y, t.size * 1.8, outer);
            let mut inner = t.color;
            inner.a = life * 0.85;
            draw_circle(t.x, t.y, t.size, inner);
        }
        // 冲击波环：双笔实现“描边发光”
        for s in &self.shocks {
            let life = s.life.clamp(0.0, 1.0);
            let mut outer = s.color;
            outer.a = life * 0.35;
            draw_circle_lines(s.x, s.y, s.r, s.thickness * 2.4, outer);
            let mut inner = s.color;
            inner.a = life * 0.85;
            draw_circle_lines(s.x, s.y, s.r, s.thickness, inner);
        }
        // 粒子：双层 halo
        for p in &self.particles {
            let life = p.life.clamp(0.0, 1.0);
            let mut halo = p.color;
            halo.a = life * 0.35;
            draw_circle(p.x, p.y, p.size * 2.2, halo);
            let mut core = p.color;
            core.a = life;
            draw_circle(p.x, p.y, p.size, core);
        }
        for b in &self.bolts {
            if b.points.len() < 2 {
                continue;
            }
            let life = b.life.clamp(0.0, 1.0);
            // 三层叠加：外软辉光 → 中等光晕 → 主线 + 起止处亮点
            for window in b.points.windows(2) {
                let (x1, y1) = window[0];
                let (x2, y2) = window[1];
                let mut g = b.color;
                g.a = life * 0.18;
                draw_line(x1, y1, x2, y2, b.width * 4.0, g);
                g.a = life * 0.40;
                draw_line(x1, y1, x2, y2, b.width * 2.2, g);
                let mut c = b.color;
                c.a = life;
                draw_line(x1, y1, x2, y2, b.width, c);
                // 内芯几乎纯白，让闪电有"灼热感"
                let mut hot = WHITE;
                hot.a = life * 0.85;
                draw_line(x1, y1, x2, y2, b.width * 0.4, hot);
            }
            // 折点亮斑
            for (px, py) in &b.points {
                let mut sp = b.color;
                sp.a = life * 0.55;
                draw_circle(*px, *py, b.width * 1.2, sp);
            }
        }
        for t in &self.texts {
            let mut c = t.color;
            c.a = t.life.clamp(0.0, 1.0);
            // 文字双层投影：黑色阴影 + 主体
            let mut shadow = BLACK;
            shadow.a = c.a * 0.6;
            draw_text(&t.text, t.x + 1.0, t.y + 1.0, t.size, shadow);
            draw_text(&t.text, t.x, t.y, t.size, c);
        }
    }
}

fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}
