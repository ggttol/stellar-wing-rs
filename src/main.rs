use macroquad::prelude::*;

mod art;
mod audio;
mod bg;
mod chapter;
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
mod talents;
mod upgrade;
mod weapon;
mod world;

use audio::{Audio, BgmTrack};
use config::CFG;
use lang::Lang;
use save::{RunReward, Save};
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
    let mut last_reward: Option<RunReward> = None;

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
                    menu_ship = step_unlocked_ship(menu_ship, -1, &save_data);
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    menu_ship = step_unlocked_ship(menu_ship, 1, &save_data);
                }
                if is_key_pressed(KeyCode::T) {
                    audio_inst.play_click();
                    scene = Scene::Talents(0);
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let ship = ShipType::ALL[menu_ship];
                    if save_data.ship_unlocked(ship) {
                        world = World::new(ship);
                        talents::apply_to_world(&mut world, &save_data);
                        fx = fx::Fx::default();
                        audio_inst.play_click();
                        audio_inst.set_track(BgmTrack::Play);
                        scene = Scene::Playing;
                    } else {
                        audio_inst.play_pause(); // 锁定提示音
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                fx.update(dt);
            }
            Scene::Talents(cursor) => {
                let n = talents::TALENTS.len();
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *cursor = (*cursor + n - 1) % n;
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *cursor = (*cursor + 1) % n;
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let id = talents::TALENTS[*cursor].id;
                    if talents::try_buy(&mut save_data, id) {
                        save::write(&save_data);
                        audio_inst.play_powerup();
                    } else {
                        audio_inst.play_pause();
                    }
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
                    audio_inst.play_click();
                    scene = Scene::Menu;
                }
                fx.update(dt);
            }
            Scene::Playing => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    scene = Scene::Paused;
                    audio_inst.play_pause();
                } else {
                    step_play(&mut world, &mut fx, &audio_inst, dt, t_acc, lang);
                    if world.player.dead {
                        fx.explode(
                            world.player.x,
                            world.player.y,
                            2.5,
                            Color::from_rgba(125, 249, 255, 255),
                        );
                        let reward = save_data.record_run(
                            world.score,
                            world.level,
                            world.bosses_killed_run,
                            world.chapter_idx,
                        );
                        save::write(&save_data);
                        last_reward = Some(reward);
                        audio_inst.play_gameover();
                        audio_inst.set_track(BgmTrack::None);
                        scene = Scene::GameOver;
                    } else if world.xp >= world.xp_to_next {
                        world.xp -= world.xp_to_next;
                        world.level += 1;
                        world.xp_to_next = 6 + world.level * 4;
                        let cards = upgrade::draw_n(3, &mut world.player, &world.weapons);
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
                    talents::apply_to_world(&mut world, &save_data);
                    fx = fx::Fx::default();
                    audio_inst.set_track(BgmTrack::Play);
                    scene = Scene::Playing;
                } else if is_key_pressed(KeyCode::Escape) {
                    audio_inst.set_track(BgmTrack::Menu);
                    scene = Scene::Menu;
                }
                fx.update(dt);
            }
        }

        clear_background(Color::from_rgba(2, 3, 10, 255));
        // 屏幕震动偏移（Boss 攻击 / 大爆炸触发）
        let (sx, sy) = if fx.shake > 0.0 {
            use ::rand::{thread_rng, Rng};
            let mut rng = thread_rng();
            (
                rng.gen_range(-fx.shake..fx.shake),
                rng.gen_range(-fx.shake * 0.7..fx.shake * 0.7),
            )
        } else {
            (0.0, 0.0)
        };
        match &scene {
            Scene::Playing | Scene::Paused | Scene::UpgradePick(_) | Scene::GameOver => {
                let chap = chapter::get(world.chapter_idx);
                bg.draw_themed(chap.bg_top, chap.bg_mid, chap.star_tint);
            }
            _ => bg.draw(),
        }

        match &scene {
            Scene::Menu => hud::draw_menu(
                t_acc,
                &save_data,
                &audio_inst,
                ShipType::ALL[menu_ship],
                font,
                lang,
            ),
            Scene::Talents(cursor) => {
                hud::draw_talents(&save_data, *cursor, t_acc, font, lang);
            }
            Scene::Playing => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_chapter_intro(&world, font, lang);
            }
            Scene::Paused => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_pause(font, lang);
            }
            Scene::UpgradePick(cards) => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_upgrade_pick(cards, t_acc, card_cursor, font, lang);
            }
            Scene::GameOver => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_gameover(t_acc, &world, &save_data, last_reward.as_ref(), font, lang);
            }
        }

        next_frame().await;
    }
}

