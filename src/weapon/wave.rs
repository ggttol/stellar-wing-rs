//! 波动炮：发射沿正弦波轨迹前进的弹丸，左右摆动覆盖横向区域。
//! 等级提升 → 波数 + 振幅 + 频率 + 射速。

use crate::entity::{Bullet, Enemy, HitSource, Player};
use crate::fx::Fx;
use crate::weapon::{roll_crit, SubWeapon};

pub struct WaveCannon {
    level: u8,
    last_shot: f32,
}

impl WaveCannon {
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
        (0.90 - (self.level as f32 - 1.0) * 0.075).max(0.60)
    }

    fn amplitude(&self) -> f32 {
        40.0 + self.level as f32 * 9.0
    }

    fn frequency(&self) -> f32 {
        1.5 + self.level as f32 * 0.2
    }
}

impl SubWeapon for WaveCannon {
    fn id(&self) -> &'static str {
        "wave"
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
        _damage_acc: &mut [f32; 9],
    ) {
        if t - self.last_shot < self.interval() {
            return;
        }
        self.last_shot = t;
        let evo = player.perks.evo_wave;
        let n = self.count() + if evo { 1 } else { 0 };
        let speed = player.stats.bullet_speed;
        let amp = self.amplitude() * if evo { 1.30 } else { 1.0 };
        let dmg_evo_mul = if evo { 1.30 } else { 1.0 };

        for i in 0..n {
            let off = (i as f32 - (n as f32 - 1.0) * 0.5) * 18.0;
            let x = player.x + off;
            let y = player.y - player.h * 0.5;

            let mut b = Bullet::player_shot(x, y, 0.0, -speed);
            let (dmg, crit) = roll_crit(player, (1.05 + self.level as f32 * 0.13) * dmg_evo_mul);
            b.damage = dmg;
            b.is_crit = crit;
            b.w = 5.0;
            b.h = 12.0;
            b.source = HitSource::Wave;
            b.spawn_x = x;
            b.wave_amp = amp;
            b.wave_freq = self.frequency();
            b.wave_phase = i as f32 * 0.8; // 错开相位
            bullets.push(b);
        }
    }

    fn draw(&self, _player: &Player, _t: f32, _ox: f32, _oy: f32) {}
}
