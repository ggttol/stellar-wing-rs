use macroquad::prelude::*;

use crate::art::draw_enemy_ship;
use crate::config::CFG;
use crate::entity::{Bullet, HitSource};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EliteMod {
    Armored,
    Berserk,
    Dasher,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BossMod {
    Frenzied,
    Bulwark,
    Summoner,
    StormCore,
    /// 周期性瞬移；瞬移期间无敌（短暂淡出）。
    Phantom,
    /// 50% HP 时分裂出两架精英 Large 护卫；自身保留剩余血。
    Hydra,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TelegraphKind {
    None,
    EliteDash,
    BossAim,
    BossFan,
    BossNova,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Small,
    Medium,
    Large,
    Boss,
    /// 自爆冲撞：锁定玩家初始位置高速直线冲击。
    Kamikaze,
    /// 横向扫射：从屏幕一侧滑过，沿途持续射击。
    Strafer,
}

impl EnemyKind {
    pub fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::Small => EnemyStats {
                w: 34.0,
                h: 34.0,
                radius: 15.0,
                hp: 1.0,
                score: 10,
                xp: 1,
                speed: 90.0,
                fire_rate: 0.0,
                color: Color::from_rgba(255, 136, 102, 255),
            },
            EnemyKind::Medium => EnemyStats {
                w: 46.0,
                h: 46.0,
                radius: 20.0,
                hp: 2.0,
                score: 30,
                xp: 3,
                speed: 70.0,
                fire_rate: 1.8,
                color: Color::from_rgba(201, 124, 255, 255),
            },
            EnemyKind::Large => EnemyStats {
                w: 78.0,
                h: 72.0,
                radius: 32.0,
                hp: 5.0,
                score: 100,
                xp: 8,
                speed: 45.0,
                fire_rate: 1.1,
                color: Color::from_rgba(255, 77, 109, 255),
            },
            EnemyKind::Kamikaze => EnemyStats {
                w: 26.0,
                h: 30.0,
                radius: 13.0,
                hp: 1.5,
                score: 25,
                xp: 2,
                speed: 260.0,
                fire_rate: 0.0,
                color: Color::from_rgba(255, 90, 110, 255),
            },
            EnemyKind::Strafer => EnemyStats {
                w: 38.0,
                h: 30.0,
                radius: 16.0,
                hp: 2.0,
                score: 45,
                xp: 3,
                speed: 220.0,
                fire_rate: 0.55,
                color: Color::from_rgba(125, 220, 255, 255),
            },
            EnemyKind::Boss => EnemyStats {
                w: 180.0,
                h: 110.0,
                radius: 70.0,
                hp: 80.0,
                score: 2500,
                xp: 120,
                speed: 35.0,
                fire_rate: 0.9,
                color: Color::from_rgba(255, 90, 140, 255),
            },
        }
    }
}

pub struct EnemyStats {
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub hp: f32,
    pub score: u32,
    pub xp: u32,
    pub speed: f32,
    pub fire_rate: f32,
    pub color: Color,
}

#[allow(dead_code)] // vx 给 M7 用
pub struct Enemy {
    pub kind: EnemyKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub score: u32,
    pub xp: u32,
    pub fire_rate: f32,
    pub last_shot: f32,
    pub hit_flash: f32,
    pub color: Color,
    pub spawn_t: f32,
    pub is_elite: bool,
    pub elite_mod: Option<EliteMod>,
    pub dash_charge: f32,
    pub dash_cooldown: f32,
    pub telegraph: f32,
    pub telegraph_kind: TelegraphKind,
    pub marked_until: f32,
    pub static_mark: bool,
    pub boss_mod: Option<BossMod>,
    pub boss_phase: u8,
    pub pending_boss_attack: TelegraphKind,
    /// Phantom：剩余瞬移无敌时间（>0 时不可被击伤）
    pub phantom_invuln: f32,
    /// Phantom：距离下次瞬移的倒计时
    pub phantom_blink_in: f32,
    /// Hydra：50% HP 触发的"分裂"是否已经放过
    pub hydra_split: bool,
    pub last_hit: HitSource,
    /// 本敌人发射的子弹伤害，由 spawn 端根据 run_time 设置。
    pub bullet_damage: f32,
    pub dead: bool,
}

