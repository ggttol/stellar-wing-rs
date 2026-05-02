# Stellar Wing — 跨平台移植设计规格

本文档把当前 Rust + macroquad 实现的所有玩法、数值、系统抽出来，**让移植者不需要读 Rust 源码**也能在 Cocos Creator / Unity / 其他引擎里复刻一份。

> 数据 / 公式以"代码里实际跑的值"为准。后期 tune 后请回头同步本文。

---

## 0. 工程目标

- **逻辑分辨率**：480 × 800（竖屏，移动端友好）
- **目标平台**：当前 macOS / Windows / Linux 桌面；规划微信小游戏 / 移动端
- **每帧 dt 上限**：0.05 秒（防卡顿后大跳步）
- **运行节奏**：单线程，60fps 目标

---

## 1. 场景状态机

```
Menu ──[Enter]──► Playing
Menu ──[T]─────► Talents      ──[Esc]──► Menu
Menu ──[O]─────► Settings     ──[Esc]──► Menu
Menu ──[H]─────► Achievements ──[Esc]──► Menu
Menu ──[C]─────► Codex        ──[Esc]──► Menu
Menu ──[Y]─────► Playing(daily_mode=true)
Menu ──[N]─────► (cycle difficulty in-place)
Menu ──[K]─────► (cycle current ship's skin in-place)

Playing ──[Esc/P]──► Paused        ──[Esc/P]──► Playing / [Q]──► Menu
Playing ──[xp 满]──► UpgradePick   ──[选完]──► Playing
Playing ──[Boss 死, 章节切]──► ChapterChoice ──[选完]──► Playing
Playing ──[player.dead]──► GameOver ──[Enter]──► Playing / [Esc]──► Menu
```

**触屏移植提示**：所有"按键进入子页"应当映射成菜单上的可点按钮；按 [N][K] 这类"在原地循环切换"应当变成多按钮 segmented control。

---

## 2. 主循环（`step_play`）每帧顺序

每个玩法 tick 的执行顺序**绝对不能乱**——不然连击 / 标记 / 掉落归属都会错：

```
0. 累加 t_acc（真实时间）
1. fx.tick_time_modifiers(real_dt) → play_dt（处理 hit-pause / slow-mo）
2. world.run_time += play_dt
3. 连击衰减（combo_timer / combo_flash / overload_flash）
4. synergy.tick(play_dt)
5. player.update(play_dt) — 输入、摩擦、推进器粒子
6. weapons.tick(play_dt) — 主炮触发 fire / 副武器各自 tick
   - 副武器修改 enemies.hp 时同步累计 damage_by_source[HitSource]
7. chapter_time += play_dt（仅 boss 不在场时；chapter_intro > 0 时也跳过）
8. spawn_chapter_wave + spawn_chapter_boss（根据 chapter_time / chap.duration）
9. 敌人 update（dt 乘以 diff_mul, boss 不乘）+ enemy_spawns 入列
10. steer_homing_bullets（追踪弹转向）
11. 所有 bullets.update(dt)
12. 给玩家子弹生成 trail 点
13. collect_pickups
14. SUPER 触发（Space）
15. resolve_player_bullets — 玩家子弹打敌人，写 damage_by_source
16. process_kills — 阵亡敌人：连击/分数/SUPER/爆破/掉落，更新 codex_*_run
17. 章节推进（boss 全清 → chapter_idx +=1，重置 chapter_modifier=None / chapter_no_hit=true）
18. resolve_enemy_bullets / resolve_enemy_player_contact（玩家受击 → damage_flash + chapter_no_hit=false）
19. 低血量 beep（lives==1，每 1.2 秒）
20. 躲避 Kamikaze 奖励
21. 清理 dead bullet/enemy
22. fx.update(play_dt)
23. 成就检查（每帧；已解锁的会跳过）
24. 升级判断（xp >= xp_to_next → 进 UpgradePick 场景）
```

---

## 3. 数值核心

### 3.1 经验曲线
```
xp_to_next(level) = 6 + 4*(L-1) + 4*(L-1)^2
```
| Lv | 6→2  | 2→3  | 3→4  | 4→5  | 5→6  | 8→9  | 12→13 |
|----|------|------|------|------|------|------|-------|
| XP | 6    | 14   | 30   | 54   | 86   | 202  | 490   |

### 3.2 难度
| Difficulty | HP×  | Bullet Spd× | XP× | Score× | Stardust× |
|------------|------|-------------|-----|--------|-----------|
| Normal     | 1.00 | 1.00        | 1.00| 1.00   | 1.0       |
| Hard       | 1.25 | 1.15        | 1.10| 1.20   | 1.5       |
| Nightmare  | 1.60 | 1.25        | 1.25| 1.50   | 2.0       |

通关 Normal 解锁 Hard；通关 Hard 解锁 Nightmare。

### 3.3 章节修饰（章末 2 选 1）
| Mod        | HP×  | Score× | Buff Drop× | Duration× |
|------------|------|--------|------------|-----------|
| None       | 1.00 | 1.00   | 1.00       | 1.00      |
| Onslaught  | 1.30 | 1.20   | 1.25       | 1.00      |
| Blitz      | 0.85 | 1.00   | 1.00       | 0.75      |
| Harvest    | 1.00 | 1.30   | 1.50       | 1.00      |

