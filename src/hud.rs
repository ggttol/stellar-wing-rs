//! 菜单 / HUD / 暂停 / 升级选卡 / 游戏结束的渲染。

use macroquad::prelude::*;

use crate::art::draw_player_preview_skin;
use crate::audio::Audio;
use crate::chapter;
use crate::config::CFG;
use crate::entity::{enemy::TelegraphKind, EnemyKind};
use crate::lang::{t, Lang};
use crate::save::{RunReward, Save};
use crate::ship::ShipType;
use crate::talents::{self, TALENTS};
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

// —— 菜单视觉：色板 ——————————————————————————————————————

const NEON_CYAN: Color = Color::new(0.0, 0.831, 1.0, 1.0); // #00D4FF
const ICE_CYAN: Color = Color::new(0.490, 0.976, 1.0, 1.0); // #7DF9FF
const GOLD: Color = Color::new(1.0, 0.819, 0.4, 1.0); // #FFD166
const MAGENTA: Color = Color::new(0.788, 0.486, 1.0, 1.0); // #C97CFF
const SOFT_WHITE: Color = Color::new(0.902, 0.945, 1.0, 1.0); // #E6F1FF
const MUTED: Color = Color::new(0.490, 0.545, 0.659, 1.0); // #7D8BA8
const PANEL_FILL: Color = Color::new(0.031, 0.055, 0.110, 0.86); // 半透深蓝
const PANEL_EDGE: Color = Color::new(0.0, 0.831, 1.0, 0.55);
const BANNER_BG: Color = Color::new(0.012, 0.020, 0.055, 1.0);

// —— 通用绘制原语 ——————————————————————————————————————

fn alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

/// 画一组 L 形 sci-fi 角标。`len` 是 L 边长，`thick` 厚度。
fn draw_corner_brackets(x: f32, y: f32, w: f32, h: f32, len: f32, thick: f32, color: Color) {
    let bars = [
        // 左上
        (x, y, len, thick),
        (x, y, thick, len),
        // 右上
        (x + w - len, y, len, thick),
        (x + w - thick, y, thick, len),
        // 左下
        (x, y + h - thick, len, thick),
        (x, y + h - len, thick, len),
        // 右下
        (x + w - len, y + h - thick, len, thick),
        (x + w - thick, y + h - len, thick, len),
    ];
    for (px, py, pw, ph) in bars {
        draw_rectangle(px, py, pw, ph, color);
    }
}

/// 带角标的"控制台"面板。可选标题嵌在上沿。
fn draw_console_panel(x: f32, y: f32, w: f32, h: f32, title: Option<&str>, font: Option<&Font>) {
    draw_rectangle(x, y, w, h, PANEL_FILL);
    draw_rectangle_lines(x, y, w, h, 1.0, alpha(PANEL_EDGE, 0.35));
    draw_corner_brackets(x, y, w, h, 14.0, 2.0, PANEL_EDGE);
    if let Some(title) = title {
        let pad = 10.0;
        let dim = mt(title, 11, font);
        let label_w = dim.width + 12.0;
        // 标题底色一小条，盖住上边线营造嵌入感
        draw_rectangle(
            x + pad,
            y - 6.0,
            label_w,
            12.0,
            Color::new(0.012, 0.020, 0.055, 1.0),
        );
        dt(title, x + pad + 6.0, y + 4.0, 11.0, ICE_CYAN, font);
    }
}

/// 带左右滑动着色的"扫描线"装饰。
fn draw_scan_underline(x: f32, y: f32, w: f32, t: f32, color: Color) {
    draw_rectangle(x, y, w, 1.0, alpha(color, 0.30));
    let head = ((t * 0.65).fract()) * w;
    let glow_w = 80.0_f32.min(w);
    let start = (head - glow_w * 0.5).clamp(0.0, w - glow_w);
    draw_rectangle(x + start, y - 1.0, glow_w, 3.0, alpha(color, 0.55));
}

/// 像素小键帽：`[K] Label`。返回占用宽度。
fn draw_key_cap(x: f32, y: f32, key: &str, label: &str, font: Option<&Font>, on: bool) -> f32 {
    let kdim = mt(key, 11, font);
    let cap_w = kdim.width + 10.0;
    let cap_h = 16.0;
    let edge = if on {
        alpha(NEON_CYAN, 0.85)
    } else {
        alpha(MUTED, 0.7)
    };
    let fill = if on {
        Color::new(0.0, 0.18, 0.30, 1.0)
    } else {
        Color::new(0.06, 0.08, 0.14, 1.0)
    };
    draw_rectangle(x, y - cap_h * 0.7, cap_w, cap_h, fill);
    draw_rectangle_lines(x, y - cap_h * 0.7, cap_w, cap_h, 1.0, edge);
    dt(key, x + 5.0, y + 3.0, 11.0, edge, font);
    let ldim = mt(label, 11, font);
    dt(label, x + cap_w + 5.0, y + 3.0, 11.0, MUTED, font);
    cap_w + 10.0 + ldim.width + 12.0
}

/// 三段式 pip 进度条。
fn draw_stat_bar(
    x: f32,
    y: f32,
    label: &str,
    value: f32,
    color: Color,
    font: Option<&Font>,
    lang: Lang,
) {
    let pip_w = 16.0;
    let pip_h = 6.0;
    let gap = 2.0;
    let segments = 8;
    let label_w = 38.0;
    dt(t(label, lang), x, y + pip_h, 11.0, MUTED, font);
    let filled = (value * segments as f32).round() as i32;
    for i in 0..segments {
        let bx = x + label_w + i as f32 * (pip_w + gap);
        let on = i < filled;
        let c = if on {
            color
        } else {
            Color::new(0.06, 0.10, 0.18, 1.0)
        };
        draw_rectangle(bx, y, pip_w, pip_h, c);
        if on {
            draw_rectangle(bx, y, pip_w, 1.0, alpha(SOFT_WHITE, 0.35));
        }
    }
}

/// 透视网格地板。每帧基于 t 滚动，营造"飞行"视感。
fn draw_perspective_grid(x: f32, y: f32, w: f32, h: f32, t: f32, color: Color) {
    let rows = 8;
    let scroll = (t * 0.6).fract();
    for i in 0..rows {
        let f = (i as f32 + scroll) / rows as f32;
        let curve = f * f; // 加速远近
        let line_y = y + h - curve * h;
        let inset = (1.0 - curve) * (w * 0.42);
        let a = (1.0 - curve).powf(1.4) * 0.7;
        draw_line(
            x + inset,
            line_y,
            x + w - inset,
            line_y,
            1.0,
            alpha(color, a),
        );
    }
    // 中央竖向消失线
    let cx = x + w * 0.5;
    let vp_y = y + h * 0.18; // 消失点
    let cols = 5;
    for i in -(cols / 2)..=cols / 2 {
        let bx = cx + i as f32 * w * 0.18;
        draw_line(bx, y + h, cx, vp_y, 1.0, alpha(color, 0.18));
    }
}

