//! 升级卡池 ~25 张。三档稀有度 + eligible 过滤。

use ::rand::seq::SliceRandom;
use ::rand::thread_rng;
use macroquad::color::Color;

use crate::entity::Player;
use crate::weapon::{Chain, Drone, Laser, Missile, WeaponSlot};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
}

impl Rarity {
    pub fn color(self) -> Color {
        match self {
            Rarity::Common => Color::from_rgba(200, 220, 255, 255),
            Rarity::Rare => Color::from_rgba(125, 200, 255, 255),
            Rarity::Epic => Color::from_rgba(220, 140, 255, 255),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
        }
    }
    pub fn weight(self) -> u32 {
        match self {
            Rarity::Common => 60,
            Rarity::Rare => 30,
            Rarity::Epic => 10,
        }
    }
}

#[allow(dead_code)] // id 字段用于调试
#[derive(Clone)]
pub struct Card {
    pub id: &'static str,
    pub rarity: Rarity,
    pub name: &'static str,
    pub desc: &'static str,
    pub apply: fn(&mut Player, &mut WeaponSlot),
    pub eligible: fn(&Player, &WeaponSlot) -> bool,
}

// ---- 数值卡 ---------------------------------------------------------------
fn fire_rate_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.fire_rate = (p.stats.fire_rate * 0.90).max(0.18);
}
fn damage_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.damage_mul = (p.stats.damage_mul * 1.12).min(2.35);
}
fn bullet_speed_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.bullet_speed = (p.stats.bullet_speed * 1.15).min(1200.0);
}
fn move_speed_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.speed = (p.stats.speed * 1.10).min(2200.0);
}
fn max_hp_up(p: &mut Player, _: &mut WeaponSlot) {
    if p.stats.max_lives < 5 {
        p.stats.max_lives = p.stats.max_lives.saturating_add(1);
        p.lives = p.lives.saturating_add(1).min(p.stats.max_lives);
    }
}
fn pickup_radius_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.attract_radius = (p.stats.attract_radius * 1.35).min(230.0);
}
fn crit_chance_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.crit_chance = (p.stats.crit_chance + 0.08).min(0.35);
}
fn crit_dmg_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.crit_mul = (p.stats.crit_mul + 0.35).min(2.8);
}
fn crit_dmg_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.crit_chance > 0.0
}
fn score_mul_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.score_mul = (p.stats.score_mul * 1.18).min(2.2);
}
fn xp_mul_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.xp_mul = (p.stats.xp_mul * 1.18).min(2.0);
}
fn regen_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.regen_per_min = (p.stats.regen_per_min + 0.35).min(1.2);
}
fn invincible_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.invincible = (p.stats.invincible + 0.12).min(1.9);
}
fn shield_grant(p: &mut Player, _: &mut WeaponSlot) {
    p.shield = true;
}
fn shield_eligible(p: &Player, _: &WeaponSlot) -> bool {
    !p.shield
}
fn heal_now(p: &mut Player, _: &mut WeaponSlot) {
    p.lives = p.stats.max_lives;
}
fn heal_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.lives < p.stats.max_lives
}
fn heat_lock_apply(p: &mut Player, _: &mut WeaponSlot) {
    p.perks.heat_lock = true;
}
fn heat_lock_eligible(p: &Player, w: &WeaponSlot) -> bool {
    !p.perks.heat_lock && w.has("missile") && w.has("laser")
}
fn static_mark_apply(p: &mut Player, _: &mut WeaponSlot) {
    p.perks.static_mark = true;
}
fn static_mark_eligible(p: &Player, w: &WeaponSlot) -> bool {
    !p.perks.static_mark && w.has("chain")
}
fn drone_relay_apply(p: &mut Player, _: &mut WeaponSlot) {
    p.perks.drone_relay = true;
}
fn drone_relay_eligible(p: &Player, w: &WeaponSlot) -> bool {
    !p.perks.drone_relay && w.has("drone") && w.has("missile")
}

