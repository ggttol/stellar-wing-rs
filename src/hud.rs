//! 菜单 / HUD / 暂停 / 升级选卡 / 游戏结束的渲染。

use macroquad::prelude::*;

use crate::art::draw_player_preview;
use crate::audio::Audio;
use crate::config::CFG;
use crate::entity::{enemy::TelegraphKind, EnemyKind};
use crate::lang::{t, Lang};
use crate::save::Save;
use crate::ship::ShipType;
use crate::upgrade::Card;
use crate::world::World;

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

pub fn draw_menu(
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

pub fn draw_play_hud(world: &World, high: u32, font: Option<&Font>, lang: Lang) {
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

pub fn draw_pause(font: Option<&Font>, lang: Lang) {
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

pub fn draw_gameover(t_acc: f32, world: &World, save: &Save, font: Option<&Font>, lang: Lang) {
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

pub fn card_layout(n: usize) -> (f32, f32, f32, f32, f32) {
    let card_w = 138.0;
    let card_h = 220.0;
    let gap = 12.0;
    let total_w = card_w * n as f32 + gap * n.saturating_sub(1) as f32;
    let start_x = CFG.w * 0.5 - total_w * 0.5;
    let y0 = 220.0;
    (start_x, y0, card_w, card_h, gap)
}

pub fn card_at(lx: f32, ly: f32, n: usize) -> Option<usize> {
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

pub fn draw_upgrade_pick(
    cards: &[Card],
    t_acc: f32,
    cursor: usize,
    font: Option<&Font>,
    lang: Lang,
) {
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

#[allow(clippy::too_many_arguments)]
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

pub fn draw_world(world: &World, t: f32) {
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