/// 标题：双层位移 + 呼吸缩放，营造霓虹"色散"感。
fn draw_title(cx: f32, y: f32, t_acc: f32, font: Option<&Font>, lang: Lang) {
    let title = t("STELLAR WING", lang);
    let breath = 1.0 + (t_acc * 1.6).sin() * 0.025;
    let size = 50.0 * breath;
    let dim = mt(title, size as u16, font);
    let lx = cx - dim.width * 0.5;

    // 软外光
    for k in [10.0, 6.0, 3.0] {
        let a = match k {
            x if x > 8.0 => 0.10,
            x if x > 4.0 => 0.18,
            _ => 0.30,
        };
        dt(title, lx + k, y, size, alpha(NEON_CYAN, a), font);
        dt(title, lx - k, y, size, alpha(MAGENTA, a * 0.85), font);
    }
    // 主体白蓝
    dt(title, lx, y, size, ICE_CYAN, font);

    // 副标题
    let sub = t("Rust Edition  ·  Roguelike Mode", lang);
    let sd = mt(sub, 13, font);
    dt(sub, cx - sd.width * 0.5, y + 24.0, 13.0, MUTED, font);

    // 装饰扫描线
    draw_scan_underline(cx - 130.0, y + 38.0, 260.0, t_acc, NEON_CYAN);
}

/// 顶部 HIGH SCORE 横幅。
fn draw_high_score_chip(cx: f32, y: f32, high: u32, font: Option<&Font>, lang: Lang) {
    let label = t("HIGH SCORE", lang);
    let value = format!("{:>06}", high);
    let dl = mt(label, 11, font);
    let dv = mt(&value, 22, font);
    let total = dl.width + 12.0 + dv.width;
    let x = cx - total * 0.5;
    dt(label, x, y - 4.0, 11.0, MUTED, font);
    dt(&value, x + dl.width + 12.0, y + 4.0, 22.0, GOLD, font);
}

/// TOP 5 排行榜面板。
fn draw_leaderboard(x: f32, y: f32, w: f32, save: &Save, font: Option<&Font>, lang: Lang) {
    let h = 14.0 + save.leaderboard.len() as f32 * 16.0;
    draw_console_panel(x, y, w, h + 8.0, Some(t("TOP 5", lang)), font);
    for (i, r) in save.leaderboard.iter().enumerate() {
        let row_y = y + 18.0 + i as f32 * 16.0;
        let rank_color = match i {
            0 => GOLD,
            1 => ICE_CYAN,
            2 => MAGENTA,
            _ => MUTED,
        };
        dt(
            &format!("{}", i + 1),
            x + 12.0,
            row_y,
            12.0,
            rank_color,
            font,
        );
        let score = format!("{:>6}", r.score);
        dt(&score, x + 30.0, row_y, 12.0, SOFT_WHITE, font);
        let lv = format!("{}{:<2}", t("LV", lang), r.level);
        dt(&lv, x + 92.0, row_y, 11.0, ICE_CYAN, font);
        dt(&r.date, x + w - 80.0, row_y, 11.0, MUTED, font);
    }
}

/// 中央"机库"面板：透视网格 + 飞船预览 + 名称/描述 + 切换箭头。
#[allow(clippy::too_many_arguments)]
fn draw_hangar(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ship: ShipType,
    save: &Save,
    t_acc: f32,
    font: Option<&Font>,
    lang: Lang,
) {
    let unlocked = save.ship_unlocked(ship);
    draw_console_panel(x, y, w, h, Some(t("HANGAR", lang)), font);

    draw_perspective_grid(
        x + 6.0,
        y + h * 0.45,
        w - 12.0,
        h * 0.50,
        t_acc,
        alpha(NEON_CYAN, if unlocked { 0.6 } else { 0.25 }),
    );

    let cx = x + w * 0.5;
    let beam_y = y + h * 0.55;
    if unlocked {
        let beam_pulse = 0.55 + (t_acc * 3.0).sin() * 0.08;
        for r in [70.0, 50.0, 30.0] {
            let a = (1.0 - r / 80.0) * beam_pulse * 0.35;
            draw_circle(cx, beam_y, r, alpha(ICE_CYAN, a));
        }
    }

    let bob = (t_acc * 1.8).sin() * 4.0;
    let ship_idx = ShipType::ALL.iter().position(|s| *s == ship).unwrap_or(0);
    let skin = save.ship_skin_choice[ship_idx];
    draw_player_preview_skin(ship, skin, cx, beam_y - 14.0 + bob, 1.35, t_acc);

    if !unlocked {
        // 锁定遮罩
        draw_rectangle(x, y, w, h, alpha(BANNER_BG, 0.55));
        let lock = t("LOCKED", lang);
        let dl = mt(lock, 28, font);
        dt(
            lock,
            cx - dl.width * 0.5,
            y + h * 0.45,
            28.0,
            alpha(MAGENTA, 0.85),
            font,
        );
        if let Some(cost) = Save::ship_unlock_cost(ship) {
            let progress = (save.lifetime_score as f32 / cost as f32).clamp(0.0, 1.0);
            let bar_w = w * 0.7;
            let bar_x = cx - bar_w * 0.5;
            let bar_y = y + h * 0.55;
            draw_rectangle(bar_x, bar_y, bar_w, 6.0, Color::new(0.06, 0.10, 0.18, 1.0));
            draw_rectangle(bar_x, bar_y, bar_w * progress, 6.0, GOLD);
            let cap = format!(
                "{}  {} / {}",
                t("Lifetime score", lang),
                save.lifetime_score,
                cost
            );
            let dc = mt(&cap, 11, font);
            dt(&cap, cx - dc.width * 0.5, bar_y + 20.0, 11.0, MUTED, font);
        }
    }

    let name = t(ship.name(), lang);
    let dn = mt(name, 22, font);
    let name_y = y + h * 0.78;
    let chev_a = 0.45 + (t_acc * 4.5).sin() * 0.3;
    dt(
        "<",
        cx - dn.width * 0.5 - 22.0 + (t_acc * 4.5).sin() * 1.5,
        name_y,
        20.0,
        alpha(NEON_CYAN, chev_a),
        font,
    );
    dt(
        ">",
        cx + dn.width * 0.5 + 8.0 - (t_acc * 4.5).sin() * 1.5,
        name_y,
        20.0,
        alpha(NEON_CYAN, chev_a),
        font,
    );
    dt(
        name,
        cx - dn.width * 0.5,
        name_y,
        22.0,
        if unlocked { GOLD } else { MUTED },
        font,
    );

    let desc = t(ship.desc(), lang);
    let dd = mt(desc, 12, font);
    dt(
        desc,
        cx - dd.width * 0.5,
        y + h - 16.0,
        12.0,
        if unlocked { SOFT_WHITE } else { MUTED },
        font,
    );
}

/// 飞船属性条（DMG / SPD / TECH）。
fn draw_ship_stats(cx: f32, y: f32, ship: ShipType, font: Option<&Font>, lang: Lang) {
    let stats = ship.stats_preview();
    let row_h = 14.0;
    let bar_w = 160.0;
    let total_w = bar_w + 38.0;
    let x = cx - total_w * 0.5;
    let palette = [GOLD, ICE_CYAN, MAGENTA];
    for (i, (label, value)) in stats.iter().enumerate() {
        draw_stat_bar(
            x,
            y + i as f32 * row_h,
            label,
            *value,
            palette[i % palette.len()],
            font,
            lang,
        );
    }
}

