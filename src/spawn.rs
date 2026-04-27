//! 敌人生成与掉落。

use ::rand::{thread_rng, Rng};

use crate::chapter;
use crate::config::CFG;
use crate::entity::{EliteMod, Enemy, EnemyKind, Pickup, PickupKind};
use crate::world::World;

pub fn spawn_chapter_wave(world: &mut World, dt: f32, t: f32) {
    let chap = chapter::get(world.chapter_idx);
    let rt = world.run_time;
    let intensity = chap.spawn_intensity;
    let lerp = |t01: f32, a: f32, b: f32| -> f32 { a + (b - a) * t01.clamp(0.0, 1.0) };

    // 章节内时钟主导基础密度；越后章节密度越大。
    let chap_t = world.chapter_time;
    let sm_intv = lerp(chap_t / 90.0, 1.4, 0.50) / intensity;
    let md_intv = if chap_t < 8.0 {
        f32::INFINITY
    } else {
        lerp((chap_t - 8.0) / 70.0, 3.0, 1.4) / intensity
    };
    let lg_intv = if chap_t < 30.0 {
        f32::INFINITY
    } else {
        lerp((chap_t - 30.0) / 100.0, 7.0, 3.0) / intensity
    };

    world.spawn.small += dt;
    world.spawn.medium += dt;
    world.spawn.large += dt;

    let mut rng = thread_rng();
    if world.spawn.small >= sm_intv {
        world.spawn.small = 0.0;
        let x = rng.gen_range(40.0..(CFG.w - 40.0));
        let kind = if rng.gen::<f32>() < chap.kamikaze_chance {
            EnemyKind::Kamikaze
        } else {
            EnemyKind::Small
        };
        let mul = endless_extra_mul(world);
        let mut e = spawn_one(kind, x, t, rt, &(world.player.x, world.player.y));
        apply_endless_scaling(&mut e, mul);
        world.enemies.push(e);
    }
    if world.spawn.medium >= md_intv {
        world.spawn.medium = 0.0;
        let x = rng.gen_range(60.0..(CFG.w - 60.0));
        let mul = endless_extra_mul(world);
        let mut e = spawn_one(EnemyKind::Medium, x, t, rt, &(world.player.x, world.player.y));
        apply_endless_scaling(&mut e, mul);
        world.enemies.push(e);
    }
    if world.spawn.large >= lg_intv {
        world.spawn.large = 0.0;
        let x = rng.gen_range(80.0..(CFG.w - 80.0));
        let mul = endless_extra_mul(world);
        let mut e = spawn_one(EnemyKind::Large, x, t, rt, &(world.player.x, world.player.y));
        apply_endless_scaling(&mut e, mul);
        world.enemies.push(e);
    }

    // Strafer：章节级独立间隔
    if chap.strafer_interval > 0.0 {
        world.strafer_timer += dt;
        if world.strafer_timer >= chap.strafer_interval / intensity {
            world.strafer_timer = 0.0;
            let mul = endless_extra_mul(world);
            let mut e = spawn_strafer(t, rt);
            apply_endless_scaling(&mut e, mul);
            world.enemies.push(e);
        }
    }
}

/// 给定 kind、原始 x，按当前难度增益生成一只敌人。Kamikaze 会在此处锁定冲撞向量。
pub fn spawn_one(
    kind: EnemyKind,
    x: f32,
    t: f32,
    run_time: f32,
    player_pos: &(f32, f32),
) -> Enemy {
    let mut e = spawn_enemy(kind, x, t, run_time);
    if matches!(kind, EnemyKind::Kamikaze) {
        // 锁定向玩家位置的方向向量
        let (px, py) = *player_pos;
        let dx = px - e.x;
        let dy = (py - e.y).max(60.0);
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let speed = 280.0 + run_time * 0.4;
        e.vx = dx / len * speed;
        e.vy = dy / len * speed;
    }
    e
}

