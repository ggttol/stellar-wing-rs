//! 跟踪导弹：发射后自动锁敌。等级提升 → 间隔缩短 + 数量增加。

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

pub struct Missile {
    level: u8,
    last_shot: f32,
}

impl Missile {
    pub fn new() -> Self {
        Self {
            level: 1,
            last_shot: -1.0,
        }
    }

    fn interval(&self) -> f32 {
        (1.7 - (self.level as f32 - 1.0) * 0.15).max(0.85)
    }

    fn count(&self) -> usize {
        match self.level {
            1 => 1,
            2 => 1,
            3 => 2,
            4 => 2,
            _ => 2,
        }
    }
}

impl SubWeapon for Missile {
    fn id(&self) -> &'static str {
        "missile"
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
        for i in 0..n {
            let off = (i as f32 - (n as f32 - 1.0) * 0.5) * 14.0;
            // 初始向上 + 轻微外扩
            let vx = if n > 1 { off * 4.0 } else { 0.0 };
            let mut b = Bullet::player_shot(player.x + off, player.y - player.h * 0.5, vx, -260.0);
            let (dmg, crit) = roll_crit(player, 1.50);
            b.damage = dmg;
            b.is_crit = crit;
            b.homing = true;
            b.w = 6.0;
            b.h = 12.0;
            b.source = HitSource::Missile;
            bullets.push(b);
        }
    }

    fn draw(&self, _player: &Player, _t: f32, _ox: f32, _oy: f32) {}
}