/// "PRESS ENTER TO LAUNCH" 大字脉冲。
fn draw_launch_prompt(cx: f32, y: f32, t_acc: f32, font: Option<&Font>, lang: Lang) {
    let pulse = 0.6 + (t_acc * 4.0).sin() * 0.4;
    let size = 22.0;
    let prompt = t("PRESS ENTER TO LAUNCH", lang);
    let dim = mt(prompt, size as u16, font);
    let arrow_off = (t_acc * 4.0).sin() * 4.0;
    let total_w = dim.width + 60.0;
    let lx = cx - total_w * 0.5 + 30.0;

    dt(
        "▶",
        lx - 24.0 + arrow_off,
        y,
        size,
        alpha(NEON_CYAN, pulse),
        font,
    );
    dt(
        "◀",
        lx + dim.width + 8.0 - arrow_off,
        y,
        size,
        alpha(NEON_CYAN, pulse),
        font,
    );
    dt(
        prompt,
        lx,
        y,
        size,
        alpha(ICE_CYAN, 0.85 + pulse * 0.15),
        font,
    );
}

/// 底部一行键提示。
fn draw_key_row(cx: f32, y: f32, audio: &Audio, font: Option<&Font>, lang: Lang) {
    let mute_label = if audio.muted {
        t("Sound Off", lang)
    } else {
        t("Sound", lang)
    };
    let hints: [(&str, &str, bool); 5] = [
        ("WASD", t("Move", lang), true),
        ("←→", t("Pick Ship", lang), true),
        ("L", t("Lang", lang), true),
        ("M", mute_label, !audio.muted),
        ("F", t("Full", lang), true),
    ];
    // 估算总宽以居中
    let mut total = 0.0;
    for (k, l, _) in hints {
        let kw = mt(k, 11, font).width + 10.0;
        let lw = mt(l, 11, font).width;
        total += kw + 10.0 + lw + 12.0;
    }
    total -= 12.0;
    let mut x = cx - total * 0.5;
    for (k, l, on) in hints {
        x += draw_key_cap(x, y, k, l, font, on);
    }
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

    // 背景晕染：菜单页中心一圈低饱和辉光，让面板更浮在前景
    draw_circle(cx, 360.0, 360.0, alpha(NEON_CYAN, 0.025));
    draw_circle(cx, 360.0, 240.0, alpha(MAGENTA, 0.020));

    draw_title(cx, 86.0, t_acc, font, lang);
    draw_high_score_chip(cx, 142.0, save.high, font, lang);
    // Stardust 余额 + 当前难度（菜单角标）
    let star_text = format!("✦ {}", save.stardust);
    let ds = mt(&star_text, 12, font);
    dt(&star_text, cx - ds.width * 0.5, 158.0, 12.0, ICE_CYAN, font);
    // 难度提示
    let diff_label = format!(
        "[N] {}",
        t(World::difficulty_label(save.difficulty), lang)
    );
    let dd = mt(&diff_label, 11, font);
    let diff_color = match save.difficulty {
        0 => MUTED,
        1 => GOLD,
        _ => Color::from_rgba(255, 90, 110, 255),
    };
    dt(&diff_label, cx - dd.width * 0.5, 174.0, 11.0, diff_color, font);

    if !save.leaderboard.is_empty() {
        draw_leaderboard(20.0, 168.0, CFG.w - 40.0, save, font, lang);
    }

    // 排行榜会撑高度，机库相应下移；排行榜空时机库上抬。
    let lb_bottom = if save.leaderboard.is_empty() {
        168.0
    } else {
        180.0 + save.leaderboard.len() as f32 * 16.0
    };
    let hangar_y = lb_bottom + 16.0;
    let hangar_h = 290.0;
    draw_hangar(
        20.0,
        hangar_y,
        CFG.w - 40.0,
        hangar_h,
        ship,
        save,
        t_acc,
        font,
        lang,
    );

    draw_ship_stats(cx, hangar_y + hangar_h + 18.0, ship, font, lang);

    draw_launch_prompt(cx, CFG.h - 96.0, t_acc, font, lang);

    // 当日挑战分数（如果今天打过）
    let today = crate::save::today();
    if save.daily_date == today && save.daily_high > 0 {
        let line = format!("{}: {}", t("Daily best", lang), save.daily_high);
        let dlim = mt(&line, 11, font);
        dt(
            &line,
            cx - dlim.width * 0.5,
            190.0,
            11.0,
            alpha(GOLD, 0.85),
            font,
        );
    }

    // 入口提示行（两行，键散开）
    let hints_a: [(&str, &str, Color); 2] = [
        ("[T]", t("TALENTS", lang), GOLD),
        ("[O]", t("SETTINGS", lang), ICE_CYAN),
    ];
    let hints_b: [(&str, &str, Color); 3] = [
        ("[H]", t("ACHIEVEMENTS", lang), GOLD),
        ("[C]", t("CODEX", lang), ICE_CYAN),
        ("[Y]", t("DAILY", lang), MAGENTA),
    ];
    fn row_w(items: &[(&str, &str, Color)], font: Option<&Font>) -> f32 {
        let mut w = 0.0;
        for (k, l, _) in items {
            let kd = mt(k, 11, font);
            let ld = mt(l, 11, font);
            w += kd.width + 4.0 + ld.width + 14.0;
        }
        (w - 14.0).max(0.0)
    }
    let pulse_a = 0.8 + (t_acc * 2.5).sin() * 0.15;
    let pulse_b = 0.8 + (t_acc * 2.5 + 1.5).sin() * 0.15;
    let row_a = row_w(&hints_a, font);
    let mut x = cx - row_a * 0.5;
    let y_a = CFG.h - 86.0;
    for (k, l, c) in hints_a {
        let kd = mt(k, 11, font);
        dt(k, x, y_a, 11.0, alpha(c, pulse_a), font);
        let ld = mt(l, 11, font);
        dt(l, x + kd.width + 4.0, y_a, 11.0, alpha(SOFT_WHITE, 0.85), font);
        x += kd.width + 4.0 + ld.width + 14.0;
    }
    let row_b = row_w(&hints_b, font);
    let mut x = cx - row_b * 0.5;
    let y_b = CFG.h - 70.0;
    for (k, l, c) in hints_b {
        let kd = mt(k, 11, font);
        dt(k, x, y_b, 11.0, alpha(c, pulse_b), font);
        let ld = mt(l, 11, font);
        dt(l, x + kd.width + 4.0, y_b, 11.0, alpha(SOFT_WHITE, 0.85), font);
        x += kd.width + 4.0 + ld.width + 14.0;
    }

    draw_key_row(cx, CFG.h - 38.0, audio, font, lang);

    // 语言指示靠左下角，不抢戏
    let lang_line = format!("◆ {}", lang.name());
    dt(&lang_line, 16.0, CFG.h - 16.0, 11.0, MUTED, font);
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
    // 每日挑战标签
    if world.daily_mode {
        let lbl = t("DAILY", lang);
        let dl = mt(lbl, 11, font);
        let pulse = 0.65 + (world.run_time * 3.0).sin() * 0.20;
        dt(
            lbl,
            CFG.w * 0.5 - dl.width * 0.5,
            42.0,
            11.0,
            alpha(GOLD, pulse),
            font,
        );
    }

    draw_resonance(world, font, lang);

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

    // 武器等级面板（仅文字 —— 等级永不回退）
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

pub fn draw_pause(world: &World, font: Option<&Font>, lang: Lang) {
    draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::from_rgba(0, 5, 20, 200));
    let cx = CFG.w * 0.5;
    let title = t("PAUSED", lang);
    let d = mt(title, 42, font);
    dt(
        title,
        cx - d.width * 0.5,
        90.0,
        42.0,
        Color::from_rgba(0, 212, 255, 255),
        font,
    );

    // build summary panel
    draw_build_summary(world, font, lang, 30.0, 150.0, CFG.w - 60.0);

    // 提示
    let lines = [t("P / ESC — resume", lang), t("Q — quit to menu", lang)];
    let mut y = CFG.h - 80.0;
    for l in lines {
        let d2 = mt(l, 16, font);
        dt(
            l,
            cx - d2.width * 0.5,
            y,
            16.0,
            Color::from_rgba(180, 200, 220, 255),
            font,
        );
        y += 24.0;
    }
}

