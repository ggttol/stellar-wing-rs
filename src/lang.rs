//! 简易 i18n：中英双语，按文本字面量做查表翻译。
//! 默认中文；找不到 CJK 字体时会自动降级为英文。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Lang {
    En,
    #[default]
    Zh,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::En => Lang::Zh,
            Lang::Zh => Lang::En,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
        }
    }
}

/// 查表翻译。`s` 是英文字面量；未命中时原样返回。
/// 返回的生命周期跟随入参——字面量自然是 'static，命中分支也是 'static。
pub fn t(s: &str, lang: Lang) -> &str {
    if lang == Lang::En {
        return s;
    }
    match s {
        // 标题 & 主菜单
        "STELLAR WING" => "星际战机",
        "Rust Edition  ·  Roguelike Mode" => "Rust 版 · 肉鸽模式",
        "HIGH SCORE" => "最高分",
        "— TOP 5 —" => "— 排行榜 —",
        "Press ENTER to start" => "按 回车 开始",
        "[Muted]" => "[静音]",
        "WASD / Arrows — Move    P / ESC — Pause" => "WASD / 方向键 — 移动    P / ESC — 暂停",
        "M — Mute    F — Fullscreen" => "M — 静音    F — 全屏    L — 语言",
        "Auto-fire · Collect XP gems → pick a card" => "自动开火 · 拾取经验 → 三选一升级",
        "A / D or ← / → — Select ship" => "A / D 或 ← / → — 选择机体",
        // HUD
        "SCORE" => "分数",
        "HIGH" => "最高",
        "LV" => "等级",
        "Gun Lv" => "主炮 Lv",
        "Ship" => "机体",
        "SUPER" => "必杀",
        // Pause / GameOver
        "PAUSED" => "暂停",
        "P / ESC — resume" => "P / ESC — 继续",
        "Q — quit to menu" => "Q — 返回菜单",
        "GAME OVER" => "游戏结束",
        "Score" => "分数",
        "High" => "最高",
        "Level reached:" => "达到等级:",
        "★ NEW RECORD ★" => "★ 新纪录 ★",
        "ENTER restart  ·  ESC menu" => "回车 重开 · ESC 菜单",
        // Boss
        "— BOSS —" => "— 首领 —",
        // Upgrade UI
        "LEVEL UP" => "升级！",
        "1 / 2 / 3   ·   ← →   ·   Enter   ·   click" => {
            "1 / 2 / 3   ·   ← →   ·   回车   ·   点击"
        }
        "Enter / Click" => "回车 / 点击",
        // 稀有度
        "Common" => "普通",
        "Rare" => "稀有",
        "Epic" => "史诗",
        "Legendary" => "传奇",
        // 卡片名
        "Rapid Fire" => "急速射击",
        "High Caliber" => "高威力弹",
        "Velocity" => "高速弹道",
        "Afterburner" => "后燃推进",
        "Magnetic Field" => "磁力场",
        "Sharp Eyes" => "鹰眼",
        "Bounty Hunter" => "赏金猎人",
        "Hull Plating" => "装甲强化",
        "Sniper Lens" => "狙击镜",
        "Devastator" => "毁灭打击",
        "Auto-Repair" => "自动修复",
        "Adrenaline" => "肾上腺素",
        "Energy Shield" => "能量护盾",
        "Repair Kit" => "修理包",
        "Main Gun +1" => "主炮 +1",
        "Homing Missile" => "跟踪导弹",
        "Orbit Drone" => "环绕僚机",
        "Pulse Laser" => "脉冲激光",
        "Chain Bolt" => "闪电链",
        "Missile +1" => "导弹 +1",
        "Drone +1" => "僚机 +1",
        "Laser +1" => "激光 +1",
        "Chain +1" => "闪电链 +1",
        "Heat Lock" => "热锁定",
        "Static Mark" => "静电印记",
        "Drone Relay" => "僚机中继",
        "Void Rift" => "虚空裂隙",
        "Wave Cannon" => "波动炮",
        "Rift +1" => "裂隙 +1",
        "Wave +1" => "波动炮 +1",
        "Reflector +1" => "反射镜 +1",
        "Gravity Well" => "重力井",
        "Resonance" => "谐振",
        "Prism" => "棱镜",
        "Frenzied" => "狂暴核心",
        "Bulwark" => "堡垒装甲",
        "Summoner" => "召唤核心",
        "Storm Core" => "风暴核心",
        "Vanguard" => "先锋型",
        "Striker" => "突击型",
        "Engineer" => "工程型",
        // 卡片描述
        "Fire rate +15%" => "射速 +15%",
        "Damage +20%" => "伤害 +20%",
        "Bullet speed +25%" => "子弹速度 +25%",
        "Move speed +15%" => "移动速度 +15%",
        "Pickup range +50%" => "拾取范围 +50%",
        "XP gain +30%" => "经验 +30%",
        "Score +25%" => "得分 +25%",
        "Max HP +1" => "最大生命 +1",
        "Crit chance +10%" => "暴击率 +10%",
        "Crit damage +50%" => "暴击伤害 +50%",
        "+0.5 HP / minute" => "每分钟回 0.5 血",
        "I-frames +30%" => "无敌时长 +30%",
        "Block one hit" => "格挡一次",
        "Refill HP now" => "立即满血",
        "Single→Dual→Triple→5w→Pierce" => "单→双→三→五→穿透",
        "Auto-lock target" => "自动锁敌",
        "Spinning satellite" => "环绕僚机",
        "Vertical beam, DPS" => "垂直光束 持续伤害",
        "Lightning jumps targets" => "闪电跳跃目标",
        "More & faster missiles" => "更多更快导弹",
        "More & faster drones" => "更多更快僚机",
        "Wider beam · more DPS" => "更宽光束 更高DPS",
        "More jumps · damage" => "更多跳数与伤害",
        "Missile marks targets · laser deals bonus damage" => "导弹标记目标 · 激光追加伤害",
        "Chain-charged targets are guaranteed crits once" => "被闪电充能的目标下一次必定暴击",
        "Drone kills launch a homing follow-up missile" => "僚机击杀会补发一枚追踪弹",
        "Deploy a pulsing damage field" => "部署脉冲伤害场",
        "Sine-wave bullets sweep the field" => "正弦波弹丸扫荡战场",
        "Bullets bounce off screen edges" => "弹丸碰壁反弹",
        "More rifts · faster pulses · wider" => "更多裂隙 · 更快脉冲 · 更广范围",
        "More waves · amplitude · speed" => "更多波数 · 更大振幅 · 更快射速",
        "More shots · bounces · speed" => "更多弹数 · 更多反弹 · 更快射速",
        "Rifts slowly pull enemies inward" => "裂隙缓慢吸入敌人",
        "Wave + Chain: hits trigger extra jumps" => "波动炮 + 闪电链：命中触发额外跳数",
        "Reflector + Laser: bounce through beam = +50% dmg & pierce" => {
            "反射镜 + 激光：穿过光束 +50% 伤害并穿透"
        }
        "Main Gun Lv2, stronger frontal burst" => "开局主炮 Lv2，正面火力更强",
        "Move speed +18%, tighter evasion" => "移动速度 +18%，闪避更灵活",
        "Starts with a support weapon, weaker gun" => "开局自带副武器，但主炮稍弱",
        "Lock-on volley" => "锁定齐射",
        "Fan barrage" => "扇面弹幕",
        "Core burst" => "核心爆发",
        // 副武器名（HUD pretty_id）
        "Missile" => "导弹",
        "Drone" => "僚机",
        "Laser" => "激光",
        "Chain" => "闪电链",
        "Rift" => "裂隙",
        "Wave" => "波动炮",
        // 其它
        "Language:" => "语言:",
        "Magnet" => "磁吸",
        "COMBO" => "连击",
        // 拾取浮字
        "MAGNET" => "磁吸",
        "SHIELD" => "护盾",
        "+SUPER" => "+必杀",
        "DODGED!" => "躲避成功！",
        _ => s,
    }
}
