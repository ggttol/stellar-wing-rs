//! 升级卡池 35 张。四档稀有度 + eligible 过滤 + 保底出解锁卡。

use ::rand::seq::SliceRandom;
use ::rand::thread_rng;
use macroquad::color::Color;

use crate::entity::Player;
use crate::weapon::{Chain, Drone, Laser, Missile, Reflector, VoidRift, WaveCannon, WeaponSlot};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(self) -> Color {
        match self {
            Rarity::Common => Color::from_rgba(200, 220, 255, 255),
            Rarity::Rare => Color::from_rgba(125, 200, 255, 255),
            Rarity::Epic => Color::from_rgba(220, 140, 255, 255),
            Rarity::Legendary => Color::from_rgba(255, 200, 60, 255),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        }
    }
    pub fn weight(self) -> u32 {
        match self {
            Rarity::Common => 48,
            Rarity::Rare => 30,
            Rarity::Epic => 15,
            Rarity::Legendary => 7,
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
fn fire_rate_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.fire_rate > 0.18
}
fn damage_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.damage_mul = (p.stats.damage_mul * 1.12).min(2.35);
}
fn damage_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.damage_mul < 2.35
}
fn bullet_speed_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.bullet_speed = (p.stats.bullet_speed * 1.15).min(1200.0);
}
fn bullet_speed_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.bullet_speed < 1200.0
}
fn move_speed_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.speed = (p.stats.speed * 1.10).min(2200.0);
}
fn move_speed_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.speed < 2200.0
}
fn max_hp_up(p: &mut Player, _: &mut WeaponSlot) {
    if p.perks.hull_plating_picks < 2 {
        p.stats.max_lives = p.stats.max_lives.saturating_add(1);
        p.lives = p.lives.saturating_add(1).min(p.stats.max_lives);
        p.perks.hull_plating_picks += 1;
    }
}
fn max_hp_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.perks.hull_plating_picks < 2
}
fn pickup_radius_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.attract_radius = (p.stats.attract_radius * 1.35).min(230.0);
}
fn pickup_radius_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.attract_radius < 230.0
}
fn crit_chance_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.crit_chance = (p.stats.crit_chance + 0.08).min(0.35);
}
fn crit_chance_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.crit_chance < 0.35
}
fn crit_dmg_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.crit_mul = (p.stats.crit_mul + 0.35).min(2.8);
}
fn crit_dmg_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.crit_chance > 0.0 && p.stats.crit_mul < 2.8
}
fn score_mul_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.score_mul = (p.stats.score_mul * 1.18).min(2.2);
}
fn score_mul_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.score_mul < 2.2
}
fn xp_mul_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.xp_mul = (p.stats.xp_mul * 1.18).min(2.0);
}
fn xp_mul_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.xp_mul < 2.0
}
fn regen_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.regen_per_min = (p.stats.regen_per_min + 0.35).min(1.2);
}
fn regen_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.regen_per_min < 1.2
}
fn invincible_up(p: &mut Player, _: &mut WeaponSlot) {
    p.stats.invincible = (p.stats.invincible + 0.12).min(1.9);
}
fn invincible_eligible(p: &Player, _: &WeaponSlot) -> bool {
    p.stats.invincible < 1.9
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

// —— 新武器：Void Rift / Wave / Reflector ——————————————————————

fn unlock_rift(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("rift") && w.subs.len() < 4 {
        w.subs.push(Box::new(VoidRift::new()));
    }
}
fn unlock_rift_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("rift")(p, w)
}
fn rift_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("rift") {
        s.level_up();
    }
}
fn rift_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("rift")(p, w)
}

fn unlock_wave(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("wave") && w.subs.len() < 4 {
        w.subs.push(Box::new(WaveCannon::new()));
    }
}
fn unlock_wave_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("wave")(p, w)
}
fn wave_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("wave") {
        s.level_up();
    }
}
fn wave_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("wave")(p, w)
}

fn unlock_reflector(_: &mut Player, w: &mut WeaponSlot) {
    if !w.has("reflector") && w.subs.len() < 4 {
        w.subs.push(Box::new(Reflector::new()));
    }
}
fn unlock_reflector_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_unlock_eligible("reflector")(p, w)
}
fn reflector_up(_: &mut Player, w: &mut WeaponSlot) {
    if let Some(s) = w.find_mut("reflector") {
        s.level_up();
    }
}
fn reflector_up_eligible(p: &Player, w: &WeaponSlot) -> bool {
    mk_up_eligible("reflector")(p, w)
}

// —— 新联动 Perks ————————————————————————————————

fn gravity_well_apply(p: &mut Player, _: &mut WeaponSlot) {
    p.perks.gravity_well = true;
}
fn gravity_well_eligible(p: &Player, w: &WeaponSlot) -> bool {
    !p.perks.gravity_well && w.has("rift") && w.has("drone")
}