impl Enemy {
    pub fn new(kind: EnemyKind, x: f32, t: f32) -> Self {
        let s = kind.stats();
        let y = -s.h * 0.5;
        Self {
            kind,
            x,
            y,
            vx: 0.0,
            vy: s.speed,
            w: s.w,
            h: s.h,
            radius: s.radius,
            hp: s.hp,
            max_hp: s.hp,
            score: s.score,
            xp: s.xp,
            fire_rate: s.fire_rate,
            last_shot: t + match kind {
                EnemyKind::Medium => 1.0,
                EnemyKind::Large => 0.6,
                EnemyKind::Boss => 1.5,
                _ => 0.0,
            },
            hit_flash: 0.0,
            color: s.color,
            spawn_t: t,
            is_elite: false,
            elite_mod: None,
            dash_charge: 0.0,
            dash_cooldown: 0.0,
            telegraph: 0.0,
            telegraph_kind: TelegraphKind::None,
            marked_until: 0.0,
            static_mark: false,
            boss_mod: None,
            boss_phase: 0,
            pending_boss_attack: TelegraphKind::None,
            phantom_invuln: 0.0,
            phantom_blink_in: 6.0,
            hydra_split: false,
            last_hit: HitSource::Enemy,
            bullet_damage: 1.0,
            dead: false,
        }
    }

    /// 创建一发敌方子弹，自动使用本敌人的 bullet_damage。
    fn make_bullet(&self, x: f32, y: f32, vx: f32, vy: f32) -> Bullet {
        let mut b = Bullet::enemy_shot(x, y, vx, vy);
        b.damage = self.bullet_damage;
        b
    }

    pub fn into_elite(mut self, elite_mod: EliteMod) -> Self {
        self.is_elite = true;
        self.elite_mod = Some(elite_mod);
        self.hp *= match elite_mod {
            EliteMod::Armored => 3.2,
            EliteMod::Berserk => 2.4,
            EliteMod::Dasher => 2.2,
        };
        self.max_hp = self.hp;
        self.score = ((self.score as f32) * 3.0) as u32;
        self.xp = ((self.xp as f32) * 2.5).ceil() as u32;
        self.color = match elite_mod {
            EliteMod::Armored => Color::from_rgba(255, 219, 99, 255),
            EliteMod::Berserk => Color::from_rgba(255, 110, 150, 255),
            EliteMod::Dasher => Color::from_rgba(125, 249, 255, 255),
        };
        self
    }

    pub fn into_boss_mod(mut self, boss_mod: BossMod) -> Self {
        self.boss_mod = Some(boss_mod);
        match boss_mod {
            BossMod::Frenzied => {
                self.fire_rate *= 0.88;
                self.color = Color::from_rgba(255, 120, 120, 255);
            }
            BossMod::Bulwark => {
                self.max_hp *= 1.18;
                self.hp = self.max_hp;
                self.color = Color::from_rgba(255, 209, 102, 255);
            }
            BossMod::Summoner => {
                self.color = Color::from_rgba(255, 150, 210, 255);
            }
            BossMod::StormCore => {
                self.fire_rate *= 0.95;
                self.color = Color::from_rgba(125, 249, 255, 255);
            }
            BossMod::Phantom => {
                self.fire_rate *= 0.92;
                self.color = Color::from_rgba(180, 140, 255, 255);
            }
            BossMod::Hydra => {
                self.fire_rate *= 0.88;
                self.color = Color::from_rgba(120, 230, 170, 255);
            }
        }
        self
    }

    pub fn update(&mut self, dt: f32, t: f32, player_x: f32, bullets: &mut Vec<Bullet>) {
        if self.hit_flash > 0.0 {
            self.hit_flash -= dt;
        }
        if self.telegraph > 0.0 {
            self.telegraph -= dt;
            if self.telegraph <= 0.0 && self.telegraph_kind == TelegraphKind::EliteDash {
                self.telegraph_kind = TelegraphKind::None;
            }
        }
        if self.dash_cooldown > 0.0 {
            self.dash_cooldown -= dt;
        }

        match self.kind {
            EnemyKind::Boss => self.update_boss(dt, t, player_x, bullets),
            EnemyKind::Kamikaze => {
                // 自爆冲撞：直线冲刺，越界即销毁。vx/vy 由 spawn 阶段锁定。
                self.x += self.vx * dt;
                self.y += self.vy * dt;
                if self.y > CFG.h + 60.0
                    || self.y < -60.0
                    || self.x < -60.0
                    || self.x > CFG.w + 60.0
                {
                    self.dead = true;
                }
            }
            EnemyKind::Strafer => {
                // 横扫：vx 方向移动，越界即销毁。沿途朝下射击。
                self.x += self.vx * dt;
                self.y += self.vy * dt; // 通常 vy=0，留个口子做斜扫
                if self.x < -60.0 || self.x > CFG.w + 60.0 {
                    self.dead = true;
                }
                if self.fire_rate > 0.0 && t - self.last_shot >= self.fire_rate {
                    self.last_shot = t;
                    bullets.push(self.make_bullet(
                        self.x,
                        self.y + self.h * 0.4,
                        0.0,
                        300.0,
                    ));
                }
            }
            _ => {
                let mut speed_mul = 1.0;
                if self.is_elite && matches!(self.elite_mod, Some(EliteMod::Berserk)) {
                    speed_mul += (1.0 - self.hp / self.max_hp).max(0.0) * 0.9;
                }
                self.y += self.vy * dt;
                match self.kind {
                    EnemyKind::Medium => {
                        self.x += (self.y / 60.0).sin() * 18.0 * dt * speed_mul;
                    }
                    EnemyKind::Large => {
                        self.x += (self.y / 80.0).sin() * 30.0 * dt * speed_mul;
                    }
                    _ => {}
                }
                if self.is_elite && matches!(self.elite_mod, Some(EliteMod::Dasher)) {
                    self.update_dash(dt, player_x);
                }
                if self.y > CFG.h + 60.0 {
                    self.dead = true;
                }
                if self.fire_rate > 0.0 && self.y > 0.0 && t - self.last_shot >= self.fire_rate {
                    self.last_shot = t;
                    self.fire_normal(player_x, bullets);
                }
            }
        }
    }

