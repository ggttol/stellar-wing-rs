//! 战斗解算：寻的、子弹↔敌人/玩家、击杀处理、超必杀、玩家近身碰撞。

use macroquad::prelude::*;

use crate::audio::Audio;
use crate::collision::{bullet_hits_enemy, bullet_hits_player, hit_circle};
use crate::entity::{Bullet, Enemy, EnemyKind, HitSource, PickupKind, Player};
use crate::fx::Fx;
use crate::lang::{t, Lang};
use crate::spawn::{drop_xp_gems, maybe_drop_special};
use crate::world::World;

/// 寻的子弹朝最近敌人转向并加速。
pub fn steer_homing_bullets(world: &mut World, dt: f32) {
    for b in &mut world.bullets {
        if !b.homing || b.dead {
            continue;
        }
        steer_one(b, &world.enemies, dt);
    }
}

fn steer_one(b: &mut Bullet, enemies: &[Enemy], dt: f32) {
    let mut best: Option<(f32, f32)> = None;
    let mut best_d2 = f32::MAX;
    for e in enemies {
        if e.dead {
            continue;
        }
        let dx = e.x - b.x;
        let dy = e.y - b.y;
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            best_d2 = d2;
            best = Some((dx, dy));
        }
    }
    let Some((dx, dy)) = best else {
        return;
    };
    let d = (dx * dx + dy * dy).sqrt().max(1.0);
    let cur_speed = (b.vx * b.vx + b.vy * b.vy).sqrt().max(1.0);
    let blend = (8.0 * dt).min(1.0);
    let nx = b.vx / cur_speed * (1.0 - blend) + dx / d * blend;
    let ny = b.vy / cur_speed * (1.0 - blend) + dy / d * blend;
    let nl = (nx * nx + ny * ny).sqrt().max(1e-3);
    let target_speed = (cur_speed + 240.0 * dt).min(620.0);
    b.vx = nx / nl * target_speed;
    b.vy = ny / nl * target_speed;
}

/// 玩家子弹打敌人，结算伤害与暴击。
pub fn resolve_player_bullets(world: &mut World, fx: &mut Fx, audio: &Audio, t: f32, dt: f32) {
    let crit_mul = world.player.stats.crit_mul;
    let overload_mul = world.synergy.damage_mul();
    // combo 伤害加成：50 连 5%，100 连 10%
    let combo_dmg = if world.combo >= 100 {
        1.10
    } else if world.combo >= 50 {
        1.05
    } else {
        1.0
    };
    // 副武器递减：≥3 个副武器时所有副武器伤害打折
    let sub_penalty = world.weapons.sub_penalty();
    // 无尽模式每圈伤害加成
    let endless_bonus = 1.0 + world.endless_damage_bonus;
    if world.hit_sfx_cooldown > 0.0 {
        world.hit_sfx_cooldown -= dt;
    }
    for b in &mut world.bullets {
        if b.dead || !b.from_player {
            continue;
        }
        for e in &mut world.enemies {
            if e.dead {
                continue;
            }
            if !bullet_hits_enemy(b, e) {
                continue;
            }
            let mut damage = b.damage * e.damage_mul() * overload_mul * combo_dmg * endless_bonus;
            // 副武器递减（主武器和敌方子弹不受影响）
            if b.source != HitSource::MainGun && b.source != HitSource::Enemy {
                damage *= sub_penalty;
            }
            if e.static_mark && !b.is_crit && b.source != HitSource::Enemy {
                damage *= crit_mul;
                b.is_crit = true;
                e.static_mark = false;
            }
            // Prism：Reflector 弹丸穿过激光束时 +50% 伤害 & 穿透 +1
            if b.source == HitSource::Reflector && world.player.perks.prism && !b.prism_boosted {
                let in_beam = (b.x - world.player.x).abs() < 22.0 && b.y < world.player.y;
                if in_beam {
                    damage *= 1.5;
                    b.pierce = b.pierce.saturating_add(1);
                    b.prism_boosted = true;
                }
            }
            e.hp -= damage;
            e.hit_flash = 0.08;
            e.last_hit = b.source;
            if b.source == HitSource::Missile {
                e.marked_until = t + 2.0;
            }
            // Wave Cannon 标记（配合 Resonance 联动）
            if b.source == HitSource::Wave {
                e.wave_marked = true;
            }
            fx.burst(
                b.x,
                b.y,
                4,
                2.0,
                Color::from_rgba(125, 249, 255, 255),
                120.0,
            );
            if world.hit_sfx_cooldown <= 0.0 {
                audio.play_hit();
                world.hit_sfx_cooldown = 0.05;
            }
            if b.pierce == 0 {
                b.dead = true;
                break;
            } else {
                b.pierce -= 1;
            }
        }
    }
}

