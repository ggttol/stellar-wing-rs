//! Load authored WAV sound effects for clearer arcade feedback.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub struct Audio {
    pub muted: bool,
    pub vol: f32,
    pub shoot: Option<Sound>,
    pub hit: Option<Sound>,
    pub kill_a: Option<Sound>,
    pub kill_b: Option<Sound>,
    pub kill_c: Option<Sound>,
    pub explode_small: Option<Sound>,
    pub explode_big: Option<Sound>,
    pub powerup: Option<Sound>,
    pub hurt: Option<Sound>,
    pub gameover: Option<Sound>,
    pub levelup: Option<Sound>,
    pub click: Option<Sound>,
    pub combo_mid: Option<Sound>,
    pub combo_high: Option<Sound>,
    pub combo_peak: Option<Sound>,
}

impl Audio {
    pub async fn load(muted: bool) -> Self {
        Self {
            muted,
            vol: 0.42,
            shoot: snd(include_bytes!("../assets/sfx_piano/shoot.wav")).await,
            hit: snd(include_bytes!("../assets/sfx_piano/hit.wav")).await,
            kill_a: snd(include_bytes!("../assets/sfx_piano/kill1.wav")).await,
            kill_b: snd(include_bytes!("../assets/sfx_piano/kill2.wav")).await,
            kill_c: snd(include_bytes!("../assets/sfx_piano/kill3.wav")).await,
            explode_small: snd(include_bytes!("../assets/sfx_piano/explode_small.wav")).await,
            explode_big: snd(include_bytes!("../assets/sfx_piano/explode_big.wav")).await,
            powerup: snd(include_bytes!("../assets/sfx_piano/powerup.wav")).await,
            hurt: snd(include_bytes!("../assets/sfx_piano/hurt.wav")).await,
            gameover: snd(include_bytes!("../assets/sfx_piano/gameover.wav")).await,
            levelup: snd(include_bytes!("../assets/sfx_piano/levelup.wav")).await,
            click: snd(include_bytes!("../assets/sfx_piano/click.wav")).await,
            combo_mid: snd(include_bytes!("../assets/sfx_safe/click.wav")).await,
            combo_high: snd(include_bytes!("../assets/sfx_safe/powerup.wav")).await,
            combo_peak: snd(include_bytes!("../assets/sfx_safe/levelup.wav")).await,
        }
    }

    pub fn play(&self, snd: &Option<Sound>, vol_mul: f32) {
        if self.muted {
            return;
        }
        if let Some(s) = snd {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: self.vol * vol_mul,
                },
            );
        }
    }

    pub fn play_kill_combo(&self, combo: u32) {
        match combo % 3 {
            1 => self.play(&self.kill_a, 0.62),
            2 => self.play(&self.kill_b, 0.62),
            _ => self.play(&self.kill_c, 0.62),
        }

        if combo >= 5 {
            self.play(&self.combo_mid, 0.30);
        }
        if combo >= 12 {
            self.play(&self.combo_high, 0.22);
        }
        if combo >= 20 {
            self.play(&self.combo_peak, 0.18);
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }
}

async fn snd(bytes: &[u8]) -> Option<Sound> {
    load_sound_from_bytes(bytes).await.ok()
}