pub fn draw_gameover(
    t_acc: f32,
    world: &World,
    save: &Save,
    reward: Option<&RunReward>,
    font: Option<&Font>,
    lang: Lang,
) {
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

    // —— 局内 build 总览（伤害分布 / 武器 / perks）——
    draw_build_summary(world, font, lang, 30.0, 360.0, CFG.w - 60.0);

    // —— 奖励 / 解锁面板 ————————————————————————————
    if let Some(r) = reward {
        let panel_y = 568.0;
        let panel_h = 100.0;
        draw_console_panel(
            20.0,
            panel_y,
            CFG.w - 40.0,
            panel_h,
            Some(t("REWARDS", lang)),
            font,
        );
        let stardust_label = format!("✦ +{}", r.stardust_gained);
        dt(&stardust_label, 36.0, panel_y + 28.0, 22.0, GOLD, font);
        let stardust_caption = t("Stardust earned", lang);
        dt(stardust_caption, 36.0, panel_y + 46.0, 11.0, MUTED, font);

        let total_label = format!("{}: {}", t("Total", lang), save.stardust);
        let dt_total = mt(&total_label, 12, font);
        dt(
            &total_label,
            CFG.w - 36.0 - dt_total.width,
            panel_y + 28.0,
            12.0,
            ICE_CYAN,
            font,
        );
        let lifetime = format!("{}: {}", t("Lifetime", lang), r.lifetime_after);
        let dt_life = mt(&lifetime, 11, font);
        dt(
            &lifetime,
            CFG.w - 36.0 - dt_life.width,
            panel_y + 46.0,
            11.0,
            MUTED,
            font,
        );

        if !r.newly_unlocked.is_empty() {
            let unlock_pulse = (t_acc * 5.0).sin().abs() * 0.4 + 0.6;
            let mut yy = panel_y + 70.0;
            for ship in &r.newly_unlocked {
                let line = format!("✦ {}: {}", t("UNLOCKED", lang), t(ship.name(), lang));
                let dl = mt(&line, 13, font);
                dt(
                    &line,
                    cx - dl.width * 0.5,
                    yy,
                    13.0,
                    alpha(GOLD, unlock_pulse),
                    font,
                );
                yy += 16.0;
            }
        }
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
        "rift" => "Rift",
        "wave" => "Wave",
        "reflector" => "Reflector",
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

/// 永久天赋购买页。
pub fn draw_talents(save: &Save, cursor: usize, t_acc: f32, font: Option<&Font>, lang: Lang) {
    let cx = CFG.w * 0.5;

    // 标题
    let title = t("TALENTS", lang);
    let dim = mt(title, 36, font);
    dt(title, cx - dim.width * 0.5, 70.0, 36.0, ICE_CYAN, font);
    let sub = t("Spend stardust to gain permanent power", lang);
    let ds = mt(sub, 12, font);
    dt(sub, cx - ds.width * 0.5, 92.0, 12.0, MUTED, font);

    // 顶部 stardust 余额（带闪烁星标）
    let pulse = 0.7 + (t_acc * 3.0).sin() * 0.2;
    let dust = format!("✦ {}", save.stardust);
    let dd = mt(&dust, 22, font);
    dt(
        &dust,
        cx - dd.width * 0.5,
        128.0,
        22.0,
        alpha(GOLD, pulse),
        font,
    );

    // 列表
    let row_h = 64.0;
    let list_x = 20.0;
    let list_w = CFG.w - 40.0;
    let list_y = 156.0;

    for (i, def) in TALENTS.iter().enumerate() {
        let y = list_y + i as f32 * row_h;
        let cur = talents::level_of(save, def.id);
        let max = def.max_level();
        let next_cost = def.next_cost(cur);
        let can_afford = next_cost.is_some_and(|c| save.stardust >= c);
        let selected = i == cursor;

        // 行底
        let fill = if selected {
            Color::new(0.06, 0.10, 0.18, 1.0)
        } else {
            Color::new(0.020, 0.035, 0.075, 1.0)
        };
        draw_rectangle(list_x, y, list_w, row_h - 6.0, fill);
        let edge_c = if selected {
            alpha(NEON_CYAN, 0.85)
        } else {
            alpha(PANEL_EDGE, 0.30)
        };
        draw_rectangle_lines(list_x, y, list_w, row_h - 6.0, 1.0, edge_c);
        if selected {
            draw_corner_brackets(list_x, y, list_w, row_h - 6.0, 10.0, 2.0, NEON_CYAN);
        }

        // 名称 + 描述
        let name = if lang == Lang::Zh {
            def.name_zh
        } else {
            def.name_en
        };
        dt(name, list_x + 14.0, y + 18.0, 14.0, ICE_CYAN, font);
        let desc = if lang == Lang::Zh {
            def.desc_zh
        } else {
            def.desc_en
        };
        dt(desc, list_x + 14.0, y + 36.0, 11.0, MUTED, font);

        // 等级 pip 条
        let pip_w = 14.0;
        let pip_h = 6.0;
        let gap = 3.0;
        let pip_total = max as f32 * (pip_w + gap) - gap;
        let pip_x = list_x + list_w - pip_total - 14.0;
        for k in 0..max {
            let bx = pip_x + k as f32 * (pip_w + gap);
            let on = k < cur;
            let c = if on {
                GOLD
            } else {
                Color::new(0.06, 0.10, 0.18, 1.0)
            };
            draw_rectangle(bx, y + 14.0, pip_w, pip_h, c);
        }

        // 右下：成本 / MAX
        let cost_label = if let Some(c) = next_cost {
            format!("✦ {}", c)
        } else {
            t("MAX", lang).to_string()
        };
        let dc = mt(&cost_label, 12, font);
        let cost_color = if next_cost.is_none() {
            GOLD
        } else if can_afford {
            ICE_CYAN
        } else {
            MUTED
        };
        dt(
            &cost_label,
            list_x + list_w - dc.width - 14.0,
            y + 40.0,
            12.0,
            cost_color,
            font,
        );
    }

    // 底部提示
    let hint = t("↑↓ select   ENTER buy   ESC back", lang);
    let dh = mt(hint, 12, font);
    dt(hint, cx - dh.width * 0.5, CFG.h - 28.0, 12.0, MUTED, font);
}

/// 设置页可调节的总行数：Master / BGM / SFX / Shake / Mute / Fullscreen。
pub const SETTINGS_ROWS: usize = 6;

/// 设置页：音量、震动、静音、全屏。
pub fn draw_settings(
    save: &Save,
    audio: &Audio,
    cursor: usize,
    t_acc: f32,
    font: Option<&Font>,
    lang: Lang,
) {
    let cx = CFG.w * 0.5;

    let title = t("SETTINGS", lang);
    let dim = mt(title, 36, font);
    dt(title, cx - dim.width * 0.5, 70.0, 36.0, ICE_CYAN, font);
    let sub = t("Volume / Effects / Display", lang);
    let ds = mt(sub, 12, font);
    dt(sub, cx - ds.width * 0.5, 92.0, 12.0, MUTED, font);

    let row_h = 56.0;
    let list_x = 60.0;
    let list_w = CFG.w - 120.0;
    let list_y = 150.0;

    let rows: [(&str, SettingValue); SETTINGS_ROWS] = [
        ("Master Volume", SettingValue::Slider(save.master_vol)),
        ("Music Volume", SettingValue::Slider(save.bgm_vol)),
        ("SFX Volume", SettingValue::Slider(save.sfx_vol)),
        (
            "Screen Shake",
            SettingValue::Slider(save.screen_shake / 1.5),
        ),
        (
            "Audio Mute",
            SettingValue::Toggle(audio.muted, "On", "Off"),
        ),
        (
            "Fullscreen",
            SettingValue::Toggle(save.fullscreen, "On", "Off"),
        ),
    ];

    for (i, (label, value)) in rows.iter().enumerate() {
        let y = list_y + i as f32 * row_h;
        let selected = i == cursor;

        let fill = if selected {
            Color::new(0.06, 0.10, 0.18, 1.0)
        } else {
            Color::new(0.020, 0.035, 0.075, 1.0)
        };
        draw_rectangle(list_x, y, list_w, row_h - 8.0, fill);
        let edge_c = if selected {
            alpha(NEON_CYAN, 0.85)
        } else {
            alpha(PANEL_EDGE, 0.30)
        };
        draw_rectangle_lines(list_x, y, list_w, row_h - 8.0, 1.0, edge_c);
        if selected {
            draw_corner_brackets(list_x, y, list_w, row_h - 8.0, 10.0, 2.0, NEON_CYAN);
        }

        dt(t(label, lang), list_x + 16.0, y + 22.0, 14.0, ICE_CYAN, font);

        // 右侧：滑条 / 文字
        let bar_w = 240.0;
        let bar_h = 8.0;
        let bar_x = list_x + list_w - bar_w - 60.0;
        let bar_y = y + 24.0;
        match value {
            SettingValue::Slider(v) => {
                let v = v.clamp(0.0, 1.0);
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.06, 0.10, 0.18, 1.0));
                let fc = if selected { GOLD } else { ICE_CYAN };
                draw_rectangle(bar_x, bar_y, bar_w * v, bar_h, fc);
                draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.0, alpha(MUTED, 0.5));
                let pct = (v * 100.0).round() as u32;
                let pct_label = format!("{}%", pct);
                let dp = mt(&pct_label, 12, font);
                dt(
                    &pct_label,
                    list_x + list_w - dp.width - 14.0,
                    bar_y + 8.0,
                    12.0,
                    SOFT_WHITE,
                    font,
                );
            }
            SettingValue::Toggle(on, on_label, off_label) => {
                let label_str = t(if *on { on_label } else { off_label }, lang);
                let dl = mt(label_str, 14, font);
                let color = if *on { GOLD } else { MUTED };
                dt(
                    label_str,
                    list_x + list_w - dl.width - 14.0,
                    bar_y + 8.0,
                    14.0,
                    color,
                    font,
                );
            }
        }
    }

    // 底部提示
    let pulse = 0.7 + (t_acc * 3.5).sin() * 0.2;
    let hint = t("↑↓ select   ←→ adjust   ENTER toggle   ESC back", lang);
    let dh = mt(hint, 12, font);
    dt(
        hint,
        cx - dh.width * 0.5,
        CFG.h - 28.0,
        12.0,
        alpha(ICE_CYAN, pulse),
        font,
    );
}

