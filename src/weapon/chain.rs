//! 闪电链：周期性命中最近敌人，闪电跳跃到下一个最近敌人，最多 N 跳。
//! 等级提升 → 跳数 + 伤害 + 频率。

use macroquad::prelude::*;

use crate::entity::{Bullet, Enemy, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

pub struct Chain {
    level: u8,
    last_shot: f32,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            level: 1,
            last_shot: -10.0,
        }
    }
    fn interval(&self) -> f32 {
        (1.65 - (self.level as f32 - 1.0) * 0.14).max(0.9)
    }
    fn jumps(&self) -> usize {
        1 + self.level as usize
    }
    fn range(&self) -> f32 {
        140.0 + self.level as f32 * 10.0
    }
    fn damage_mul(&self) -> f32 {
        1.45 + self.level as f32 * 0.28
    }
}

impl SubWeapon for Chain {
    fn id(&self) -> &'static str {
        "chain"
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
        enemies: &mut [Enemy],
        _bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
    ) {
        if t - self.last_shot < self.interval() {
            return;
        }
        self.last_shot = t;
        let max_jumps = self.jumps();
        let range = self.range();
        let color = Color::from_rgba(150, 220, 255, 255);

        let mut from = (player.x, player.y - player.h * 0.5);
        let mut hit: Vec<usize> = Vec::with_capacity(max_jumps);

        for _ in 0..max_jumps {
            let mut best: Option<usize> = None;
            let mut best_d2 = range * range;
            for (i, e) in enemies.iter().enumerate() {
                if e.dead || hit.contains(&i) {
                    continue;
                }
                let dx = e.x - from.0;
                let dy = e.y - from.1;
                let d2 = dx * dx + dy * dy;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best = Some(i);
                }
            }
            let Some(idx) = best else {
                break;
            };
            let (dmg, _crit) = roll_crit(player, self.damage_mul());
            let e = &mut enemies[idx];
            e.hp -= dmg;
            e.hit_flash = 0.08;
            e.last_hit = crate::entity::HitSource::Chain;
            // Resonance: Wave 标记触发 +2 额外跳（不消耗主跳数）
            let extra_jumps = if player.perks.resonance && e.wave_marked {
                e.wave_marked = false;
                2
            } else {
                0
            };
            if player.perks.static_mark {
                e.static_mark = true;
            }
            fx.bolt(from.0, from.1, e.x, e.y, color);
            fx.burst(e.x, e.y, 3, 2.0, color, 100.0);
            from = (e.x, e.y);
            hit.push(idx);

            // Resonance 额外跳
            for _ in 0..extra_jumps {
                let mut best2: Option<usize> = None;
                let mut best_d2 = range * range;
                for (j, ej) in enemies.iter().enumerate() {
                    if ej.dead || hit.contains(&j) {
                        continue;
                    }
                    let dx2 = ej.x - from.0;
                    let dy2 = ej.y - from.1;
                    let d2 = dx2 * dx2 + dy2 * dy2;
                    if d2 < best_d2 {
                        best_d2 = d2;
                        best2 = Some(j);
                    }
                }
                if let Some(jdx) = best2 {
                    let (bdmg, _) = roll_crit(player, self.damage_mul());
                    let ej = &mut enemies[jdx];
                    ej.hp -= bdmg;
                    ej.hit_flash = 0.08;
                    ej.last_hit = crate::entity::HitSource::Chain;
                    fx.bolt(from.0, from.1, ej.x, ej.y, Color::from_rgba(120, 255, 200, 255));
                    fx.burst(ej.x, ej.y, 2, 1.5, Color::from_rgba(120, 255, 200, 255), 80.0);
                    from = (ej.x, ej.y);
                    hit.push(jdx);
                }
            }
        }
    }

    fn draw(&self, _player: &Player, _t: f32, _ox: f32, _oy: f32) {}
}
