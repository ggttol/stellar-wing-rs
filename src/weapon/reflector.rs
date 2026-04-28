//! 反射镜：发射碰到屏幕边缘会反弹的弹丸，在封闭空间中制造复杂弹道路径。
//! 等级提升 → 弹数 + 反弹次数 + 射速。

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

pub struct Reflector {
    level: u8,
    last_shot: f32,
}

impl Reflector {
    pub fn new() -> Self {
        Self {
            level: 1,
            last_shot: -10.0,
        }
    }

    fn count(&self) -> usize {
        match self.level {
            1 => 1,
            2 => 1,
            3 => 2,
            4 => 2,
            _ => 3,
        }
    }

    fn interval(&self) -> f32 {
        (1.10 - (self.level as f32 - 1.0) * 0.09).max(0.75)
    }

    fn bounces(&self) -> u8 {
        match self.level {
            1 => 2,
            2 => 3,
            3 => 3,
            4 => 4,
            _ => 4,
        }
    }
}

impl SubWeapon for Reflector {
    fn id(&self) -> &'static str {
        "reflector"
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
        _dt: f32,
        t: f32,
        player: &Player,
        _enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        _fx: &mut Fx,
    ) {
        if t - self.last_shot < self.interval() {
            return;
        }
        self.last_shot = t;
        let n = self.count();
        let speed = 340.0; // 较慢的弹速，给反弹留观察空间
        let bounces = self.bounces();

        for i in 0..n {
            // 扇形发射，角度越大反弹路径越丰富
            let angles: [f32; 3] = [-0.30, 0.0, 0.30];
            let ang = if n <= 1 {
                0.0
            } else {
                angles[i.min(2)]
            };
            let vx = ang.sin() * speed * 1.5;
            let vy = -ang.cos() * speed;

            let mut b = Bullet::player_shot(
                player.x,
                player.y - player.h * 0.5,
                vx,
                vy,
            );
            let (dmg, crit) = roll_crit(player, 0.85 + self.level as f32 * 0.12);
            b.damage = dmg;
            b.is_crit = crit;
            b.w = 5.0;
            b.h = 5.0;
            b.source = HitSource::Reflector;
            b.bounces = bounces;
            bullets.push(b);
        }
    }

    fn draw(&self, _player: &Player, _t: f32, _ox: f32, _oy: f32) {}
}