enum SettingValue {
    Slider(f32),
    Toggle(bool, &'static str, &'static str),
}

/// 章节分叉选择页：在 boss 死亡 / 章节切换瞬间弹出，2 选 1 修饰。
pub fn draw_chapter_choice(
    world: &World,
    opts: &[crate::world::ChapterMod; 2],
    cursor: usize,
    t_acc: f32,
    font: Option<&Font>,
    lang: Lang,
) {
    draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::from_rgba(0, 5, 20, 220));

    let cx = CFG.w * 0.5;
    let title = t("CHOOSE YOUR PATH", lang);
    let dim = mt(title, 26, font);
    dt(title, cx - dim.width * 0.5, 130.0, 26.0, GOLD, font);

    let chap = chapter::get(world.chapter_idx);
    let chap_label = if chap.endless {
        format!("◆ {} ◆", t("ENDLESS", lang))
    } else {
        format!("{} {} / {}", t("CHAPTER", lang), chap.id, chapter::total())
    };
    let dlim = mt(&chap_label, 12, font);
    dt(&chap_label, cx - dlim.width * 0.5, 156.0, 12.0, ICE_CYAN, font);

    let card_w = 200.0;
    let card_h = 200.0;
    let gap = 24.0;
    let total_w = card_w * 2.0 + gap;
    let start_x = cx - total_w * 0.5;
    let card_y = 200.0;

    for (i, opt) in opts.iter().enumerate() {
        let x = start_x + i as f32 * (card_w + gap);
        let selected = i == cursor;
        let pulse = if selected {
            0.7 + (t_acc * 6.0).sin() * 0.2
        } else {
            0.4
        };
        let bg = if selected {
            Color::from_rgba(20, 35, 60, 240)
        } else {
            Color::from_rgba(10, 20, 40, 220)
        };
        draw_rectangle(x, card_y, card_w, card_h, bg);
        let edge = if selected { GOLD } else { alpha(MUTED, 0.5) };
        draw_rectangle_lines(x, card_y, card_w, card_h, if selected { 3.0 } else { 1.5 }, edge);
        if selected {
            draw_corner_brackets(x, card_y, card_w, card_h, 14.0, 2.0, GOLD);
        }

        // 路线编号
        dt(
            &format!("{}", i + 1),
            x + 12.0,
            card_y + 22.0,
            14.0,
            alpha(MUTED, pulse),
            font,
        );

        // 标题
        let nm = t(opt.name(), lang);
        let dn = mt(nm, 22, font);
        dt(
            nm,
            x + (card_w - dn.width) * 0.5,
            card_y + 70.0,
            22.0,
            ICE_CYAN,
            font,
        );

        // 描述
        let desc = t(opt.desc(), lang);
        let dd = mt(desc, 11, font);
        dt(
            desc,
            x + (card_w - dd.width) * 0.5,
            card_y + 110.0,
            11.0,
            SOFT_WHITE,
            font,
        );

        if selected {
            let hint = t("Enter / 1·2", lang);
            let dh = mt(hint, 11, font);
            dt(
                hint,
                x + (card_w - dh.width) * 0.5,
                card_y + card_h - 16.0,
                11.0,
                alpha(GOLD, pulse),
                font,
            );
        }
    }
}