    fn update_dash(&mut self, dt: f32, player_x: f32) {
        if self.dash_cooldown <= 0.0 && self.telegraph <= 0.0 && self.y > 70.0 {
            self.telegraph = 0.45;
            self.telegraph_kind = TelegraphKind::EliteDash;
            self.dash_charge = if player_x >= self.x { 240.0 } else { -240.0 };
            self.dash_cooldown = 3.8;
        }
        if self.telegraph <= 0.0 && self.dash_charge.abs() > 1.0 {
            self.x += self.dash_charge * dt;
            self.dash_charge *= 0.88_f32.powf(dt * 60.0);
        }
    }

    pub fn damage_mul(&self) -> f32 {
        // Phantom：瞬移期间完全无敌（淡出状态）
        if matches!(self.boss_mod, Some(BossMod::Phantom)) && self.phantom_invuln > 0.0 {
            return 0.0;
        }
        if matches!(self.kind, EnemyKind::Boss)
            && matches!(self.boss_mod, Some(BossMod::Bulwark))
            && self.hp / self.max_hp > 0.75
        {
            return 0.55;
        }
        if self.is_elite && matches!(self.elite_mod, Some(EliteMod::Armored)) {
            0.72
        } else {
            1.0
        }
    }

    fn fire_normal(&self, player_x: f32, bullets: &mut Vec<Bullet>) {
        match self.kind {
            EnemyKind::Medium => {
                let dx = player_x - self.x;
                let dy = (CFG.h - 100.0) - self.y;
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                let speed = 320.0;
                bullets.push(self.make_bullet(
                    self.x,
                    self.y + self.h * 0.5,
                    dx / len * speed,
                    dy / len * speed,
                ));
            }
            EnemyKind::Large => {
                let speed = 320.0;
                bullets.push(self.make_bullet(
                    self.x,
                    self.y + self.h * 0.5,
                    0.0,
                    speed,
                ));
                bullets.push(self.make_bullet(
                    self.x - 15.0,
                    self.y + self.h * 0.5 - 6.0,
                    -75.0,
                    speed * 0.9,
                ));
                bullets.push(self.make_bullet(
                    self.x + 15.0,
                    self.y + self.h * 0.5 - 6.0,
                    75.0,
                    speed * 0.9,
                ));
            }
            _ => {}
        }
    }

