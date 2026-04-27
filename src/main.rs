use ::rand::{thread_rng, Rng};
use macroquad::prelude::*;

mod art;
mod audio;
mod bg;
mod collision;
mod config;
mod entity;
mod fx;
mod lang;
mod save;
mod scene;
mod ship;
mod upgrade;
mod weapon;

use art::draw_player_preview;
use audio::Audio;
use config::CFG;
use entity::enemy::TelegraphKind;
use entity::{BossMod, Bullet, EliteMod, Enemy, EnemyKind, HitSource, Pickup, PickupKind, Player};
use lang::{t, Lang};
use save::Save;
use scene::Scene;
use ship::ShipType;
use upgrade::Card;
use weapon::WeaponSlot;

fn window_conf() -> Conf {
    Conf {
        window_title: "Stellar Wing".to_string(),
        window_width: CFG.w as i32,
        window_height: CFG.h as i32,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}

struct SpawnTimers {
    small: f32,
    medium: f32,
    large: f32,
}
impl SpawnTimers {
    fn new() -> Self {
        Self {
            small: 0.0,
            medium: 0.0,
            large: 0.0,
        }
    }
}

struct World {
    player: Player,
    weapons: WeaponSlot,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    pickups: Vec<Pickup>,
    spawn: SpawnTimers,
    score: u32,
    run_time: f32,
    xp: u32,
    level: u32,
    xp_to_next: u32,
    next_boss_at: f32,
    boss_alive: bool,
    super_charge: f32,
    combo: u32,
    combo_timer: f32,
    combo_flash: f32,
    combo_note_idx: usize,
}

impl World {
    fn new(ship: ShipType) -> Self {
        let mut player = Player::with_ship(ship);
        let mut weapons = WeaponSlot::new();
        ship.apply(&mut player, &mut weapons);
        Self {
            player,
            weapons,
            bullets: Vec::with_capacity(512),
            enemies: Vec::with_capacity(64),
            pickups: Vec::with_capacity(64),
            spawn: SpawnTimers::new(),
            score: 0,
            run_time: 0.0,
            xp: 0,
            level: 1,
            xp_to_next: 6,
            next_boss_at: 60.0,
            boss_alive: false,
            super_charge: 0.2,
            combo: 0,
            combo_timer: 0.0,
            combo_flash: 0.0,
            combo_note_idx: 0,
        }
    }
    fn diff_mul(&self) -> f32 {
        (0.85 + self.run_time / 200.0).min(1.8)
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut save_data = save::load();
    save_data.muted = true;
    save::write(&save_data);
    let mut audio_inst = Audio::load(save_data.muted).await;

    if save_data.fullscreen {
        set_fullscreen(true);
    }

    // 尝试加载系统 CJK 字体；失败则降级英文
    let cjk_font = try_load_cjk_font().await;
    if cjk_font.is_none() && save_data.lang == Lang::Zh {
        save_data.lang = Lang::En;
    }

    let mut bg = bg::StarField::new();
    let mut fx = fx::Fx::default();
    let mut scene = Scene::Menu;
    let mut t_acc: f32 = 0.0;
    let mut menu_ship = 0usize;
    let mut world = World::new(ShipType::ALL[menu_ship]);
    let mut card_cursor: usize = 0;

    loop {
        let dt = get_frame_time().min(0.05);
        t_acc += dt;

        bg.update(dt);

        // 全局快捷键
        if is_key_pressed(KeyCode::F) {
            save_data.fullscreen = !save_data.fullscreen;
            set_fullscreen(save_data.fullscreen);
            save::write(&save_data);
        }
        if is_key_pressed(KeyCode::M) {
            audio_inst.toggle_mute();
            save_data.muted = audio_inst.muted;
            save::write(&save_data);
        }
        if is_key_pressed(KeyCode::L) {
            // 字体可用才允许切到中文
            let next = save_data.lang.toggle();
            if next == Lang::Zh && cjk_font.is_none() {
                // 找不到 CJK 字体：保持英文
            } else {
                save_data.lang = next;
                save::write(&save_data);
            }
        }

        let lang = save_data.lang;
        let font = cjk_font.as_ref();

        match &mut scene {
            Scene::Menu => {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    menu_ship = (menu_ship + ShipType::ALL.len() - 1) % ShipType::ALL.len();
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    menu_ship = (menu_ship + 1) % ShipType::ALL.len();
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    world = World::new(ShipType::ALL[menu_ship]);
                    fx = fx::Fx::default();
                    audio_inst.play(&audio_inst.click, 0.6);
                    scene = Scene::Playing;
                }
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                fx.update(dt);
            }
            Scene::Playing => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    scene = Scene::Paused;
                    audio_inst.play(&audio_inst.click, 0.5);
                } else {
                    step_play(&mut world, &mut fx, &audio_inst, dt, t_acc);
                    if world.player.dead {
                        fx.explode(
                            world.player.x,
                            world.player.y,
                            2.5,
                            Color::from_rgba(125, 249, 255, 255),
                        );
                        save_data.push_record(world.score, world.level);
                        save::write(&save_data);
                        audio_inst.play(&audio_inst.gameover, 1.0);
                        scene = Scene::GameOver;
                    } else if world.xp >= world.xp_to_next {
                        world.xp -= world.xp_to_next;
                        world.level += 1;
                        world.xp_to_next = 6 + world.level * 4;
                        let cards = upgrade::draw_n(3, &world.player, &world.weapons);
                        card_cursor = 0;
                        audio_inst.play(&audio_inst.levelup, 0.32);
                        scene = Scene::UpgradePick(cards);
                    }
                }
            }
            Scene::Paused => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    scene = Scene::Playing;
                } else if is_key_pressed(KeyCode::Q) {
                    scene = Scene::Menu;
                }
                fx.update(dt);
            }
            Scene::UpgradePick(cards) => {
                let n = cards.len();
                let mut picked: Option<usize> = None;
                if is_key_pressed(KeyCode::Key1) {
                    picked = Some(0);
                }
                if is_key_pressed(KeyCode::Key2) {
                    picked = Some(1);
                }
                if is_key_pressed(KeyCode::Key3) {
                    picked = Some(2);
                }
                if n > 0 {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        card_cursor = (card_cursor + n - 1) % n;
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        card_cursor = (card_cursor + 1) % n;
                    }
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                        picked = Some(card_cursor);
                    }
                    let (mx, my) = mouse_position();
                    let lx = mx * (CFG.w / screen_width());
                    let ly = my * (CFG.h / screen_height());
                    if let Some(i) = card_at(lx, ly, n) {
                        card_cursor = i;
                        if is_mouse_button_pressed(MouseButton::Left) {
                            picked = Some(i);
                        }
                    }
                }
                if let Some(i) = picked {
                    if i < n {
                        (cards[i].apply)(&mut world.player, &mut world.weapons);
                        fx.float_text(
                            world.player.x,
                            world.player.y - 30.0,
                            t(cards[i].name, lang).to_string(),
                            cards[i].rarity.color(),
                            18.0,
                        );
                        audio_inst.play(&audio_inst.powerup, 0.28);
                        scene = Scene::Playing;
                    }
                }
                fx.update(dt);
            }
            Scene::GameOver => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    world = World::new(world.player.ship);
                    fx = fx::Fx::default();
                    scene = Scene::Playing;
                } else if is_key_pressed(KeyCode::Escape) {
                    scene = Scene::Menu;
                }
                fx.update(dt);
            }
            _ => {
                fx.update(dt);
            }
        }

        clear_background(Color::from_rgba(2, 3, 10, 255));
        bg.draw();

        match &scene {
            Scene::Menu => draw_menu(
                t_acc,
                &save_data,
                &audio_inst,
                ShipType::ALL[menu_ship],
                font,
                lang,
            ),
            Scene::Playing => {
                draw_world(&world, t_acc);
                fx.draw();
                draw_play_hud(&world, save_data.high, font, lang);
            }
            Scene::Paused => {
                draw_world(&world, t_acc);
                fx.draw();
                draw_play_hud(&world, save_data.high, font, lang);
                draw_pause(font, lang);
            }
            Scene::UpgradePick(cards) => {
                draw_world(&world, t_acc);
                fx.draw();
                draw_play_hud(&world, save_data.high, font, lang);
                draw_upgrade_pick(cards, t_acc, card_cursor, font, lang);
            }
            Scene::GameOver => {
                draw_world(&world, t_acc);
                fx.draw();
                draw_play_hud(&world, save_data.high, font, lang);
                draw_gameover(t_acc, &world, &save_data, font, lang);
            }
            _ => {}
        }

        next_frame().await;
    }
}