每章 (chapter_idx % 3) 决定哪两条出现：`{0:[Ons,Blitz], 1:[Blitz,Harv], 2:[Ons,Harv]}`。

### 3.4 难度时间增长（每只敌人在 spawn 时叠）
- HP_mul = (1 + run_time/55) × difficulty_hp × chap_mod_hp
- Bullet damage = 1 + run_time/100
- Bullet speed mul = (0.58 + warmup×0.42) × difficulty_bspd 其中 warmup = clamp(run_time/120, 0, 1)
- Fire rate = base × (1.35 - warmup×0.35)（>90s 再 ×0.92）
- Score = base × (1 + run_time/180) × score_mul × diff_score × chap_mod_score
- XP = base × (1 + run_time/220) × diff_xp

### 3.5 精英敌人
当 run_time ≥ 25：每只非 Boss/Kamikaze/Strafer 敌人按 kind 概率变精英
- Small 7%, Medium 12%, Large 18%, Sniper/Weaver/MineLayer 10%
- 精英词缀（随机）：Armored / Berserk / Dasher
- 精英 buff 掉落概率 ×2

---

## 4. 玩家

### 4.1 PlayerStats（默认值）
| 字段             | 默认  | 说明                       |
|------------------|-------|----------------------------|
| speed            | 1500  | 加速度（与 friction 配合） |
| friction         | 0.86  | 每帧基于 60fps 衰减系数    |
| fire_rate        | 0.30  | 主炮射击间隔（s），cap 0.18 |
| bullet_speed     | 800   | 子弹速度，cap 1200          |
| damage_mul       | 1.0   | cap 2.35                    |
| max_lives        | 3     |                             |
| invincible       | 1.5   | 受击无敌时长（s），cap 1.9  |
| pickup_radius    | 22    | 必拾半径                    |
| attract_radius   | 90    | 吸附半径，cap 230           |
| crit_chance      | 0.0   | cap 0.35                    |
| crit_mul         | 2.0   | cap 2.8                     |
| score_mul        | 1.0   | cap 2.2                     |
| xp_mul           | 1.0   | cap 2.0                     |
| regen_per_min    | 0.0   | 每分钟回 X 血，cap 1.2      |

**摩擦实现**：`vx *= friction.powf(dt * 60.0)`（连续衰减，不依赖固定帧率）

### 4.2 三种飞船
| Ship      | Apply 内容                               |
|-----------|-----------------------------------------|
| Vanguard  | 主炮起手 Lv2，正面火力强                |
| Striker   | speed ×1.18，闪避更灵活                  |
| Engineer  | 开局自带一件副武器，主炮稍弱             |

### 4.3 输入（桌面版）
- 移动：WASD / 方向键（8 方向，归一化）
- 释放 SUPER：Space
- 选卡：1/2/3 或 ←→ + Enter 或 鼠标点击

**触屏映射建议**：
- 左下虚拟摇杆（半径 100px）→ 输出 ax, ay
- 主炮自动开火（已经是）
- 右下两个圆按钮：[SUPER]（亮起=可放）+ [PAUSE]
- 选卡：直接点卡片

### 4.4 受击 hit(t)
1. 若 t < invincible_until → 返回 false（无效）
2. 若有 shield → 消耗 shield + 0.5s i-frames，返回 false
3. lives -= 1，invincible_until = t + invincible，若 lives==0 → dead=true

---

## 5. 敌人

### 5.1 9 种敌人 stats
| Kind       | HP  | Score | XP  | W×H   | Radius | Speed | Fire/s |
|------------|-----|-------|-----|-------|--------|-------|--------|
| Small      | 1.0 | 10    | 1   | 34×34 | 15     | 90    | 0      |
| Medium     | 2.0 | 30    | 3   | 46×46 | 20     | 70    | 1.8    |
| Large      | 5.0 | 100   | 8   | 78×72 | 32     | 45    | 1.1    |
| Boss       | 80.0| 2500  | 120 | 180×110| 70    | 35    | 0.9    |
| Kamikaze   | 1.5 | 25    | 2   | 26×30 | 13     | 260   | 0      |
| Strafer    | 2.0 | 45    | 3   | 38×30 | 16     | 220   | 0.55   |
| Sniper     | 2.4 | 55    | 4   | 42×50 | 19     | 52    | 2.35   |
| Weaver     | 2.2 | 50    | 4   | 44×38 | 18     | 78    | 1.15   |
| MineLayer  | 4.0 | 85    | 6   | 62×54 | 25     | 46    | 1.65   |

### 5.2 Boss 修饰（6 种，章节抽取）
- **Frenzied**：射速更快
- **Bulwark**：HP 翻倍 / 重装甲
- **Summoner**：周期召唤援军
- **StormCore**：旋转弹环
- **Phantom**：周期瞬移（短暂无敌）
- **Hydra**：50% 血时分裂

### 5.3 攻击告警（telegraph）
所有 boss 远程攻击前 0.6-1.0s 显示 telegraph 动画（红色十字光圈 / 扇面预警），防"屏外打死"。