/// 处理本帧"刚被打死"的敌人：分数、combo、超表回收、爆破、掉落。
/// 返回是否有 boss 在本帧死亡。
pub fn process_kills(world: &mut World, fx: &mut Fx, audio: &Audio, t: f32) -> bool {
    let score_mul = world.player.stats.score_mul;
    let mut boss_died = false;

    // 先把刚阵亡的敌人收集出来（按引用处理，回头再 retain），
    // 这样我们能调用 spawn_relay_missile 同时持有 &mut world.bullets。
    for i in 0..world.enemies.len() {
        let e = &mut world.enemies[i];
        if e.dead || e.hp > 0.0 {
            continue;
        }
        e.dead = true;
        // 共鸣槽填充。返回 true 表示这一击触发了过载。
        let triggered_overload = world.synergy.add_kill(e.kind);
        if triggered_overload {
            world.overload_flash = 1.0;
            audio.play_super(); // 过载触发的"轰"音
            fx.explode(
                world.player.x,
                world.player.y,
                2.2,
                Color::from_rgba(255, 220, 110, 255),
            );
        }
        world.combo = world.combo.saturating_add(1);
        world.combo_timer = 1.2;
        world.combo_flash = 0.4;
        let combo_mul = if world.combo >= 30 {
            1.5
        } else if world.combo >= 15 {
            1.25
        } else if world.combo >= 5 {
            1.1
        } else {
            1.0
        };
        world.score += ((e.score as f32) * score_mul * combo_mul) as u32;
        world.super_charge = (world.super_charge
            + match e.kind {
                EnemyKind::Small => 0.018,
                EnemyKind::Medium => 0.035,
                EnemyKind::Large => 0.08,
                EnemyKind::Boss => 0.3,
                EnemyKind::Kamikaze => 0.030,
                EnemyKind::Strafer => 0.045,
                EnemyKind::Sniper => 0.050,
                EnemyKind::Weaver => 0.042,
                EnemyKind::MineLayer => 0.070,
            })
        .min(1.0);
        if world.combo.is_multiple_of(10) {
            world.super_charge = (world.super_charge + 0.05).min(1.0);
        }
        let (scale, color, big) = match e.kind {
            EnemyKind::Small => (0.9, Color::from_rgba(255, 136, 102, 255), false),
            EnemyKind::Medium => (1.3, Color::from_rgba(201, 124, 255, 255), false),
            EnemyKind::Large => (2.0, Color::from_rgba(255, 77, 109, 255), true),
            EnemyKind::Boss => (4.0, Color::from_rgba(255, 90, 140, 255), true),
            EnemyKind::Kamikaze => (1.1, Color::from_rgba(255, 100, 130, 255), false),
            EnemyKind::Strafer => (1.4, Color::from_rgba(125, 220, 255, 255), false),
            EnemyKind::Sniper => (1.5, Color::from_rgba(255, 206, 96, 255), false),
            EnemyKind::Weaver => (1.35, Color::from_rgba(92, 240, 210, 255), false),
            EnemyKind::MineLayer => (1.8, Color::from_rgba(255, 158, 76, 255), true),
        };
        fx.explode(e.x, e.y, scale, color);
        drop_xp_gems(&mut world.pickups, e);
        maybe_drop_special(&mut world.pickups, e, t);

        let relay = world.player.perks.drone_relay && e.last_hit == HitSource::Drone;
        let kind = e.kind;
        let (ex, ey) = (e.x, e.y);
        let raw_score = ((e.score as f32) * score_mul * combo_mul) as u32;

        if relay {
            spawn_relay_missile(&mut world.bullets, &world.player, ex, ey);
        }
        if big {
            fx.float_text(
                ex - 18.0,
                ey,
                format!("+{}", raw_score),
                Color::from_rgba(255, 209, 102, 255),
                if matches!(kind, EnemyKind::Boss) {
                    28.0
                } else {
                    22.0
                },
            );
            audio.play_explode_big();
        } else {
            audio.play_kill_combo(world.combo);
            if matches!(kind, EnemyKind::Large) {
                audio.play_explode_small();
            }
        }
        if matches!(kind, EnemyKind::Boss) {
            boss_died = true;
        }
    }

    boss_died
}

