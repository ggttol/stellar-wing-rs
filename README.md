# Stellar Wing

A Rust roguelike vertical shoot-'em-up built on [macroquad](https://github.com/not-fl3/macroquad). Pick a ship, dodge bullet patterns, level up, and choose from procedurally drawn upgrade cards run after run.

## Features

- **Roguelike upgrade loop** — kill enemies → drop XP gems → level up → pick 1 of 3 cards (common / rare / epic / legendary).
- **Three ships** — *Vanguard* (frontal burst), *Striker* (mobility), *Engineer* (starts with a sub-weapon).
- **Five weapons** — main gun + four sub-weapon slots (Missile, Drone, Laser, Chain).
- **Elite & boss modifiers** — `Armored / Berserk / Dasher` elites and `Frenzied / Bulwark / Summoner / StormCore` bosses, each with distinct telegraphs.
- **Combo / SUPER** — chain kills for score multipliers; charge a screen-clearing super bomb.
- **Bilingual UI** — English and Simplified Chinese, auto-detects a system CJK font.
- **Local leaderboard** — top-5 scores persisted to `~/Library/Application Support/dev.ggttol.stellar-wing/save.json` (macOS) via the `directories` crate.

## Build & run

Requires a recent stable Rust toolchain.

```sh
cargo run --release
```

`--release` is strongly recommended — debug builds noticeably affect frame pacing.

### macOS `.app` bundle

```sh
bash scripts/package_macos_app.sh
```

Produces `dist/Stellar Wing.app` (uses `sips` and `iconutil`, macOS only).

## Controls

| Key | Action |
| --- | --- |
| `WASD` / arrows | Move |
| `Space` | Super bomb (when SUPER gauge full) |
| `1` `2` `3` / `← →` + `Enter` / click | Pick upgrade card |
| `P` / `Esc` | Pause |
| `Q` (paused) | Quit to menu |
| `M` | Toggle mute |
| `F` | Toggle fullscreen |
| `L` | Toggle language (EN / 中文) |

The main gun is auto-firing.

## Project layout

```
src/
  main.rs        # Game loop, scene state machine, HUD/menu drawing
  scene.rs       # Top-level Scene enum
  entity/        # Player, Bullet, Enemy (+ elite/boss mods), Pickup
  weapon/        # MainGun + SubWeapon trait (Missile, Drone, Laser, Chain)
  upgrade.rs     # Upgrade card pool and weighted draw
  ship.rs        # Ship presets
  save.rs        # JSON persistence (high score, leaderboard, settings)
  lang.rs        # EN / ZH translations
  art.rs bg.rs fx.rs audio.rs  # Rendering, starfield, particles, SFX
assets/sfx*/     # Multiple sound packs; Audio::load picks one
scripts/         # macOS .app packaging
```

See [`CLAUDE.md`](./CLAUDE.md) for deeper notes on the code architecture.

## License

MIT. See [LICENSE](./LICENSE).
