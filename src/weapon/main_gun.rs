//! 主武器：5 级
//!  Lv1 单发  ·  Lv2 双发  ·  Lv3 三向扇形  ·  Lv4 五向  ·  Lv5 五向 + 穿透 2

use crate::entity::{Bullet, HitSource, Player};
use crate::weapon::roll_crit;

pub struct MainGun {
    pub level: u8,
    last_shot: f32,
}

impl MainGun {
    pub fn new() -> Self {
        Self {
            level: 1,
            last_shot: -10.0,
        }
    }

    pub fn level_up(&mut self) {
        if self.level < 5 {
            self.level += 1;
        }
    }

    pub fn is_max(&self) -> bool {
        self.level >= 5
    }

    pub fn tick(&mut self, t: f32, player: &Player, bullets: &mut Vec<Bullet>) -> bool {
        if t - self.last_shot < player.stats.fire_rate {
            return false;
        }
        self.last_shot = t;
        let speed = player.stats.bullet_speed;
        let pierce = if self.level >= 5 { 2 } else { 0 };
        let x = player.x;
        let y = player.y - player.h * 0.5;

        match self.level {
            1 => emit(bullets, player, x, y, 0.0, -speed, pierce),
            2 => {
                emit(bullets, player, x - 10.0, y, 0.0, -speed, pierce);
                emit(bullets, player, x + 10.0, y, 0.0, -speed, pierce);
            }
            3 => {
                emit(bullets, player, x, y, 0.0, -speed, pierce);
                let a = 0.20_f32;
                emit(
                    bullets,
                    player,
                    x - 6.0,
                    y,
                    -a.sin() * speed,
                    -a.cos() * speed,
                    pierce,
                );
                emit(
                    bullets,
                    player,
                    x + 6.0,
                    y,
                    a.sin() * speed,
                    -a.cos() * speed,
                    pierce,
                );
            }
            _ => {
                emit(bullets, player, x, y, 0.0, -speed, pierce);
                let a1 = 0.18_f32;
                let a2 = 0.36_f32;
                emit(
                    bullets,
                    player,
                    x - 5.0,
                    y,
                    -a1.sin() * speed,
                    -a1.cos() * speed,
                    pierce,
                );
                emit(
                    bullets,
                    player,
                    x + 5.0,
                    y,
                    a1.sin() * speed,
                    -a1.cos() * speed,
                    pierce,
                );
                emit(
                    bullets,
                    player,
                    x - 10.0,
                    y + 4.0,
                    -a2.sin() * speed,
                    -a2.cos() * speed,
                    pierce,
                );
                emit(
                    bullets,
                    player,
                    x + 10.0,
                    y + 4.0,
                    a2.sin() * speed,
                    -a2.cos() * speed,
                    pierce,
                );
            }
        }

        true
    }
}

fn emit(bullets: &mut Vec<Bullet>, player: &Player, x: f32, y: f32, vx: f32, vy: f32, pierce: u8) {
    let mut b = Bullet::player_shot(x, y, vx, vy);
    let (dmg, crit) = roll_crit(player, 1.0);
    b.damage = dmg;
    b.is_crit = crit;
    b.pierce = pierce;
    b.source = HitSource::MainGun;
    bullets.push(b);
}
