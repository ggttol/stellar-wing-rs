use macroquad::prelude::*;

mod art;
mod audio;
mod bg;
mod collision;
mod combat;
mod config;
mod entity;
mod fx;
mod hud;
mod lang;
mod save;
mod scene;
mod ship;
mod spawn;
mod upgrade;
mod weapon;
mod world;

use audio::{Audio, BgmTrack};
use config::CFG;
use lang::Lang;
use save::Save;
use scene::Scene;
use ship::ShipType;
use world::World;

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

#[macroquad::main(window_conf)]
async fn main() {
    let mut save_data = save::load();
    let mut audio_inst = Audio::load(save_data.muted).await;

    if save_data.fullscreen {
        set_fullscreen(true);
    }

    // 尝试加载系统 CJK 字体；失败则降级英文。
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

    audio_inst.set_track(BgmTrack::Menu);

    loop {
        let dt = get_frame_time().min(0.05);
        t_acc += dt;

        bg.update(dt);

        global_keys(&mut save_data, &mut audio_inst, &cjk_font);
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
                    audio_inst.play_click();
                    audio_inst.set_track(BgmTrack::Play);
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
                    audio_inst.play_pause();
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
                        audio_inst.play_gameover();
                        audio_inst.set_track(BgmTrack::None);
                        scene = Scene::GameOver;
                    } else if world.xp >= world.xp_to_next {
                        world.xp -= world.xp_to_next;
                        world.level += 1;
                        world.xp_to_next = 6 + world.level * 4;
                        let cards = upgrade::draw_n(3, &world.player, &world.weapons);
                        card_cursor = 0;
                        audio_inst.play_levelup();
                        scene = Scene::UpgradePick(cards);
                    } else {
                        // BGM 跟随 boss 状态。
                        let want = if world.boss_alive {
                            BgmTrack::Boss
                        } else {
                            BgmTrack::Play
                        };
                        audio_inst.set_track(want);
                    }
                }
            }
            Scene::Paused => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    scene = Scene::Playing;
                } else if is_key_pressed(KeyCode::Q) {
                    scene = Scene::Menu;
                    audio_inst.set_track(BgmTrack::Menu);
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
                    if let Some(i) = hud::card_at(lx, ly, n) {
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
                            lang::t(cards[i].name, lang).to_string(),
                            cards[i].rarity.color(),
                            18.0,
                        );
                        audio_inst.play_powerup();
                        scene = Scene::Playing;
                    }
                }
                fx.update(dt);
            }
            Scene::GameOver => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    world = World::new(world.player.ship);
                    fx = fx::Fx::default();
                    audio_inst.set_track(BgmTrack::Play);
                    scene = Scene::Playing;
                } else if is_key_pressed(KeyCode::Escape) {
                    audio_inst.set_track(BgmTrack::Menu);
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
            Scene::Menu => hud::draw_menu(
                t_acc,
                &save_data,
                &audio_inst,
                ShipType::ALL[menu_ship],
                font,
                lang,
            ),
            Scene::Playing => {
                hud::draw_world(&world, t_acc);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
            }
            Scene::Paused => {
                hud::draw_world(&world, t_acc);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_pause(font, lang);
            }
            Scene::UpgradePick(cards) => {
                hud::draw_world(&world, t_acc);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_upgrade_pick(cards, t_acc, card_cursor, font, lang);
            }
            Scene::GameOver => {
                hud::draw_world(&world, t_acc);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_gameover(t_acc, &world, &save_data, font, lang);
            }
            _ => {}
        }

        next_frame().await;
    }
}

fn global_keys(save_data: &mut Save, audio_inst: &mut Audio, cjk_font: &Option<Font>) {
    if is_key_pressed(KeyCode::F) {
        save_data.fullscreen = !save_data.fullscreen;
        set_fullscreen(save_data.fullscreen);
        save::write(save_data);
    }
    if is_key_pressed(KeyCode::M) {
        audio_inst.toggle_mute();
        save_data.muted = audio_inst.muted;
        save::write(save_data);
    }
    if is_key_pressed(KeyCode::L) {
        let next = save_data.lang.toggle();
        // CJK 字体不可用时不允许切到中文。
        if !(next == Lang::Zh && cjk_font.is_none()) {
            save_data.lang = next;
            save::write(save_data);
        }
    }
}

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

/// 主战斗 tick：编排各子系统的执行顺序。
fn step_play(world: &mut World, fx: &mut fx::Fx, audio: &Audio, dt: f32, t: f32) {
    world.run_time += dt;

    // combo 衰减
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
        audio.play_shoot();
    }

    // 生成
    if !world.boss_alive {
        spawn::spawn_normals(world, dt, t);
        if world.run_time >= world.next_boss_at {
            let x = CFG.w * 0.5;
            world.enemies.push(spawn::spawn_boss(x, t));
            world.boss_alive = true;
            audio.play_boss_intro();
        }
    }

    // 敌人 / 子弹更新
    let speed_mul = world.diff_mul();
    for e in &mut world.enemies {
        let scale = if matches!(e.kind, entity::EnemyKind::Boss) {
            1.0
        } else {
            speed_mul
        };
        e.update(dt * scale, t, world.player.x, &mut world.bullets);
    }
    combat::steer_homing_bullets(world, dt);
    for b in &mut world.bullets {
        b.update(dt);
    }

    // 拾取
    combat::collect_pickups(world, fx, dt);

    // Super
    if is_key_pressed(KeyCode::Space) && world.super_charge >= 1.0 {
        combat::trigger_super(world, fx, audio);
    }

    // 碰撞结算
    combat::resolve_player_bullets(world, fx, audio, t, dt);

    let boss_died = combat::process_kills(world, fx, audio, t);
    if boss_died {
        world.boss_alive = false;
        world.next_boss_at = world.run_time + 90.0;
    }

    combat::resolve_enemy_bullets(world, audio, t);
    combat::resolve_enemy_player_contact(world, audio, t);

    world.bullets.retain(|b| !b.dead);
    world.enemies.retain(|e| !e.dead);

    fx.update(dt);
}