// ---- 武器卡 ---------------------------------------------------------------
fn main_gun_up(_: &mut Player, w: &mut WeaponSlot) {
    w.main.level_up();
}
fn main_gun_eligible(_: &Player, w: &WeaponSlot) -> bool {
    !w.main.is_max()
}

fn mk_unlock_eligible(id: &'static str) -> impl Fn(&Player, &WeaponSlot) -> bool + 'static {
    move |_p, w| !w.has(id) && w.subs.len() < 4
}

fn mk_up_eligible(id: &'static str) -> impl Fn(&Player, &WeaponSlot) -> bool + 'static {
    move |_p, w| {
        w.subs
            .iter()
            .find(|s| s.id() == id)
            .map(|s| s.level() < s.max_level())
            .unwrap_or(false)
    }
}

fn unlock_missile(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("missile") && w.subs.len() < 4 {
        w.subs.push(Box::new(Missile::new()));
    }
}
fn unlock_drone(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("drone") && w.subs.len() < 4 {
        w.subs.push(Box::new(Drone::new()));
    }
}
fn unlock_laser(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("laser") && w.subs.len() < 4 {
        w.subs.push(Box::new(Laser::new()));
    }
}
fn unlock_chain(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("chain") && w.subs.len() < 4 {
        w.subs.push(Box::new(Chain::new()));
    }
}

fn missile_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("missile") {
        s.level_up();
    }
}
fn drone_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("drone") {
        s.level_up();
    }
}
fn laser_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("laser") {
        s.level_up();
    }
}
fn chain_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("chain") {
        s.level_up();
    }
}

// 因为 fn 字段不能用闭包，需要显式声明 wrapper
fn unlock_missile_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("missile")(p, w)
}
fn unlock_drone_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("drone")(p, w)
}
fn unlock_laser_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("laser")(p, w)
}
fn unlock_chain_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("chain")(p, w)
}
fn missile_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("missile")(p, w)
}
fn drone_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("drone")(p, w)
}
fn laser_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("laser")(p, w)
}
fn chain_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("chain")(p, w)
}

fn always(_: &Player, _: &WeaponSlot) -> bool {
    true
}