### 5.4 XP 宝石分裂掉落
| Kind        | 碎片数 |
|-------------|--------|
| Small/Kami  | 1      |
| Medium/Strafer/Sniper/Weaver | 2 |
| MineLayer   | 3      |
| Large       | 4      |
| Boss        | 16     |

每片 XP = ceil(enemy.xp / pieces)。

---

## 6. 武器

### 6.1 主炮（5 级演化）
| Lv | 形态         |
|----|--------------|
| 1  | 单发         |
| 2  | 双发并排     |
| 3  | 三向（↖↑↗）  |
| 4  | 五向         |
| 5  | 穿透         |

子弹 base_damage = 1.0 × player.damage_mul，速度 = player.bullet_speed，从 player 头顶发射，方向 ↑（除三/五向有夹角）。

### 6.2 副武器（7 种，每种 5 级）

#### Missile
- interval = max(1.55 - (L-1)×0.14, 0.78)
- count by L: [1, 1, 2, 2, 3]
- damage_mul = 1.65 + L×0.05
- homing=true, w=6, h=12, vy=-300, vx=外扩
- HitSource::Missile（被击敌人 marked_until = t+2，给 Heat Lock 用）

#### Drone
- count by L: [1, 2, 2, 3, 3]
- orbit_radius = 52
- fire_rate by L: [0.62, 0.56, 0.52, 0.48, 0.44]
- damage_mul = 1.00 + L×0.05
- orbit angular speed = 1.8 rad/s
- 子弹速度 = 650 + L×25，朝最近敌人发射
- HitSource::Drone

#### Laser（持续光束）
- cycle = 2.0s（ON+OFF 一周期）
- on_duty = 0.45 + (L-1)×0.06
- dps = (1.6 + L×0.55) × player.damage_mul
- width = 14 + L×3
- 跟踪：beam_x 朝最近 player.y 之上的敌人 x 缓动，track_speed = 3.8 + L×0.35
- 命中条件：|e.x - beam_x| < width/2 + e.radius 且 e.y ≤ player.y
- HitSource::Laser；Heat Lock 加成：被 mark 的敌人 ×1.4 dmg

#### Chain（闪电链）
- interval = max(1.65 - (L-1)×0.14, 0.9)
- jumps = 1 + L
- range = 140 + L×10（除首跳）
- first_range = 520 + L×35
- damage_mul = 1.45 + L×0.28
- 算法：从 player 头顶起，BFS 选最近未命中敌人，伤害 + bolt 特效，重复 jumps 次
- Resonance perk 触发条件：被击敌人有 wave_marked=true → 额外 +2 跳（不消耗主跳数计数）
- Static Mark perk：每次 chain 命中给敌人 static_mark=true（下次任意武器命中保证暴击）
- HitSource::Chain

#### Rift（虚空裂隙）
- max_rifts by L: [1, 1, 2, 2, 3]
- lifetime = 3.8 + L×0.45
- radius = 66 + L×8
- pulse_interval = max(1.0 - (L-1)×0.08, 0.68)
- place_interval by L: [3.5, 3.0, 2.8, 2.5, 2.2]
- chase_speed = 190 + L×22
- base_damage = 1.2 + L×0.28
- 行为：从 player 附近生成，朝最近敌人加速移动；范围内敌人每 0.22s 受 base×0.34 灼烧；每 pulse_interval 释放强脉冲（base 全额）
- Gravity Well perk：每帧把范围内敌人朝裂隙拖 40 px/s
- HitSource::Rift

#### Wave（波动炮）
- count by L: [1, 1, 2, 2, 3]
- interval = max(0.90 - (L-1)×0.075, 0.60)
- amplitude = 40 + L×9（横向摆动幅度）
- frequency = 1.5 + L×0.2
- damage_mul = 1.05 + L×0.13
- 子弹运动：spawn_x 沿 vx 推进，x = spawn_x + amp×sin(phase)；vy=-bullet_speed 上行
- HitSource::Wave；命中后给敌人 wave_marked=true（给 Resonance 用）

#### Reflector（反弹弹）
- count by L: [1, 2, 2, 2, 3]
- interval = max(1.10 - (L-1)×0.09, 0.75)
- bounces by L: [2, 3, 3, 4, 4]
- damage_mul = 1.0 + L×0.14
- speed = 430，三连发夹角 ±0.24 rad（朝最近敌人）
- L≥4：+pierce 1
- 边界反弹：撞到屏幕四边 vx/vy 取反，bounces -= 1，归零后正常越界销毁
- HitSource::Reflector
- Prism perk：穿过激光束（|x - player.x| < 22 且 y < player.y）→ 该弹一次性 ×1.5 dmg + pierce +1

### 6.3 副武器递减
槽位 0-2 个：penalty=1.0；3 个：×0.92；4 个：×0.85（不影响主炮、敌方弹）

### 6.4 暴击
- chance = clamp(crit_chance, 0, 0.35)
- 每次伤害结算前 roll：random < chance → damage ×= crit_mul
- Static Mark 标记的敌人：下一次任意命中**保证暴击**（消耗标记）

