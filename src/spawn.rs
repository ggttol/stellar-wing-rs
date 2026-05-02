//! 敌人生成与掉落。

use ::rand::{thread_rng, Rng};

use crate::chapter;
use crate::config::CFG;
use crate::entity::{BuffKind, EliteMod, Enemy, EnemyKind, Pickup, PickupKind};
use crate::world::World;

pub fn spawn_chapter_wave(world: &mut World, dt: f32, t: f32) {
    let chap = chapter::get(world.chapter_idx);
    let rt = world.run_time;
    let intensity = chap.spawn_intensity;
    let lerp = |t01: f32, a: f32, b: f32| -> f32 { a + (b - a) * t01.clamp(0.0, 1.0) };

    // 章节内时钟主导基础密度；越后章节密度越大。
    let chap_t = world.chapter_time;
    let sm_intv = (lerp(chap_t / 90.0, 1.4, 0.50) / intensity).min(0.75); // 防止章节开头散步真空期
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
        } else if world.run_time >= 18.0 && rng.gen::<f32>() < 0.18 {
            EnemyKind::Weaver
        } else {
            EnemyKind::Small
        };
        let mul = endless_extra_mul(world);
        let mut e = spawn_one_full(
            kind,
            x,
            t,
            rt,
            &(world.player.x, world.player.y),
            world.difficulty,
            world.chapter_modifier,
        );
        apply_endless_scaling(&mut e, mul);
        world.enemies.push(e);
    }
    if world.spawn.medium >= md_intv {
        world.spawn.medium = 0.0;
        let x = rng.gen_range(60.0..(CFG.w - 60.0));
        let mul = endless_extra_mul(world);
        let kind = if world.run_time >= 45.0 && rng.gen::<f32>() < 0.22 {
            EnemyKind::Sniper
        } else {
            EnemyKind::Medium
        };
        let mut e = spawn_one_full(
            kind,
            x,
            t,
            rt,
            &(world.player.x, world.player.y),
            world.difficulty,
            world.chapter_modifier,
        );
        apply_endless_scaling(&mut e, mul);
        world.enemies.push(e);
    }
    if world.spawn.large >= lg_intv {
        world.spawn.large = 0.0;
        let x = rng.gen_range(80.0..(CFG.w - 80.0));
        let mul = endless_extra_mul(world);
        let kind = if world.run_time >= 60.0 && rng.gen::<f32>() < 0.24 {
            EnemyKind::MineLayer
        } else {
            EnemyKind::Large
        };
        let mut e = spawn_one_full(
            kind,
            x,
            t,
            rt,
            &(world.player.x, world.player.y),
            world.difficulty,
            world.chapter_modifier,
        );
        apply_endless_scaling(&mut e, mul);
        world.enemies.push(e);
    }

    // Strafer：章节级独立间隔
    if chap.strafer_interval > 0.0 {
        world.strafer_timer += dt;
        if world.strafer_timer >= chap.strafer_interval / intensity {
            world.strafer_timer = 0.0;
            let mul = endless_extra_mul(world);
            let mut e = spawn_strafer(t, rt, world.difficulty);
            apply_endless_scaling(&mut e, mul);
            world.enemies.push(e);
        }
    }
}