fn resonance_apply(p: &mut Player, _: &mut WeaponSlot) {
    p.perks.resonance = true;
}
fn resonance_eligible(p: &Player, w: &WeaponSlot) -> bool {
    !p.perks.resonance && w.has("wave") && w.has("chain")
}

fn prism_apply(p: &mut Player, _: &mut WeaponSlot) {
    p.perks.prism = true;
}
fn prism_eligible(p: &Player, w: &WeaponSlot) -> bool {
    !p.perks.prism && w.has("reflector") && w.has("laser")
}

static CARD_POOL: std::sync::OnceLock<Vec<Card>> = std::sync::OnceLock::new();

pub fn pool() -> &'static [Card] {
    CARD_POOL.get_or_init(|| {
        vec![
            // 数值（白卡）
            c(
                "fire_rate",
                Rarity::Common,
                "Rapid Fire",
                "Fire rate +15%",
                fire_rate_up,
                fire_rate_eligible,
            ),
            c(
                "damage",
                Rarity::Common,
                "High Caliber",
                "Damage +20%",
                damage_up,
                damage_eligible,
            ),
            c(
                "bullet_speed",
                Rarity::Common,
                "Velocity",
                "Bullet speed +25%",
                bullet_speed_up,
                bullet_speed_eligible,
            ),
            c(
                "move_speed",
                Rarity::Common,
                "Afterburner",
                "Move speed +15%",
                move_speed_up,
                move_speed_eligible,
            ),
            c(
                "pickup_r",
                Rarity::Common,
                "Magnetic Field",
                "Pickup range +50%",
                pickup_radius_up,
                pickup_radius_eligible,
            ),
            c(
                "xp_mul",
                Rarity::Common,
                "Sharp Eyes",
                "XP gain +30%",
                xp_mul_up,
                xp_mul_eligible,
            ),
            c(
                "score_mul",
                Rarity::Common,
                "Bounty Hunter",
                "Score +25%",
                score_mul_up,
                score_mul_eligible,
            ),
            // 数值（蓝卡）
            c(
                "max_hp",
                Rarity::Rare,
                "Hull Plating",
                "Max HP +1",
                max_hp_up,
                max_hp_eligible,
            ),
            c(
                "crit_chance",
                Rarity::Rare,
                "Sniper Lens",
                "Crit chance +10%",
                crit_chance_up,
                crit_chance_eligible,
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
                regen_eligible,
            ),
            c(
                "invincible",
                Rarity::Rare,
                "Adrenaline",
                "I-frames +30%",
                invincible_up,
                invincible_eligible,
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
                "Orbit drones aim at nearby targets",
                unlock_drone,
                unlock_drone_eligible,
            ),
            c(
                "u_laser",
                Rarity::Epic,
                "Pulse Laser",
                "Tracking beam, sustained DPS",
                unlock_laser,
                unlock_laser_eligible,
            ),
            c(
                "u_chain",
                Rarity::Epic,
                "Chain Bolt",
                "Long-range lightning jumps targets",
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
            // —— 联动 Perks（金卡）————————————
            c(
                "heat_lock",
                Rarity::Legendary,
                "Heat Lock",
                "Missile marks targets · laser deals bonus damage",
                heat_lock_apply,
                heat_lock_eligible,
            ),
            c(
                "static_mark",
                Rarity::Legendary,
                "Static Mark",
                "Chain-charged targets are guaranteed crits once",
                static_mark_apply,
                static_mark_eligible,
            ),
            c(
                "drone_relay",
                Rarity::Legendary,
                "Drone Relay",
                "Drone kills launch a homing follow-up missile",
                drone_relay_apply,
                drone_relay_eligible,
            ),
            // —— 新武器解锁（紫卡）————————————
            c(
                "u_rift",
                Rarity::Epic,
                "Void Rift",
                "Hunting damage field",
                unlock_rift,
                unlock_rift_eligible,
            ),
            c(
                "u_wave",
                Rarity::Epic,
                "Wave Cannon",
                "Sine-wave bullets sweep the field",
                unlock_wave,
                unlock_wave_eligible,
            ),
            c(
                "u_reflector",
                Rarity::Epic,
                "Reflector",
                "Aimed ricochet shots",
                unlock_reflector,
                unlock_reflector_eligible,
            ),
            // —— 新武器升级（蓝卡）————————————
            c(
                "rift_up",
                Rarity::Rare,
                "Rift +1",
                "More rifts · faster pulses · wider",
                rift_up,
                rift_up_eligible,
            ),
            c(
                "wave_up",
                Rarity::Rare,
                "Wave +1",
                "More waves · amplitude · speed",
                wave_up,
                wave_up_eligible,
            ),
            c(
                "reflector_up",
                Rarity::Rare,
                "Reflector +1",
                "More shots · bounces · speed",
                reflector_up,
                reflector_up_eligible,
            ),
            c(
                "gravity_well",
                Rarity::Legendary,
                "Gravity Well",
                "Rifts slowly pull enemies inward",
                gravity_well_apply,
                gravity_well_eligible,
            ),
            c(
                "resonance",
                Rarity::Legendary,
                "Resonance",
                "Wave + Chain: hits trigger extra jumps",
                resonance_apply,
                resonance_eligible,
            ),
            c(
                "prism",
                Rarity::Legendary,
                "Prism",
                "Reflector + Laser: bounce through beam = +50% dmg & pierce",
                prism_apply,
                prism_eligible,
            ),
        ]
    })
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
pub fn draw_n(n: usize, player: &mut Player, weapons: &WeaponSlot) -> Vec<Card> {
    use ::rand::Rng;
    let mut rng = thread_rng();
    let mut pool: Vec<Card> = pool()
        .iter()
        .filter(|c| (c.eligible)(player, weapons))
        .cloned()
        .collect();
    if pool.is_empty() {
        return vec![];
    }

    // 保底：连续 5 次未见副武器解锁卡，且还有空槽 → 强制出一张
    let unlock_count = pool.iter().filter(|c| c.id.starts_with("u_")).count();
    let pity_trigger = player.perks.pity_unlock >= 5 && weapons.subs.len() < 4 && unlock_count > 0;

    // 加权随机（带去重）
    let mut picks: Vec<Card> = Vec::with_capacity(n);

    // 保底触发：从池中随机选一张解锁卡
    if pity_trigger {
        let unlock_indices: Vec<usize> = pool
            .iter()
            .enumerate()
            .filter(|(_, c)| c.id.starts_with("u_"))
            .map(|(i, _)| i)
            .collect();
        if !unlock_indices.is_empty() {
            let idx = unlock_indices[rng.gen_range(0..unlock_indices.len())];
            picks.push(pool.swap_remove(idx));
            player.perks.pity_unlock = 0;
        }
    }

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

    // 更新保底计数：本轮出了解锁卡则重置，否则 +1
    let got_unlock = picks.iter().any(|c| c.id.starts_with("u_"));
    if got_unlock {
        player.perks.pity_unlock = 0;
    } else {
        player.perks.pity_unlock = player.perks.pity_unlock.saturating_add(1);
    }

    picks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Player;
    use crate::ship::ShipType;

    fn test_player() -> Player {
        Player::with_ship(ShipType::Vanguard)
    }

    #[test]
    fn capped_stat_cards_become_ineligible() {
        let mut player = test_player();
        let weapons = WeaponSlot::new();

        player.stats.fire_rate = 0.18;
        player.stats.damage_mul = 2.35;
        player.stats.bullet_speed = 1200.0;
        player.stats.speed = 2200.0;
        player.stats.attract_radius = 230.0;
        player.stats.crit_chance = 0.35;
        player.stats.crit_mul = 2.8;
        player.stats.score_mul = 2.2;
        player.stats.xp_mul = 2.0;
        player.stats.regen_per_min = 1.2;
        player.stats.invincible = 1.9;
        player.perks.hull_plating_picks = 2;

        let ids: Vec<&str> = pool()
            .iter()
            .filter(|c| (c.eligible)(&player, &weapons))
            .map(|c| c.id)
            .collect();

        for capped in [
            "fire_rate",
            "damage",
            "bullet_speed",
            "move_speed",
            "pickup_r",
            "crit_chance",
            "crit_dmg",
            "score_mul",
            "xp_mul",
            "regen",
            "invincible",
            "max_hp",
        ] {
            assert!(!ids.contains(&capped), "{capped} should be filtered out");
        }
    }

    #[test]
    fn pity_forces_unlock_when_slots_are_available() {
        let mut player = test_player();
        player.perks.pity_unlock = 5;
        let weapons = WeaponSlot::new();

        let cards = draw_n(3, &mut player, &weapons);

        assert!(cards.iter().any(|c| c.id.starts_with("u_")));
        assert_eq!(player.perks.pity_unlock, 0);
    }

    #[test]
    fn full_subweapon_slots_filter_unlock_cards() {
        let player = test_player();
        let mut weapons = WeaponSlot::new();
        weapons.subs.push(Box::new(Missile::new()));
        weapons.subs.push(Box::new(Drone::new()));
        weapons.subs.push(Box::new(Laser::new()));
        weapons.subs.push(Box::new(Chain::new()));

        let ids: Vec<&str> = pool()
            .iter()
            .filter(|c| (c.eligible)(&player, &weapons))
            .map(|c| c.id)
            .collect();

        assert!(!ids.iter().any(|id| id.starts_with("u_")));
    }
}