---

## 7. 玩家加成

### 7.1 6 个 Perk（联动 Legendary）
| Perk         | 解锁条件             | 效果 |
|--------------|----------------------|------|
| Heat Lock    | 持有 missile + laser | Missile 命中给 marked_until=t+2；Laser 对 marked 敌 ×1.4 dmg |
| Static Mark  | 持有 chain           | Chain 命中给 static_mark；下次任意命中保证暴击 |
| Drone Relay  | 持有 drone + missile | Drone 击杀 → 自动从尸体补发一枚追踪导弹（伤害 1.2×damage_mul） |
| Gravity Well | 持有 rift + drone    | Rift 范围内每帧把敌人朝中心拖 40 px/s |
| Resonance    | 持有 wave + chain    | Wave 标记的敌人被 chain 命中 → 额外 +2 跳 |
| Prism        | 持有 reflector + laser | Reflector 弹穿过激光束 = 一次性 ×1.5 dmg + pierce +1 |

### 7.2 6 个武器进化（金卡）
| Evolution    | 触发                     | 效果 |
|--------------|--------------------------|------|
| Heatseeker   | Missile L5 + Heat Lock   | +1 弹/轮，dmg ×1.5，弹头 6×12 → 8×14 |
| Swarm        | Drone L5 + Drone Relay   | +1 僚机，fire_rate ×0.85 |
| Annihilator  | Laser L5 + Heat Lock     | DPS ×1.6，width ×1.5，on_duty +0.15 |
| Tempest      | Chain L5 + Static Mark   | +2 jumps，dmg ×1.4 |
| Cascade      | Wave L5 + Resonance      | +1 wave，amp ×1.3，dmg ×1.3 |
| Kaleidoscope | Reflector L5 + Prism     | +1 子弹，+2 bounces，dmg ×1.3 |

进化以 Player.perks.evo_X 标志位记录；武器 tick 检查标志放大数值。

---

## 8. 卡池（升级弹窗）

### 8.1 稀有度权重
- Common 44 / Rare 34 / Epic 15 / Legendary 7（总权重 100）
- 颜色：Common=#C8DCFF，Rare=#7DC8FF，Epic=#DC8CFF，Legendary=#FFC83C

### 8.2 卡列表（约 25 张构筑性卡）

**主炮 / 副武器**
| ID                | 稀有度 | 效果                        | eligible 条件 |
|-------------------|--------|-----------------------------|--------------|
| main_gun_up       | Rare   | 主炮 +1 级                  | main 未满 5  |
| u_missile         | Epic   | 解锁导弹                    | 未持有 + slot<4 |
| u_drone           | Epic   | 解锁僚机                    | 同上          |
| u_laser           | Epic   | 解锁激光                    | 同上          |
| u_chain           | Epic   | 解锁闪电链                  | 同上          |
| u_rift            | Epic   | 解锁裂隙                    | 同上          |
| u_wave            | Epic   | 解锁波动炮                  | 同上          |
| u_reflector       | Epic   | 解锁反射镜                  | 同上          |
| missile_up etc.   | Rare   | 对应副武器 +1 级            | 已持有且未满 5 |

**生存 / 杂项**
| ID         | 稀有度 | 效果                           |
|------------|--------|--------------------------------|
| max_hp     | Rare   | max_lives +1，最多 2 次       |
| regen      | Rare   | regen_per_min +0.35（cap 1.2） |
| invincible | Rare   | invincible +0.12（cap 1.9）   |
| shield     | Rare   | 给一次格挡（已有则不出）      |
| heal       | Common | 立即满血                       |

**6 Legendary Perks** + **6 Legendary Evolutions** —— 见上 §7

> ⚠️ 数值小卡（fire_rate/damage/bullet_speed/move_speed/pickup_r/xp_mul/score_mul/crit_chance/crit_dmg）**不在卡池里**，已经迁到 buff 掉落（§9）。

### 8.3 抽卡算法
1. filter pool by `eligible(player, weapons)`
2. 检查保底：`pity_unlock ≥ 4 && weapons.subs.len() < 4 && pool 中有 u_*` → 直接从可用 u_* 中随机选 1 张，pity_unlock=0
3. 剩余按权重加权随机抽（去重 swap_remove）直到 3 张
4. 抽出 shuffle 一次
5. 若本次没出 u_* 卡：pity_unlock += 1；否则归 0

---

## 9. 拾取系统

### 9.1 6 类拾取
| Kind          | 颜色          | 形状           | 效果 |
|---------------|---------------|----------------|------|
| Xp(value)     | 青/紫/金（按 value 分级） | 双三角钻石 | 加 xp（×xp_mul） |
| Heal          | 绿色          | 红十字         | lives += 1（不超 max） |
| Magnet        | 粉            | 磁铁形         | magnet_until = run_time + 8（吸附半径×2.4） |
| Ammo          | 橙            | 圆+三角        | super_charge += 0.25 |
| Barrier       | 蓝            | 圆环+白点      | shield = true |
| Buff(BuffKind)| 9 色          | 菱形+字母      | 见 §9.2 |