/// 成就页：列出所有成就，按已解锁 / 未解锁着色。
pub fn draw_achievements(
    save: &Save,
    cursor: usize,
    t_acc: f32,
    font: Option<&Font>,
    lang: Lang,
) {
    let cx = CFG.w * 0.5;
    let title = t("ACHIEVEMENTS", lang);
    let dim = mt(title, 32, font);
    dt(title, cx - dim.width * 0.5, 60.0, 32.0, ICE_CYAN, font);

    let total = crate::achievements::ACHIEVEMENTS.len();
    let unlocked = (0..total)
        .filter(|i| crate::achievements::is_unlocked(save, *i as u8))
        .count();
    let prog = format!("{} / {}", unlocked, total);
    let dp = mt(&prog, 14, font);
    dt(&prog, cx - dp.width * 0.5, 84.0, 14.0, GOLD, font);

    // 列表
    let row_h = 38.0;
    let visible = 12_usize;
    let start = if cursor < visible / 2 {
        0
    } else {
        (cursor - visible / 2).min(total.saturating_sub(visible))
    };
    let list_x = 30.0;
    let list_w = CFG.w - 60.0;
    let list_y = 110.0;

    for i in 0..visible.min(total) {
        let idx = start + i;
        if idx >= total {
            break;
        }
        let a = &crate::achievements::ACHIEVEMENTS[idx];
        let y = list_y + i as f32 * row_h;
        let is_done = crate::achievements::is_unlocked(save, a.id);
        let selected = idx == cursor;

        let fill = if selected {
            Color::new(0.06, 0.10, 0.18, 1.0)
        } else {
            Color::new(0.020, 0.035, 0.075, 0.9)
        };
        draw_rectangle(list_x, y, list_w, row_h - 4.0, fill);
        let edge = if selected {
            alpha(NEON_CYAN, 0.85)
        } else if is_done {
            alpha(GOLD, 0.55)
        } else {
            alpha(MUTED, 0.30)
        };
        draw_rectangle_lines(list_x, y, list_w, row_h - 4.0, 1.0, edge);

        // 状态星
        let star = if is_done { "★" } else { "☆" };
        let sc = if is_done {
            GOLD
        } else {
            alpha(MUTED, 0.6)
        };
        dt(star, list_x + 12.0, y + 22.0, 18.0, sc, font);

        let name = if lang == Lang::Zh { a.name_zh } else { a.name_en };
        let desc = if lang == Lang::Zh { a.desc_zh } else { a.desc_en };
        let name_color = if is_done { ICE_CYAN } else { SOFT_WHITE };
        dt(name, list_x + 36.0, y + 16.0, 13.0, name_color, font);
        dt(desc, list_x + 36.0, y + 30.0, 11.0, MUTED, font);

        // 奖励
        if a.stardust > 0 {
            let rew = format!("✦ {}", a.stardust);
            let dr = mt(&rew, 11, font);
            dt(
                &rew,
                list_x + list_w - dr.width - 14.0,
                y + 22.0,
                11.0,
                if is_done { alpha(GOLD, 0.6) } else { GOLD },
                font,
            );
        }
    }

    // 底部提示
    let hint = t("↑↓ select   ESC back", lang);
    let dh = mt(hint, 12, font);
    let pulse = 0.7 + (t_acc * 3.5).sin() * 0.2;
    dt(
        hint,
        cx - dh.width * 0.5,
        CFG.h - 28.0,
        12.0,
        alpha(ICE_CYAN, pulse),
        font,
    );
}

/// 图鉴页：tab 0 = 敌人、1 = Boss 修饰、2 = 副武器
pub fn draw_codex(save: &Save, tab: u8, cursor: usize, t_acc: f32, font: Option<&Font>, lang: Lang) {
    let cx = CFG.w * 0.5;
    let title = t("CODEX", lang);
    let dim = mt(title, 32, font);
    dt(title, cx - dim.width * 0.5, 60.0, 32.0, ICE_CYAN, font);

    // tab 选择
    let tabs = [t("ENEMIES", lang), t("BOSS MODS", lang), t("WEAPONS", lang)];
    let tab_y = 100.0;
    let tab_w = (CFG.w - 60.0) / 3.0;
    for (i, label) in tabs.iter().enumerate() {
        let tx = 30.0 + i as f32 * tab_w;
        let is_sel = i as u8 == tab;
        let edge = if is_sel { NEON_CYAN } else { alpha(MUTED, 0.4) };
        draw_rectangle_lines(tx + 4.0, tab_y, tab_w - 8.0, 26.0, 1.5, edge);
        let dim2 = mt(label, 13, font);
        let cc = if is_sel { ICE_CYAN } else { MUTED };
        dt(
            label,
            tx + tab_w * 0.5 - dim2.width * 0.5,
            tab_y + 18.0,
            13.0,
            cc,
            font,
        );
    }

    // 内容区
    let list_x = 30.0;
    let list_y = 144.0;
    let list_w = CFG.w * 0.42;
    let detail_x = list_x + list_w + 12.0;
    let detail_w = CFG.w - detail_x - 30.0;

    // 数据：根据 tab 决定项目集
    let items: Vec<(usize, &'static str, &'static str, bool)> = match tab {
        0 => {
            // 9 种 EnemyKind
            let kinds = [
                ("Small", "Lightweight scout"),
                ("Medium", "Mid-tier shooter"),
                ("Large", "Heavy armored"),
                ("Boss", "Chapter boss"),
                ("Kamikaze", "Suicide ram"),
                ("Strafer", "Side-sweeper"),
                ("Sniper", "High-velocity shots"),
                ("Weaver", "Sine-wave bullets"),
                ("MineLayer", "Slow heavy bombs"),
            ];
            kinds
                .iter()
                .enumerate()
                .map(|(i, (n, d))| {
                    let unlocked = (save.codex_enemies & (1u32 << i)) != 0;
                    (i, *n, *d, unlocked)
                })
                .collect()
        }
        1 => {
            let mods = [
                ("Frenzied", "Faster fire rate"),
                ("Bulwark", "Heavy armor"),
                ("Summoner", "Calls reinforcements"),
                ("Storm Core", "Spinning bullet rings"),
                ("Phantom", "Teleports"),
                ("Hydra", "Splits at 50%"),
            ];
            mods.iter()
                .enumerate()
                .map(|(i, (n, d))| {
                    let unlocked = (save.codex_bosses & (1u32 << i)) != 0;
                    (i, *n, *d, unlocked)
                })
                .collect()
        }
        _ => {
            let weapons = [
                ("Homing Missile", "Auto-lock target"),
                ("Orbit Drone", "Orbit drones aim at nearby targets"),
                ("Pulse Laser", "Tracking beam, sustained DPS"),
                ("Chain Bolt", "Long-range lightning jumps targets"),
                ("Void Rift", "Hunting damage field"),
                ("Wave Cannon", "Sine-wave bullets sweep the field"),
                ("Reflector", "Aimed ricochet shots"),
            ];
            weapons
                .iter()
                .enumerate()
                .map(|(i, (n, d))| {
                    let unlocked = (save.codex_weapons & (1u32 << i)) != 0;
                    (i, *n, *d, unlocked)
                })
                .collect()
        }
    };

    let row_h = 30.0;
    for (i, (_idx, name, _desc, unlocked)) in items.iter().enumerate() {
        let y = list_y + i as f32 * row_h;
        let selected = i == cursor;
        let fill = if selected {
            Color::new(0.06, 0.10, 0.18, 1.0)
        } else {
            Color::new(0.020, 0.035, 0.075, 0.9)
        };
        draw_rectangle(list_x, y, list_w, row_h - 4.0, fill);
        let edge = if selected {
            NEON_CYAN
        } else {
            alpha(MUTED, 0.3)
        };
        draw_rectangle_lines(list_x, y, list_w, row_h - 4.0, 1.0, edge);
        let display_name = if *unlocked {
            t(name, lang).to_string()
        } else {
            "??? ???".to_string()
        };
        let nc = if *unlocked { ICE_CYAN } else { MUTED };
        dt(&display_name, list_x + 12.0, y + 18.0, 13.0, nc, font);
    }

    // 详情面板
    if let Some((_, name, desc, unlocked)) = items.get(cursor) {
        draw_console_panel(
            detail_x,
            list_y,
            detail_w,
            260.0,
            Some(t("DETAIL", lang)),
            font,
        );
        if *unlocked {
            dt(t(name, lang), detail_x + 14.0, list_y + 28.0, 16.0, GOLD, font);
            dt(t(desc, lang), detail_x + 14.0, list_y + 52.0, 12.0, SOFT_WHITE, font);
        } else {
            dt("???", detail_x + 14.0, list_y + 30.0, 22.0, MUTED, font);
            dt(
                t("Encounter to reveal", lang),
                detail_x + 14.0,
                list_y + 60.0,
                11.0,
                MUTED,
                font,
            );
        }
    }

    let hint = t("←→ tab   ↑↓ select   ESC back", lang);
    let dh = mt(hint, 12, font);
    let pulse = 0.7 + (t_acc * 3.5).sin() * 0.2;
    dt(
        hint,
        cx - dh.width * 0.5,
        CFG.h - 28.0,
        12.0,
        alpha(ICE_CYAN, pulse),
        font,
    );
}

