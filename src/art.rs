use macroquad::prelude::*;

use crate::entity::EnemyKind;
use crate::ship::ShipType;

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn tint(c: Color, mul: f32) -> Color {
    Color::new(
        (c.r * mul).clamp(0.0, 1.0),
        (c.g * mul).clamp(0.0, 1.0),
        (c.b * mul).clamp(0.0, 1.0),
        c.a,
    )
}

fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

/// 飞船涂装表：(body, trim, canopy, engine)。第 0 套是默认，1/2 是解锁皮肤。
pub fn ship_palette(ship: ShipType, variant: u8) -> (Color, Color, Color, Color) {
    match (ship, variant) {
        // —— Vanguard ——
        (ShipType::Vanguard, 0) => (
            rgba(196, 227, 255, 255),
            rgba(0, 205, 255, 255),
            rgba(255, 208, 102, 255),
            rgba(110, 248, 255, 255),
        ),
        (ShipType::Vanguard, 1) => (
            // Crimson：深红装甲 + 金色衬边
            rgba(255, 196, 188, 255),
            rgba(220, 60, 80, 255),
            rgba(255, 230, 110, 255),
            rgba(255, 130, 90, 255),
        ),
        (ShipType::Vanguard, _) => (
            // Voidshade：石墨灰 + 紫色辉光
            rgba(110, 116, 130, 255),
            rgba(170, 100, 240, 255),
            rgba(255, 200, 110, 255),
            rgba(180, 130, 255, 255),
        ),
        // —— Striker ——
        (ShipType::Striker, 0) => (
            rgba(227, 240, 255, 255),
            rgba(30, 255, 197, 255),
            rgba(120, 255, 240, 255),
            rgba(68, 225, 255, 255),
        ),
        (ShipType::Striker, 1) => (
            // Sunburst：明黄机身 + 暖橙
            rgba(255, 240, 180, 255),
            rgba(255, 170, 60, 255),
            rgba(255, 110, 60, 255),
            rgba(255, 200, 100, 255),
        ),
        (ShipType::Striker, _) => (
            // Frostbyte：冰蓝 + 银白
            rgba(220, 240, 255, 255),
            rgba(120, 200, 255, 255),
            rgba(180, 240, 255, 255),
            rgba(160, 220, 255, 255),
        ),
        // —— Engineer ——
        (ShipType::Engineer, 0) => (
            rgba(238, 230, 255, 255),
            rgba(170, 140, 255, 255),
            rgba(255, 217, 130, 255),
            rgba(125, 249, 255, 255),
        ),
        (ShipType::Engineer, 1) => (
            // Verdant：森林绿 + 青柠
            rgba(220, 250, 200, 255),
            rgba(80, 200, 110, 255),
            rgba(220, 255, 130, 255),
            rgba(160, 230, 130, 255),
        ),
        (ShipType::Engineer, _) => (
            // Obsidian：黑底 + 蓝紫光
            rgba(100, 110, 140, 255),
            rgba(100, 130, 230, 255),
            rgba(180, 200, 255, 255),
            rgba(160, 180, 255, 255),
        ),
    }
}