    fn update_boss(&mut self, dt: f32, t: f32, player_x: f32, bullets: &mut Vec<Bullet>) {
        // 入场：从 -h*0.5 飘到 y=140
        let target_y = 140.0;
        if self.y < target_y {
            self.y = (self.y + self.vy * dt).min(target_y);
        }

        // Phantom：周期性瞬移 + 短暂无敌淡出
        if matches!(self.boss_mod, Some(BossMod::Phantom)) {
            if self.phantom_invuln > 0.0 {
                self.phantom_invuln -= dt;
            }
            self.phantom_blink_in -= dt;
            if self.phantom_blink_in <= 0.0 {
                use ::rand::{thread_rng, Rng};
                let mut rng = thread_rng();
                self.x = rng.gen_range(80.0..(CFG.w - 80.0));
                self.phantom_invuln = 0.45;
                self.phantom_blink_in = 6.5;
            }
        }
        // 横向正弦巡航
        let life = (t - self.spawn_t).max(0.0);
        let osc = (life * 0.9).sin();
        let center = CFG.w * 0.5;
        let target_x = center + osc * (CFG.w * 0.30);
        // 平滑趋近
        self.x += (target_x - self.x) * (2.5 * dt).min(1.0);

        let ratio = self.hp / self.max_hp;
        self.advance_boss_phase(ratio, t, bullets);

        let interval = if ratio > 0.66 {
            self.fire_rate
        } else if ratio > 0.33 {
            self.fire_rate * 0.85
        } else {
            self.fire_rate * 0.7
        };
        let interval = if matches!(self.boss_mod, Some(BossMod::Frenzied)) {
            interval * 0.9
        } else {
            interval
        };
        if self.telegraph > 0.0
            && matches!(
                self.telegraph_kind,
                TelegraphKind::BossAim | TelegraphKind::BossFan | TelegraphKind::BossNova
            )
        {
            return;
        }
        if self.pending_boss_attack != TelegraphKind::None && self.telegraph <= 0.0 {
            self.fire_pending_boss_attack(player_x, bullets, life);
            self.pending_boss_attack = TelegraphKind::None;
            self.telegraph_kind = TelegraphKind::None;
            self.last_shot = t;
            return;
        }
        if t - self.last_shot < interval || self.y < target_y - 1.0 {
            return;
        }
        if ratio > 0.66 {
            self.telegraph = 0.55;
            self.telegraph_kind = TelegraphKind::BossAim;
            self.pending_boss_attack = TelegraphKind::BossAim;
        } else if ratio > 0.33 {
            self.telegraph = 0.7;
            self.telegraph_kind = TelegraphKind::BossFan;
            self.pending_boss_attack = TelegraphKind::BossFan;
        } else {
            self.telegraph = 0.8;
            self.telegraph_kind = TelegraphKind::BossNova;
            self.pending_boss_attack = TelegraphKind::BossNova;
        }
    }

    fn advance_boss_phase(&mut self, ratio: f32, t: f32, bullets: &mut Vec<Bullet>) {
        if !matches!(self.kind, EnemyKind::Boss) {
            return;
        }

        // Hydra：50% HP 触发一次"分裂"爆发，与三段相位独立
        if matches!(self.boss_mod, Some(BossMod::Hydra)) && !self.hydra_split && ratio <= 0.5 {
            self.hydra_split = true;
            let muzzle_y = self.y + self.h * 0.45;
            // 12 道圆环
            let spokes = 14;
            let speed = 280.0;
            for i in 0..spokes {
                let ang = i as f32 * std::f32::consts::TAU / spokes as f32;
                bullets.push(self.make_bullet(
                    self.x,
                    self.y,
                    ang.cos() * speed,
                    ang.sin() * speed,
                ));
            }
            // 4 发左右散弹
            for i in [-2.0_f32, -1.0, 1.0, 2.0] {
                bullets.push(self.make_bullet(
                    self.x + i * 18.0,
                    muzzle_y,
                    i * 60.0,
                    320.0,
                ));
            }
            self.telegraph = 0.0;
            self.pending_boss_attack = TelegraphKind::None;
        }

        let phase = if ratio > 0.66 {
            0
        } else if ratio > 0.33 {
            1
        } else {
            2
        };
        if phase <= self.boss_phase {
            return;
        }
        self.boss_phase = phase;
        if matches!(self.boss_mod, Some(BossMod::Summoner)) {
            let count = 2 + phase as usize;
            for i in 0..count {
                let off = (i as f32 - (count as f32 - 1.0) * 0.5) * 42.0;
                let mut minion = Enemy::new(
                    EnemyKind::Small,
                    (self.x + off).clamp(32.0, CFG.w - 32.0),
                    t,
                );
                minion.y = self.y + 24.0;
                bullets.push(self.make_bullet(
                    minion.x,
                    minion.y + 8.0,
                    0.0,
                    220.0 + phase as f32 * 30.0,
                ));
            }
        }
    }