/// 局内 build 总览：当前主炮/副武器等级、生效 perks、关键属性 + 伤害分布。
/// 在暂停页和 GameOver 上调用，给玩家"我现在叠了什么"的明确反馈。
pub fn draw_build_summary(world: &World, font: Option<&Font>, lang: Lang, x: f32, y: f32, w: f32) {
    // 面板
    draw_console_panel(x, y, w, 198.0, Some(t("BUILD", lang)), font);

    // 第 1 列：武器
    let col1 = x + 14.0;
    let mut yy = y + 18.0;
    dt(t("Weapons", lang), col1, yy, 12.0, GOLD, font);
    yy += 16.0;
    let gun = format!("{}{}", t("Gun Lv", lang), world.weapons.main.level);
    dt(&gun, col1, yy, 12.0, ICE_CYAN, font);
    yy += 14.0;
    for s in &world.weapons.subs {
        let label = t(pretty_id(s.id()), lang);
        let txt = format!("{} Lv{}", label, s.level());
        dt(&txt, col1, yy, 12.0, SOFT_WHITE, font);
        yy += 14.0;
    }

    // 第 2 列：Perks + 关键数值
    let col2 = x + w * 0.36;
    yy = y + 18.0;
    dt(t("Perks", lang), col2, yy, 12.0, MAGENTA, font);
    yy += 16.0;
    let perks = &world.player.perks;
    let mut perk_lines: Vec<&str> = Vec::new();
    if perks.heat_lock {
        perk_lines.push(t("Heat Lock", lang));
    }
    if perks.static_mark {
        perk_lines.push(t("Static Mark", lang));
    }
    if perks.drone_relay {
        perk_lines.push(t("Drone Relay", lang));
    }
    if perks.gravity_well {
        perk_lines.push(t("Gravity Well", lang));
    }
    if perks.resonance {
        perk_lines.push(t("Resonance", lang));
    }
    if perks.prism {
        perk_lines.push(t("Prism", lang));
    }
    if perk_lines.is_empty() {
        dt("—", col2, yy, 12.0, MUTED, font);
    } else {
        for p in perk_lines {
            dt(p, col2, yy, 12.0, SOFT_WHITE, font);
            yy += 14.0;
        }
    }

    // 第 3 列：核心属性
    let col3 = x + w * 0.66;
    yy = y + 18.0;
    dt(t("Stats", lang), col3, yy, 12.0, ICE_CYAN, font);
    yy += 16.0;
    let st = &world.player.stats;
    let lines = [
        format!("{}: ×{:.2}", t("DMG", lang), st.damage_mul),
        format!("{}: {:.0}%", t("CRIT", lang), st.crit_chance * 100.0),
        format!("{}: ×{:.2}", t("CDMG", lang), st.crit_mul),
        format!("{}: {:.0}", t("SPD", lang), st.speed),
        format!("{}: ×{:.2}", t("XP", lang), st.xp_mul),
        format!("{}: ×{:.2}", t("SCORE", lang), st.score_mul),
    ];
    for line in lines {
        dt(&line, col3, yy, 12.0, SOFT_WHITE, font);
        yy += 14.0;
    }

    // 底部：伤害分布条（占 1 行）
    let bar_y = y + 156.0;
    let bar_x = x + 14.0;
    let bar_w = w - 28.0;
    let bar_h = 12.0;
    dt(
        t("Damage by source", lang),
        bar_x,
        bar_y - 4.0,
        11.0,
        MUTED,
        font,
    );
    let labels = ["MAIN", "MISL", "DRN", "LSR", "CHN", "RFT", "WAV", "RFL"];
    let palette = [
        Color::from_rgba(125, 249, 255, 255),
        Color::from_rgba(255, 200, 120, 255),
        Color::from_rgba(0, 212, 255, 255),
        Color::from_rgba(220, 250, 255, 255),
        Color::from_rgba(150, 220, 255, 255),
        Color::from_rgba(160, 100, 255, 255),
        Color::from_rgba(120, 255, 200, 255),
        Color::from_rgba(255, 255, 255, 255),
    ];
    let total: f32 = world.damage_by_source[..8].iter().sum();
    let total = total.max(1.0);
    let mut cur_x = bar_x;
    draw_rectangle(bar_x, bar_y + 8.0, bar_w, bar_h, Color::new(0.04, 0.07, 0.13, 1.0));
    for (i, &dmg) in world.damage_by_source[..8].iter().enumerate() {
        if dmg <= 0.0 {
            continue;
        }
        let frac = dmg / total;
        let seg_w = bar_w * frac;
        draw_rectangle(cur_x, bar_y + 8.0, seg_w, bar_h, palette[i]);
        // 段标签：占比足够大才显示
        if seg_w > 32.0 {
            let pct = (frac * 100.0).round() as u32;
            let lab = format!("{} {}%", labels[i], pct);
            let dim = mt(&lab, 10, font);
            if dim.width < seg_w {
                dt(
                    &lab,
                    cur_x + (seg_w - dim.width) * 0.5,
                    bar_y + 18.0,
                    10.0,
                    Color::new(0.04, 0.07, 0.13, 1.0),
                    font,
                );
            }
        }
        cur_x += seg_w;
    }

    // 角落小字：连击 / 击杀
    let stats_line = format!(
        "{} {}    {} {}",
        t("Kills", lang),
        world.kills,
        t("Peak", lang),
        world.max_combo,
    );
    let dim = mt(&stats_line, 11, font);
    dt(
        &stats_line,
        x + w - dim.width - 14.0,
        y + 14.0,
        11.0,
        MUTED,
        font,
    );
}