pub fn draw_player_ship_skin(
    ship: ShipType,
    variant: u8,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    t: f32,
) {
    let (body, trim, canopy, engine) = ship_palette(ship, variant);
    let dark = tint(body, 0.42);
    let mid = tint(body, 0.75);
    let glow = with_alpha(trim, 0.22 + (t * 4.0).sin().abs() * 0.10);

    draw_circle(x, y + h * 0.06, w * 0.44, glow);

    match ship {
        ShipType::Vanguard => {
            draw_triangle(
                vec2(x, y - h * 0.54),
                vec2(x + w * 0.18, y + h * 0.16),
                vec2(x - w * 0.18, y + h * 0.16),
                body,
            );
            draw_triangle(
                vec2(x - w * 0.48, y + h * 0.18),
                vec2(x - w * 0.12, y - h * 0.02),
                vec2(x - w * 0.18, y + h * 0.40),
                mid,
            );
            draw_triangle(
                vec2(x + w * 0.48, y + h * 0.18),
                vec2(x + w * 0.12, y - h * 0.02),
                vec2(x + w * 0.18, y + h * 0.40),
                mid,
            );
            draw_rectangle(x - w * 0.13, y - h * 0.02, w * 0.26, h * 0.42, dark);
            draw_rectangle(x - w * 0.36, y + h * 0.22, w * 0.22, h * 0.10, dark);
            draw_rectangle(x + w * 0.14, y + h * 0.22, w * 0.22, h * 0.10, dark);
            draw_triangle(
                vec2(x, y - h * 0.48),
                vec2(x + w * 0.10, y - h * 0.06),
                vec2(x - w * 0.10, y - h * 0.06),
                trim,
            );
        }
        ShipType::Striker => {
            draw_triangle(
                vec2(x, y - h * 0.56),
                vec2(x + w * 0.20, y + h * 0.18),
                vec2(x - w * 0.20, y + h * 0.18),
                body,
            );
            draw_triangle(
                vec2(x - w * 0.50, y + h * 0.14),
                vec2(x - w * 0.04, y - h * 0.14),
                vec2(x - w * 0.12, y + h * 0.34),
                trim,
            );
            draw_triangle(
                vec2(x + w * 0.50, y + h * 0.14),
                vec2(x + w * 0.04, y - h * 0.14),
                vec2(x + w * 0.12, y + h * 0.34),
                trim,
            );
            draw_triangle(
                vec2(x - w * 0.16, y + h * 0.10),
                vec2(x + w * 0.16, y + h * 0.10),
                vec2(x, y + h * 0.42),
                dark,
            );
            draw_triangle(
                vec2(x, y - h * 0.42),
                vec2(x + w * 0.08, y),
                vec2(x - w * 0.08, y),
                rgba(210, 255, 252, 255),
            );
        }
        ShipType::Engineer => {
            draw_rectangle(x - w * 0.12, y - h * 0.24, w * 0.24, h * 0.54, body);
            draw_triangle(
                vec2(x, y - h * 0.56),
                vec2(x + w * 0.14, y - h * 0.12),
                vec2(x - w * 0.14, y - h * 0.12),
                trim,
            );
            draw_rectangle(x - w * 0.40, y - h * 0.02, w * 0.18, h * 0.34, mid);
            draw_rectangle(x + w * 0.22, y - h * 0.02, w * 0.18, h * 0.34, mid);
            draw_triangle(
                vec2(x - w * 0.50, y + h * 0.18),
                vec2(x - w * 0.12, y + h * 0.02),
                vec2(x - w * 0.20, y + h * 0.36),
                dark,
            );
            draw_triangle(
                vec2(x + w * 0.50, y + h * 0.18),
                vec2(x + w * 0.12, y + h * 0.02),
                vec2(x + w * 0.20, y + h * 0.36),
                dark,
            );
            draw_rectangle(x - w * 0.08, y + h * 0.22, w * 0.16, h * 0.16, dark);
        }
    }

    draw_circle(x, y - h * 0.14, w * 0.09, with_alpha(canopy, 0.45));
    draw_circle(x, y - h * 0.14, w * 0.05, canopy);

    for off in [-1.0_f32, 1.0] {
        let ex = x + off * w * 0.14;
        draw_circle(ex, y + h * 0.34, w * 0.06, with_alpha(engine, 0.25));
        draw_rectangle(ex - w * 0.028, y + h * 0.24, w * 0.056, h * 0.10, engine);
    }
}

pub fn draw_player_preview_skin(
    ship: ShipType,
    variant: u8,
    x: f32,
    y: f32,
    scale: f32,
    t: f32,
) {
    let (_, trim, _, _) = ship_palette(ship, variant);
    let mut aura = trim;
    aura.a = 0.35;
    draw_circle(x, y + 6.0, 34.0 * scale, aura);
    draw_player_ship_skin(ship, variant, x, y, 58.0 * scale, 72.0 * scale, t);
}