### 9.2 9 种 Buff（爆炸掉落，自动应用）
| BuffKind     | 字母 | 颜色          | 应用                         |
|--------------|------|---------------|------------------------------|
| FireRate     | F    | #FF825A       | fire_rate ×0.93（cap 0.18）  |
| Damage       | D    | #FF5A6E       | damage_mul ×1.06（cap 2.35） |
| BulletSpeed  | V    | #FFDC64       | bullet_speed ×1.08（cap 1200）|
| MoveSpeed    | S    | #7DF9FF       | speed ×1.05（cap 2200）       |
| PickupR      | M    | #FF82DC       | attract_radius ×1.18（cap 230）|
| XpMul        | X    | #96E6FF       | xp_mul ×1.08（cap 2.0）      |
| ScoreMul     | $    | #FFC85A       | score_mul ×1.08（cap 2.2）    |
| CritChance   | C    | #FF6E50       | crit_chance + 0.03（cap 0.35）|
| CritDamage   | K    | #FF503C       | crit_mul + 0.15（cap 2.8）   |

### 9.3 Buff 掉落概率
按 enemy.kind：
- Small/Kamikaze: 5-6%
- Medium/Strafer/Sniper/Weaver: 12-13%
- Large: 30%, MineLayer: 24%
- Boss: 必掉 4 个
- 精英 ×2，章节 mod ×buff_drop_mul，cap 1.0

加权选 BuffKind：
- FireRate 18, Damage 18, BulletSpeed 14, MoveSpeed 14
- PickupR 8, XpMul 8, ScoreMul 8, CritChance 6, CritDamage 6

### 9.4 Special 掉落（独立于 Buff）
按 enemy.kind 概率：
- Small/Kamikaze 0%, Medium/Strafer/Sniper/Weaver 5%, Large/MineLayer 18%
- 精英 100%, Boss 100%（掉 2 个）
- 在 Heal/Magnet/Ammo/Barrier 中按 (t+i)%4 循环选

### 9.5 拾取行为
- 距离 < pickup_radius → 立即吃
- 距离 < attract_radius → 朝 player 加速 900 px/s²（封顶 600 px/s）
- 距离 ≥ attract_radius → 缓慢漂浮下沉（vy 限 [-50, 45]）
- 边界：x ∈ [18, w-18], y ∈ [18, h-18]

---

## 10. 连击 / SUPER / 共鸣（Synergy）

### 10.1 Combo
- 每杀一个敌人 combo += 1，combo_timer = 1.2s
- combo_timer 到 0 → combo = 0
- 分数倍率：≥5 ×1.10，≥15 ×1.25，≥30 ×1.5
- 伤害倍率：≥50 ×1.05，≥100 ×1.10
- 每 10 连：super_charge += 0.05

### 10.2 SUPER charge（按击杀敌人 kind 累加）
| Kind | charge |
|------|--------|
| Small/Kamikaze | 0.018 / 0.030 |
| Medium | 0.035 |
| Strafer | 0.045 |
| Sniper | 0.050 |
| Weaver | 0.042 |
| MineLayer | 0.070 |
| Large | 0.080 |
| Boss | 0.300 |

满（≥1.0）按 Space → trigger_super：
- 清屏所有敌方子弹
- 对所有敌人造成伤害（Boss = max_hp×0.06，其余 = 6.0 固定）
- 玩家位置爆炸特效，super_charge=0

### 10.3 Synergy（共鸣 / 过载）
- 每杀填充 charge：Small/Kami=0.05，Med/Strafer/Sniper/Weaver=0.08，Large/Mine=0.15，Boss=0.50
- 满 1.0 → 进入"过载"6 秒，期间所有伤害 ×1.30
- 过载结束 charge=0，回到 idle 衰减（每秒 -0.05）

---

## 11. FX / 游戏手感

### 11.1 全局 FX 状态（每帧主循环消费）
- `shake`：Boss 攻击 / 大爆炸触发，draw 时给 (sx, sy) 随机偏移；衰减率 18 px/s²；用户偏好可缩放
- `damage_flash`：受击 0.85，过载 1.0；每秒 -2.4；屏幕红色半透矩形
- `time_freeze`：>0 时 play_dt = 0
- `slow_mo` + `slow_mo_scale`：>0 时 play_dt = real_dt × scale

### 11.2 命中冻帧 / 慢动作触发
| 事件 | 效果 |
|------|------|
| 暴击命中 | hit_pause(0.04) + 命中点冲击波环 |
| 大型敌人死亡 | hit_pause(0.06) + explode_big |
| Boss 死亡 | hit_pause(0.10) + slow_mo(0.45, 0.30) |
| 玩家受击 | damage_flash(0.85) + hit_pause(0.06) + shake 8 |
| 玩家撞 boss | damage_flash(0.95) + hit_pause(0.08) + shake 10 |

### 11.3 Vignette（屏幕边缘暗角）
- 低 HP（lives ≤ 1）：红色脉冲 0.45-0.95 alpha
- 过载：金色 0.6-0.75
- SUPER 蓄满：青色 0.5-0.7

实现：4 个矩形从屏幕边缘渐变到中心，5 层叠加。

