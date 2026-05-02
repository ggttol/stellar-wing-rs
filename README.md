# Stellar Wing

一款基于 [macroquad](https://github.com/not-fl3/macroquad) 的 Rust 俯视角肉鸽飞行射击游戏。选择战机，穿越 5 个章节进入无尽模式；用 buff 拾取叠加细节、用升级弹窗做构筑决策、用武器进化把流派推到极致。

## 游戏特色

### 核心循环
- **章节推进** — 5 个故事章节（外环带 → 无畏舰核心），每章固定时长 + 主题色 + 专属敌人组合，通关后进入无尽模式，难度持续攀升。
- **章节分叉** — 每章 Boss 死后弹一次 2 选 1 路线：`Onslaught`（敌人 +30% HP，分数 +20%）/ `Blitz`（HP -15%，章节时长 -25%）/ `Harvest`（buff 掉率 ×1.5，分数 +30%）。让每次 run 风格都不同。
- **难度档位** — Normal / Hard / Nightmare 三档，按通关解锁。难度提升 HP / 弹速 / XP / 分数 / 星尘倍率。
- **每日挑战** — 菜单按 `Y`，用当日日期作种子启动；当日最高分单独记录。

### 战斗系统
- **三种战机** — `Vanguard`（正面火力强化，开局主炮 Lv2）、`Striker`（高速机动，闪避灵活）、`Engineer`（开局自带僚机，但主炮稍弱）。
- **八种武器** — 主炮 5 级进化（单发 → 双发 → 三向 → 五向 → 穿透）+ 7 种副武器，最多携带 4 种：`Homing Missile`、`Orbit Drone`、`Pulse Laser`、`Chain Bolt`、`Void Rift`、`Wave Cannon`、`Reflector`。副武器 ≥3 个时有递减收益。
- **六种协同 Perk** — 跨武器联动效果：`Heat Lock`（导弹标记 + 激光追伤）、`Static Mark`（闪电链标记保证暴击）、`Drone Relay`（僚机击杀补发追踪弹）、`Gravity Well`（裂隙吸入敌人）、`Resonance`（波动炮 + 闪电链额外跳数）、`Prism`（反射镜弹丸穿过光束 +50% 伤害并穿透）。
- **六种武器进化** — 副武器满级 + 对应 Perk 后解锁的金卡：`Heatseeker` / `Swarm` / `Annihilator` / `Tempest` / `Cascade` / `Kaleidoscope`，对应武器伤害大幅放大、形态升级。
- **共鸣槽 + SUPER** — 击杀填充共鸣槽，满槽进入 6 秒"过载"状态（全伤害 ×1.30）。连杀积攒 SUPER 能量，满时按 `Space` 释放清屏炸弹。

### 强化系统
- **战斗中 buff 拾取** — 9 种小数值卡（射速 / 伤害 / 弹速 / 移速 / 拾取范围 / XP / 分数 / 暴击率 / 暴伤）从敌人爆炸掉落，自动吸附 + 立即应用，无需中断战斗。
- **构筑性升级弹窗** — 升级时从 3 张卡选 1，仅保留构筑性卡（武器解锁 / 升级 / Perk / 进化 / HP / Shield / Heal 等）。四档稀有度（Common / Rare / Epic / Legendary），加权随机 + 保底（连续 4 抽未出副武器解锁卡时强制出）。
- **永续成长** — 6 条跨局天赋（伤害 / HP / 移速 / XP / 星尘收益 / 开局 SUPER），用每局获得的星尘购买升级。

### 留存与收集
- **20 条成就** — 首杀 Boss / 单局 100 连击 / 不死过章 / 武器累计伤害门槛 / 难度通关 / 跨局总数 …… 解锁后给星尘奖励。
- **图鉴** — 击杀解锁敌人、Boss 修饰、副武器条目，可在菜单查看已解锁内容。
- **飞船涂装** — 每艘飞船 3 套配色（默认 / 中级 / 终级），按 Normal / Hard 通关解锁，可在菜单切换。
- **结算 / 暂停 build 总览** — 暂停页和 GameOver 都显示当前武器 + Perks + 关键属性 + 8 种武器伤害分布占比条。

### 敌人 & 关卡
- **精英、特殊敌人与 Boss** — 普通敌人外，还包含 `Kamikaze` 自爆冲锋、`Strafer` 横扫机、`Sniper` 狙击机、`Weaver` 编织机、`MineLayer` 布雷机等差异化敌人；精英词缀 `Armored / Berserk / Dasher`；每章 Boss 从 `Frenzied / Bulwark / Summoner / StormCore / Phantom / Hydra` 池中随机抽取，各有独立的预警和攻击模式。

### 表现与手感
- **全程序化音频** — 所有音效和 3 轨 BGM（菜单 / 战斗 / Boss）均由代码合成，不依赖外部音频文件。`shoot` 是低频 sci-fi blaster（多层叠加 + 膛口噪声 click）；`hit` 有 3 个变体随机播放避免疲劳。
- **手感打磨** — 命中冻帧（hit-pause）、Boss 死亡慢动作（slow-mo）、暴击 / 受击冲击波环、子弹分类拖尾、屏幕受击红闪、低血量 / 过载 / SUPER 蓄满边缘 vignette、Boss 攻击屏幕震动（用户可调倍率）、屏幕外敌人方向指示器、成功躲避 Kamikaze 得分奖励。
- **设置面板** — 总音量 / BGM / SFX 独立滑条、屏震强度、静音、全屏，菜单按 `O` 进入。
- **双语界面** — 英文和简体中文，自动检测系统 CJK 字体，运行时按 `L` 切换。

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

### 测试 / Lint

```sh
cargo test                                  # 17 个单元测试
cargo clippy --all-targets -- -D warnings   # 0 警告才算过
```

### macOS `.app` 打包

```sh
bash scripts/package_macos_app.sh
```

生成 `dist/Stellar Wing.app`（仅 macOS，依赖 `sips` 和 `iconutil`）。

### Windows 版本

Windows 版由 GitHub Actions 在 `windows-latest` runner 上构建，并自动上传到当前版本的 GitHub Release。对应流程位于 `.github/workflows/windows-release.yml`。

## 操作说明

### 战斗中

| 按键 | 功能 |
| --- | --- |
| `WASD` / 方向键 | 移动 |
| `Space` | 释放 SUPER 炸弹（能量满时） |
| `1` `2` `3` / `← →` + `Enter` / 鼠标点击 | 升级时选卡 |
| `P` / `Esc` | 暂停（同时显示 build 总览） |
| `Q`（暂停时） | 返回菜单 |

主武器自动开火；buff 拾取自动应用，不打断战斗。

### 菜单

| 按键 | 功能 |
| --- | --- |
| `Enter` / `Space` | 启动一局 |
| `Y` | 启动每日挑战 |
| `A` `D` / `← →` | 选择战机 |
| `T` | 进入天赋页 |
| `O` | 进入设置页 |
| `H` | 进入成就页 |
| `C` | 进入图鉴页 |
| `N` | 切换难度（已解锁内循环） |
| `K` | 切换当前飞船的涂装 |
| `L` | 切换语言（EN / 中文） |
| `M` | 静音开关 |
| `F` | 全屏切换 |
| `Esc` | 退出 |

## 项目结构

```text
src/
  main.rs          # 游戏主循环、场景调度
  scene.rs         # Scene 顶层状态枚举（11 个变体）
  config.rs        # 全局常量 CFG（逻辑分辨率 480×800）
  world.rs         # World 运行时状态 + 难度 / 章节修饰枚举
  entity/          # 玩家、子弹、敌人（含精英/Boss 修饰）、拾取（含 BuffKind）
  weapon/          # MainGun + SubWeapon trait（7 种副武器）+ SynergyGauge
    main_gun.rs    #   主炮 5 级进化
    missile.rs     #   追踪导弹
    drone.rs       #   自动瞄准僚机
    laser.rs       #   追踪持续光束
    chain.rs       #   远程闪电链
    rift.rs        #   虚空裂隙（追猎伤害场）
    wave.rs        #   波动炮（正弦波弹丸）
    reflector.rs   #   反射镜（瞄准式反弹弹丸）
  upgrade.rs       # ~25 张构筑性卡池、权重抽取、保底
  ship.rs          # 3 种战机预设
  chapter.rs       # 5 个故事章节 + 无尽模式定义
  talents.rs       # 6 条跨局永久天赋（星尘购买）
  achievements.rs  # 20 条成就 + 触发检查
  spawn.rs         # 敌人生成、难度缩放、buff/special 掉落
  combat.rs        # 碰撞结算、击杀处理、拾取、SUPER、伤害分布
  collision.rs     # 几何碰撞原语
  save.rs          # JSON 持久化（成就/图鉴/难度/每日/涂装/天赋）
  lang.rs          # 中英双语文本查表
  audio/           # 全流程程序化音效与 BGM 合成
  art.rs           # 战机/敌人精灵 + 涂装表
  bg.rs            # 星空背景渲染
  fx.rs            # 粒子/拖尾/冲击波/屏震/闪屏/hit-pause/slow-mo
  hud.rs           # 菜单/HUD/暂停/选卡/章节分叉/成就/图鉴/设置/GameOver 绘制
scripts/            # macOS .app 打包脚本
.github/workflows/  # Windows release 构建与上传流程
docs/
  PORTING-SPEC.md   # 跨引擎 / 微信小游戏移植的完整玩法/数值规格
```

更详细的代码架构说明请查看 [`CLAUDE.md`](./CLAUDE.md)。

跨引擎 / 平台移植（含微信小游戏）的完整设计与数值规格见 [`docs/PORTING-SPEC.md`](./docs/PORTING-SPEC.md)——零依赖 Rust 源码即可复刻。

## 许可证

MIT。见 [LICENSE](./LICENSE)。