    fn fire_pending_boss_attack(&mut self, player_x: f32, bullets: &mut Vec<Bullet>, life: f32) {
        let muzzle_y = self.y + self.h * 0.45;
        match self.pending_boss_attack {
            TelegraphKind::BossAim => {
                for off in [-22.0_f32, 22.0_f32] {
                    let dx = player_x - (self.x + off);
                    let dy = (CFG.h - 100.0) - muzzle_y;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    let speed = if matches!(self.boss_mod, Some(BossMod::Frenzied)) {
                        390.0
                    } else {
                        360.0
                    };
                    bullets.push(self.make_bullet(
                        self.x + off,
                        muzzle_y,
                        dx / len * speed,
                        dy / len * speed,
                    ));
                }
            }
            TelegraphKind::BossFan => {
                let speed = 320.0;
                for i in -2..=2 {
                    let ang = i as f32 * 0.20_f32;
                    bullets.push(self.make_bullet(
                        self.x,
                        muzzle_y,
                        ang.sin() * speed,
                        ang.cos() * speed,
                    ));
                }
                if matches!(self.boss_mod, Some(BossMod::StormCore)) {
                    for i in -3..=3 {
                        if i == 0 {
                            continue;
                        }
                        let ang = i as f32 * 0.15_f32;
                        bullets.push(self.make_bullet(
                            self.x,
                            muzzle_y,
                            ang.sin() * 260.0,
                            ang.cos() * 260.0,
                        ));
                    }
                }
            }
            TelegraphKind::BossNova => {
                let speed = 280.0;
                let spokes = if matches!(self.boss_mod, Some(BossMod::StormCore)) {
                    12
                } else {
                    8
                };
                for i in 0..spokes {
                    let ang =
                        i as f32 * std::f32::consts::TAU / spokes as f32 + (life * 0.6).sin() * 0.2;
                    bullets.push(self.make_bullet(
                        self.x,
                        self.y,
                        ang.cos() * speed,
                        ang.sin() * speed,
                    ));
                }
                let dx = player_x - self.x;
                let dy = (CFG.h - 100.0) - muzzle_y;
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                bullets.push(self.make_bullet(
                    self.x,
                    muzzle_y,
                    dx / len * 380.0,
                    dy / len * 380.0,
                ));
            }
            _ => {}
        }
    }

    pub fn boss_mod_label(&self) -> Option<&'static str> {
        match self.boss_mod {
            Some(BossMod::Frenzied) => Some("Frenzied"),
            Some(BossMod::Bulwark) => Some("Bulwark"),
            Some(BossMod::Summoner) => Some("Summoner"),
            Some(BossMod::StormCore) => Some("Storm Core"),
            Some(BossMod::Phantom) => Some("Phantom"),
            Some(BossMod::Hydra) => Some("Hydra"),
            None => None,
        }
    }

    pub fn draw(&self) {
        let phantom_fade = if matches!(self.boss_mod, Some(BossMod::Phantom))
            && self.phantom_invuln > 0.0
        {
            (self.phantom_invuln / 0.45).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let alpha = 1.0 - phantom_fade * 0.75;
        let mut c = if self.hit_flash > 0.0 {
            WHITE
        } else {
            self.color
        };
        c.a = alpha;
        let mut g = c;
        g.a = if self.is_elite { 0.38 } else { 0.25 } * alpha;
        draw_circle(self.x, self.y, self.radius * 1.4, g);
        if self.is_elite {
            let mut ring = c;
            ring.a = 0.8;
            draw_circle_lines(self.x, self.y, self.radius + 6.0, 2.0, ring);
            if self.telegraph > 0.0 && self.telegraph_kind == TelegraphKind::EliteDash {
                let tx = self.x + self.dash_charge.signum() * 40.0;
                draw_line(
                    self.x,
                    self.y,
                    tx,
                    self.y,
                    3.0,
                    Color::from_rgba(255, 90, 110, 220),
                );
            }
        }
        if matches!(self.kind, EnemyKind::Boss) && self.telegraph > 0.0 {
            let warn = Color::from_rgba(255, 90, 110, 220);
            let muzzle_y = self.y + self.h * 0.45;
            match self.telegraph_kind {
                TelegraphKind::BossAim => {
                    for off in [-22.0_f32, 22.0_f32] {
                        draw_line(
                            self.x + off,
                            muzzle_y,
                            self.x + off,
                            CFG.h - 110.0,
                            2.0,
                            warn,
                        );
                    }
                }
                TelegraphKind::BossFan => {
                    for ang in [-0.4_f32, -0.2, 0.0, 0.2, 0.4] {
                        draw_line(
                            self.x,
                            muzzle_y,
                            self.x + ang.sin() * 180.0,
                            muzzle_y + ang.cos() * 180.0,
                            2.0,
                            warn,
                        );
                    }
                }
                TelegraphKind::BossNova => {
                    draw_circle_lines(self.x, self.y, self.radius + 18.0, 3.0, warn);
                }
                _ => {}
            }
        }

        draw_enemy_ship(
            self.kind,
            self.x,
            self.y,
            self.w,
            self.h,
            c,
            (self.hp / self.max_hp).max(0.0),
        );
    }
}
