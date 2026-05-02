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
        "U — pick upgrade card" => "U — 选择升级卡",
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
        "Orbit drones aim at nearby targets" => "环绕僚机自动瞄准附近目标",
        "Tracking beam, sustained DPS" => "追踪光束 持续伤害",
        "Long-range lightning jumps targets" => "远程闪电跳跃目标",
        "More & faster missiles" => "更多更快导弹",
        "More & faster drones" => "更多更快僚机",
        "Wider beam · more DPS" => "更宽光束 更高DPS",
        "More jumps · damage" => "更多跳数与伤害",
        "Missile marks targets · laser deals bonus damage" => "导弹标记目标 · 激光追加伤害",
        "Chain-charged targets are guaranteed crits once" => "被闪电充能的目标下一次必定暴击",
        "Drone kills launch a homing follow-up missile" => "僚机击杀会补发一枚追踪弹",
        "Hunting damage field" => "追猎型伤害场",
        "Sine-wave bullets sweep the field" => "正弦波弹丸扫荡战场",
        "Aimed ricochet shots" => "瞄准式反弹弹丸",
        "More rifts · faster pulses · wider" => "更多裂隙 · 更快脉冲 · 更广范围",
        "More waves · amplitude · speed" => "更多波数 · 更大振幅 · 更快射速",
        "More shots · bounces · speed" => "更多弹数 · 更多反弹 · 更快射速",
        "Rifts slowly pull enemies inward" => "裂隙缓慢吸入敌人",
        "Wave + Chain: hits trigger extra jumps" => "波动炮 + 闪电链：命中触发额外跳数",
        "Reflector + Laser: bounce through beam = +50% dmg & pierce" => {
            "反射镜 + 激光：穿过光束 +50% 伤害并穿透"
        }
        // 武器进化卡
        "Heatseeker" => "热寻者",
        "Swarm" => "蜂群",
        "Annihilator" => "歼灭者",
        "Tempest" => "风暴",
        "Cascade" => "瀑布",
        "Kaleidoscope" => "万花筒",
        "Missile evolved: +50% dmg, +1 per volley, larger blast" => "导弹进化：伤害 +50%、每次多 1 发、爆炸更大",
        "Drone evolved: +1 drone, faster fire" => "僚机进化：+1 僚机、射速更快",
        "Laser evolved: +60% DPS, +50% width, longer ON duty" => "激光进化：DPS +60%、宽度 +50%、持续更长",
        "Chain evolved: +2 jumps, +40% damage" => "闪电链进化：+2 跳数、伤害 +40%",
        "Wave evolved: +1 wave, +30% amplitude & dmg" => "波动炮进化：+1 波、振幅与伤害 +30%",
        "Reflector evolved: +1 shot, +2 bounces, +30% dmg" => "反射镜进化：+1 弹、+2 反弹、伤害 +30%",
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
        // 设置页
        "SETTINGS" => "设置",
        "Volume / Effects / Display" => "音量 / 特效 / 显示",
        "Master Volume" => "总音量",
        "Music Volume" => "音乐音量",
        "SFX Volume" => "音效音量",
        "Screen Shake" => "屏幕震动",
        "Audio Mute" => "静音",
        "Fullscreen" => "全屏",
        "On" => "开启",
        "Off" => "关闭",
        "↑↓ select   ←→ adjust   ENTER toggle   ESC back" => {
            "↑↓ 选择   ←→ 调节   回车 切换   ESC 返回"
        }
        "[O]  SETTINGS" => "[O]  设置",
        // build summary
        "BUILD" => "构筑",
        "Weapons" => "武器",
        "Perks" => "联动",
        "Stats" => "属性",
        "DMG" => "伤害",
        "CRIT" => "暴击率",
        "CDMG" => "暴伤",
        "SPD" => "速度",
        "XP" => "经验",
        "Damage by source" => "伤害分布",
        "Kills" => "击杀",
        "Peak" => "巅峰",
        "NORMAL" => "普通",
        "HARD" => "困难",
        "NIGHTMARE" => "噩梦",
        "ACHIEVEMENTS" => "成就",
        "DAILY" => "每日挑战",
        "Daily best" => "今日最佳",
        "TALENTS" => "天赋",
        "CHOOSE YOUR PATH" => "选择路线",
        "Standard" => "标准路线",
        "Onslaught" => "猛攻路",
        "Blitz" => "速攻路",
        "Harvest" => "收成路",
        "Balanced encounter" => "标准强度",
        "+30% HP · +20% score · extra buffs" => "敌人 +30% HP · 分数 +20% · 多掉 buff",
        "-15% HP · -25% chapter time" => "敌人 -15% HP · 章节时长 -25%",
        "Buff drops ×1.5 · +30% score" => "Buff 掉率 ×1.5 · 分数 +30%",
        "Enter / 1·2" => "回车 / 1·2",
        "CHAPTER" => "章节",
        "ENDLESS" => "无尽",
        "↑↓ select   ESC back" => "↑↓ 选择   ESC 返回",
        "CODEX" => "图鉴",
        "ENEMIES" => "敌人",
        "BOSS MODS" => "首领",
        "WEAPONS" => "武器",
        "DETAIL" => "详情",
        "Encounter to reveal" => "遭遇后解锁",
        "←→ tab   ↑↓ select   ESC back" => "←→ 切换分类   ↑↓ 选择   ESC 返回",
        "Lightweight scout" => "轻型侦察机",
        "Mid-tier shooter" => "中级射手",
        "Heavy armored" => "重型装甲",
        "Chapter boss" => "章节首领",
        "Suicide ram" => "自爆冲锋",
        "Side-sweeper" => "横扫机",
        "High-velocity shots" => "高速射击",
        "Sine-wave bullets" => "正弦弹幕",
        "Slow heavy bombs" => "慢速重弹",
        "Faster fire rate" => "射速加快",
        "Heavy armor" => "重装甲",
        "Calls reinforcements" => "召唤援军",
        "Spinning bullet rings" => "旋转弹环",
        "Teleports" => "瞬移",
        "Splits at 50%" => "半血分裂",
        // 拾取浮字
        "MAGNET" => "磁吸",
        "SHIELD" => "护盾",
        "+SUPER" => "+必杀",
        "DODGED!" => "躲避成功！",
        // buff 拾取浮字
        "+RATE" => "+射速",
        "+DMG" => "+伤害",
        "+VEL" => "+弹速",
        "+SPD" => "+移速",
        "+RANGE" => "+范围",
        "+XP" => "+经验",
        "+SCORE" => "+分数",
        "+CRIT" => "+暴击率",
        "+CRIT DMG" => "+暴击伤害",
        _ => s,
    }
}
