# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Stellar Wing — a Rust roguelike vertical shoot-em-up built on [macroquad](https://github.com/not-fl3/macroquad). Single binary crate (`stellar-wing`).

## Common commands

- `cargo run` — build and launch the game (debug).
- `cargo run --release` — much smoother frame rate; prefer this when actually playing/testing gameplay feel.
- `cargo build --release` — release binary at `target/release/stellar-wing`.
- `cargo check` / `cargo clippy` — fast iteration on type/lint errors.
- `bash scripts/package_macos_app.sh` — build release binary and assemble `dist/Stellar Wing.app` (uses `sips` + `iconutil`, macOS only).

There is no test suite; `cargo test` is a no-op.

## Architecture

The game is a single-threaded macroquad app. `main.rs` owns the loop and is large (~1500 lines) because it inlines scene transitions, spawning, collision dispatch, and HUD/menu drawing. When making changes, expect cross-cutting edits here rather than in many small files.

### Game loop (`src/main.rs`)

- `World` is the run-time game state (player, weapons, bullets, enemies, pickups, score, level, XP, super-charge, combo, boss timer). It is recreated per run via `World::new(ShipType)`.
- `Scene` (`src/scene.rs`) is the top-level state machine: `Menu → Playing ↔ Paused → UpgradePick(cards) → Playing → … → GameOver → Menu`. The match in `main()` per-frame drives input, then dispatches to a single `step_play` for gameplay and to `draw_*` helpers for rendering.
- `step_play` is the per-frame gameplay tick: weapons fire → enemies/bullets update → homing steering → pickup pickup/XP → super bomb (Space) → bullet↔enemy collisions (with crit/static-mark logic) → enemy deaths (combo, super-charge gain, drops, "drone relay" perk) → bullet↔player and enemy↔player collisions → retain dead. Difficulty scales from `world.run_time` via `diff_mul()` and the spawn-interval `lerp` in `spawn_normals`.

### Entities (`src/entity/`)

`mod.rs` re-exports `Player`, `Bullet`, `Enemy` (+ `EnemyKind`, `EliteMod`, `BossMod`, `TelegraphKind`), `Pickup` (+ `PickupKind`), and `HitSource`. Bullets carry a `HitSource` tag so kill-credit logic (e.g. "drone relay") and damage modifiers (missile mark, static mark crit) work without per-weapon collision code. `Player::stats` holds the multipliers (`damage_mul`, `crit_chance`, `crit_mul`, `xp_mul`, `score_mul`, `pickup_radius`, `max_lives`, …) that upgrade cards mutate.

### Weapons (`src/weapon/`)

- `MainGun` is a concrete type, not a trait object — hot path stays monomorphic.
- Sub-weapons (`Missile`, `Drone`, `Laser`, `Chain`) implement `SubWeapon` and live as `Box<dyn SubWeapon>` in `WeaponSlot.subs` (max 4). Adding a new sub-weapon = new file in `src/weapon/`, impl `SubWeapon`, expose via `mod.rs`, and add corresponding upgrade cards in `src/upgrade.rs`.
- `DecayGauge` (in `weapon/mod.rs`) is the shared "weapon level decays over time unless refreshed" mechanic; weapons hold one and call `decay_tick`/`refill`.
- `roll_crit(player, base_mul)` is the unified crit roll all weapons should use so `crit_chance`/`crit_mul` stats apply consistently.

### Other modules

- `upgrade.rs` — card pool + `draw_n` weighted sampler; each `Card` has an `apply: fn(&mut Player, &mut WeaponSlot)`.
- `ship.rs` — `ShipType::ALL` defines selectable ships and `apply()` mutates starting `Player`/`WeaponSlot`.
- `art.rs`, `bg.rs`, `fx.rs` — pure rendering (ship sprites, starfield, particle/float-text FX).
- `audio.rs` — preloads SFX from `assets/sfx*/`. Multiple sfx packs exist (`sfx`, `sfx_focus`, `sfx_piano`, `sfx_piano_combo`, `sfx_real`, `sfx_safe`); `Audio::load` picks one — check there before swapping packs. `play_kill_combo` advances a melodic note index per kill.
- `save.rs` — JSON persistence via `directories::ProjectDirs("dev", "ggttol", "stellar-wing")` → `~/Library/Application Support/dev.ggttol.stellar-wing/save.json` on macOS. Stores high score, top-5 leaderboard, mute, fullscreen, language. Date is computed without `chrono` (see `epoch_days_to_ymd`).
- `lang.rs` — `t(key, lang)` lookup for English/Chinese strings. `main.rs` tries to load a system CJK font (`try_load_cjk_font`) and falls back to English if none is found.
- `config.rs` — global `CFG` with logical resolution (`w`, `h`); the window is fixed-size and not resizable.

### Conventions worth knowing

- Logical coordinates use `CFG.w` × `CFG.h`; mouse input is rescaled (`mx * CFG.w / screen_width()`) — follow that pattern for any new mouse hit-tests.
- Time is mostly tracked as `t_acc` (wall-clock-ish accumulator for animation) vs. `world.run_time` (gameplay clock, paused with the game). Don't mix them.
- `dt` is clamped to `0.05` per frame in the main loop — assume bounded steps.
- New comments/strings are routinely written in Chinese in this codebase; match the surrounding style rather than translating existing ones.
