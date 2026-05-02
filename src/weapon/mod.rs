pub mod chain;
pub mod drone;
pub mod laser;
pub mod main_gun;
pub mod missile;
pub mod reflector;
pub mod rift;
pub mod wave;

pub use chain::Chain;
pub use drone::Drone;
pub use laser::Laser;
pub use main_gun::MainGun;
pub use missile::Missile;
pub use reflector::Reflector;
pub use rift::VoidRift;
pub use wave::WaveCannon;

use crate::entity::{Bullet, Enemy, Player};
use crate::fx::Fx;

use crate::entity::EnemyKind;

/// 共鸣槽：击杀填充，满槽进入"过载"状态，全武器伤害 ×OVERLOAD_DAMAGE_MUL。
/// 替代了旧的 DecayGauge —— 武器等级现在永久保留，节奏感靠这个槽提供。
pub struct SynergyGauge {
    /// 0..1，过载期间不再累计（视觉上变成倒计时条）
    pub charge: f32,
    /// >0 = 过载中，剩余秒数
    pub overload_remaining: f32,
}

pub const OVERLOAD_DURATION: f32 = 6.0;
pub const OVERLOAD_DAMAGE_MUL: f32 = 1.30;
const IDLE_DRAIN_PER_SEC: f32 = 0.05;

impl SynergyGauge {
    pub fn new() -> Self {
        Self {
            charge: 0.0,
            overload_remaining: 0.0,
        }
    }

    /// 击杀按敌人体型填充共鸣。返回 true = 这一击触发了过载。
    pub fn add_kill(&mut self, kind: EnemyKind) -> bool {
        if self.overload_remaining > 0.0 {
            return false;
        }
        let amount = match kind {
            EnemyKind::Small | EnemyKind::Kamikaze => 0.05,
            EnemyKind::Medium | EnemyKind::Strafer | EnemyKind::Sniper | EnemyKind::Weaver => 0.08,
            EnemyKind::Large | EnemyKind::MineLayer => 0.15,
            EnemyKind::Boss => 0.50,
        };
        self.charge += amount;
        if self.charge >= 1.0 {
            self.charge = 0.0;
            self.overload_remaining = OVERLOAD_DURATION;
            return true;
        }
        false
    }

    pub fn tick(&mut self, dt: f32) {
        if self.overload_remaining > 0.0 {
            self.overload_remaining = (self.overload_remaining - dt).max(0.0);
        } else {
            self.charge = (self.charge - IDLE_DRAIN_PER_SEC * dt).max(0.0);
        }
    }

    pub fn is_overloaded(&self) -> bool {
        self.overload_remaining > 0.0
    }

    pub fn damage_mul(&self) -> f32 {
        if self.is_overloaded() {
            OVERLOAD_DAMAGE_MUL
        } else {
            1.0
        }
    }

    /// HUD 用进度条比例。过载时表现为剩余时长。
    pub fn ratio(&self) -> f32 {
        if self.is_overloaded() {
            (self.overload_remaining / OVERLOAD_DURATION).clamp(0.0, 1.0)
        } else {
            self.charge.clamp(0.0, 1.0)
        }
    }
}

/// 副武器接口。主武器有自己的具体类型，不走 trait（性能更稳）。
pub trait SubWeapon {
    fn id(&self) -> &'static str;
    fn level(&self) -> u8;
    fn level_up(&mut self);
    fn max_level(&self) -> u8 {
        5
    }
    /// `damage_acc[HitSource as usize]` 累加本 tick 直接造成的伤害；
    /// 只有不通过 Bullet 走 resolve_player_bullets 的武器（laser/chain/rift）需要写。
    #[allow(clippy::too_many_arguments)]
    fn tick(
        &mut self,
        dt: f32,
        t: f32,
        player: &Player,
        enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
        damage_acc: &mut [f32; 9],
    );
    fn draw(&self, player: &Player, t: f32, ox: f32, oy: f32);
}

pub struct WeaponSlot {
    pub main: MainGun,
    pub subs: Vec<Box<dyn SubWeapon>>,
}

impl WeaponSlot {
    pub fn new() -> Self {
        Self {
            main: MainGun::new(),
            subs: Vec::with_capacity(4),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        dt: f32,
        t: f32,
        player: &Player,
        enemies: &mut [Enemy],
        bullets: &mut Vec<Bullet>,
        fx: &mut Fx,
        damage_acc: &mut [f32; 9],
    ) -> bool {
        let fired_main = self.main.tick(t, player, bullets);
        for s in &mut self.subs {
            s.tick(dt, t, player, enemies, bullets, fx, damage_acc);
        }
        fired_main
    }

    pub fn draw(&self, player: &Player, t: f32, ox: f32, oy: f32) {
        for s in &self.subs {
            s.draw(player, t, ox, oy);
        }
    }

    pub fn has(&self, id: &str) -> bool {
        self.subs.iter().any(|s| s.id() == id)
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Box<dyn SubWeapon>> {
        self.subs.iter_mut().find(|s| s.id() == id)
    }

    /// 副武器递减收益：0-2 个无惩罚，3 个 ×0.92，4 个 ×0.85。
    pub fn sub_penalty(&self) -> f32 {
        match self.subs.len() {
            0..=2 => 1.0,
            3 => 0.92,
            _ => 0.85,
        }
    }
}

/// 武器统一的暴击滚动：返回 (damage, is_crit)。`base_mul` 是该武器对玩家伤害的基础倍率。
pub fn roll_crit(player: &Player, base_mul: f32) -> (f32, bool) {
    use ::rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    let base = player.stats.damage_mul * base_mul;
    if player.stats.crit_chance > 0.0 && rng.gen::<f32>() < player.stats.crit_chance {
        (base * player.stats.crit_mul, true)
    } else {
        (base, false)
    }
}
