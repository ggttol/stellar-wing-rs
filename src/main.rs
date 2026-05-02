use macroquad::prelude::*;

mod achievements;
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
    let mut audio_inst = Audio::load(
        save_data.muted,
        save_data.master_vol,
        save_data.sfx_vol,
        save_data.bgm_vol,
    )
    .await;

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
                if is_key_pressed(KeyCode::O) {
                    audio_inst.play_click();
                    scene = Scene::Settings(0);
                }
                if is_key_pressed(KeyCode::H) {
                    audio_inst.play_click();
                    scene = Scene::Achievements(0);
                }
                if is_key_pressed(KeyCode::C) {
                    audio_inst.play_click();
                    scene = Scene::Codex(0, 0);
                }
                if is_key_pressed(KeyCode::Y) {
                    // 进入每日挑战：用 today() 当种子
                    let ship = ShipType::ALL[menu_ship];
                    if save_data.ship_unlocked(ship) {
                        let seed = daily_seed_from_date(&save::today());
                        world = World::new(ship);
                        world.difficulty = save_data.difficulty;
                        world.daily_mode = true;
                        world.run_seed = seed;
                        talents::apply_to_world(&mut world, &save_data);
                        fx = fx::Fx::default();
                        audio_inst.play_click();
                        audio_inst.set_track(BgmTrack::Play);
                        scene = Scene::Playing;
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let ship = ShipType::ALL[menu_ship];
                    if save_data.ship_unlocked(ship) {
                        world = World::new(ship);
                        world.difficulty = save_data.difficulty;
                        let ship_idx = ShipType::ALL.iter().position(|s| *s == ship).unwrap_or(0);
                        world.player.skin = save_data.ship_skin_choice[ship_idx];
                        talents::apply_to_world(&mut world, &save_data);
                        fx = fx::Fx::default();
                        audio_inst.play_click();
                        audio_inst.set_track(BgmTrack::Play);
                        scene = Scene::Playing;
                    } else {
                        audio_inst.play_pause(); // 锁定提示音
                    }
                }
                // [K] 切换当前飞船的涂装（仅在已解锁的之间循环）
                if is_key_pressed(KeyCode::K) {
                    let ship_idx = menu_ship;
                    // 涂装位：bit = ship_idx*3 + variant
                    let cur = save_data.ship_skin_choice[ship_idx];
                    for step in 1..3u8 {
                        let next = (cur + step) % 3;
                        let bit = ship_idx as u32 * 3 + next as u32;
                        if (save_data.ship_skins_unlocked & (1u32 << bit)) != 0 {
                            save_data.ship_skin_choice[ship_idx] = next;
                            save::write(&save_data);
                            audio_inst.play_click();
                            break;
                        }
                    }
                }
                // [N] 切换难度（已解锁的范围内循环）
                if is_key_pressed(KeyCode::N) {
                    let max_d = if save_data.nightmare_unlocked {
                        2
                    } else if save_data.hard_unlocked {
                        1
                    } else {
                        0
                    };
                    save_data.difficulty = (save_data.difficulty + 1) % (max_d + 1);
                    save::write(&save_data);
                    audio_inst.play_click();
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
            Scene::ChapterChoice(opts, cursor) => {
                if is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::A)
                    || is_key_pressed(KeyCode::Key1)
                {
                    *cursor = 0;
                }
                if is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::D)
                    || is_key_pressed(KeyCode::Key2)
                {
                    *cursor = 1;
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    world.chapter_modifier = opts[*cursor];
                    audio_inst.play_powerup();
                    scene = Scene::Playing;
                }
                fx.update(dt);
            }
            Scene::Achievements(cursor) => {
                let n = achievements::ACHIEVEMENTS.len();
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *cursor = (*cursor + n - 1) % n;
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *cursor = (*cursor + 1) % n;
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
                    audio_inst.play_click();
                    scene = Scene::Menu;
                }
                fx.update(dt);
            }
            Scene::Codex(tab, cursor) => {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    *tab = (*tab + 2) % 3;
                    *cursor = 0;
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    *tab = (*tab + 1) % 3;
                    *cursor = 0;
                }
                let max_n = match *tab {
                    0 => 9,  // 9 种敌人
                    1 => 6,  // 6 种 BossMod
                    _ => 7,  // 7 种副武器
                };
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *cursor = (*cursor + max_n - 1) % max_n.max(1);
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *cursor = (*cursor + 1) % max_n.max(1);
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
                    audio_inst.play_click();
                    scene = Scene::Menu;
                }
                fx.update(dt);
            }
            Scene::Settings(cursor) => {
                let n = hud::SETTINGS_ROWS;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *cursor = (*cursor + n - 1) % n;
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *cursor = (*cursor + 1) % n;
                }
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    settings_adjust(*cursor, -1, &mut save_data, &mut audio_inst);
                    save::write(&save_data);
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    settings_adjust(*cursor, 1, &mut save_data, &mut audio_inst);
                    save::write(&save_data);
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    settings_adjust(*cursor, 1, &mut save_data, &mut audio_inst);
                    save::write(&save_data);
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
                    // 玩法 dt 受 hit-pause / slow-mo 影响；UI / 背景仍按真实 dt
                    let play_dt = fx.tick_time_modifiers(dt);
                    step_play(&mut world, &mut fx, &audio_inst, play_dt, t_acc, lang);

                    // 成就检查（每帧；已解锁的会跳过）
                    let new_unlocks = achievements::check_all(&world, &save_data);
                    for (idx, reward) in &new_unlocks {
                        achievements::mark_unlocked(&mut save_data, *idx);
                        save_data.stardust = save_data.stardust.saturating_add(*reward);
                        let name = if lang == lang::Lang::Zh {
                            achievements::ACHIEVEMENTS[*idx as usize].name_zh
                        } else {
                            achievements::ACHIEVEMENTS[*idx as usize].name_en
                        };
                        fx.float_text(
                            world.player.x,
                            world.player.y - 50.0,
                            format!("★ {}", name),
                            Color::from_rgba(255, 215, 90, 255),
                            16.0,
                        );
                        fx.shock_ring(
                            world.player.x,
                            world.player.y,
                            Color::from_rgba(255, 215, 90, 255),
                            1.6,
                        );
                        audio_inst.play_powerup();
                    }
                    if !new_unlocks.is_empty() {
                        save::write(&save_data);
                    }

                    // 章节切换：如果 boss_alive 转 false 且新章节还在故事范围，弹路线选择
                    // （World 在 boss 死亡后已 chapter_idx +=1 + boss_alive=false）
                    if !world.boss_alive
                        && world.chapter_intro > 0.0
                        && world.chapter_idx >= 1
                        && (world.chapter_idx as usize) < chapter::CHAPTERS.len()
                        && world.chapter_modifier == world::ChapterMod::None
                        && world.chapter_time < 0.5
                    {
                        let opts = world::ChapterMod::options();
                        // 用 chapter_idx 决定哪两条出现：[0,1]、[1,2]、[0,2]，循环
                        let pick = (world.chapter_idx as usize) % 3;
                        let pair = match pick {
                            0 => [opts[0], opts[1]],
                            1 => [opts[1], opts[2]],
                            _ => [opts[0], opts[2]],
                        };
                        scene = Scene::ChapterChoice(pair, 0);
                        audio_inst.play_levelup();
                    }
                    if world.player.dead {
                        fx.explode(
                            world.player.x,
                            world.player.y,
                            2.5,
                            Color::from_rgba(125, 249, 255, 255),
                        );
                        let cleared_story =
                            world.chapter_idx as usize >= chapter::CHAPTERS.len();
                        let reward = save_data.record_run(
                            world.score,
                            world.level,
                            world.bosses_killed_run,
                            world.chapter_idx,
                            world.difficulty,
                            cleared_story,
                        );
                        // 合并图鉴（局内位 → 持久 save）
                        save_data.codex_enemies |= world.codex_enemies_run;
                        save_data.codex_bosses |= world.codex_bosses_run;
                        save_data.codex_weapons |= world.codex_weapons_run;
                        // 每日挑战分数单独记录
                        if world.daily_mode {
                            let today = save::today();
                            if save_data.daily_date != today {
                                save_data.daily_date = today;
                                save_data.daily_high = 0;
                            }
                            if world.score > save_data.daily_high {
                                save_data.daily_high = world.score;
                            }
                        }
                        save::write(&save_data);
                        last_reward = Some(reward);
                        audio_inst.play_gameover();
                        audio_inst.set_track(BgmTrack::None);
                        scene = Scene::GameOver;
                    } else if world.xp >= world.xp_to_next {
                        world.xp -= world.xp_to_next;
                        world.level += 1;
                        world.xp_to_next = World::xp_required_for(world.level + 1);
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
        // 屏幕震动偏移（Boss 攻击 / 大爆炸触发），用户偏好可缩放
        let shake = fx.shake * save_data.screen_shake;
        let (sx, sy) = if shake > 0.0 {
            use ::rand::{thread_rng, Rng};
            let mut rng = thread_rng();
            (
                rng.gen_range(-shake..shake),
                rng.gen_range(-shake * 0.7..shake * 0.7),
            )
        } else {
            (0.0, 0.0)
        };
        match &scene {
            Scene::Playing
            | Scene::Paused
            | Scene::UpgradePick(_)
            | Scene::ChapterChoice(_, _)
            | Scene::GameOver => {
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
            Scene::Settings(cursor) => {
                hud::draw_settings(&save_data, &audio_inst, *cursor, t_acc, font, lang);
            }
            Scene::Achievements(cursor) => {
                hud::draw_achievements(&save_data, *cursor, t_acc, font, lang);
            }
            Scene::Codex(tab, cursor) => {
                hud::draw_codex(&save_data, *tab, *cursor, t_acc, font, lang);
            }
            Scene::Playing => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_screen_fx(&world, &fx);
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_chapter_intro(&world, font, lang);
            }
            Scene::Paused => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_screen_fx(&world, &fx);
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_pause(&world, font, lang);
            }
            Scene::UpgradePick(cards) => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_screen_fx(&world, &fx);
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_upgrade_pick(cards, t_acc, card_cursor, font, lang);
            }
            Scene::ChapterChoice(opts, cursor) => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_screen_fx(&world, &fx);
                hud::draw_play_hud(&world, save_data.high, font, lang);
                hud::draw_chapter_choice(&world, opts, *cursor, t_acc, font, lang);
            }
            Scene::GameOver => {
                hud::draw_world(&world, t_acc, sx, sy);
                fx.draw();
                hud::draw_screen_fx(&world, &fx);
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

/// 把 YYYY-MM-DD 字符串哈希成 64-bit 种子（每日挑战用）
fn daily_seed_from_date(date: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    date.hash(&mut h);
    h.finish()
}

/// 设置项调整：把光标行映射到具体字段，按 step 增减。
/// 调整后立即同步到 audio_inst（保证音量条拉到时直接听到变化）。
fn settings_adjust(row: usize, dir: i32, save_data: &mut Save, audio: &mut Audio) {
    let step_vol = 0.05;
    let step_shake = 0.1;
    match row {
        0 => {
            save_data.master_vol = (save_data.master_vol + dir as f32 * step_vol).clamp(0.0, 1.0);
            audio.set_master_vol(save_data.master_vol);
        }
        1 => {
            save_data.bgm_vol = (save_data.bgm_vol + dir as f32 * step_vol).clamp(0.0, 1.0);
            audio.set_bgm_vol(save_data.bgm_vol);
        }
        2 => {
            save_data.sfx_vol = (save_data.sfx_vol + dir as f32 * step_vol).clamp(0.0, 1.0);
            audio.set_sfx_vol(save_data.sfx_vol);
            audio.play_click(); // 让用户听到当前 SFX 强度
        }
        3 => {
            save_data.screen_shake =
                (save_data.screen_shake + dir as f32 * step_shake).clamp(0.0, 1.5);
        }
        4 => {
            // Mute toggle：只接受 dir = 1（Enter / →），方向键左右都允许
            audio.toggle_mute();
            save_data.muted = audio.muted;
        }
        5 => {
            save_data.fullscreen = !save_data.fullscreen;
            set_fullscreen(save_data.fullscreen);
        }
        _ => {}
    }
}

/// 给所有子弹补一个拖尾点：玩家子弹按来源差异化，敌方子弹只在大子弹上加。
fn spawn_bullet_trails(bullets: &[entity::Bullet], fx: &mut fx::Fx) {
    use entity::HitSource;
    for b in bullets {
        if b.dead {
            continue;
        }
        let c = b.tint();
        if b.from_player {
            // 不同武器轨迹的"质感"不同：
            // 导弹尾焰最长最亮（火箭弹味），反射镜中等（弹道感），僚机最短（光弹），其余默认。
            let (size, decay) = match b.source {
                HitSource::Missile => (4.5, 3.4),
                HitSource::Reflector => (3.4, 4.8),
                HitSource::Drone => (2.0, 7.5),
                _ if b.is_crit => (4.2, 4.0),
                _ => (2.6, 6.5),
            };
            fx.trail(b.x, b.y, size, c, decay);
        } else if b.h >= 14.0 || b.w >= 12.0 {
            // 只给"大"敌方弹（Boss / 狙击 / 重弹）加 trail，避免普通弹幕画面拥堵
            fx.trail(b.x, b.y, 3.0, c, 5.0);
        }
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
        &mut world.damage_by_source,
    );
    // Codex：当前局已解锁的副武器
    for s in &world.weapons.subs {
        if let Some(b) = World::weapon_codex_bit(s.id()) {
            world.codex_weapons_run |= 1u32 << b;
        }
    }
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
        let dur = chap.duration * world.chapter_modifier.duration_mul();
        if !world.chapter_boss_spawned && world.chapter_time >= dur {
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
    spawn_bullet_trails(&world.bullets, fx);

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
            // 重置章节修饰 / no-hit 状态，准备下一章的分叉选择
            world.chapter_modifier = world::ChapterMod::None;
            world.chapter_no_hit = true;
        }
    }

    combat::resolve_enemy_bullets(world, fx, audio, t);
    combat::resolve_enemy_player_contact(world, fx, audio, t);

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
