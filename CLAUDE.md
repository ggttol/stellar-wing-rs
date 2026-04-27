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

Single-threaded macroquad app. `main.rs` is slim — it owns the window/loop, the `Scene` state machine, and orchestrates a per-frame `step_play` that delegates to focused modules.

### Game loop (`src/main.rs`)

- `Scene` (`src/scene.rs`) is the top-level state machine: `Menu → Playing ↔ Paused → UpgradePick(cards) → Playing → … → GameOver → Menu`. The match in `main()` per-frame drives input + scene transitions and dispatches to `hud::*` for rendering.
- `World` (`src/world.rs`) is the run-time game state (player, weapons, bullets, enemies, pickups, score, level, XP, super-charge, combo, boss timer). All fields are `pub` because gameplay systems are split across modules — that's intentional, not sloppy.
- `step_play` (in `main.rs`) is the per-frame gameplay tick. It encodes the *order* of subsystems: combo decay → player/weapons/enemy/bullet update → spawn → homing → pickup collection → super bomb → player-bullet vs enemies → kill processing (combo, drops, drone relay) → enemy-bullet vs player → enemy contact → retain dead. Each subsystem lives in `combat.rs` or `spawn.rs`.

### Modules in dependency order

- `world.rs` — `World`, `SpawnTimers`. Plain data, no logic.
- `spawn.rs` — `spawn_normals`, `spawn_enemy`, `spawn_boss`, `drop_xp_gems`, `maybe_drop_special`. Difficulty scaling from `world.run_time` lives here (interval `lerp`, hp/score multipliers, elite roll).
- `combat.rs` — gameplay collisions and kill resolution: `steer_homing_bullets`, `resolve_player_bullets`, `process_kills`, `resolve_enemy_bullets`, `resolve_enemy_player_contact`, `trigger_super`, `collect_pickups`, `spawn_relay_missile`. The crit / static-mark / missile-mark logic and the combo→super-charge/score multipliers live in `process_kills`.
- `collision.rs` — only the geometric primitives (`hit_circle`, AABB-vs-circle helpers). Don't put gameplay logic here.
- `hud.rs` — Menu/HUD/Pause/UpgradePick/GameOver rendering, plus `draw_world` (pickups → bullets → enemies → weapons → player). The mouse-coordinate rescale pattern (`mx * CFG.w / screen_width()`) is in `main.rs` next to `card_at`.

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
- `audio/` — fully procedural. `synth.rs` has oscillators (sine/square/saw/triangle/noise), ADSR, one-pole LP, frequency-swept note builder, and an in-memory 16-bit PCM mono WAV encoder. `sfx.rs` and `bgm.rs` use those primitives to render every SFX and BGM track at startup into `Vec<u8>`, fed to `macroquad::audio::load_sound_from_bytes`. **Don't add WAV files to `assets/`** — design the sound in code so it stays parameterizable. BGM has three tracks (`Menu` / `Play` / `Boss`); `Audio::set_track` is idempotent and `main.rs` calls it on Scene transitions and per-frame from `Playing` (to flip Play↔Boss when boss spawns/dies). `play_kill_combo` picks a kill-step pitch by combo count.
- `save.rs` — JSON persistence via `directories::ProjectDirs("dev", "ggttol", "stellar-wing")` → `~/Library/Application Support/dev.ggttol.stellar-wing/save.json` on macOS. Stores high score, top-5 leaderboard, mute, fullscreen, language. Date is computed without `chrono` (see `epoch_days_to_ymd`).
- `lang.rs` — `t(key, lang)` lookup for English/Chinese strings. `main.rs` tries to load a system CJK font (`try_load_cjk_font`) and falls back to English if none is found.
- `config.rs` — global `CFG` with logical resolution (`w`, `h`); the window is fixed-size and not resizable.

### Conventions worth knowing

- Logical coordinates use `CFG.w` × `CFG.h`; mouse input is rescaled (`mx * CFG.w / screen_width()`) — follow that pattern for any new mouse hit-tests.
- Time is mostly tracked as `t_acc` (wall-clock-ish accumulator for animation) vs. `world.run_time` (gameplay clock, paused with the game). Don't mix them.
- `dt` is clamped to `0.05` per frame in the main loop — assume bounded steps.
- New comments/strings are routinely written in Chinese in this codebase; match the surrounding style rather than translating existing ones.
