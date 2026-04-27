//! 敌人生成与掉落。

use ::rand::{thread_rng, Rng};

use crate::config::CFG;
use crate::entity::{BossMod, EliteMod, Enemy, EnemyKind, Pickup, PickupKind};
use crate::world::World;

pub fn spawn_normals(world: &mut World, dt: f32, t: f32) {
    let rt = world.run_time;
    let lerp = |t01: f32, a: f32, b: f32| -> f32 { a + (b - a) * t01.clamp(0.0, 1.0) };

    let sm_intv = lerp(rt / 120.0, 1.4, 0.50);
    let md_intv = if rt < 12.0 {
        f32::INFINITY
    } else {
        lerp((rt - 12.0) / 90.0, 3.0, 1.4)
    };
    let lg_intv = if rt < 40.0 {
        f32::INFINITY
    } else {
        lerp((rt - 40.0) / 120.0, 7.0, 3.0)
    };

    world.spawn.small += dt;
    world.spawn.medium += dt;
    world.spawn.large += dt;

    let mut rng = thread_rng();
    if world.spawn.small >= sm_intv {
        world.spawn.small = 0.0;
        let x = rng.gen_range(40.0..(CFG.w - 40.0));
        world.enemies.push(spawn_enemy(EnemyKind::Small, x, t, rt));
    }
    if world.spawn.medium >= md_intv {
        world.spawn.medium = 0.0;
        let x = rng.gen_range(60.0..(CFG.w - 60.0));
        world.enemies.push(spawn_enemy(EnemyKind::Medium, x, t, rt));
    }
    if world.spawn.large >= lg_intv {
        world.spawn.large = 0.0;
        let x = rng.gen_range(80.0..(CFG.w - 80.0));
        world.enemies.push(spawn_enemy(EnemyKind::Large, x, t, rt));
    }
}

pub fn spawn_enemy(kind: EnemyKind, x: f32, t: f32, run_time: f32) -> Enemy {
    let mut enemy = Enemy::new(kind, x, t);
    let hp_mul = (1.0 + run_time / 95.0).min(4.0);
    let score_mul = (1.0 + run_time / 180.0).min(2.4);
    enemy.hp *= hp_mul;
    enemy.max_hp = enemy.hp;
    enemy.score = ((enemy.score as f32) * score_mul) as u32;
    enemy.xp = ((enemy.xp as f32) * (1.0 + run_time / 220.0)).ceil() as u32;
    if run_time > 90.0 {
        enemy.fire_rate *= 0.92;
    }
    if run_time >= 25.0 && !matches!(kind, EnemyKind::Boss) {
        let mut rng = thread_rng();
        let elite_chance = match kind {
            EnemyKind::Small => 0.07,
            EnemyKind::Medium => 0.12,
            EnemyKind::Large => 0.18,
            EnemyKind::Boss => 0.0,
        };
        if rng.gen::<f32>() < elite_chance {
            let roll = rng.gen_range(0..3);
            let elite_mod = match roll {
                0 => EliteMod::Armored,
                1 => EliteMod::Berserk,
                _ => EliteMod::Dasher,
            };
            enemy = enemy.into_elite(elite_mod);
        }
    }
    enemy
}

pub fn spawn_boss(x: f32, t: f32) -> Enemy {
    let mut rng = thread_rng();
    let boss_mod = match rng.gen_range(0..4) {
        0 => BossMod::Frenzied,
        1 => BossMod::Bulwark,
        2 => BossMod::Summoner,
        _ => BossMod::StormCore,
    };
    let mut boss = Enemy::new(EnemyKind::Boss, x, t);
    boss.hp *= 1.35;
    boss.max_hp = boss.hp;
    boss.into_boss_mod(boss_mod)
}

pub fn drop_xp_gems(pickups: &mut Vec<Pickup>, e: &Enemy) {
    let pieces = match e.kind {
        EnemyKind::Small => 1,
        EnemyKind::Medium => 2,
        EnemyKind::Large => 4,
        EnemyKind::Boss => 16,
    };
    let per = (e.xp / pieces.max(1)).max(1);
    let mut rng = thread_rng();
    for _ in 0..pieces {
        let ox: f32 = rng.gen_range(-18.0..18.0);
        let oy: f32 = rng.gen_range(-12.0..12.0);
        pickups.push(Pickup::xp(e.x + ox, e.y + oy, per));
    }
}

pub fn maybe_drop_special(pickups: &mut Vec<Pickup>, e: &Enemy, t: f32) {
    let mut rng = thread_rng();
    let drop_roll = if e.is_elite {
        1.0
    } else {
        match e.kind {
            EnemyKind::Small => 0.0,
            EnemyKind::Medium => 0.0,
            EnemyKind::Large => 0.18,
            EnemyKind::Boss => 1.0,
        }
    };
    if rng.gen::<f32>() > drop_roll {
        return;
    }
    let drops = if matches!(e.kind, EnemyKind::Boss) {
        2
    } else {
        1
    };
    for i in 0..drops {
        let kind = match (t as i32 + i) % 4 {
            0 => PickupKind::Heal,
            1 => PickupKind::Magnet,
            2 => PickupKind::Ammo,
            _ => PickupKind::Barrier,
        };
        pickups.push(Pickup::special(
            e.x + rng.gen_range(-10.0..10.0),
            e.y + rng.gen_range(-10.0..10.0),
            kind,
        ));
    }
}
