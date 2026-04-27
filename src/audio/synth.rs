//! 程序化合成原语：振荡器、ADSR、滤波器、扫频，以及内存 WAV 编码。
//!
//! 全部产出 16-bit PCM 单声道，采样率 [`SAMPLE_RATE`]。
//! 每个 SFX / BGM 在游戏启动时合成成 `Vec<u8>`，再喂给 `macroquad::audio::load_sound_from_bytes`。

#![allow(dead_code)] // 振荡器/工具留作后续音效设计的"调色板"，刻意保留。

pub const SAMPLE_RATE: u32 = 44_100;

/// 1 秒等于多少样本。
pub const SR_F: f32 = SAMPLE_RATE as f32;

#[inline]
pub fn samples_for(seconds: f32) -> usize {
    (seconds * SR_F) as usize
}

#[inline]
pub fn time_at(idx: usize) -> f32 {
    idx as f32 / SR_F
}

// —— 振荡器 ——————————————————————————————————————————————————————

#[inline]
pub fn sine(phase: f32) -> f32 {
    (phase * std::f32::consts::TAU).sin()
}

#[inline]
pub fn square(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty {
        1.0
    } else {
        -1.0
    }
}

#[inline]
pub fn saw(phase: f32) -> f32 {
    2.0 * phase.fract() - 1.0
}

#[inline]
pub fn triangle(phase: f32) -> f32 {
    let p = phase.fract();
    if p < 0.5 {
        4.0 * p - 1.0
    } else {
        3.0 - 4.0 * p
    }
}

/// xorshift 风格的低开销噪声，避免依赖外部 RNG。
pub struct NoiseRng {
    state: u32,
}

impl NoiseRng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed.max(1),
        }
    }
    /// 返回 [-1, 1]。
    pub fn next(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x as i32 as f32) / (i32::MAX as f32)
    }
}

// —— 音名 → 频率 ————————————————————————————————————————————————

/// MIDI 音高（A4=69）转频率（Hz）。
#[inline]
pub fn midi_hz(note: f32) -> f32 {
    440.0 * (2f32).powf((note - 69.0) / 12.0)
}

// —— ADSR ——————————————————————————————————————————————————————

#[derive(Clone, Copy)]
pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Adsr {
    pub fn pluck(release: f32) -> Self {
        Self {
            attack: 0.005,
            decay: 0.04,
            sustain: 0.0,
            release,
        }
    }
    pub fn percussive(decay: f32) -> Self {
        Self {
            attack: 0.001,
            decay,
            sustain: 0.0,
            release: 0.0,
        }
    }
    pub fn pad(attack: f32, release: f32) -> Self {
        Self {
            attack,
            decay: 0.05,
            sustain: 0.85,
            release,
        }
    }
    /// 在长度为 `total_seconds` 的音上，求 `t` 处的振幅。
    /// 假设 `release` 紧贴在持续段后；release 段从 sustain 衰减到 0。
    pub fn at(&self, t: f32, total: f32) -> f32 {
        let sustain_end = (total - self.release).max(self.attack + self.decay);
        if t < self.attack {
            t / self.attack.max(1e-5)
        } else if t < self.attack + self.decay {
            let k = (t - self.attack) / self.decay.max(1e-5);
            1.0 + (self.sustain - 1.0) * k
        } else if t < sustain_end {
            self.sustain
        } else {
            let k = ((t - sustain_end) / self.release.max(1e-5)).clamp(0.0, 1.0);
            self.sustain * (1.0 - k)
        }
    }
}

// —— 单极低通 ——————————————————————————————————————————————————

pub struct OnePoleLp {
    a: f32,
    z: f32,
}

impl OnePoleLp {
    pub fn new(cutoff_hz: f32) -> Self {
        let dt = 1.0 / SR_F;
        let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
        let a = dt / (rc + dt);
        Self { a, z: 0.0 }
    }
    pub fn process(&mut self, x: f32) -> f32 {
        self.z += self.a * (x - self.z);
        self.z
    }
}

// —— 缓冲区辅助 ——————————————————————————————————————————————————

/// 把 `src` 加到 `dst` 上（从 `offset` 起），自动扩展长度。`gain` 控制混入幅度。
pub fn mix_into(dst: &mut Vec<f32>, src: &[f32], offset: usize, gain: f32) {
    let needed = offset + src.len();
    if dst.len() < needed {
        dst.resize(needed, 0.0);
    }
    for (i, s) in src.iter().enumerate() {
        dst[offset + i] += s * gain;
    }
}

/// 把整段乘以一个增益 envelope（长度需 ≥ buf.len()）。
pub fn apply_env(buf: &mut [f32], env: impl Fn(usize) -> f32) {
    for (i, s) in buf.iter_mut().enumerate() {
        *s *= env(i);
    }
}

// —— Soft clip & WAV 编码 ————————————————————————————————————————

#[inline]
fn soft_clip(x: f32) -> f32 {
    // tanh 近似，避免 std 依赖（其实 f32::tanh 也行）。-1.5..1.5 内大致线性。
    let x = x.clamp(-1.5, 1.5);
    x - x * x * x / 3.0
}

/// 将 [-1, 1] 浮点缓冲区编码为 RIFF/WAVE 16-bit PCM 单声道字节流。
/// `peak_gain` 是写出前的整段增益；超 1.0 会被 soft-clip。
pub fn encode_wav(samples: &[f32], peak_gain: f32) -> Vec<u8> {
    let n = samples.len();
    let data_size = (n * 2) as u32;
    let riff_size = 36 + data_size;

    let mut out = Vec::with_capacity(44 + n * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");

    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // channels
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());

    for s in samples {
        let v = soft_clip(*s * peak_gain).clamp(-1.0, 1.0);
        let i = (v * 32767.0) as i16;
        out.extend_from_slice(&i.to_le_bytes());
    }
    out
}

// —— 高层"打音符"辅助 ——————————————————————————————————————————

/// 在缓冲区某偏移处叠加一个带 ADSR 的简单音符。
/// `wave` 接受相位（0..1 循环）返回 [-1,1]。
pub fn add_note(
    dst: &mut Vec<f32>,
    offset_sec: f32,
    duration: f32,
    freq_hz: f32,
    wave: impl Fn(f32) -> f32,
    env: Adsr,
    gain: f32,
) {
    let n = samples_for(duration);
    let off = samples_for(offset_sec);
    let needed = off + n;
    if dst.len() < needed {
        dst.resize(needed, 0.0);
    }
    let mut phase = 0.0_f32;
    let dphase = freq_hz / SR_F;
    for i in 0..n {
        let t = i as f32 / SR_F;
        let amp = env.at(t, duration) * gain;
        dst[off + i] += wave(phase) * amp;
        phase += dphase;
        if phase > 1.0 {
            phase -= 1.0;
        }
    }
}

/// 为打击/扫频做的"频率随时间变化"叠加。`freq_at(t)` 返回当前 Hz。
pub fn add_swept(
    dst: &mut Vec<f32>,
    offset_sec: f32,
    duration: f32,
    freq_at: impl Fn(f32) -> f32,
    wave: impl Fn(f32) -> f32,
    env: Adsr,
    gain: f32,
) {
    let n = samples_for(duration);
    let off = samples_for(offset_sec);
    let needed = off + n;
    if dst.len() < needed {
        dst.resize(needed, 0.0);
    }
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR_F;
        let f = freq_at(t).max(1.0);
        let dphase = f / SR_F;
        let amp = env.at(t, duration) * gain;
        dst[off + i] += wave(phase) * amp;
        phase += dphase;
        if phase > 1.0 {
            phase -= 1.0;
        }
    }
}
