# Stellar Wing

一款基于 [macroquad](https://github.com/not-fl3/macroquad) 的 Rust 俯视角肉鸽飞行射击游戏。选择战机、躲避弹幕、升级成长，在每一局里从程序生成的强化卡牌中不断构筑自己的流派。

## 游戏特色

- **肉鸽式强化循环** - 击败敌人 → 掉落经验宝石 → 升级 → 从 3 张卡牌中选 1 张（普通 / 稀有 / 史诗 / 传奇）。
- **三种战机** - `Vanguard`（正面爆发）、`Striker`（高机动）、`Engineer`（开局自带副武器）。
- **五种武器** - 主武器 + 四个副武器槽位（Missile、Drone、Laser、Chain）。
- **精英与 Boss 修饰** - 精英词缀 `Armored / Berserk / Dasher`，Boss 词缀 `Frenzied / Bulwark / Summoner / StormCore`，都有独立的预警表现。
- **Combo / SUPER** - 连杀提升分数倍率，并积攒可清屏的超级炸弹。
- **双语界面** - 支持英文和简体中文，会自动尝试检测系统中的 CJK 字体。
- **本地排行榜** - 前 5 名分数会通过 `directories` crate 持久化到 macOS 的 `~/Library/Application Support/dev.ggttol.stellar-wing/save.json`。

## 构建与运行

需要较新的稳定版 Rust 工具链。

```sh
cargo run --release
```

强烈建议使用 `--release`，因为调试构建会明显影响帧率和手感。

### macOS `.app` 打包

```sh
bash scripts/package_macos_app.sh
```

会生成 `dist/Stellar Wing.app`（仅 macOS 可用，依赖 `sips` 和 `iconutil`）。

## 操作说明

| 按键 | 功能 |
| --- | --- |
| `WASD` / 方向键 | 移动 |
| `Space` | 释放超级炸弹（SUPER 满时） |
| `1` `2` `3` / `← →` + `Enter` / 鼠标点击 | 选择强化卡牌 |
| `P` / `Esc` | 暂停 |
| `Q`（暂停时） | 返回菜单 |
| `M` | 静音开关 |
| `F` | 全屏切换 |
| `L` | 切换语言（EN / 中文） |

主武器会自动开火。

## 项目结构

```text
src/
  main.rs        # 游戏主循环、场景状态机、HUD / 菜单绘制
  scene.rs       # 顶层 Scene 枚举
  entity/        # 玩家、子弹、敌人（含精英 / Boss 修饰）、掉落物
  weapon/        # MainGun + SubWeapon trait（Missile、Drone、Laser、Chain）
  upgrade.rs     # 强化卡牌池与权重抽取
  ship.rs        # 战机预设
  save.rs        # JSON 持久化（高分、排行榜、设置）
  lang.rs        # 英文 / 中文文本
  art.rs bg.rs fx.rs audio.rs  # 渲染、星空、粒子、音效
assets/sfx*/     # 多套音效资源；Audio::load 会自动选择一套
scripts/         # macOS .app 打包脚本
```

更详细的代码架构说明请查看 [`CLAUDE.md`](./CLAUDE.md)。

## 许可证

MIT。见 [LICENSE](./LICENSE)。
