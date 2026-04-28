# Stellar Wing

一款基于 [macroquad](https://github.com/not-fl3/macroquad) 的 Rust 俯视角肉鸽飞行射击游戏。选择战机、穿越 5 个章节闯入无尽模式，在每一局里从 35 张程序生成的强化卡牌中不断构筑自己的流派。

## 游戏特色

- **章节推进** — 5 个故事章节（外环带 → 无畏舰核心），每章固定时长 + 主题色 + 专属敌人组合，通关后进入无尽模式，难度持续攀升。
- **肉鸽强化循环** — 击杀敌人 → 掉落经验宝石 → 升级 → 从 3 张卡牌中选 1 张。三档稀有度（Common / Rare / Epic），加权随机 + 保底机制（每 5 抽至少出一张副武器解锁卡）。
- **三种战机** — `Vanguard`（正面火力强化，开局主炮 Lv2）、`Striker`（高速机动，闪避灵活）、`Engineer`（开局自带僚机，但主炮稍弱）。
- **八种武器** — 主炮 5 级进化（单发→双发→三向→五向→穿透） + 7 种副武器，最多携带 4 种：`Homing Missile`（追踪导弹）、`Orbit Drone`（自动瞄准僚机）、`Pulse Laser`（追踪持续光束）、`Chain Bolt`（远程闪电链跳）、`Void Rift`（追猎型脉冲伤害场）、`Wave Cannon`（正弦波弹丸扫荡）、`Reflector`（瞄准式反弹弹丸）。副武器 ≥3 个时有递减收益。
- **六种协同 Perk** — 跨武器联动效果：`Heat Lock`（导弹标记 + 激光追伤）、`Static Mark`（闪电链标记保证暴击）、`Drone Relay`（僚机击杀补发追踪弹）、`Gravity Well`（裂隙吸入敌人）、`Resonance`（波动炮 + 闪电链额外跳数）、`Prism`（反射镜弹丸穿过光束 +50% 伤害并穿透）。
- **共鸣槽 + SUPER** — 击杀填充共鸣槽，满槽进入 6 秒"过载"状态（全伤害 ×1.30）。连杀积攒 SUPER 能量，满时可释放清屏炸弹。
- **精英、特殊敌人与 Boss** — 普通敌人外，还包含 `Kamikaze` 自爆冲锋、`Strafer` 横扫机、`Sniper` 狙击机、`Weaver` 编织机、`MineLayer` 布雷机等差异化敌人；精英词缀 `Armored / Berserk / Dasher`；每章 Boss 从 `Frenzied / Bulwark / Summoner / StormCore / Phantom / Hydra` 池中随机抽取，各有独立的预警和攻击模式。
- **永续成长** — 6 条跨局天赋（伤害 / HP / 移速 / XP / 星尘收益 / 开局 SUPER），用每局获得的星尘购买升级。
- **全程序化音频** — 所有音效和 3 轨 BGM（菜单 / 战斗 / Boss）均由代码合成，不依赖外部音频文件。
- **双语界面** — 英文和简体中文，自动检测系统 CJK 字体，运行时按 `L` 切换。
- **游戏体验打磨** — Boss 攻击屏幕震动、屏幕外敌人方向指示器、低血量脉冲警告、成功躲避 Kamikaze 得分奖励。

## 近期玩法调整

- **武器可靠性增强** — 僚机会主动瞄准附近敌人；激光会轻微追踪目标横坐标；闪电链第一跳改为远距离锁敌；反射镜优先朝最近敌人发射，再利用反弹路径；导弹冷却、伤害和满级数量略有增强。
- **裂隙重做** — `Void Rift` 不再原地等待敌人撞入，而是从玩家附近生成后主动追猎最近敌人，范围内持续灼烧并周期性释放强脉冲。
- **敌方弹幕丰富化** — 新增狙击弹、蛇形编织弹、慢速布雷弹等不同弹型，并给对应敌人绘制了不同外形和子弹视觉。
- **开局难度回调** — 敌方子弹速度和开火频率加入前 120 秒暖场曲线，新型敌人分阶段登场，避免开局弹幕压迫过强。

## 构建与运行

### 下载发布版

最新版本见 [GitHub Releases](https://github.com/ggttol/stellar-wing-rs/releases/latest)。

- macOS：下载 `Stellar.Wing.zip`，解压后运行 `Stellar Wing.app`。
- Windows：下载 `Stellar.Wing.Windows.x86_64.zip`，解压后运行 `stellar-wing.exe`。

macOS 版本目前未做 Apple Developer ID 签名和公证，首次打开时可能被 Gatekeeper 拦截。

### 从源码运行

需要较新的稳定版 Rust 工具链。

```sh
cargo run --release
```

强烈建议使用 `--release`，调试构建会显著影响帧率和手感。

### macOS `.app` 打包

```sh
bash scripts/package_macos_app.sh
```

生成 `dist/Stellar Wing.app`（仅 macOS，依赖 `sips` 和 `iconutil`）。

### Windows 版本

Windows 版由 GitHub Actions 在 `windows-latest` runner 上构建，并自动上传到当前版本的 GitHub Release。对应流程位于 `.github/workflows/windows-release.yml`。

## 操作说明

| 按键 | 功能 |
| --- | --- |
| `WASD` / 方向键 | 移动 |
| `Space` | 释放 SUPER 炸弹（能量满时） |
| `1` `2` `3` / `← →` + `Enter` / 鼠标点击 | 选择强化卡牌 |
| `P` / `Esc` | 暂停 |
| `Q`（暂停时） | 返回菜单 |
| `T`（菜单中） | 打开天赋面板 |
| `M` | 静音开关 |
| `F` | 全屏切换 |
| `L` | 切换语言（EN / 中文） |
| `A` `D` / `← →`（菜单中） | 选择战机 |

主武器自动开火。

## 项目结构

```text
src/
  main.rs          # 游戏主循环、场景状态机、HUD/菜单绘制
  config.rs        # 全局常量 CFG（逻辑分辨率 480×800）
  world.rs         # World 运行时状态容器
  scene.rs         # Scene 顶层状态枚举
  entity/          # 玩家、子弹、敌人（含精英/Boss 修饰）、掉落物
  weapon/          # MainGun + SubWeapon trait（7 种副武器）
    main_gun.rs    #   主炮 5 级进化
    missile.rs     #   追踪导弹
    drone.rs       #   自动瞄准僚机
    laser.rs       #   追踪持续光束
    chain.rs       #   远程闪电链
    rift.rs        #   虚空裂隙（追猎伤害场）
    wave.rs        #   波动炮（正弦波弹丸）
    reflector.rs   #   反射镜（瞄准式反弹弹丸）
  upgrade.rs       # 35 张强化卡池、权重抽取、保底机制
  ship.rs          # 3 种战机预设
  chapter.rs       # 5 个故事章节 + 无尽模式定义
  talents.rs       # 6 条跨局永久天赋（星尘购买）
  spawn.rs         # 敌人生成、难度缩放、Boss 入场
  combat.rs        # 碰撞结算、击杀处理、拾取、SUPER
  collision.rs     # 几何碰撞原语
  save.rs          # JSON 持久化（高分/排行榜/设置/天赋）
  lang.rs          # 中英双语文本查表
  audio/           # 全流程程序化音效与 BGM 合成
  art.rs           # 战机精灵渲染
  bg.rs            # 星空背景渲染
  fx.rs            # 粒子特效、浮字、屏幕震动
  hud.rs           # 菜单/HUD/暂停/选卡/GameOver 绘制
scripts/            # macOS .app 打包脚本
.github/workflows/  # Windows release 构建与上传流程
```

更详细的代码架构说明请查看 [`CLAUDE.md`](./CLAUDE.md)。

## 许可证

MIT。见 [LICENSE](./LICENSE)。