/// Endless 模式额外的 HP/分数倍率：每跑完一圈线性叠加。
pub fn endless_extra_mul(world: &World) -> f32 {
    if !world.is_endless() {
        return 1.0;
    }
    let lap = (world.chapter_idx as i64 - chapter::CHAPTERS.len() as i64).max(0) as f32 + 1.0;
    1.0 + lap * 0.35
}

/// 在 spawn_one 之后给 endless 局再叠一层。
pub fn apply_endless_scaling(e: &mut Enemy, mul: f32) {
    if mul <= 1.0 {
        return;
    }
    e.hp *= mul;
    e.max_hp = e.hp;
    e.score = ((e.score as f32) * mul.sqrt()) as u32;
    e.xp = ((e.xp as f32) * mul.sqrt()).ceil() as u32;
}

fn spawn_strafer(t: f32, run_time: f32) -> Enemy {
    use ::rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    let from_left = rng.gen::<bool>();
    let speed = 220.0 + run_time * 0.35;
    let x = if from_left { -30.0 } else { CFG.w + 30.0 };
    let mut e = Enemy::new(EnemyKind::Strafer, x, t);
    e.y = rng.gen_range(80.0..220.0);
    e.vx = if from_left { speed } else { -speed };
    e.vy = 0.0;
    let hp_mul = (1.0 + run_time / 95.0).min(4.0);
    e.hp *= hp_mul;
    e.max_hp = e.hp;
    e.score = ((e.score as f32) * (1.0 + run_time / 180.0)) as u32;
    e
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
            // Kamikaze / Strafer 已经是"特殊敌人"，不再叠加 elite。
            EnemyKind::Kamikaze | EnemyKind::Strafer => 0.0,
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

/// 按当前章节配置生成 Boss(s)：故事章节单 Boss，无尽双 Boss。
/// 把 Boss 直接 push 到 world.enemies。
pub fn spawn_chapter_boss(world: &mut World, t: f32) {
    let chap = chapter::get(world.chapter_idx);
    let mut rng = thread_rng();
    let pool = chap.boss_pool;
    let mut pick_mod = || pool[rng.gen_range(0..pool.len())];

    let scaled_hp_mul = chapter_boss_hp_mul(world.chapter_idx);

    if chap.endless {
        // 双 Boss：左右站位
        for x in [CFG.w * 0.30, CFG.w * 0.70] {
            let mut boss = Enemy::new(EnemyKind::Boss, x, t);
            boss.hp *= 1.20 * scaled_hp_mul;
            boss.max_hp = boss.hp;
            world.enemies.push(boss.into_boss_mod(pick_mod()));
        }
    } else {
        let mut boss = Enemy::new(EnemyKind::Boss, CFG.w * 0.5, t);
        boss.hp *= 1.35 * scaled_hp_mul;
        boss.max_hp = boss.hp;
        world.enemies.push(boss.into_boss_mod(pick_mod()));
    }
}

/// Boss HP 随章节进阶；故事章节封顶 ×3，无尽继续指数增长。
fn chapter_boss_hp_mul(chapter_idx: u32) -> f32 {
    if (chapter_idx as usize) < chapter::CHAPTERS.len() {
        1.0 + chapter_idx as f32 * 0.45
    } else {
        let endless_lap = (chapter_idx as f32) - chapter::CHAPTERS.len() as f32 + 1.0;
        3.25 * 1.25_f32.powf(endless_lap)
    }
}

pub fn drop_xp_gems(pickups: &mut Vec<Pickup>, e: &Enemy) {
    let pieces = match e.kind {
        EnemyKind::Small => 1,
        EnemyKind::Medium => 2,
        EnemyKind::Large => 4,
        EnemyKind::Boss => 16,
        EnemyKind::Kamikaze => 1,
        EnemyKind::Strafer => 2,
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
            EnemyKind::Small | EnemyKind::Kamikaze => 0.0,
            EnemyKind::Medium | EnemyKind::Strafer => 0.05,
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