### 11.4 粒子 / 拖尾 / 冲击波
- `Particle`：x/y/vx/vy/life/decay/size/color；轻微减速 0.6×dt；alpha=life
- `TrailDot`：玩家子弹按 HitSource 长度不同：
  - Missile: size 4.5, decay 3.4
  - Reflector: size 3.4, decay 4.8
  - Drone: size 2.0, decay 7.5
  - Crit: size 4.2, decay 4.0
  - 默认: size 2.6, decay 6.5
- `Shock`：圆环扩张 + 淡出，双笔（外软辉光 + 内亮）
- `Bolt`：闪电折线（segs=4-14 by length，amp=clamp(len×0.08, 6, 22)，正弦包络中段最抖），4 层叠加（外软 + 中等 + 主线 + 白热芯）

---

## 12. 元进度（持久化）

### 12.1 Save 数据结构
```ts
interface Save {
  // 战绩
  high: u32
  leaderboard: { score, level, date(YYYY-MM-DD) }[]  // top 5

  // 偏好
  muted: bool
  fullscreen: bool
  lang: 'En' | 'Zh'
  master_vol: f32 (default 0.5)
  bgm_vol: f32 (default 0.55)
  sfx_vol: f32 (default 1.0)
  screen_shake: f32 (default 1.0, range 0..1.5)

  // 跨局
  stardust: u64
  lifetime_score: u64
  bosses_killed: u32
  runs: u32
  unlocked_ships: u32 (bitmask, bit0=Vanguard 永久)
  furthest_chapter: u32

  // 6 项天赋等级
  talent_dmg, talent_hp, talent_speed, talent_xp, talent_stardust, talent_super: u32

  // 成就 / 图鉴
  achievements: u64 (bitmask, 20 条)
  codex_bosses: u32 (bitmask, 6 BossMod)
  codex_enemies: u32 (bitmask, 9 EnemyKind)
  codex_weapons: u32 (bitmask, 7 sub-weapons)

  // 难度 / 每日
  difficulty: u8 (0/1/2)
  hard_unlocked: bool
  nightmare_unlocked: bool
  daily_date: string
  daily_high: u32

  // 涂装
  ship_skins_unlocked: u32 (bit = ship_idx*3 + variant)
  ship_skin_choice: [u8; 3]
}
```

### 12.2 6 个天赋
| ID         | 名称           | 效果（每级）   | 成本（升 1→max）         |
|------------|----------------|----------------|--------------------------|
| Damage     | 穿甲弹         | dmg_mul ×1.06  | 120 / 280 / 600 / 1200 / 2400 |
| Health     | 强化机壳       | max_lives +1   | 260 / 700 / 1700 |
| Speed      | 敏捷推进器     | speed ×1.05    | 180 / 420 / 950 |
| Xp         | 数据收割       | xp_mul ×1.10   | 200 / 480 / 1100 |
| Stardust   | 星尘精炼       | 局后 stardust ×1.15 | 300 / 750 / 1700 |
| SuperStart | 预充能核心     | 开局 super_charge +0.20 | 220 / 520 / 1200 |

局开始时 `apply_to_world(world, save)` 把这些叠到 player.stats 上。

### 12.3 局后结算
```
raw = score/100 + bosses_in_run × 50
stardust = raw × stardust_multiplier × difficulty_mul
其中 stardust_multiplier = 1.0 + talent_stardust × 0.15
     difficulty_mul = {0:1.0, 1:1.5, 2:2.0}[difficulty]
```
通关故事章节（chapter_idx ≥ 5）→ 解锁更高难度 + 涂装：
- Normal 通关：解锁 Hard + 每艘飞船的中级涂装（变体 1）
- Hard 通关：解锁 Nightmare + 每艘飞船的终级涂装（变体 2）

### 12.4 20 条成就
（id, 名称, 条件, 奖励 stardust）
- 0 First Boss Down — bosses_killed_run ≥ 1 — 30
- 1 Climber — 单局 level ≥ 10 — 30
- 2 Combo Streak — combo ≥ 50 — 50
- 3 Combo Maniac — combo ≥ 100 — 100
- 4 Story Cleared — chapter_idx ≥ 5 — 200
- 5 Hard Done — Hard 通关 — 400
- 6 Nightmare Done — Nightmare 通关 — 800
- 7 Endless Lap — chapter_idx > 5 — 200
- 8 Missile Master — damage_by_source[Missile] ≥ 5000 — 80
- 9 Beam Wielder — damage_by_source[Laser] ≥ 8000 — 80
- 10 Stormbringer — damage_by_source[Chain] ≥ 6000 — 80
- 11 Riftwalker — damage_by_source[Rift] ≥ 6000 — 80
- 12 Quartet — 单局 4 副武器 — 100
- 13 Untouched — chapter_no_hit && chapter_idx ≥ 1 — 150
- 14 Pacifist Boss — 击杀 boss 时满血 — 100
- 15 Six-figure — 单局 score ≥ 100,000 — 200
- 16 Hundred Runs — runs ≥ 100 — 500
- 17 Boss Slayer — bosses_killed ≥ 50 — 300
- 18 Star Saver — stardust ≥ 10,000 — 0（仅徽章）
- 19 Synergist — 单局激活 ≥ 3 perk — 200

