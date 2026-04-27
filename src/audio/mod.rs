//! 程序化合成的 SFX + BGM。所有声音在启动时合成成内存 WAV 字节流，再交给 macroquad 加载。

mod bgm;
mod sfx;
mod synth;

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

pub struct Audio {
    pub muted: bool,
    pub vol: f32,
    sfx_vol: f32,
    bgm_vol: f32,

    // SFX
    shoot: Option<Sound>,
    hit: Option<Sound>,
    kill_steps: Vec<Option<Sound>>, // 8 级
    explode_small: Option<Sound>,
    explode_big: Option<Sound>,
    powerup: Option<Sound>,
    super_bomb: Option<Sound>,
    hurt: Option<Sound>,
    gameover: Option<Sound>,
    levelup: Option<Sound>,
    click: Option<Sound>,
    boss_intro: Option<Sound>,

    // BGM
    bgm_menu: Option<Sound>,
    bgm_play: Option<Sound>,
    bgm_boss: Option<Sound>,
    current_track: BgmTrack,
}

/// BGM 轨道标识。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BgmTrack {
    None,
    Menu,
    Play,
    Boss,
}

impl Audio {
    pub async fn load(muted: bool) -> Self {
        // 在启动线程上把所有 PCM 合成到字节流，然后交给 macroquad。
        let kill_bytes: Vec<Vec<u8>> = (0..8).map(sfx::kill_at).collect();

        let mut kill_steps = Vec::with_capacity(kill_bytes.len());
        for b in &kill_bytes {
            kill_steps.push(load(b).await);
        }

        Self {
            muted,
            vol: 0.5,
            sfx_vol: 1.0,
            bgm_vol: 0.55,

            shoot: load(&sfx::shoot()).await,
            hit: load(&sfx::hit()).await,
            kill_steps,
            explode_small: load(&sfx::explode_small()).await,
            explode_big: load(&sfx::explode_big()).await,
            powerup: load(&sfx::powerup()).await,
            super_bomb: load(&sfx::super_bomb()).await,
            hurt: load(&sfx::hurt()).await,
            gameover: load(&sfx::gameover()).await,
            levelup: load(&sfx::levelup()).await,
            click: load(&sfx::click()).await,
            boss_intro: load(&sfx::boss_intro()).await,

            bgm_menu: load(&bgm::menu()).await,
            bgm_play: load(&bgm::play()).await,
            bgm_boss: load(&bgm::boss()).await,
            current_track: BgmTrack::None,
        }
    }

    fn play_one(&self, snd: &Option<Sound>, vol_mul: f32) {
        if self.muted {
            return;
        }
        if let Some(s) = snd {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: self.vol * self.sfx_vol * vol_mul,
                },
            );
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        if self.muted {
            self.stop_current_track();
        } else {
            // 恢复当前轨道
            let t = self.current_track;
            self.current_track = BgmTrack::None;
            self.set_track(t);
        }
        self.muted
    }

    // —— SFX 公共 API —————————————————————————————

    pub fn play_shoot(&self) {
        self.play_one(&self.shoot, 0.55);
    }
    pub fn play_hit(&self) {
        self.play_one(&self.hit, 0.35);
    }
    pub fn play_hurt(&self) {
        self.play_one(&self.hurt, 0.9);
    }
    pub fn play_explode_small(&self) {
        self.play_one(&self.explode_small, 0.55);
    }
    pub fn play_explode_big(&self) {
        self.play_one(&self.explode_big, 0.95);
    }
    pub fn play_powerup(&self) {
        self.play_one(&self.powerup, 0.55);
    }
    pub fn play_super(&self) {
        self.play_one(&self.super_bomb, 0.95);
    }
    pub fn play_levelup(&self) {
        self.play_one(&self.levelup, 0.6);
    }
    pub fn play_gameover(&self) {
        self.play_one(&self.gameover, 0.85);
    }
    pub fn play_click(&self) {
        self.play_one(&self.click, 0.55);
    }
    pub fn play_pause(&self) {
        self.play_one(&self.click, 0.45);
    }
    pub fn play_boss_intro(&self) {
        self.play_one(&self.boss_intro, 0.85);
    }

    /// combo 越高音越亮；combo 1 用最低音，每涨 3 升一阶。
    pub fn play_kill_combo(&self, combo: u32) {
        if self.kill_steps.is_empty() {
            return;
        }
        let step = ((combo.saturating_sub(1) / 3) as usize).min(self.kill_steps.len() - 1);
        self.play_one(&self.kill_steps[step], 0.55);
    }

    // —— BGM ————————————————————————————————————

    pub fn set_track(&mut self, track: BgmTrack) {
        if track == self.current_track {
            return;
        }
        self.stop_current_track();
        self.current_track = track;
        if self.muted {
            return;
        }
        let snd = match track {
            BgmTrack::None => return,
            BgmTrack::Menu => &self.bgm_menu,
            BgmTrack::Play => &self.bgm_play,
            BgmTrack::Boss => &self.bgm_boss,
        };
        if let Some(s) = snd {
            play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: self.vol * self.bgm_vol,
                },
            );
        }
    }

    fn stop_current_track(&self) {
        let snd = match self.current_track {
            BgmTrack::None => return,
            BgmTrack::Menu => &self.bgm_menu,
            BgmTrack::Play => &self.bgm_play,
            BgmTrack::Boss => &self.bgm_boss,
        };
        if let Some(s) = snd {
            stop_sound(s);
        }
    }
}

async fn load(bytes: &[u8]) -> Option<Sound> {
    load_sound_from_bytes(bytes).await.ok()
}
