# CLAUDE.md

这个文件为 Claude Code（claude.ai/code）在本仓库中工作时提供指引。

## 项目概览

Stellar Wing 是一款基于 [macroquad](https://github.com/not-fl3/macroquad) 的 Rust 肉鸽俯视角飞行射击游戏。项目是单一二进制 crate（`stellar-wing`）。

## 常用命令

- `cargo run` - 构建并启动游戏（调试模式）。
- `cargo run --release` - 帧率更稳定；实际游玩或测试手感时优先使用。
- `cargo build --release` - 在 `target/release/stellar-wing` 生成发布版二进制。
- `cargo check` / `cargo clippy` - 快速迭代类型错误和 lint 问题。
- `bash scripts/package_macos_app.sh` - 构建发布版并组装 `dist/Stellar Wing.app`（使用 `sips` + `iconutil`，仅 macOS 可用）。

项目没有测试套件，`cargo test` 基本不会做任何事。

## 架构

这是一个单线程的 macroquad 应用。`main.rs` 保持尽量精简，它负责窗口/主循环、`Scene` 状态机，以及按帧调度 `step_play`，把具体逻辑分发给各个模块。

### 游戏主循环（`src/main.rs`）

- `Scene`（`src/scene.rs`）是顶层状态机：`Menu → Playing ↔ Paused → UpgradePick(cards) → Playing → … → GameOver → Menu`。`main()` 里的每帧 `match` 负责输入、场景切换，并调用 `hud::*` 进行绘制。
- `World`（`src/world.rs`）是运行时游戏状态，包含玩家、武器、子弹、敌人、掉落物、分数、等级、经验、SUPER 能量、连击、Boss 计时器等。所有字段都设为 `pub`，因为不同的玩法系统分散在多个模块里，这样设计是有意为之。
- `step_play`（在 `main.rs` 中）是每帧的核心玩法 tick。它明确规定了各系统的执行顺序：连击衰减 → 玩家/武器/敌人/子弹更新 → 刷怪 → 追踪 → 拾取 → SUPER 炸弹 → 玩家子弹打敌人 → 击杀处理（连击、掉落、无人机接力）→ 敌方子弹打玩家 → 敌人碰撞 → 清理死亡对象。各个子系统分别位于 `combat.rs` 或 `spawn.rs`。

### 模块依赖顺序

- `world.rs` - `World`、`SpawnTimers`。只有数据，没有逻辑。
- `spawn.rs` - `spawn_normals`、`spawn_enemy`、`spawn_boss`、`drop_xp_gems`、`maybe_drop_special`。这里负责从 `world.run_time` 进行难度缩放（间隔 `lerp`、生命/分数倍率、精英判定）。
- `combat.rs` - 玩法碰撞与击杀结算：`steer_homing_bullets`、`resolve_player_bullets`、`process_kills`、`resolve_enemy_bullets`、`resolve_enemy_player_contact`、`trigger_super`、`collect_pickups`、`spawn_relay_missile`。暴击 / 静电标记 / 导弹标记逻辑，以及连击 → SUPER 能量 / 分数倍率的处理都在 `process_kills` 里。
- `collision.rs` - 只放几何原语（`hit_circle`、AABB 与圆形的辅助函数）。不要把玩法逻辑放进来。
- `hud.rs` - 菜单 / HUD / 暂停 / 强化选择 / GameOver 的绘制，以及 `draw_world`（拾取物 → 子弹 → 敌人 → 武器 → 玩家）。鼠标坐标重映射模式（`mx * CFG.w / screen_width()`）写在 `main.rs` 的 `card_at` 附近。

### 实体（`src/entity/`）

`mod.rs` 重新导出 `Player`、`Bullet`、`Enemy`（以及 `EnemyKind`、`EliteMod`、`BossMod`、`TelegraphKind`）、`Pickup`（以及 `PickupKind`）和 `HitSource`。子弹会携带 `HitSource` 标记，这样击杀归属逻辑（例如“无人机接力”）和伤害修正（导弹标记、静电标记暴击）就可以在不为每种武器单独写碰撞代码的前提下工作。`Player::stats` 保存各种倍率（`damage_mul`、`crit_chance`、`crit_mul`、`xp_mul`、`score_mul`、`pickup_radius`、`max_lives` 等），这些会被强化卡修改。

### 武器（`src/weapon/`）

- `MainGun` 是具体类型，不是 trait object，这样热点路径可以保持单态化。
- 副武器（`Missile`、`Drone`、`Laser`、`Chain`）实现 `SubWeapon`，并以 `Box<dyn SubWeapon>` 的形式存放在 `WeaponSlot.subs` 中（最多 4 个）。新增副武器的流程是：在 `src/weapon/` 新建文件，实现 `SubWeapon`，在 `mod.rs` 里导出，并在 `src/upgrade.rs` 里补对应的强化卡。
- `DecayGauge`（在 `weapon/mod.rs` 中）是共享的“武器等级会随时间衰减，除非被刷新”的机制；武器会持有它，并调用 `decay_tick` / `refill`。
- `roll_crit(player, base_mul)` 是所有武器都应使用的统一暴击逻辑，这样 `crit_chance` / `crit_mul` 这两个属性才能一致生效。

### 其他模块

- `upgrade.rs` - 卡牌池和 `draw_n` 加权抽取器；每张 `Card` 都带有 `apply: fn(&mut Player, &mut WeaponSlot)`。
- `ship.rs` - `ShipType::ALL` 定义可选战机，`apply()` 会修改初始 `Player` / `WeaponSlot`。
- `art.rs`、`bg.rs`、`fx.rs` - 纯渲染代码（战机精灵、星空、粒子 / 浮字特效）。
- `audio/` - 完全程序生成。`synth.rs` 里有振荡器（sine / square / saw / triangle / noise）、ADSR、一极点低通、频率扫频音符构建器，以及内存中的 16 位 PCM 单声道 WAV 编码器。`sfx.rs` 和 `bgm.rs` 会在启动时使用这些原语把所有音效和 BGM 轨道渲染成 `Vec<u8>`，再喂给 `macroquad::audio::load_sound_from_bytes`。**不要往 `assets/` 里新增 WAV 文件**，要把声音设计放在代码里，这样更容易参数化。BGM 有三个轨道（`Menu` / `Play` / `Boss`）；`Audio::set_track` 是幂等的，`main.rs` 会在 Scene 切换时调用它，并在 `Playing` 场景中每帧调用它，以便在 Boss 出现或死亡时在 `Play` 和 `Boss` 之间切换。`play_kill_combo` 会根据连击数选择击杀音阶。
- `save.rs` - 通过 `directories::ProjectDirs("dev", "ggttol", "stellar-wing")` 做 JSON 持久化，在 macOS 上对应 `~/Library/Application Support/dev.ggttol.stellar-wing/save.json`。保存内容包括最高分、前 5 排行榜、静音、全屏、语言。日期计算不依赖 `chrono`（见 `epoch_days_to_ymd`）。
- `lang.rs` - 英文 / 中文文本查找函数 `t(key, lang)`。`main.rs` 会尝试加载系统 CJK 字体（`try_load_cjk_font`），如果没有找到就回退到英文。
- `config.rs` - 全局 `CFG`，包含逻辑分辨率（`w`、`h`）；窗口尺寸固定，不能调整大小。

### 需要记住的约定

- 逻辑坐标使用 `CFG.w` × `CFG.h`；鼠标输入需要重映射（`mx * CFG.w / screen_width()`）- 任何新的鼠标命中检测都要沿用这个模式。
- 时间主要分为 `t_acc`（更接近真实时间的动画累加器）和 `world.run_time`（玩法时钟，暂停时会停止）- 不要混用。
- 主循环里每帧的 `dt` 会被夹到 `0.05` - 可以假设步长是有上限的。
- 这个代码库里的新注释 / 字符串经常会直接写中文；新增内容时尽量跟随周围风格，而不是把已有内容硬翻成英文。
