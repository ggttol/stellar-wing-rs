use macroquad::prelude::*;

use crate::config::CFG;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    Xp,
    Heal,
    Magnet,
    Ammo,
    Barrier,
}

pub struct Pickup {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: PickupKind,
    pub value: u32,
    pub spin: f32,
    pub dead: bool,
}

impl Pickup {
    pub fn xp(x: f32, y: f32, value: u32) -> Self {
        // 初始向上小幅飘起，再下落，被吸附前像礼物盒
        Self {
            x,
            y,
            vx: 0.0,
            vy: -40.0,
            kind: PickupKind::Xp,
            value,
            spin: 0.0,
            dead: false,
        }
    }

    pub fn special(x: f32, y: f32, kind: PickupKind) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: -20.0,
            kind,
            value: 1,
            spin: 0.0,
            dead: false,
        }
    }

    /// 朝玩家吸附 + 自由下落。返回是否被玩家拾取。
    pub fn update(
        &mut self,
        dt: f32,
        player_x: f32,
        player_y: f32,
        attract_r: f32,
        pickup_r: f32,
    ) -> bool {
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let d2 = dx * dx + dy * dy;
        let pr = pickup_r * pickup_r;
        if d2 < pr {
            return true;
        }

        let ar = attract_r * attract_r;
        if d2 < ar {
            // 吸附：朝玩家加速
            let d = d2.sqrt().max(0.001);
            let pull = 900.0 * dt;
            self.vx += dx / d * pull;
            self.vy += dy / d * pull;
            // 限速
            let v2 = self.vx * self.vx + self.vy * self.vy;
            let max = 600.0;
            if v2 > max * max {
                let v = v2.sqrt();
                self.vx = self.vx / v * max;
                self.vy = self.vy / v * max;
            }
        } else {
            // 自由漂浮：略微下沉，但不会很快滑出屏幕
            self.vy += 30.0 * dt;
            self.vy = self.vy.clamp(-50.0, 45.0);
            self.vx *= 0.98;
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.spin += dt * 6.0;

        self.x = self.x.clamp(18.0, CFG.w - 18.0);
        self.y = self.y.clamp(18.0, CFG.h - 18.0);
        false
    }

    pub fn draw(&self, t: f32) {
        let c = match self.kind {
            PickupKind::Xp => match self.value {
                0..=2 => Color::from_rgba(125, 249, 255, 255),
                3..=6 => Color::from_rgba(190, 140, 255, 255),
                _ => Color::from_rgba(255, 209, 102, 255),
            },
            PickupKind::Heal => Color::from_rgba(118, 255, 122, 255),
            PickupKind::Magnet => Color::from_rgba(255, 120, 210, 255),
            PickupKind::Ammo => Color::from_rgba(255, 180, 80, 255),
            PickupKind::Barrier => Color::from_rgba(125, 200, 255, 255),
        };
        let mut g = c;
        g.a = 0.35;
        let pulse = 1.0 + (t * 6.0 + self.spin).sin() * 0.15;
        draw_circle(self.x, self.y, 8.0 * pulse, g);
        match self.kind {
            PickupKind::Xp => {
                let s = 5.0;
                draw_triangle(
                    vec2(self.x, self.y - s),
                    vec2(self.x + s, self.y),
                    vec2(self.x - s, self.y),
                    c,
                );
                draw_triangle(
                    vec2(self.x, self.y + s),
                    vec2(self.x + s, self.y),
                    vec2(self.x - s, self.y),
                    c,
                );
            }
            PickupKind::Heal => {
                draw_rectangle(self.x - 2.0, self.y - 6.0, 4.0, 12.0, c);
                draw_rectangle(self.x - 6.0, self.y - 2.0, 12.0, 4.0, c);
            }
            PickupKind::Magnet => {
                draw_circle_lines(self.x, self.y, 6.0, 2.0, c);
                draw_line(
                    self.x - 5.0,
                    self.y - 6.0,
                    self.x - 1.0,
                    self.y + 2.0,
                    2.0,
                    c,
                );
                draw_line(
                    self.x + 5.0,
                    self.y - 6.0,
                    self.x + 1.0,
                    self.y + 2.0,
                    2.0,
                    c,
                );
            }
            PickupKind::Ammo => {
                draw_circle(self.x, self.y, 5.0, c);
                draw_triangle(
                    vec2(self.x, self.y - 8.0),
                    vec2(self.x + 4.0, self.y - 1.0),
                    vec2(self.x - 4.0, self.y - 1.0),
                    c,
                );
            }
            PickupKind::Barrier => {
                draw_circle_lines(self.x, self.y, 7.0, 2.0, c);
                draw_circle(self.x, self.y, 3.0, WHITE);
            }
        }
    }
}