每帧调 `check_all(world, save)` 返回新解锁列表，加 stardust 并落盘。

### 12.5 图鉴
3 个 tab：
- ENEMIES（9 种，按 kill 解锁）
- BOSS MODS（6 种，按 kill boss 时记录其 boss_mod 解锁）
- WEAPONS（7 种，装备过即解锁）

未解锁画 `???`；解锁画名+描述。

### 12.6 涂装（每艘 3 套）
| Ship | 0 (Default)    | 1 (Mid)      | 2 (Top)        |
|------|----------------|--------------|----------------|
| Vanguard  | 蓝白    | Crimson 红   | Voidshade 紫灰 |
| Striker   | 青绿    | Sunburst 橙  | Frostbyte 冰蓝 |
| Engineer  | 紫白    | Verdant 绿   | Obsidian 黑紫  |

每套涂装 4 色：(body, trim, canopy, engine) RGB 见源码 art.rs::ship_palette。
ship_skins_unlocked bit = ship_idx × 3 + variant。

---

## 13. 章节定义

5 个故事章节 + 1 个无尽模板：

| Ch | Name              | Duration | Spawn Intensity | Kami | Strafer | Boss Pool                          |
|----|-------------------|----------|-----------------|------|---------|------------------------------------|
| 1  | OUTER BELT        | 60s      | 1.00            | 0%   | —       | Frenzied                           |
| 2  | CRIMSON DRIFT     | 75s      | 1.10            | 30%  | —       | Frenzied, Bulwark                  |
| 3  | ION STORM         | 90s      | 1.18            | 10%  | 6s      | Summoner, StormCore                |
| 4  | GHOST TIDE        | 100s     | 1.30            | 20%  | 8s      | Phantom, Summoner                  |
| 5  | DREADNOUGHT CORE  | 110s     | 1.45            | 25%  | 5s      | Hydra, StormCore, Bulwark          |
| ∞  | ENDLESS           | 60s      | 1.80            | 30%  | 4s      | 全 6 种                             |

每章主题色（star_tint, bg_top, bg_mid）见 chapter.rs。无尽模式每圈 HP/分数/XP 额外 ×(1 + lap×0.35)，每圈双 boss。

---

## 14. 刷怪密度

每章基于 chapter_time 生成普通敌人：
```
sm_intv = clamp(lerp(chap_t/90, 1.4, 0.50) / intensity, _, 0.75)
md_intv = chap_t < 8 ? ∞ : lerp((chap_t-8)/70, 3.0, 1.4) / intensity
lg_intv = chap_t < 30 ? ∞ : lerp((chap_t-30)/100, 7.0, 3.0) / intensity
```
- Small spawn：随机 X，10% 概率（>=18s）变 Weaver，否则按 chapter.kamikaze_chance 变 Kamikaze
- Medium spawn：22% 概率（>=45s）变 Sniper
- Large spawn：24% 概率（>=60s）变 MineLayer
- Strafer：章节专用 strafer_interval，从屏幕左/右进，y ∈ [80, 220]

---

## 15. 音频

### 15.1 SFX 列表
全部启动时合成（SR=44100，16-bit PCM mono WAV）：
- shoot — 800→160 Hz 锯齿 + 250→70 Hz sine sub-bass + 6ms 噪声 click + 低通尾噪
- hit ×3 变体（起始 2800/3200/3600 Hz 三角波下行）
- kill_combo ×8（C5/E5/G5/A5/C6/E6/G6/A6 + 五度泛音）
- explode_small / explode_big
- powerup, super, hurt, gameover, levelup, click, boss_intro

### 15.2 BGM
- Menu / Play / Boss 三轨，循环；切换通过 `set_track`
- BGM 音量改动需 `set_sound_volume(snd, vol)` 实时生效

### 15.3 播放时
- play_one_jitter：±10% 音量随机抖动（shoot/hit 用）
- play_one：固定音量
- 击杀 SFX：按 combo 选 kill_steps[(combo-1)/3] 阶

**移植小游戏注意**：内存 WAV 不能直接喂 InnerAudioContext，需要：
1. 启动时把每段 WAV bytes 写到 `wx.env.USER_DATA_PATH/sfx_*.wav`
2. 加载时用 `wx.createInnerAudioContext({ src: ... })`
3. 池化 audio context（每个 SFX 至少 4 个池实例避免高频开新句柄）

---

## 16. 输入键位（桌面，移植参考）

### 16.1 菜单
| Key | Action |
|-----|--------|
| Enter / Space | 启动一局 |
| Y | 启动每日挑战 |
| ←/→ A/D | 选飞船 |
| T | 进入天赋 |
| O | 进入设置 |
| H | 进入成就 |
| C | 进入图鉴 |
| N | 切换难度（已解锁内循环） |
| K | 切换当前飞船的涂装 |
| L | 切换语言 |
| M | 静音 |
| F | 全屏 |
| Esc | 退出 |