pub fn spawn_one_full(
    kind: EnemyKind,
    x: f32,
    t: f32,
    run_time: f32,
    player_pos: &(f32, f32),
    difficulty: u8,
    chap_mod: crate::world::ChapterMod,
) -> Enemy {
    let mut e = spawn_enemy_full(kind, x, t, run_time, difficulty, chap_mod);
    if matches!(kind, EnemyKind::Kamikaze) {
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

fn spawn_strafer(t: f32, run_time: f32, difficulty: u8) -> Enemy {
    use ::rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    let from_left = rng.gen::<bool>();
    let speed = 220.0 + run_time * 0.35;
    let x = if from_left { -30.0 } else { CFG.w + 30.0 };
    let mut e = Enemy::new(EnemyKind::Strafer, x, t);
    e.y = rng.gen_range(80.0..220.0);
    e.vx = if from_left { speed } else { -speed };
    e.vy = 0.0;
    let (d_hp, d_bspd, _, _) = World::difficulty_mods(difficulty);
    let hp_mul = (1.0 + run_time / 55.0) * d_hp;
    let warmup = (run_time / 120.0).clamp(0.0, 1.0);
    e.hp *= hp_mul;
    e.bullet_damage = 1.0 + run_time / 100.0;
    e.bullet_speed_mul = (0.58 + warmup * 0.42) * d_bspd;
    e.fire_rate *= 1.25 - warmup * 0.25;
    e.max_hp = e.hp;
    e.score = ((e.score as f32) * (1.0 + run_time / 180.0)) as u32;
    e
}

pub fn spawn_enemy_full(
    kind: EnemyKind,
    x: f32,
    t: f32,
    run_time: f32,
    difficulty: u8,
    chap_mod: crate::world::ChapterMod,
) -> Enemy {
    let mut enemy = Enemy::new(kind, x, t);
    let (d_hp, d_bspd, _xp, _score) = World::difficulty_mods(difficulty);
    let hp_mul = (1.0 + run_time / 55.0) * d_hp * chap_mod.hp_mul();
    let score_mul = 1.0 + run_time / 180.0; // 分数倍率也不再封顶
    let warmup = (run_time / 120.0).clamp(0.0, 1.0);
    enemy.hp *= hp_mul;
    enemy.bullet_damage = 1.0 + run_time / 100.0; // 敌方子弹伤害随时间增长
    enemy.bullet_speed_mul = (0.58 + warmup * 0.42) * d_bspd;
    enemy.fire_rate *= 1.35 - warmup * 0.35;
    enemy.max_hp = enemy.hp;
    let (_, _, d_xp, d_score) = World::difficulty_mods(difficulty);
    enemy.score =
        ((enemy.score as f32) * score_mul * d_score * chap_mod.score_mul()) as u32;
    enemy.xp = ((enemy.xp as f32) * (1.0 + run_time / 220.0) * d_xp).ceil() as u32;
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
            EnemyKind::Sniper | EnemyKind::Weaver | EnemyKind::MineLayer => 0.10,
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

    let (d_hp, _, _, _) = World::difficulty_mods(world.difficulty);
    if chap.endless {
        // 双 Boss：左右站位
        for x in [CFG.w * 0.30, CFG.w * 0.70] {
            let mut boss = Enemy::new(EnemyKind::Boss, x, t);
            boss.hp *= 1.20 * scaled_hp_mul * (1.0 + world.run_time / 70.0) * d_hp;
            boss.bullet_damage = 1.0 + world.run_time / 100.0;
            boss.max_hp = boss.hp;
            world.enemies.push(boss.into_boss_mod(pick_mod()));
        }
    } else {
        let mut boss = Enemy::new(EnemyKind::Boss, CFG.w * 0.5, t);
        boss.hp *= 1.35 * scaled_hp_mul * (1.0 + world.run_time / 70.0) * d_hp;
        boss.bullet_damage = 1.0 + world.run_time / 100.0;
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
        EnemyKind::Sniper => 2,
        EnemyKind::Weaver => 2,
        EnemyKind::MineLayer => 3,
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
            EnemyKind::Medium | EnemyKind::Strafer | EnemyKind::Sniper | EnemyKind::Weaver => 0.05,
            EnemyKind::Large | EnemyKind::MineLayer => 0.18,
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

/// 战斗中掉落的"小数值卡"。频次比 `maybe_drop_special` 高，提供持续的微量
/// 增益，省掉了原先升级弹窗里大量的小数值卡。
pub fn maybe_drop_buff(
    pickups: &mut Vec<Pickup>,
    e: &Enemy,
    chap_mod: crate::world::ChapterMod,
) {
    let mut rng = thread_rng();
    let base = match e.kind {
        EnemyKind::Small => 0.06,
        EnemyKind::Kamikaze => 0.05,
        EnemyKind::Medium => 0.13,
        EnemyKind::Strafer | EnemyKind::Sniper | EnemyKind::Weaver => 0.12,
        EnemyKind::Large => 0.30,
        EnemyKind::MineLayer => 0.24,
        EnemyKind::Boss => 1.0, // 自带保底，下面强制多掉几张
    };
    // 精英 ×2、章节修饰按 buff_drop_mul，再 cap 到 1
    let mut chance: f32 = if e.is_elite { base * 2.0 } else { base };
    chance *= chap_mod.buff_drop_mul();
    chance = chance.min(1.0);

    let drops = if matches!(e.kind, EnemyKind::Boss) {
        (4.0 * chap_mod.buff_drop_mul()).round() as u32
    } else if rng.gen::<f32>() < chance {
        1
    } else {
        return;
    };

    for _ in 0..drops {
        let kind = pick_buff_kind(&mut rng);
        let ox = rng.gen_range(-14.0..14.0);
        let oy = rng.gen_range(-10.0..10.0);
        pickups.push(Pickup::buff(e.x + ox, e.y + oy, kind));
    }
}

/// 加权随机选一种 buff。FireRate / Damage 出现得更勤，crit 类相对稀少。
fn pick_buff_kind(rng: &mut impl Rng) -> BuffKind {
    let weights: &[(BuffKind, u32)] = &[
        (BuffKind::FireRate, 18),
        (BuffKind::Damage, 18),
        (BuffKind::BulletSpeed, 14),
        (BuffKind::MoveSpeed, 14),
        (BuffKind::PickupR, 8),
        (BuffKind::XpMul, 8),
        (BuffKind::ScoreMul, 8),
        (BuffKind::CritChance, 6),
        (BuffKind::CritDamage, 6),
    ];
    let total: u32 = weights.iter().map(|(_, w)| *w).sum();
    let mut r = rng.gen_range(0..total);
    for (k, w) in weights {
        if r < *w {
            return *k;
        }
        r -= w;
    }
    BuffKind::Damage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship::ShipType;

    #[test]
    fn endless_extra_multiplier_only_applies_after_story_chapters() {
        let mut world = World::new(ShipType::Vanguard);
        assert_eq!(endless_extra_mul(&world), 1.0);

        world.chapter_idx = chapter::CHAPTERS.len() as u32;
        assert!(endless_extra_mul(&world) > 1.0);
    }

    #[test]
    fn endless_scaling_updates_enemy_rewards_and_hp() {
        let mut enemy = Enemy::new(EnemyKind::Medium, 120.0, 0.0);
        let hp = enemy.hp;
        let score = enemy.score;
        let xp = enemy.xp;

        apply_endless_scaling(&mut enemy, 2.0);

        assert!(enemy.hp > hp);
        assert_eq!(enemy.max_hp, enemy.hp);
        assert!(enemy.score > score);
        assert!(enemy.xp > xp);
    }
}