pub fn pool() -> Vec<Card> {
    vec![
        // 数值（白卡）
        c(
            "fire_rate",
            Rarity::Common,
            "Rapid Fire",
            "Fire rate +15%",
            fire_rate_up,
            always,
        ),
        c(
            "damage",
            Rarity::Common,
            "High Caliber",
            "Damage +20%",
            damage_up,
            always,
        ),
        c(
            "bullet_speed",
            Rarity::Common,
            "Velocity",
            "Bullet speed +25%",
            bullet_speed_up,
            always,
        ),
        c(
            "move_speed",
            Rarity::Common,
            "Afterburner",
            "Move speed +15%",
            move_speed_up,
            always,
        ),
        c(
            "pickup_r",
            Rarity::Common,
            "Magnetic Field",
            "Pickup range +50%",
            pickup_radius_up,
            always,
        ),
        c(
            "xp_mul",
            Rarity::Common,
            "Sharp Eyes",
            "XP gain +30%",
            xp_mul_up,
            always,
        ),
        c(
            "score_mul",
            Rarity::Common,
            "Bounty Hunter",
            "Score +25%",
            score_mul_up,
            always,
        ),
        // 数值（蓝卡）
        c(
            "max_hp",
            Rarity::Rare,
            "Hull Plating",
            "Max HP +1",
            max_hp_up,
            always,
        ),
        c(
            "crit_chance",
            Rarity::Rare,
            "Sniper Lens",
            "Crit chance +10%",
            crit_chance_up,
            always,
        ),
        c(
            "crit_dmg",
            Rarity::Rare,
            "Devastator",
            "Crit damage +50%",
            crit_dmg_up,
            crit_dmg_eligible,
        ),
        c(
            "regen",
            Rarity::Rare,
            "Auto-Repair",
            "+0.5 HP / minute",
            regen_up,
            always,
        ),
        c(
            "invincible",
            Rarity::Rare,
            "Adrenaline",
            "I-frames +30%",
            invincible_up,
            always,
        ),
        c(
            "shield",
            Rarity::Rare,
            "Energy Shield",
            "Block one hit",
            shield_grant,
            shield_eligible,
        ),
        c(
            "heal",
            Rarity::Common,
            "Repair Kit",
            "Refill HP now",
            heal_now,
            heal_eligible,
        ),
        // 主武器（蓝卡）
        c(
            "main_gun_up",
            Rarity::Rare,
            "Main Gun +1",
            "Single→Dual→Triple→5w→Pierce",
            main_gun_up,
            main_gun_eligible,
        ),
        // 副武器解锁（紫卡）
        c(
            "u_missile",
            Rarity::Epic,
            "Homing Missile",
            "Auto-lock target",
            unlock_missile,
            unlock_missile_eligible,
        ),
        c(
            "u_drone",
            Rarity::Epic,
            "Orbit Drone",
            "Spinning satellite",
            unlock_drone,
            unlock_drone_eligible,
        ),
        c(
            "u_laser",
            Rarity::Epic,
            "Pulse Laser",
            "Vertical beam, DPS",
            unlock_laser,
            unlock_laser_eligible,
        ),
        c(
            "u_chain",
            Rarity::Epic,
            "Chain Bolt",
            "Lightning jumps targets",
            unlock_chain,
            unlock_chain_eligible,
        ),
        // 副武器升级（蓝卡）
        c(
            "missile_up",
            Rarity::Rare,
            "Missile +1",
            "More & faster missiles",
            missile_up,
            missile_up_eligible,
        ),
        c(
            "drone_up",
            Rarity::Rare,
            "Drone +1",
            "More & faster drones",
            drone_up,
            drone_up_eligible,
        ),
        c(
            "laser_up",
            Rarity::Rare,
            "Laser +1",
            "Wider beam · more DPS",
            laser_up,
            laser_up_eligible,
        ),
        c(
            "chain_up",
            Rarity::Rare,
            "Chain +1",
            "More jumps · damage",
            chain_up,
            chain_up_eligible,
        ),
        c(
            "heat_lock",
            Rarity::Epic,
            "Heat Lock",
            "Missile marks targets · laser deals bonus damage",
            heat_lock_apply,
            heat_lock_eligible,
        ),
        c(
            "static_mark",
            Rarity::Epic,
            "Static Mark",
            "Chain-charged targets are guaranteed crits once",
            static_mark_apply,
            static_mark_eligible,
        ),
        c(
            "drone_relay",
            Rarity::Epic,
            "Drone Relay",
            "Drone kills launch a homing follow-up missile",
            drone_relay_apply,
            drone_relay_eligible,
        ),
    ]
}

fn c(
    id: &'static str,
    rarity: Rarity,
    name: &'static str,
    desc: &'static str,
    apply: fn(&mut Player, &mut WeaponSlot),
    eligible: fn(&Player, &WeaponSlot) -> bool,
) -> Card {
    Card {
        id,
        rarity,
        name,
        desc,
        apply,
        eligible,
    }
}

/// 抽 N 张去重卡，按 eligible + 稀有度权重过滤。
pub fn draw_n(n: usize, player: &Player, weapons: &WeaponSlot) -> Vec<Card> {
    use ::rand::Rng;
    let mut rng = thread_rng();
    let all: Vec<Card> = pool()
        .into_iter()
        .filter(|c| (c.eligible)(player, weapons))
        .collect();
    if all.is_empty() {
        return vec![];
    }
    // 加权随机（带去重）
    let mut picks: Vec<Card> = Vec::with_capacity(n);
    let mut pool: Vec<Card> = all;
    while picks.len() < n && !pool.is_empty() {
        let total: u32 = pool.iter().map(|c| c.rarity.weight()).sum();
        let mut r = rng.gen_range(0..total);
        let mut idx = 0;
        for (i, c) in pool.iter().enumerate() {
            if r < c.rarity.weight() {
                idx = i;
                break;
            }
            r -= c.rarity.weight();
        }
        picks.push(pool.swap_remove(idx));
    }
    picks.shuffle(&mut rng);
    picks
}