/// 敌人子弹打玩家。
pub fn resolve_enemy_bullets(world: &mut World, audio: &Audio, t: f32) {
    if world.player.dead {
        return;
    }
    for b in &mut world.bullets {
        if b.dead || b.from_player {
            continue;
        }
        if bullet_hits_player(b, &world.player) {
            b.dead = true;
            if world.player.hit(t) {
                audio.play_hurt();
            }
        }
    }
}

/// 敌人和玩家近身碰撞。
pub fn resolve_enemy_player_contact(world: &mut World, audio: &Audio, t: f32) {
    if world.player.dead {
        return;
    }
    for e in &mut world.enemies {
        if e.dead {
            continue;
        }
        if hit_circle(
            e.x,
            e.y,
            e.radius,
            world.player.x,
            world.player.y,
            world.player.radius,
        ) {
            if world.player.hit(t) {
                audio.play_hurt();
            }
            if !matches!(e.kind, EnemyKind::Boss) {
                e.telegraph = 0.3;
                e.vy *= 0.6;
                let push = (e.x - world.player.x).signum();
                e.x += push * 18.0;
            }
        }
    }
}

/// 玩家按下空格触发的清屏 super 爆炸。
pub fn trigger_super(world: &mut World, fx: &mut Fx, audio: &Audio) {
    world.super_charge = 0.0;
    for b in &mut world.bullets {
        if !b.from_player {
            b.dead = true;
        }
    }
    for e in &mut world.enemies {
        if e.dead {
            continue;
        }
        let dmg = if matches!(e.kind, EnemyKind::Boss) {
            e.max_hp * 0.06
        } else {
            6.0
        };
        e.hp -= dmg;
        e.hit_flash = 0.12;
        e.last_hit = HitSource::MainGun;
    }
    fx.explode(
        world.player.x,
        world.player.y,
        3.5,
        Color::from_rgba(125, 249, 255, 255),
    );
    audio.play_super();
}

/// 拾取处理，返回新增的 XP（已乘倍率）。
pub fn collect_pickups(world: &mut World, fx: &mut Fx, dt: f32, lang: Lang) {
    let pr = world.player.stats.pickup_radius;
    let ar = world.player.attract_radius_at(world.run_time);
    let px = world.player.x;
    let py = world.player.y;
    let mut gained_xp_raw: u32 = 0;
    let mut gained_specials: Vec<PickupKind> = Vec::new();
    world.pickups.retain_mut(|g| {
        if g.update(dt, px, py, ar, pr) {
            if g.kind == PickupKind::Xp {
                gained_xp_raw += g.value;
            } else {
                gained_specials.push(g.kind);
            }
            return false;
        }
        !g.dead
    });
    if gained_xp_raw > 0 {
        let mul = world.player.stats.xp_mul;
        let gained = ((gained_xp_raw as f32) * mul) as u32;
        world.xp += gained.max(1);
        fx.float_text(
            px,
            py - 28.0,
            format!("+{} XP", gained.max(1)),
            Color::from_rgba(125, 249, 255, 255),
            14.0,
        );
    }
    for special in gained_specials {
        match special {
            PickupKind::Xp => {}
            PickupKind::Heal => {
                world.player.lives = (world.player.lives + 1).min(world.player.stats.max_lives);
                fx.float_text(
                    px,
                    py - 46.0,
                    "+1 HP",
                    Color::from_rgba(118, 255, 122, 255),
                    14.0,
                );
            }
            PickupKind::Magnet => {
                world.player.magnet_until = world.run_time + 8.0;
                fx.float_text(
                    px,
                    py - 46.0,
                    t("MAGNET", lang),
                    Color::from_rgba(255, 120, 210, 255),
                    14.0,
                );
            }
            PickupKind::Ammo => {
                world.super_charge = (world.super_charge + 0.25).min(1.0);
                fx.float_text(
                    px,
                    py - 46.0,
                    t("+SUPER", lang),
                    Color::from_rgba(255, 180, 80, 255),
                    14.0,
                );
            }
            PickupKind::Barrier => {
                world.player.shield = true;
                fx.float_text(
                    px,
                    py - 46.0,
                    t("SHIELD", lang),
                    Color::from_rgba(125, 200, 255, 255),
                    14.0,
                );
            }
        }
    }
}

fn spawn_relay_missile(bullets: &mut Vec<Bullet>, player: &Player, x: f32, y: f32) {
    let mut b = Bullet::player_shot(x, y, 0.0, -260.0);
    b.damage = player.stats.damage_mul * 1.2;
    b.homing = true;
    b.w = 5.0;
    b.h = 10.0;
    b.source = HitSource::Missile;
    bullets.push(b);
}