/// 在解锁的飞船里走一步；锁住的也允许停留（让玩家看到提示），但优先走到下一个解锁的。
fn step_unlocked_ship(cur: usize, dir: i32, save: &Save) -> usize {
    let n = ShipType::ALL.len();
    let mut next = cur;
    for _ in 0..n {
        next = ((next as i32 + dir).rem_euclid(n as i32)) as usize;
        if save.ship_unlocked(ShipType::ALL[next]) {
            return next;
        }
    }
    cur
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
fn step_play(world: &mut World, fx: &mut fx::Fx, audio: &Audio, dt: f32, t: f32, lang: Lang) {
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
    if world.overload_flash > 0.0 {
        world.overload_flash = (world.overload_flash - dt * 1.6).max(0.0);
    }
    world.synergy.tick(dt);

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

    // 章节内时钟（仅在没有 Boss 时推进）
    if !world.boss_alive {
        if world.chapter_intro > 0.0 {
            world.chapter_intro -= dt;
        } else {
            world.chapter_time += dt;
        }
    }

    // 普通敌人 / 章节专属敌人
    if !world.boss_alive {
        spawn::spawn_chapter_wave(world, dt, t);

        // Boss 入场
        let chap = chapter::get(world.chapter_idx);
        if !world.chapter_boss_spawned && world.chapter_time >= chap.duration {
            spawn::spawn_chapter_boss(world, t);
            world.chapter_boss_spawned = true;
            world.boss_alive = true;
            audio.play_boss_intro();
        }
    }

    // 敌人 / 子弹更新
    let speed_mul = world.diff_mul();
    let mut enemy_spawns = Vec::new();
    for e in &mut world.enemies {
        let scale = if matches!(e.kind, entity::EnemyKind::Boss) {
            1.0
        } else {
            speed_mul
        };
        e.update(
            dt * scale,
            t,
            world.player.x,
            &mut world.bullets,
            &mut enemy_spawns,
        );
        // Boss 攻击屏幕震动
        if e.should_shake {
            fx.shake = fx.shake.max(10.0);
            e.should_shake = false;
        }
    }
    world.enemies.extend(enemy_spawns);
    combat::steer_homing_bullets(world, dt);
    for b in &mut world.bullets {
        b.update(dt);
    }

    // 拾取
    combat::collect_pickups(world, fx, dt, lang);

    // Super
    if is_key_pressed(KeyCode::Space) && world.super_charge >= 1.0 {
        combat::trigger_super(world, fx, audio);
    }

    // 碰撞结算
    combat::resolve_player_bullets(world, fx, audio, t, dt);

    let boss_died = combat::process_kills(world, fx, audio, t);
    if boss_died {
        // 仅当所有 boss 都被击杀（无尽双 boss）才推进章节。
        let still_alive = world
            .enemies
            .iter()
            .any(|e| !e.dead && matches!(e.kind, entity::EnemyKind::Boss));
        if !still_alive {
            world.boss_alive = false;
            world.bosses_killed_run += 1;
            world.chapter_idx += 1;
            // 无尽模式每圈 +4% 伤害，抵消 Boss HP 指数增长
            if world.is_endless() {
                let lap = (world.chapter_idx - crate::chapter::total()) as f32;
                world.endless_damage_bonus = (lap * 0.04).max(0.0);
            }
            world.chapter_time = 0.0;
            world.chapter_boss_spawned = false;
            world.chapter_intro = 2.5;
            world.strafer_timer = 0.0;
        }
    }

    combat::resolve_enemy_bullets(world, audio, t);
    combat::resolve_enemy_player_contact(world, audio, t);

    // 低血量警告（每 1.2 秒一声短促 beep）
    if world.player.lives == 1 && !world.player.dead {
        let beat = (world.run_time / 1.2) as u32;
        if beat != world.last_hp_warn_beat {
            world.last_hp_warn_beat = beat;
            audio.play_hurt(); // 复用受击音效作为警告
        }
    }

    // 成功躲避 Kamikaze 的反馈
    for e in &world.enemies {
        if e.dodged {
            let reward = e.score / 4;
            world.score += reward;
            fx.float_text(
                e.x.clamp(30.0, CFG.w - 30.0),
                e.y.clamp(30.0, CFG.h - 30.0),
                format!("{} +{}", lang::t("DODGED!", lang), reward),
                Color::from_rgba(125, 249, 255, 255),
                13.0,
            );
        }
    }

    world.bullets.retain(|b| !b.dead);
    world.enemies.retain(|e| !e.dead);

    fx.update(dt);
}