### 16.2 战斗中
| Key | Action |
|-----|--------|
| WASD / 方向键 | 移动 |
| Space | SUPER（蓄满时） |
| Esc / P | 暂停 |
| Q（暂停时）| 返回菜单 |

### 16.3 选卡
| Key | Action |
|-----|--------|
| 1 / 2 / 3 | 选第 N 张 |
| ←/→ + Enter | 光标 + 确认 |
| 鼠标点击 | 直接选 |

---

## 17. 多语言（中英）

所有用户可见字符串以**英文为 key**，中文为查表 fallback。`t(key, lang)` 找不到时返回原 key。
- 系统检测可用 CJK 字体；找不到强制 lang=En。
- 完整翻译表见 src/lang.rs（约 200 条）。

---

## 18. 移植到微信小游戏的额外注意

### 18.1 必改
1. **触屏输入**：见 §4.3 的虚拟摇杆设计
2. **存档**：`directories::ProjectDirs` → `wx.setStorageSync('save', JSON)` / `wx.getStorageSync`
3. **日期**：`SystemTime::now()` → `new Date().toISOString().slice(0,10)`
4. **CJK 字体**：自带 `wenkai-subset.ttf`（约 2-3MB，常用 7000 字）走 `wx.loadFont`
5. **音频**：合成 WAV → 写文件系统 → `InnerAudioContext`（见 §15.3）
6. **全屏 / 静音 / 语言切换**：保留逻辑，UI 入口移到设置面板（不要按键）

### 18.2 性能注意
- 当前每帧大量 `draw_circle / draw_rectangle` 立即模式调用 → 在小游戏的 Canvas 上可能瓶颈
- 优化方向：
  - 把粒子 / trail / 子弹改成 batched draw（共用一张 atlas，一次 drawArrays）
  - 静态背景 / hud 用 Canvas2D 一次画好缓存
  - 避免每帧创建对象（粒子用对象池）

### 18.3 包体限制
- 小游戏主包 ≤ 4MB，总 ≤ 16MB（含子包）
- 当前 Rust 版没有外部资源依赖（音频/字体都内置）
- 移植后字体子集是最大头（CJK 子集 2-3MB）；音频如果改成预渲染则 SFX 总计 <1MB
- 美术资产建议放子包（章节背景图等）

### 18.4 隐私 / 上线
- 小游戏需要 ICP 备案 + 类目（休闲游戏）
- 排行榜如果上云需要后端（云开发免费额度足够）
- 数据上报先留 hook，不强行接入

---

## 19. 文件 / 模块对照（Rust 源 → 设计概念）

| Rust 文件         | 对应设计概念             |
|-------------------|--------------------------|
| main.rs           | 主循环 + 场景调度        |
| world.rs          | World 状态 + 难度/章节修饰 |
| entity/player.rs  | 玩家状态 + 输入 + 渲染   |
| entity/enemy.rs   | 9 种敌人 + Boss 修饰     |
| entity/bullet.rs  | 子弹 + HitSource         |
| entity/pickup.rs  | 6 类拾取 + BuffKind      |
| weapon/main_gun.rs| 主炮 5 级演化             |
| weapon/{missile,drone,laser,chain,rift,wave,reflector}.rs | 7 副武器 |
| weapon/mod.rs     | SubWeapon trait + WeaponSlot + Synergy |
| spawn.rs          | 刷怪规则 + buff/special 掉落 |
| combat.rs         | 命中 / 击杀 / 拾取 / SUPER 结算 |
| upgrade.rs        | 卡池 + 抽卡 + 保底       |
| chapter.rs        | 5 章 + 无尽配置          |
| achievements.rs   | 20 条成就                |
| talents.rs        | 6 个跨局天赋             |
| save.rs           | 持久化 schema            |
| ship.rs           | 3 飞船定义               |
| art.rs            | 飞船 / 敌人绘制 + 涂装表 |
| hud.rs            | 所有 UI 面板             |
| fx.rs             | 粒子 / trail / 冲击波 / hit-pause / vignette |
| audio/{mod,sfx,bgm,synth}.rs | 程序合成音频 |
| bg.rs             | 星空背景                 |
| collision.rs      | hit_circle / AABB        |
| lang.rs           | i18n 中英查表            |
| config.rs         | 全局 CFG（480×800）      |
| scene.rs          | Scene 枚举               |

---

## 20. MVP 移植拆分建议

如果资源紧张，可以这样切片：
- **第 1 周**：场景骨架 + 玩家移动 + 1 飞船 + 主炮 + 1 副武器（Missile）+ 1 章 + 1 Boss
- **第 2 周**：扩展到 7 副武器 + 5 章 + 6 Boss 修饰 + 卡池
- **第 3 周**：Buff 掉落 + Perk + 进化 + Synergy/SUPER
- **第 4 周**：成就 + 图鉴 + 难度 + 章节分叉 + 涂装
- **第 5 周**：每日 + 天赋 + 设置 + i18n
- **第 6 周**：手感打磨（hit-pause / slow-mo / vignette / FX）+ 性能优化 + 过审

---

> 任何具体公式 / 数值有歧义时，回头看对应 Rust 模块。本文档只是"截图式"快照，**真实数值以代码为准**。