/// 共鸣槽 / 过载状态条（顶部中央，时间 ↓ 下方）。
fn draw_resonance(world: &World, font: Option<&Font>, lang: Lang) {
    let bar_w = 240.0;
    let bar_h = 6.0;
    let cx = CFG.w * 0.5;
    let by = 38.0;
    let bx = cx - bar_w * 0.5;

    draw_rectangle(bx, by, bar_w, bar_h, Color::from_rgba(20, 30, 50, 220));

    let ratio = world.synergy.ratio();
    let overloaded = world.synergy.is_overloaded();
    let fill_color = if overloaded {
        // 过载：金色脉冲
        let pulse = 0.85 + (world.run_time * 12.0).sin() * 0.15;
        Color::new(1.0, 0.86 * pulse, 0.3, 1.0)
    } else {
        Color::new(0.49, 0.97, 1.0, 0.85)
    };
    draw_rectangle(bx, by, bar_w * ratio, bar_h, fill_color);

    // 标签
    let label = if overloaded {
        format!(
            "◆ {} ◆ {:.1}s",
            t("OVERLOAD", lang),
            world.synergy.overload_remaining
        )
    } else {
        let pct = (ratio * 100.0).round() as u32;
        format!("{}  {}%", t("RESONANCE", lang), pct)
    };
    let dl = mt(&label, 11, font);
    let label_color = if overloaded {
        Color::new(1.0, 0.86, 0.3, 1.0)
    } else {
        Color::from_rgba(180, 200, 220, 255)
    };
    dt(
        &label,
        cx - dl.width * 0.5,
        by + bar_h + 12.0,
        11.0,
        label_color,
        font,
    );

    // 触发瞬间的全屏淡金色冲击波（进入过载）
    if world.overload_flash > 0.0 {
        let a = world.overload_flash * 0.18;
        draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::new(1.0, 0.86, 0.3, a));
    }
}

/// 章节切入时的标题/副标题 + "CH N / TOTAL" 章节计数。
/// `progress` 0..1：1 = 刚出现，0 = 完全淡出。
pub fn draw_chapter_intro(world: &World, font: Option<&Font>, lang: Lang) {
    if world.chapter_intro <= 0.0 {
        return;
    }
    let progress = (world.chapter_intro / 2.5).clamp(0.0, 1.0);
    // 缓入缓出：前 0.3 上抬，后段淡出。
    let a = if progress > 0.85 {
        ((1.0 - progress) / 0.15).clamp(0.0, 1.0)
    } else if progress < 0.30 {
        (progress / 0.30).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let chap = chapter::get(world.chapter_idx);
    let cx = CFG.w * 0.5;
    let y = CFG.h * 0.36;

    // 半透横幅
    draw_rectangle(0.0, y - 38.0, CFG.w, 80.0, alpha(BANNER_BG, a * 0.65));
    draw_rectangle(0.0, y - 38.0, CFG.w, 1.5, alpha(NEON_CYAN, a));
    draw_rectangle(0.0, y + 40.0, CFG.w, 1.5, alpha(NEON_CYAN, a));

    // CH n / total（无尽则直接 ENDLESS）
    let label = if chap.endless {
        format!("◆ {} ◆", t("ENDLESS", lang))
    } else {
        format!("{} {} / {}", t("CHAPTER", lang), chap.id, chapter::total())
    };
    let dl = mt(&label, 12, font);
    dt(
        &label,
        cx - dl.width * 0.5,
        y - 18.0,
        12.0,
        alpha(GOLD, a),
        font,
    );

    // 章节名
    let name = if lang == Lang::Zh {
        chap.name_zh
    } else {
        chap.name_en
    };
    let dn = mt(name, 26, font);
    dt(
        name,
        cx - dn.width * 0.5,
        y + 10.0,
        26.0,
        alpha(ICE_CYAN, a),
        font,
    );

    // 副标题
    let tag = if lang == Lang::Zh {
        chap.tagline_zh
    } else {
        chap.tagline_en
    };
    let dt2 = mt(tag, 12, font);
    dt(
        tag,
        cx - dt2.width * 0.5,
        y + 32.0,
        12.0,
        alpha(SOFT_WHITE, a * 0.85),
        font,
    );
}

/// 屏幕边缘绘制敌人方向指示器（对屏幕外的敌人显示小三角）
pub fn draw_off_screen_indicators(world: &World) {
    let m = 24.0; // 边距
    for e in &world.enemies {
        if e.dead {
            continue;
        }
        // 敌人在屏幕内则跳过
        if e.x > m && e.x < CFG.w - m && e.y > m && e.y < CFG.h - m {
            continue;
        }
        // 计算最近屏幕边缘交点
        let cx = e.x.clamp(m, CFG.w - m);
        let cy = e.y.clamp(m, CFG.h - m);
        // 指向敌人的方向角
        let angle = (e.y - cy).atan2(e.x - cx);
        // 小三角颜色：Kamikaze 更显眼
        let color = if matches!(e.kind, EnemyKind::Kamikaze) {
            Color::from_rgba(255, 70, 90, 255)
        } else {
            Color::from_rgba(255, 136, 102, 180)
        };
        let s = 6.0;
        draw_triangle(
            vec2(cx + angle.cos() * s, cy + angle.sin() * s),
            vec2(cx + (angle + 2.6).cos() * s, cy + (angle + 2.6).sin() * s),
            vec2(cx + (angle - 2.6).cos() * s, cy + (angle - 2.6).sin() * s),
            color,
        );
    }
}

pub fn draw_world(world: &World, t: f32, ox: f32, oy: f32) {
    for g in &world.pickups {
        g.draw(t, ox, oy);
    }
    for b in &world.bullets {
        b.draw(ox, oy);
    }
    for e in &world.enemies {
        e.draw(ox, oy);
    }
    if !world.player.dead {
        world.weapons.draw(&world.player, t, ox, oy);
        world.player.draw(t, ox, oy);
    }
    // 屏幕外敌人指示器
    draw_off_screen_indicators(world);
}

/// 全屏 vignette + 受击闪屏，覆盖在世界之上、HUD 之下。
pub fn draw_screen_fx(world: &World, fx: &crate::fx::Fx) {
    // 受击红闪：alpha 与 damage_flash 成正比
    if fx.damage_flash > 0.0 {
        let a = (fx.damage_flash * 0.55).min(0.55);
        draw_rectangle(0.0, 0.0, CFG.w, CFG.h, Color::new(1.0, 0.18, 0.20, a));
    }

    // 低血量红色脉冲 vignette
    if !world.player.dead && world.player.lives <= 1 {
        let pulse = 0.45 + (world.run_time * 4.5).sin() * 0.25;
        draw_vignette(Color::new(1.0, 0.10, 0.15, 0.35 * pulse));
    }

    // 过载金色 vignette
    if world.synergy.is_overloaded() {
        let pulse = 0.6 + (world.run_time * 5.0).sin() * 0.15;
        draw_vignette(Color::new(1.0, 0.78, 0.20, 0.28 * pulse));
    }

    // SUPER 蓄满时蓝色脉冲，提示玩家可以放
    if world.super_charge >= 1.0 {
        let pulse = 0.5 + (world.run_time * 3.0).sin() * 0.20;
        draw_vignette(Color::new(0.12, 0.85, 1.0, 0.18 * pulse));
    }
}

/// 屏幕四边的暗角：靠多个矩形渐变近似实现，避免 shader 依赖。
fn draw_vignette(c: Color) {
    let bands = 5usize;
    let edge = 60.0;
    for i in 0..bands {
        let t = (i as f32 + 1.0) / bands as f32;
        let mut col = c;
        col.a = c.a * (1.0 - t).powf(1.4);
        let inset = edge * t;
        // 上下
        draw_rectangle(0.0, 0.0, CFG.w, edge - inset, col);
        draw_rectangle(0.0, CFG.h - (edge - inset), CFG.w, edge - inset, col);
        // 左右
        draw_rectangle(0.0, 0.0, edge - inset, CFG.h, col);
        draw_rectangle(CFG.w - (edge - inset), 0.0, edge - inset, CFG.h, col);
    }
}
