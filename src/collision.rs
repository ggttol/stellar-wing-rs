//! 圆形 + AABB-circle 碰撞检测。等价于原 shooter.html 的 hitCircle / hitBulletEntity。

use crate::entity::{Bullet, Enemy, Player};

pub fn hit_circle(ax: f32, ay: f32, ar: f32, bx: f32, by: f32, br: f32) -> bool {
    let dx = ax - bx;
    let dy = ay - by;
    let r = ar + br;
    dx * dx + dy * dy < r * r
}

pub fn bullet_hits_enemy(b: &Bullet, e: &Enemy) -> bool {
    let dx = (b.x - e.x).abs();
    let dy = (b.y - e.y).abs();
    dx < (b.w * 0.5 + e.radius) && dy < (b.h * 0.5 + e.radius)
}

pub fn bullet_hits_player(b: &Bullet, p: &Player) -> bool {
    let dx = (b.x - p.x).abs();
    let dy = (b.y - p.y).abs();
    dx < (b.w * 0.5 + p.radius) && dy < (b.h * 0.5 + p.radius)
}