// ---------- 字体 / 文本工具 ------------------------------------------------

async fn try_load_cjk_font() -> Option<Font> {
    let candidates: &[&str] = &[
        // macOS
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
        // Windows
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/msyh.ttf",
        "C:/Windows/Fonts/simhei.ttf",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(f) = load_ttf_font_from_bytes(&bytes) {
                return Some(f);
            }
        }
    }
    None
}

fn dt(s: &str, x: f32, y: f32, size: f32, color: Color, font: Option<&Font>) {
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font,
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

fn mt(s: &str, size: u16, font: Option<&Font>) -> TextDimensions {
    measure_text(s, font, size, 1.0)
}

// ---------- 主循环辅助 -----------------------------------------------------

fn step_play(world: &mut World, fx: &mut fx::Fx, audio: &Audio, dt: f32, t: f32) {
    world.run_time += dt;
    if world.combo_timer > 0.0 {
        world.combo_timer -= dt;
    } else if world.combo > 0 {
        world.combo = 0;
        world.combo_note_idx = 0;
    }
    if world.combo_flash > 0.0 {
        world.combo_flash -= dt;
    }
    world.player.update(dt, t, fx);

    let fired_main = world.weapons.tick(
        dt,
        t,
        &world.player,
        &mut world.enemies,
        &mut world.bullets,
        fx,
    );
    if fired_main {
        audio.play(&audio.shoot, 0.4);
    }

    if !world.boss_alive {
        spawn_normals(world, dt, t);
    }
    if !world.boss_alive && world.run_time >= world.next_boss_at {
        let x = CFG.w * 0.5;
        world.enemies.push(spawn_boss(x, t));
        world.boss_alive = true;
    }

    let speed_mul = world.diff_mul();
    for e in &mut world.enemies {
        let scale = if matches!(e.kind, EnemyKind::Boss) {
            1.0
        } else {
            speed_mul
        };
        e.update(dt * scale, t, world.player.x, &mut world.bullets);
    }

    for b in &mut world.bullets {
        if b.homing && !b.dead {
            steer_homing(b, &world.enemies, dt);
        }
    }
    for b in &mut world.bullets {
        b.update(dt);
    }

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
                    "MAGNET",
                    Color::from_rgba(255, 120, 210, 255),
                    14.0,
                );
            }
            PickupKind::Ammo => {
                world.super_charge = (world.super_charge + 0.25).min(1.0);
                fx.float_text(
                    px,
                    py - 46.0,
                    "+SUPER",
                    Color::from_rgba(255, 180, 80, 255),
                    14.0,
                );
            }
            PickupKind::Barrier => {
                world.player.shield = true;
                fx.float_text(
                    px,
                    py - 46.0,
                    "SHIELD",
                    Color::from_rgba(125, 200, 255, 255),
                    14.0,
                );
            }
        }
    }

    if is_key_pressed(KeyCode::Space) && world.super_charge >= 1.0 {
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
        audio.play(&audio.powerup, 0.26);
    }

    for b in &mut world.bullets {
        if b.dead || !b.from_player {
            continue;
        }
        for e in &mut world.enemies {
            if e.dead {
                continue;
            }
            if collision::bullet_hits_enemy(b, e) {
                let mut damage = b.damage * e.damage_mul();
                if e.static_mark && !b.is_crit && b.source != HitSource::Enemy {
                    damage *= world.player.stats.crit_mul;
                    b.is_crit = true;
                    e.static_mark = false;
                }
                e.hp -= damage;
                e.hit_flash = 0.08;
                e.last_hit = b.source;
                if b.source == HitSource::Missile {
                    e.marked_until = t + 2.0;
                }
                fx.burst(
                    b.x,
                    b.y,
                    4,
                    2.0,
                    Color::from_rgba(125, 249, 255, 255),
                    120.0,
                );
                if (t * 1000.0) as i32 % 4 == 0 {
                    audio.play(&audio.hit, 0.18);
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

    let score_mul = world.player.stats.score_mul;
    let mut boss_died_just_now = false;
    for e in &mut world.enemies {
        if !e.dead && e.hp <= 0.0 {
            e.dead = true;
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
                })
            .min(1.0);
            if world.combo % 10 == 0 {
                world.super_charge = (world.super_charge + 0.05).min(1.0);
            }
            let (scale, color, big) = match e.kind {
                EnemyKind::Small => (0.9, Color::from_rgba(255, 136, 102, 255), false),
                EnemyKind::Medium => (1.3, Color::from_rgba(201, 124, 255, 255), false),
                EnemyKind::Large => (2.0, Color::from_rgba(255, 77, 109, 255), true),
                EnemyKind::Boss => (4.0, Color::from_rgba(255, 90, 140, 255), true),
            };
            fx.explode(e.x, e.y, scale, color);
            drop_xp_gems(&mut world.pickups, e);
            maybe_drop_special(&mut world.pickups, e, t);
            if world.player.perks.drone_relay && e.last_hit == HitSource::Drone {
                spawn_relay_missile(&mut world.bullets, &world.player, e.x, e.y);
            }
            if big {
                fx.float_text(
                    e.x - 18.0,
                    e.y,
                    format!("+{}", ((e.score as f32) * score_mul * combo_mul) as u32),
                    Color::from_rgba(255, 209, 102, 255),
                    if matches!(e.kind, EnemyKind::Boss) {
                        28.0
                    } else {
                        22.0
                    },
                );
                audio.play(&audio.explode_big, 1.0);
            } else {
                audio.play_kill_combo(world.combo);
                if matches!(e.kind, EnemyKind::Large) {
                    audio.play(&audio.explode_small, 0.10);
                }
            }
            if matches!(e.kind, EnemyKind::Boss) {
                boss_died_just_now = true;
            }
        }
    }
    if boss_died_just_now {
        world.boss_alive = false;
        world.next_boss_at = world.run_time + 90.0;
    }

    if !world.player.dead {
        for b in &mut world.bullets {
            if b.dead || b.from_player {
                continue;
            }
            if collision::bullet_hits_player(b, &world.player) {
                b.dead = true;
                if world.player.hit(t) {
                    audio.play(&audio.hurt, 0.9);
                }
            }
        }
    }

    if !world.player.dead {
        for e in &mut world.enemies {
            if e.dead {
                continue;
            }
            if collision::hit_circle(
                e.x,
                e.y,
                e.radius,
                world.player.x,
                world.player.y,
                world.player.radius,
            ) {
                if world.player.hit(t) {
                    audio.play(&audio.hurt, 0.9);
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

    world.bullets.retain(|b| !b.dead);
    world.enemies.retain(|e| !e.dead);

    fx.update(dt);
}

fn spawn_normals(world: &mut World, dt: f32, t: f32) {
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

fn spawn_enemy(kind: EnemyKind, x: f32, t: f32, run_time: f32) -> Enemy {
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

fn spawn_boss(x: f32, t: f32) -> Enemy {
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

fn drop_xp_gems(pickups: &mut Vec<Pickup>, e: &Enemy) {
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

fn maybe_drop_special(pickups: &mut Vec<Pickup>, e: &Enemy, t: f32) {
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

fn spawn_relay_missile(bullets: &mut Vec<Bullet>, player: &Player, x: f32, y: f32) {
    let mut b = Bullet::player_shot(x, y, 0.0, -260.0);
    b.damage = player.stats.damage_mul * 1.2;
    b.homing = true;
    b.w = 5.0;
    b.h = 10.0;
    b.source = HitSource::Missile;
    bullets.push(b);
}

fn draw_world(world: &World, t: f32) {
    for g in &world.pickups {
        g.draw(t);
    }
    for b in &world.bullets {
        b.draw();
    }
    for e in &world.enemies {
        e.draw();
    }
    if !world.player.dead {
        world.weapons.draw(&world.player, t);
        world.player.draw(t);
    }
}

fn steer_homing(b: &mut Bullet, enemies: &[Enemy], dt: f32) {
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

// ---------- UI -----------------------------------------------------------

fn draw_menu(
    t_acc: f32,
    save: &Save,
    audio: &Audio,
    ship: ShipType,
    font: Option<&Font>,
    lang: Lang,
) {
    let cx = CFG.w * 0.5;
    let scale = 1.0 + (t_acc * 1.6).sin() * 0.03;

    let title = t("STELLAR WING", lang);
    let font_size = 56.0 * scale;
    let dim = mt(title, font_size as u16, font);
    dt(
        title,
        cx - dim.width * 0.5,
        160.0,
        font_size,
        Color::from_rgba(0, 212, 255, 255),
        font,
    );

    let sub = t("Rust Edition  ·  Roguelike Mode", lang);
    let dim2 = mt(sub, 16, font);
    dt(
        sub,
        cx - dim2.width * 0.5,
        192.0,
        16.0,
        Color::from_rgba(125, 249, 255, 255),
        font,
    );

    let hi = format!("{}  {}", t("HIGH SCORE", lang), save.high);
    let dh = mt(&hi, 18, font);
    dt(
        &hi,
        cx - dh.width * 0.5,
        228.0,
        18.0,
        Color::from_rgba(255, 209, 102, 255),
        font,
    );

    if !save.leaderboard.is_empty() {
        let head = t("— TOP 5 —", lang);
        let dh2 = mt(head, 14, font);
        dt(
            head,
            cx - dh2.width * 0.5,
            260.0,
            14.0,
            Color::from_rgba(180, 200, 220, 255),
            font,
        );
        for (i, r) in save.leaderboard.iter().enumerate() {
            let line = format!(
                "{}.  {:>6}   {}{:<2}   {}",
                i + 1,
                r.score,
                t("LV", lang),
                r.level,
                r.date
            );
            let d = mt(&line, 14, font);
            dt(
                &line,
                cx - d.width * 0.5,
                284.0 + i as f32 * 20.0,
                14.0,
                Color::from_rgba(200, 220, 240, 255),
                font,
            );
        }
    }

    let lines = [
        t("WASD / Arrows — Move    P / ESC — Pause", lang),
        t("A / D or ← / → — Select ship", lang),
        t("M — Mute    F — Fullscreen", lang),
        t("Auto-fire · Collect XP gems → pick a card", lang),
    ];
    let mut y = 460.0;
    for l in lines {
        let d = mt(l, 14, font);
        dt(
            l,
            cx - d.width * 0.5,
            y,
            14.0,
            Color::from_rgba(160, 180, 210, 255),
            font,
        );
        y += 22.0;
    }

    // 当前语言指示
    let lang_line = format!("{} {}", t("Language:", lang), lang.name());
    let dl = mt(&lang_line, 12, font);
    dt(
        &lang_line,
        cx - dl.width * 0.5,
        y + 6.0,
        12.0,
        Color::from_rgba(125, 139, 168, 255),
        font,
    );

    let ship_name = t(ship.name(), lang);
    let dsn = mt(ship_name, 20, font);
    dt(
        ship_name,
        cx - dsn.width * 0.5,
        568.0,
        20.0,
        Color::from_rgba(255, 209, 102, 255),
        font,
    );
    let ship_desc = t(ship.desc(), lang);
    let dsd = mt(ship_desc, 13, font);
    dt(
        ship_desc,
        cx - dsd.width * 0.5,
        592.0,
        13.0,
        Color::from_rgba(180, 200, 220, 255),
        font,
    );

    draw_player_preview(ship, cx, 666.0, 1.15, t_acc);

    let mute_status = if audio.muted { t("[Muted]", lang) } else { "" };
    let hint = format!("{}  {}", t("Press ENTER to start", lang), mute_status);
    let dh3 = mt(&hint, 20, font);
    dt(
        &hint,
        cx - dh3.width * 0.5,
        CFG.h - 60.0 + (t_acc * 4.0).sin() * 2.0,
        20.0,
        Color::from_rgba(125, 249, 255, 255),
        font,
    );
}

fn draw_play_hud(world: &World, high: u32, font: Option<&Font>, lang: Lang) {
    let score_txt = format!("{}  {}", t("SCORE", lang), world.score);
    dt(
        &score_txt,
        16.0,
        32.0,
        22.0,
        Color::from_rgba(0, 212, 255, 255),
        font,
    );
    let hi_txt = format!("{}  {}", t("HIGH", lang), high.max(world.score));
    dt(
        &hi_txt,
        16.0,
        54.0,
        14.0,
        Color::from_rgba(125, 249, 255, 255),
        font,
    );

    let lv = format!("{} {}", t("LV", lang), world.level);
    dt(
        &lv,
        16.0,
        76.0,
        16.0,
        Color::from_rgba(255, 209, 102, 255),
        font,
    );
    let ship_line = format!("{} {}", t("Ship", lang), t(world.player.ship.name(), lang));
    dt(
        &ship_line,
        16.0,
        94.0,
        12.0,
        Color::from_rgba(180, 200, 220, 255),
        font,
    );

    let secs = world.run_time as u32;
    let timer = format!("{:02}:{:02}", secs / 60, secs % 60);
    let dtm = mt(&timer, 16, font);
    dt(
        &timer,
        CFG.w * 0.5 - dtm.width * 0.5,
        20.0,
        16.0,
        Color::from_rgba(180, 200, 220, 255),
        font,
    );

    if let Some(boss) = world
        .enemies
        .iter()
        .find(|e| matches!(e.kind, EnemyKind::Boss))
    {
        let bar_y = 76.0;
        let pad = 70.0;
        let bw = CFG.w - pad * 2.0;
        draw_rectangle(pad, bar_y, bw, 8.0, Color::from_rgba(0, 0, 0, 200));
        let pct = (boss.hp / boss.max_hp).clamp(0.0, 1.0);
        let mut col = Color::from_rgba(255, 90, 140, 255);
        if pct < 0.66 {
            col = Color::from_rgba(255, 160, 80, 255);
        }
        if pct < 0.33 {
            col = Color::from_rgba(255, 70, 70, 255);
        }
        draw_rectangle(pad, bar_y, bw * pct, 8.0, col);
        let title = t("— BOSS —", lang);
        let dt2 = mt(title, 12, font);
        dt(
            title,
            CFG.w * 0.5 - dt2.width * 0.5,
            bar_y - 4.0,
            12.0,
            Color::from_rgba(255, 209, 102, 255),
            font,
        );
        if let Some(mod_label) = boss.boss_mod_label() {
            let text = t(mod_label, lang);
            let dm = mt(text, 12, font);
            dt(
                text,
                CFG.w * 0.5 - dm.width * 0.5,
                bar_y + 24.0,
                12.0,
                Color::from_rgba(230, 241, 255, 255),
                font,
            );
        }
        if boss.telegraph > 0.0 {
            let warn_text = match boss.telegraph_kind {
                TelegraphKind::BossAim => "Lock-on volley",
                TelegraphKind::BossFan => "Fan barrage",
                TelegraphKind::BossNova => "Core burst",
                _ => "",
            };
            if !warn_text.is_empty() {
                let dw = mt(t(warn_text, lang), 13, font);
                dt(
                    t(warn_text, lang),
                    CFG.w * 0.5 - dw.width * 0.5,
                    bar_y + 42.0,
                    13.0,
                    Color::from_rgba(255, 110, 110, 255),
                    font,
                );
            }
        }
    }

    for i in 0..world.player.lives {
        let x = CFG.w - 24.0 - i as f32 * 22.0;
        draw_heart(x, 24.0, 8.0, Color::from_rgba(255, 85, 119, 255));
    }

    let mut wy = CFG.h - 40.0;
    let wx = 16.0;
    let gun_label = format!("{}{}", t("Gun Lv", lang), world.weapons.main.level);
    dt(
        &gun_label,
        wx,
        wy,
        14.0,
        Color::from_rgba(125, 249, 255, 255),
        font,
    );
    if let Some(ratio) = world.weapons.main.decay_ratio() {
        draw_rectangle(wx, wy + 4.0, 74.0, 5.0, Color::from_rgba(20, 30, 50, 255));
        draw_rectangle(
            wx,
            wy + 4.0,
            74.0 * ratio,
            5.0,
            Color::from_rgba(0, 212, 255, 255),
        );
    }
    wy -= 18.0;
    for s in &world.weapons.subs {
        let label = t(pretty_id(s.id()), lang);
        let txt = format!("{} Lv{}", label, s.level());
        dt(
            &txt,
            wx,
            wy,
            12.0,
            Color::from_rgba(255, 209, 102, 255),
            font,
        );
        if let Some(ratio) = s.decay_ratio() {
            draw_rectangle(wx, wy + 3.0, 64.0, 4.0, Color::from_rgba(20, 30, 50, 255));
            draw_rectangle(
                wx,
                wy + 3.0,
                64.0 * ratio,
                4.0,
                Color::from_rgba(255, 160, 90, 255),
            );
        }
        wy -= 16.0;
    }
    if world.player.magnet_until > world.run_time {
        dt(
            t("Magnet", lang),
            wx,
            wy,
            12.0,
            Color::from_rgba(255, 120, 210, 255),
            font,
        );
    }

    let sx = CFG.w - 110.0;
    let sy = CFG.h - 34.0;
    draw_rectangle(sx, sy, 94.0, 8.0, Color::from_rgba(20, 30, 50, 255));
    draw_rectangle(
        sx,
        sy,
        94.0 * world.super_charge.clamp(0.0, 1.0),
        8.0,
        Color::from_rgba(255, 180, 80, 255),
    );
    dt(
        t("SUPER", lang),
        sx,
        sy - 4.0,
        12.0,
        Color::from_rgba(255, 209, 102, 255),
        font,
    );

    if world.combo >= 2 {
        let mut c = Color::from_rgba(255, 209, 102, 255);
        c.a = (0.7 + world.combo_flash * 0.7).min(1.0);
        let combo_line = format!("{} x{}", t("COMBO", lang), world.combo);
        let dc = mt(&combo_line, 16, font);
        dt(&combo_line, CFG.w - dc.width - 16.0, 56.0, 16.0, c, font);
    }

    let bar_h = 6.0;
    let pct = (world.xp as f32 / world.xp_to_next.max(1) as f32).clamp(0.0, 1.0);
    draw_rectangle(
        0.0,
        CFG.h - bar_h,
        CFG.w,
        bar_h,
        Color::from_rgba(20, 30, 50, 255),
    );
    draw_rectangle(
        0.0,
        CFG.h - bar_h,
        CFG.w * pct,
        bar_h,
        Color::from_rgba(125, 249, 255, 255),
    );
}

fn draw_pause(font: Option<&Font>, lang: Lang) {
    draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::from_rgba(0, 5, 20, 200));
    let cx = CFG.w * 0.5;
    let title = t("PAUSED", lang);
    let d = mt(title, 48, font);
    dt(
        title,
        cx - d.width * 0.5,
        CFG.h * 0.4,
        48.0,
        Color::from_rgba(0, 212, 255, 255),
        font,
    );
    let lines = [t("P / ESC — resume", lang), t("Q — quit to menu", lang)];
    let mut y = CFG.h * 0.55;
    for l in lines {
        let d2 = mt(l, 18, font);
        dt(
            l,
            cx - d2.width * 0.5,
            y,
            18.0,
            Color::from_rgba(180, 200, 220, 255),
            font,
        );
        y += 30.0;
    }
}

fn draw_gameover(t_acc: f32, world: &World, save: &Save, font: Option<&Font>, lang: Lang) {
    draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::from_rgba(0, 0, 0, 180));
    let cx = CFG.w * 0.5;
    let bob = (t_acc * 3.0).sin() * 4.0;
    let title = t("GAME OVER", lang);
    let d = mt(title, 48, font);
    dt(
        title,
        cx - d.width * 0.5,
        180.0 + bob,
        48.0,
        Color::from_rgba(255, 85, 119, 255),
        font,
    );

    let s = format!("{}  {}", t("Score", lang), world.score);
    let ds = mt(&s, 22, font);
    dt(
        &s,
        cx - ds.width * 0.5,
        260.0,
        22.0,
        Color::from_rgba(230, 241, 255, 255),
        font,
    );
    let h = format!("{}   {}", t("High", lang), save.high);
    let dh = mt(&h, 18, font);
    dt(
        &h,
        cx - dh.width * 0.5,
        290.0,
        18.0,
        Color::from_rgba(255, 209, 102, 255),
        font,
    );
    let lv = format!("{} {}", t("Level reached:", lang), world.level);
    let dlv = mt(&lv, 16, font);
    dt(
        &lv,
        cx - dlv.width * 0.5,
        316.0,
        16.0,
        Color::from_rgba(180, 200, 220, 255),
        font,
    );

    if world.score > 0 && world.score >= save.high {
        let nr = t("★ NEW RECORD ★", lang);
        let dn = mt(nr, 16, font);
        dt(
            nr,
            cx - dn.width * 0.5,
            340.0,
            16.0,
            Color::from_rgba(125, 249, 255, 255),
            font,
        );
    }

    let hint = t("ENTER restart  ·  ESC menu", lang);
    let dh2 = mt(hint, 16, font);
    dt(
        hint,
        cx - dh2.width * 0.5,
        CFG.h - 80.0,
        16.0,
        Color::from_rgba(125, 139, 168, 255),
        font,
    );
}

fn card_layout(n: usize) -> (f32, f32, f32, f32, f32) {
    let card_w = 138.0;
    let card_h = 220.0;
    let gap = 12.0;
    let total_w = card_w * n as f32 + gap * n.saturating_sub(1) as f32;
    let start_x = CFG.w * 0.5 - total_w * 0.5;
    let y0 = 220.0;
    (start_x, y0, card_w, card_h, gap)
}

fn card_at(lx: f32, ly: f32, n: usize) -> Option<usize> {
    let (start_x, y0, w, h, gap) = card_layout(n);
    if ly < y0 || ly > y0 + h {
        return None;
    }
    for i in 0..n {
        let x = start_x + i as f32 * (w + gap);
        if lx >= x && lx <= x + w {
            return Some(i);
        }
    }
    None
}

fn draw_upgrade_pick(cards: &[Card], t_acc: f32, cursor: usize, font: Option<&Font>, lang: Lang) {
    draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::from_rgba(0, 5, 20, 210));
    let cx = CFG.w * 0.5;
    let title = t("LEVEL UP", lang);
    let d = mt(title, 36, font);
    dt(
        title,
        cx - d.width * 0.5,
        140.0,
        36.0,
        Color::from_rgba(255, 209, 102, 255),
        font,
    );
    let sub = t("1 / 2 / 3   ·   ← →   ·   Enter   ·   click", lang);
    let ds = mt(sub, 14, font);
    dt(
        sub,
        cx - ds.width * 0.5,
        168.0,
        14.0,
        Color::from_rgba(180, 200, 220, 255),
        font,
    );

    let n = cards.len();
    let (start_x, y0, card_w, card_h, gap) = card_layout(n);

    for (i, c) in cards.iter().enumerate() {
        let x = start_x + i as f32 * (card_w + gap);
        let selected = i == cursor;
        let bob = (t_acc * 3.0 + i as f32 * 0.6).sin() * 3.0;
        let y = y0 + bob - if selected { 8.0 } else { 0.0 };
        draw_card(c, x, y, card_w, card_h, i + 1, selected, t_acc, font, lang);
    }
}

fn draw_card(
    c: &Card,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    idx: usize,
    selected: bool,
    t_acc: f32,
    font: Option<&Font>,
    lang: Lang,
) {
    let edge = c.rarity.color();
    if selected {
        let pulse = 0.6 + (t_acc * 6.0).sin() * 0.2;
        let mut glow = edge;
        glow.a = pulse;
        draw_rectangle_lines(x - 4.0, y - 4.0, w + 8.0, h + 8.0, 4.0, glow);
    }
    let bg = if selected {
        Color::from_rgba(20, 35, 60, 240)
    } else {
        Color::from_rgba(10, 20, 40, 230)
    };
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, if selected { 3.0 } else { 2.0 }, edge);
    draw_rectangle(x, y, w, 6.0, edge);

    dt(
        &format!("{}", idx),
        x + 8.0,
        y + 22.0,
        14.0,
        Color::from_rgba(125, 139, 168, 255),
        font,
    );
    let r = t(c.rarity.label(), lang);
    let dr = mt(r, 11, font);
    dt(r, x + w - dr.width - 8.0, y + 20.0, 11.0, edge, font);

    let name = t(c.name, lang);
    let dn = mt(name, 18, font);
    dt(
        name,
        x + (w - dn.width) * 0.5,
        y + 80.0,
        18.0,
        Color::from_rgba(230, 241, 255, 255),
        font,
    );
    let desc = t(c.desc, lang);
    let dd = mt(desc, 12, font);
    dt(
        desc,
        x + (w - dd.width) * 0.5,
        y + 130.0,
        12.0,
        Color::from_rgba(180, 200, 220, 255),
        font,
    );

    if selected {
        let hint = t("Enter / Click", lang);
        let dh = mt(hint, 12, font);
        dt(
            hint,
            x + (w - dh.width) * 0.5,
            y + h - 14.0,
            12.0,
            Color::from_rgba(125, 249, 255, 255),
            font,
        );
    }
}

fn pretty_id(id: &str) -> &'static str {
    match id {
        "missile" => "Missile",
        "drone" => "Drone",
        "laser" => "Laser",
        "chain" => "Chain",
        _ => "?",
    }
}

fn draw_heart(x: f32, y: f32, s: f32, c: Color) {
    draw_circle(x - s * 0.5, y, s * 0.5, c);
    draw_circle(x + s * 0.5, y, s * 0.5, c);
    draw_triangle(
        vec2(x - s, y + s * 0.1),
        vec2(x + s, y + s * 0.1),
        vec2(x, y + s * 1.2),
        c,
    );
}
