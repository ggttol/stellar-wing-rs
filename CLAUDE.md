# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

Stellar Wing 是一款基于 [macroquad](https://github.com/not-fl3/macroquad) 的 Rust 肉鸽俯视角飞行射击游戏。项目是单一二进制 crate（`stellar-wing`）。逻辑分辨率固定 480×800（竖屏，移动端友好），主循环单线程。

完整的玩法 / 数值规格（含 Cocos / 微信小游戏移植参考）见 [`docs/PORTING-SPEC.md`](./docs/PORTING-SPEC.md)——那是设计与数值的单一可靠来源；改了 Rust 代码里的常数后回头同步那一份。

## 常用命令

- `cargo run --release` — 实际游玩 / 测试手感时优先用，调试构建会明显掉帧。
- `cargo run` — 调试模式，编译快，但帧率不稳。
- `cargo build --release` — 在 `target/release/stellar-wing` 生成发布版二进制。
- `cargo check` — 快速类型检查，迭代时常用。
- `cargo clippy --all-targets -- -D warnings` — 0 警告才算过；CI 与本地 PR 都按这个标准。
- `cargo test` — 17 个单元测试，覆盖 Bullet / 反射弹反弹 / Save 兼容 / 升级卡 cap / 武器最小行为等关键路径。跑全套：`cargo test --quiet`；跑单个：`cargo test --quiet wave_bullet_tracks_spawn_center_with_sine_offset`。
- `bash scripts/package_macos_app.sh` — 构建 release 并组装 `dist/Stellar Wing.app`（用 `sips` + `iconutil`，仅 macOS）。

## 架构

单线程 macroquad 应用。`main.rs` 保持精简——窗口配置 / 主循环 / `Scene` 状态机 / `step_play` 调度，把具体逻辑分发给各模块。

### 主循环（`src/main.rs`）

- `Scene`（`src/scene.rs`）是顶层状态机，11 个变体：
  - `Menu`、`Talents(cursor)`、`Settings(cursor)`、`Achievements(cursor)`、`Codex(tab,cursor)` —— 菜单与子菜单
  - `Playing` ↔ `Paused`、`UpgradePick(cards)`、`ChapterChoice([ChapterMod;2], cursor)`、`GameOver` —— 战斗相关
- `World`（`src/world.rs`）是运行时游戏状态。所有字段 `pub`，因为玩法系统散落在多个模块——这是有意为之，不要追求"完美封装"。除了核心战斗状态外，World 还携带：`damage_by_source: [f32; 9]`（按 `HitSource` 分桶累计，结算页和成就用）、`difficulty`、`daily_mode`、`run_seed`、`chapter_modifier: ChapterMod`、`chapter_no_hit`、`codex_*_run` 三个临时位掩码。
- `step_play`（`main.rs` 内部函数）是每帧的核心玩法 tick。**执行顺序绝对不能乱**——参见 `docs/PORTING-SPEC.md` §2。简化版：连击/共鸣衰减 → 玩家 → 武器 tick（写 `damage_by_source`）→ 章节计时 → 刷怪 / Boss 入场 → 敌人 update → 追踪弹转向 → 子弹 update → 玩家子弹 trail → 拾取 → SUPER → 玩家子弹打敌人 → 击杀处理 → 章节切换 → 敌方子弹/碰撞 → 低血警告 → 清理。**主循环额外在 step_play 之后做成就检查 + 升级判断 + 章节分叉触发。**
- 玩法 dt 受 `fx.tick_time_modifiers(real_dt)` 缩放（`time_freeze` → 0；`slow_mo` → ×scale）。背景 / 真实时间动画用 `t_acc`，玩法时钟用 `world.run_time`，**不要混用**。

### 关键模块依赖

- `world.rs` — `World`、`SpawnTimers`、`ChapterMod`、`xp_required_for(L)`、`difficulty_mods(d)`、`weapon_codex_bit(id)`。只有数据 + 静态 helper，没有玩法逻辑。
- `spawn.rs` — `spawn_chapter_wave`、`spawn_chapter_boss`、`spawn_one_full`、`spawn_enemy_full`、`spawn_strafer`、`drop_xp_gems`、`maybe_drop_special`、`maybe_drop_buff`、`endless_extra_mul`。从 `world.run_time` 做难度时间缩放，叠加 `world.difficulty` 与 `world.chapter_modifier`。
- `combat.rs` — 命中 / 拾取 / 击杀 / SUPER / 玩家受击。`process_kills` 同时更新连击 / 分数 / SUPER 充能 / `codex_*_run` 位掩码 / 调用 `maybe_drop_buff`。`resolve_player_bullets` 直接写 `world.damage_by_source[b.source as usize]`。
- `collision.rs` — 几何原语（`hit_circle`、AABB ↔ 圆）。**不要把玩法逻辑放进来。**
- `hud.rs` — 全部 UI 面板：菜单 / HUD / 暂停 / 强化选卡 / 章节分叉 / 成就 / 图鉴 / 设置 / 天赋 / GameOver。`draw_world` 顺序：拾取 → 子弹 → 敌人 → 武器 → 玩家。`draw_screen_fx` 在世界与 HUD 之间叠 vignette / 受击红闪。鼠标坐标重映射模式（`mx * CFG.w / screen_width()`）写在 `main.rs::card_at` 附近——任何新的鼠标命中检测都要沿用这个模式。

### 实体（`src/entity/`）

`mod.rs` 重新导出 `Player`、`Bullet`、`Enemy`（含 `EnemyKind` / `EliteMod` / `BossMod` / `TelegraphKind`）、`Pickup`（含 `PickupKind` / `BuffKind`）和 `HitSource`（9 个变体，索引方式与 `damage_by_source` 一致）。

子弹携带 `HitSource` 标记，让"击杀归属"（例如 Drone Relay 接力）和"伤害修饰"（Heat Lock 标记、Static Mark 暴击、Resonance 共鸣跳数）能在不给每种武器单独写碰撞代码的前提下工作。

`Player::stats`（`PlayerStats`）保存各种倍率（`damage_mul`、`crit_chance`、`crit_mul`、`xp_mul`、`score_mul`、`pickup_radius`、`max_lives`、`fire_rate`、`bullet_speed`、`speed`、`friction`、`invincible`、`regen_per_min`），所有 cap 值见 `docs/PORTING-SPEC.md` §4.1。这些会被升级卡 / Buff 拾取 / 天赋修改。

`Player::perks`（`CombatPerks`）保存 6 个 Legendary perk 标志位 + 6 个武器进化（`evo_missile / evo_drone / evo_laser / evo_chain / evo_wave / evo_reflector`）+ `hull_plating_picks` + `pity_unlock`。

`Pickup::Buff(BuffKind)` 是 9 种小数值卡（FireRate/Damage/BulletSpeed/MoveSpeed/PickupR/XpMul/ScoreMul/CritChance/CritDamage），从敌人爆炸掉落，自动吸附 + 应用。**这些数值之前是升级卡，现在迁出来了**——卡池只保留构筑性卡（解锁 / 升级 / Perk / 进化 / HP / Shield / Heal / Regen / Invincible）。

### 武器（`src/weapon/`）

7 个副武器：`Missile`、`Drone`、`Laser`、`Chain`、`VoidRift`（`rift.rs`）、`WaveCannon`（`wave.rs`）、`Reflector`。每种 5 级。

- `MainGun` 是具体类型，不是 trait object，热点路径单态化。
- 副武器实现 `SubWeapon` trait，`Box<dyn SubWeapon>` 存在 `WeaponSlot.subs` 中（最多 4 个）。
- **`SubWeapon::tick` 现在多一个参数 `damage_acc: &mut [f32; 9]`**——不通过 Bullet 走 `resolve_player_bullets` 的武器（`laser` / `chain` / `rift`）必须把直接造成的伤害写进 `damage_acc[HitSource as usize]`，否则伤害分布与"武器累计伤害成就"会漏算。
- `SynergyGauge`（`weapon/mod.rs`）是过载机制——击杀填充 charge，满 1.0 进入 6 秒过载，期间所有伤害 ×1.30。**注意：旧的 `DecayGauge`（武器等级随时间衰减）已经被废弃换成这个**。
- `roll_crit(player, base_mul)` 是所有武器都应使用的统一暴击逻辑，让 `crit_chance` / `crit_mul` 一致生效。Static Mark 标记的敌人会消耗标记触发保证暴击，逻辑在 `combat::resolve_player_bullets` 里。
- 副武器递减：`WeaponSlot::sub_penalty()` —— 0-2 个无惩罚，3 个 ×0.92，4 个 ×0.85（不影响主炮 / 敌方弹）。
- 武器进化：每个武器 tick 内读 `player.perks.evo_X` 决定是否走"进化形态"分支（额外弹数 / 伤害倍率 / 视觉宽度）。新增进化时除了改对应武器，还要在 `upgrade.rs` 里加金卡（满级 + perk 为 eligible 条件）。
- 新增副武器流程：`src/weapon/` 新建文件，`impl SubWeapon`，`mod.rs` 导出，`upgrade.rs` 加解锁卡 / 升级卡（用 `mk_unlock_eligible` / `mk_up_eligible`），`docs/PORTING-SPEC.md` §6 同步数值。

### 其他模块

- `chapter.rs` — 5 个故事章节 + `ENDLESS` 模板。`chapter::get(chapter_idx)` 屏蔽边界（idx ≥ 5 进 endless）。每章配置：duration、星空 tint、bg 渐变、boss 池、kamikaze / strafer 概率与间隔。
- `upgrade.rs` — 卡池（构筑性卡，~25 张）+ `draw_n` 加权抽取器。`Card.apply: fn(&mut Player, &mut WeaponSlot)`、`Card.eligible: fn(&Player, &WeaponSlot) -> bool`。**保底**：连续 4 抽未出 `u_*` 解锁卡且仍有空槽 → 强制出。每张卡的稀有度权重见 `Rarity::weight()`（44/34/15/7）。
- `talents.rs` — 6 项跨局永久天赋（dmg / hp / speed / xp / stardust / super-start）。开局时 `apply_to_world(world, save)` 把等级叠到 player.stats 上。`stardust_multiplier(save)` 给 `record_run` 的星尘奖励乘个倍率。
- `achievements.rs` — 20 条成就 + `check_all(world, save)`。每帧调，已解锁的跳过；新解锁返回 (idx, stardust)。`mark_unlocked` 写位。idx 永远不能改（save 兼容性靠位序）。
- `ship.rs` — `ShipType::ALL` 三艘飞船定义，`apply()` 修改初始 `Player` / `WeaponSlot`。
- `art.rs` — 飞船 / 敌人 / Boss 的代码绘制；`ship_palette(ship, variant)` 是涂装表（每艘 3 套配色：默认 / 中级 / 终级，对应 Normal/Hard/Nightmare 通关解锁）。
- `bg.rs` — 星空背景，按章节主题色调染。
- `fx.rs` — 粒子 / 拖尾（`TrailDot`）/ 冲击波环（`Shock`）/ 浮字 / 闪电（`Bolt`，锯齿折线）/ 屏震 / 受击红闪（`damage_flash`）/ 命中冻帧（`time_freeze`）/ 慢动作（`slow_mo`）。**所有命中 / 死亡的视觉反馈都走这个模块**，不要在战斗代码里直接 draw。`fx.tick_time_modifiers(real_dt) -> play_dt` 是把"时间修饰"喂给主循环的入口。
- `audio/` — 完全程序生成。`synth.rs`：振荡器（sine/square/saw/triangle/noise）、ADSR、一极点低通、扫频音符、内存中 16-bit PCM mono WAV 编码器。`sfx.rs` 和 `bgm.rs` 在 `Audio::load` 时把所有 SFX / 3 条 BGM 渲染成 `Vec<u8>` 喂给 `macroquad::audio::load_sound_from_bytes`。**不要往 `assets/` 里新增 WAV 文件**——声音设计放在代码里，便于参数化。BGM 三轨（Menu / Play / Boss）；`Audio::set_track` 幂等，`main.rs` 在 Scene 切换时调用，并在 Playing 中每帧调用以便在 Boss 出现/死亡时切 Play↔Boss。`play_kill_combo(combo)` 按连击数选击杀音阶（8 阶）。`hit` 有 3 个变体随机播放避免疲劳；`shoot` 和 `hit` 用 `play_one_jitter` 加 ±10% 音量抖动。运行时音量改动（设置面板）通过 `set_master_vol/set_bgm_vol/set_sfx_vol`，BGM 用 `set_sound_volume` 实时同步。
- `save.rs` — 通过 `directories::ProjectDirs("dev", "ggttol", "stellar-wing")` 做 JSON 持久化（macOS：`~/Library/Application Support/dev.ggttol.stellar-wing/save.json`）。schema 大且持续扩展：最高分 / 排行榜 / 偏好（mute/fullscreen/lang/master_vol/bgm_vol/sfx_vol/screen_shake）/ 跨局（stardust/lifetime_score/bosses_killed/runs/furthest_chapter）/ 6 项天赋 / 成就 64-bit 位掩码 / 图鉴位掩码（敌人/Boss/武器）/ 难度（hard_unlocked/nightmare_unlocked）/ 每日（daily_date/daily_high）/ 涂装。**所有新字段必须 `#[serde(default)]` 兼容旧档**——有测试 `old_save_json_missing_new_fields_deserializes_with_defaults` 守护。日期不依赖 `chrono`（`epoch_days_to_ymd`）；`save::today()` 返回 `YYYY-MM-DD`，每日挑战种子来自这里。
- `lang.rs` — 英文 / 中文文本查表 `t(key, lang)`。**英文是 key**，中文为 fallback；找不到时原样返回 key。`main.rs::try_load_cjk_font` 加载系统 CJK 字体，找不到时强制 lang=En。新增 UI 字符串只在中文分支里加翻译条目。
- `config.rs` — 全局 `CFG`，逻辑分辨率（`w=480`、`h=800`）；窗口尺寸固定，**不能调整大小**。

### 需要记住的约定

- 逻辑坐标使用 `CFG.w` × `CFG.h`；鼠标 / 触屏输入需要重映射（`mx * CFG.w / screen_width()`）——任何新的命中检测都要沿用这个模式。
- 时间分两条：`t_acc`（真实时间累加器，给动画 / fx 寿命用）和 `world.run_time`（玩法时钟，受 hit-pause / slow-mo / 暂停影响）——**不要混用**。
- 主循环每帧 `dt` 被夹到 `0.05`，可以假设步长有上限。但 `fx.tick_time_modifiers` 之后的 `play_dt` 可能为 0（命中冻帧）或缩放。
- 这个代码库里的注释 / 字符串经常直接写中文；新增内容跟随周围风格，不要把已有内容硬翻成英文。UI 用户可见字符串走 `lang::t`。
- 动了核心数值（XP 曲线、武器伤害、敌人 HP、卡牌权重、章节配置等）后，建议同步更新 `docs/PORTING-SPEC.md`。
- Save schema 新字段必加 `#[serde(default = "...")]`，新增成就 / 涂装 bit 永远只追加在末尾，不要重排——会破坏存量玩家档。