pub fn draw_enemy_ship(kind: EnemyKind, x: f32, y: f32, w: f32, h: f32, base: Color, hp_pct: f32) {
    let dark = tint(base, 0.34);
    let mid = tint(base, 0.72);
    let hi = tint(base, 1.18);
    let cockpit = rgba(255, 225, 140, 255);

    match kind {
        EnemyKind::Small => {
            draw_triangle(
                vec2(x, y + h * 0.52),
                vec2(x + w * 0.28, y - h * 0.06),
                vec2(x - w * 0.28, y - h * 0.06),
                base,
            );
            draw_triangle(
                vec2(x + w * 0.46, y - h * 0.18),
                vec2(x + w * 0.10, y - h * 0.02),
                vec2(x + w * 0.20, y + h * 0.18),
                mid,
            );
            draw_triangle(
                vec2(x - w * 0.46, y - h * 0.18),
                vec2(x - w * 0.10, y - h * 0.02),
                vec2(x - w * 0.20, y + h * 0.18),
                mid,
            );
            draw_rectangle(x - w * 0.06, y - h * 0.18, w * 0.12, h * 0.18, dark);
            draw_circle(x, y + h * 0.12, w * 0.06, cockpit);
        }
        EnemyKind::Medium => {
            draw_triangle(
                vec2(x, y + h * 0.52),
                vec2(x + w * 0.20, y - h * 0.06),
                vec2(x - w * 0.20, y - h * 0.06),
                base,
            );
            draw_triangle(
                vec2(x - w * 0.52, y),
                vec2(x - w * 0.06, y - h * 0.18),
                vec2(x - w * 0.18, y + h * 0.22),
                mid,
            );
            draw_triangle(
                vec2(x + w * 0.52, y),
                vec2(x + w * 0.06, y - h * 0.18),
                vec2(x + w * 0.18, y + h * 0.22),
                mid,
            );
            draw_rectangle(x - w * 0.12, y - h * 0.26, w * 0.24, h * 0.36, dark);
            draw_triangle(
                vec2(x, y - h * 0.42),
                vec2(x + w * 0.28, y - h * 0.10),
                vec2(x - w * 0.28, y - h * 0.10),
                hi,
            );
            draw_circle(x, y - h * 0.02, w * 0.07, cockpit);
        }
        EnemyKind::Large => {
            draw_rectangle(x - w * 0.28, y - h * 0.28, w * 0.56, h * 0.52, base);
            draw_triangle(
                vec2(x, y + h * 0.56),
                vec2(x + w * 0.18, y + h * 0.10),
                vec2(x - w * 0.18, y + h * 0.10),
                mid,
            );
            draw_triangle(
                vec2(x - w * 0.56, y),
                vec2(x - w * 0.20, y - h * 0.12),
                vec2(x - w * 0.28, y + h * 0.24),
                dark,
            );
            draw_triangle(
                vec2(x + w * 0.56, y),
                vec2(x + w * 0.20, y - h * 0.12),
                vec2(x + w * 0.28, y + h * 0.24),
                dark,
            );
            draw_rectangle(x - w * 0.44, y - h * 0.10, w * 0.16, h * 0.30, mid);
            draw_rectangle(x + w * 0.28, y - h * 0.10, w * 0.16, h * 0.30, mid);
            draw_rectangle(
                x - w * 0.18,
                y - h * 0.06,
                w * 0.36,
                h * 0.12,
                rgba(22, 10, 20, 255),
            );
            draw_circle(x - w * 0.16, y - h * 0.02, w * 0.05, cockpit);
            draw_circle(x + w * 0.16, y - h * 0.02, w * 0.05, cockpit);

            draw_rectangle(
                x - w * 0.30,
                y - h * 0.42,
                w * 0.60,
                4.0,
                rgba(0, 0, 0, 130),
            );
            draw_rectangle(
                x - w * 0.30,
                y - h * 0.42,
                w * 0.60 * hp_pct.clamp(0.0, 1.0),
                4.0,
                rgba(120, 255, 160, 255),
            );
        }
        EnemyKind::Boss => {
            draw_rectangle(x - w * 0.20, y - h * 0.34, w * 0.40, h * 0.56, base);
            draw_triangle(
                vec2(x, y + h * 0.58),
                vec2(x + w * 0.16, y + h * 0.10),
                vec2(x - w * 0.16, y + h * 0.10),
                hi,
            );
            draw_triangle(
                vec2(x - w * 0.50, y - h * 0.08),
                vec2(x - w * 0.10, y - h * 0.24),
                vec2(x - w * 0.24, y + h * 0.26),
                mid,
            );
            draw_triangle(
                vec2(x + w * 0.50, y - h * 0.08),
                vec2(x + w * 0.10, y - h * 0.24),
                vec2(x + w * 0.24, y + h * 0.26),
                mid,
            );
            draw_triangle(
                vec2(x - w * 0.82, y - h * 0.04),
                vec2(x - w * 0.50, y - h * 0.02),
                vec2(x - w * 0.58, y + h * 0.22),
                dark,
            );
            draw_triangle(
                vec2(x + w * 0.82, y - h * 0.04),
                vec2(x + w * 0.50, y - h * 0.02),
                vec2(x + w * 0.58, y + h * 0.22),
                dark,
            );
            draw_rectangle(
                x - w * 0.32,
                y - h * 0.06,
                w * 0.64,
                h * 0.14,
                rgba(35, 8, 24, 255),
            );
            draw_circle(x - w * 0.16, y, w * 0.05, with_alpha(cockpit, 0.35));
            draw_circle(x + w * 0.16, y, w * 0.05, with_alpha(cockpit, 0.35));
            draw_circle(x - w * 0.16, y, w * 0.025, cockpit);
            draw_circle(x + w * 0.16, y, w * 0.025, cockpit);
            draw_circle(x, y - h * 0.18, w * 0.07, with_alpha(hi, 0.25));
            draw_circle(x, y - h * 0.18, w * 0.04, hi);
        }
        EnemyKind::Kamikaze => {
            // 红色菱形 + 双引擎尾焰，营造"突击"感。
            let pulse = with_alpha(hi, 0.55);
            draw_circle(x, y, w * 0.7, with_alpha(base, 0.20));
            draw_triangle(
                vec2(x, y - h * 0.55),
                vec2(x + w * 0.50, y),
                vec2(x, y + h * 0.30),
                base,
            );
            draw_triangle(
                vec2(x, y - h * 0.55),
                vec2(x - w * 0.50, y),
                vec2(x, y + h * 0.30),
                mid,
            );
            draw_triangle(
                vec2(x - w * 0.18, y + h * 0.10),
                vec2(x + w * 0.18, y + h * 0.10),
                vec2(x, y + h * 0.55),
                pulse,
            );
            draw_circle(x, y - h * 0.05, w * 0.10, dark);
            draw_circle(x, y - h * 0.05, w * 0.06, cockpit);
        }
        EnemyKind::Strafer => {
            // 长条扁机身 + 翼端发射点。
            draw_rectangle(x - w * 0.45, y - h * 0.20, w * 0.90, h * 0.40, base);
            draw_triangle(
                vec2(x - w * 0.45, y - h * 0.20),
                vec2(x - w * 0.65, y),
                vec2(x - w * 0.45, y + h * 0.20),
                mid,
            );
            draw_triangle(
                vec2(x + w * 0.45, y - h * 0.20),
                vec2(x + w * 0.65, y),
                vec2(x + w * 0.45, y + h * 0.20),
                mid,
            );
            draw_rectangle(x - w * 0.10, y - h * 0.40, w * 0.20, h * 0.40, dark);
            draw_circle(x, y - h * 0.20, w * 0.06, cockpit);
            draw_circle(x - w * 0.55, y + h * 0.10, w * 0.05, with_alpha(hi, 0.7));
            draw_circle(x + w * 0.55, y + h * 0.10, w * 0.05, with_alpha(hi, 0.7));
        }
        EnemyKind::Sniper => {
            // 窄长机体 + 明显炮管，让高速狙击弹有来源感。
            draw_triangle(
                vec2(x, y + h * 0.54),
                vec2(x + w * 0.24, y - h * 0.12),
                vec2(x - w * 0.24, y - h * 0.12),
                base,
            );
            draw_rectangle(x - w * 0.08, y - h * 0.48, w * 0.16, h * 0.76, dark);
            draw_rectangle(x - w * 0.05, y + h * 0.14, w * 0.10, h * 0.42, hi);
            draw_triangle(
                vec2(x - w * 0.42, y - h * 0.04),
                vec2(x - w * 0.08, y - h * 0.12),
                vec2(x - w * 0.16, y + h * 0.20),
                mid,
            );
            draw_triangle(
                vec2(x + w * 0.42, y - h * 0.04),
                vec2(x + w * 0.08, y - h * 0.12),
                vec2(x + w * 0.16, y + h * 0.20),
                mid,
            );
            draw_circle(x, y - h * 0.12, w * 0.07, cockpit);
        }
        EnemyKind::Weaver => {
            // 双翼弧形机，和它的蛇形弹呼应。
            draw_circle(x, y, w * 0.28, base);
            draw_triangle(
                vec2(x - w * 0.58, y - h * 0.18),
                vec2(x - w * 0.08, y - h * 0.08),
                vec2(x - w * 0.44, y + h * 0.30),
                mid,
            );
            draw_triangle(
                vec2(x + w * 0.58, y - h * 0.18),
                vec2(x + w * 0.08, y - h * 0.08),
                vec2(x + w * 0.44, y + h * 0.30),
                mid,
            );
            draw_triangle(
                vec2(x, y + h * 0.50),
                vec2(x + w * 0.14, y + h * 0.04),
                vec2(x - w * 0.14, y + h * 0.04),
                hi,
            );
            draw_circle(x - w * 0.20, y + h * 0.06, w * 0.05, cockpit);
            draw_circle(x + w * 0.20, y + h * 0.06, w * 0.05, cockpit);
        }
        EnemyKind::MineLayer => {
            // 厚重投弹舱，强调慢速压迫型弹幕。
            draw_rectangle(x - w * 0.30, y - h * 0.30, w * 0.60, h * 0.56, base);
            draw_triangle(
                vec2(x - w * 0.56, y - h * 0.06),
                vec2(x - w * 0.24, y - h * 0.22),
                vec2(x - w * 0.30, y + h * 0.30),
                dark,
            );
            draw_triangle(
                vec2(x + w * 0.56, y - h * 0.06),
                vec2(x + w * 0.24, y - h * 0.22),
                vec2(x + w * 0.30, y + h * 0.30),
                dark,
            );
            draw_rectangle(x - w * 0.18, y + h * 0.04, w * 0.36, h * 0.20, mid);
            draw_circle(x - w * 0.16, y + h * 0.16, w * 0.07, hi);
            draw_circle(x + w * 0.16, y + h * 0.16, w * 0.07, hi);
            draw_circle(x, y - h * 0.08, w * 0.08, cockpit);
        }
    }
}
