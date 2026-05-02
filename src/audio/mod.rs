//! 程序化合成的 SFX + BGM。所有声音在启动时合成成内存 WAV 字节流，再交给 macroquad 加载。

mod bgm;
mod sfx;
mod synth;

use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};

pub struct Audio {
    pub muted: bool,
    pub vol: f32,
    pub sfx_vol: f32,
    pub bgm_vol: f32,

    // SFX
    shoot: Option<Sound>,
    hit_variants: Vec<Option<Sound>>, // 3 个变体
    kill_steps: Vec<Option<Sound>>,   // 8 级
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
    pub async fn load(muted: bool, master: f32, sfx: f32, bgm: f32) -> Self {
        // 在启动线程上把所有 PCM 合成到字节流，然后交给 macroquad。
        let kill_bytes: Vec<Vec<u8>> = (0..8).map(sfx::kill_at).collect();

        let mut kill_steps = Vec::with_capacity(kill_bytes.len());
        for b in &kill_bytes {
            kill_steps.push(load(b).await);
        }

        // 3 个 hit 变体，随机播放避免疲劳
        let mut hit_variants = Vec::with_capacity(3);
        for k in 0..3 {
            hit_variants.push(load(&sfx::hit_variant(k)).await);
        }

        Self {
            muted,
            vol: master.clamp(0.0, 1.0),
            sfx_vol: sfx.clamp(0.0, 1.0),
            bgm_vol: bgm.clamp(0.0, 1.0),

            shoot: load(&sfx::shoot()).await,
            hit_variants,
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

    /// 与 `play_one` 相同，但音量会有 ±jitter 的随机抖动，避免重复 SFX 听起来太机械。
    fn play_one_jitter(&self, snd: &Option<Sound>, vol_mul: f32, jitter: f32) {
        if self.muted {
            return;
        }
        if let Some(s) = snd {
            use ::rand::{thread_rng, Rng};
            let j = thread_rng().gen_range(-jitter..jitter);
            let v = (self.vol * self.sfx_vol * vol_mul * (1.0 + j)).clamp(0.0, 1.0);
            play_sound(s, PlaySoundParams { looped: false, volume: v });
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

    /// 实时改写主音量，并把当前 BGM 音量同步过去（SFX 在下次播放时生效即可）。
    pub fn set_master_vol(&mut self, v: f32) {
        self.vol = v.clamp(0.0, 1.0);
        self.refresh_bgm_volume();
    }
    pub fn set_bgm_vol(&mut self, v: f32) {
        self.bgm_vol = v.clamp(0.0, 1.0);
        self.refresh_bgm_volume();
    }
    pub fn set_sfx_vol(&mut self, v: f32) {
        self.sfx_vol = v.clamp(0.0, 1.0);
    }

    fn refresh_bgm_volume(&self) {
        if self.muted {
            return;
        }
        let snd = match self.current_track {
            BgmTrack::None => return,
            BgmTrack::Menu => &self.bgm_menu,
            BgmTrack::Play => &self.bgm_play,
            BgmTrack::Boss => &self.bgm_boss,
        };
        if let Some(s) = snd {
            set_sound_volume(s, self.vol * self.bgm_vol);
        }
    }

    // —— SFX 公共 API —————————————————————————————

    pub fn play_shoot(&self) {
        // shoot 频次极高，加 ±10% 音量抖动让连发更有"质感"
        self.play_one_jitter(&self.shoot, 0.55, 0.10);
    }
    pub fn play_hit(&self) {
        if self.hit_variants.is_empty() {
            return;
        }
        use ::rand::{thread_rng, Rng};
        let i = thread_rng().gen_range(0..self.hit_variants.len());
        self.play_one_jitter(&self.hit_variants[i], 0.35, 0.12);
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
