//! 程序化合成的音效库。所有函数返回内存 WAV 字节流。

#![allow(clippy::needless_range_loop)] // DSP 循环里 `i` 同时是样本索引和时间，按索引读最清楚。

use super::synth::{
    add_note, add_swept, encode_wav, midi_hz, sine, square, triangle, Adsr, NoiseRng, OnePoleLp,
    SR_F,
};

// —— 主武器射击：短促"啾"声，下行扫频 + 微噪 ————————————————————

pub fn shoot() -> Vec<u8> {
    let dur = 0.10;
    let n = (dur * SR_F) as usize;
    let mut buf = vec![0.0_f32; n];

    // 下行方波扫频 1800 -> 600 Hz
    add_swept(
        &mut buf,
        0.0,
        dur,
        |t| 1800.0 * (1.0 - t / dur).max(0.0).powf(0.6).max(0.0).mul_add(0.66, 0.34),
        |p| square(p, 0.35),
        Adsr::pluck(0.06),
        0.55,
    );
    // 噪声底，做点空气感
    let mut rng = NoiseRng::new(0xC0FFEE);
    let mut lp = OnePoleLp::new(2400.0);
    for i in 0..n {
        let t = i as f32 / SR_F;
        let env = (1.0 - t / dur).clamp(0.0, 1.0).powf(2.0);
        let s = lp.process(rng.next()) * env * 0.18;
        buf[i] += s;
    }

    encode_wav(&buf, 0.85)
}

// —— 击中敌人：金属"叮"——————————————————————————————————————————

pub fn hit() -> Vec<u8> {
    let dur = 0.06;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    add_swept(
        &mut buf,
        0.0,
        dur,
        |t| 3200.0 - t * 8000.0,
        triangle,
        Adsr::percussive(0.04),
        0.55,
    );
    encode_wav(&buf, 0.9)
}

// —— 击杀爬升音：combo 越高音越亮 ————————————————————————————————

/// `step` 0..n-1，决定半音数。越大越高。
pub fn kill_at(step: u32) -> Vec<u8> {
    let dur = 0.18;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    // 五声音阶往上：C E G A C E G A
    let scale = [60.0, 64.0, 67.0, 69.0, 72.0, 76.0, 79.0, 81.0];
    let note = scale[(step as usize).min(scale.len() - 1)];
    let f1 = midi_hz(note);
    let f2 = midi_hz(note + 7.0); // 加一个五度泛音

    add_note(
        &mut buf,
        0.0,
        dur,
        f1,
        triangle,
        Adsr {
            attack: 0.002,
            decay: 0.06,
            sustain: 0.4,
            release: dur * 0.7,
        },
        0.5,
    );
    add_note(
        &mut buf,
        0.0,
        dur,
        f2,
        sine,
        Adsr::pluck(dur * 0.6),
        0.25,
    );
    encode_wav(&buf, 0.9)
}

// —— 小爆炸：白噪 + 低频 boom，下行截止 ————————————————————————————

pub fn explode_small() -> Vec<u8> {
    let dur = 0.32;
    let n = (dur * SR_F) as usize;
    let mut buf = vec![0.0_f32; n];

    // 噪声主体，截止从 4kHz 滑到 200Hz（每帧重建一极低通：单次构造代价极小）
    let mut rng = NoiseRng::new(0xDEADBEEF);
    for i in 0..n {
        let t = i as f32 / SR_F;
        let cutoff = 4000.0 * (1.0 - t / dur).max(0.0).powf(1.6) + 200.0;
        let mut lp = OnePoleLp::new(cutoff);
        let env = (1.0 - t / dur).clamp(0.0, 1.0).powf(0.7);
        let s = lp.process(rng.next()) * env;
        buf[i] += s * 0.85;
    }
    // 低频 boom
    add_swept(
        &mut buf,
        0.0,
        0.18,
        |t| 90.0 - t * 200.0,
        sine,
        Adsr::percussive(0.16),
        0.7,
    );
    encode_wav(&buf, 0.95)
}

// —— 大爆炸：更长更厚 ——————————————————————————————————————

pub fn explode_big() -> Vec<u8> {
    let dur = 0.85;
    let n = (dur * SR_F) as usize;
    let mut buf = vec![0.0_f32; n];

    let mut rng = NoiseRng::new(0xFEED_F00D);
    for i in 0..n {
        let t = i as f32 / SR_F;
        let cutoff = 5500.0 * (1.0 - t / dur).max(0.0).powf(1.4) + 90.0;
        let mut lp = OnePoleLp::new(cutoff);
        let env = (1.0 - t / dur).clamp(0.0, 1.0).powf(0.55);
        let s = lp.process(rng.next()) * env;
        buf[i] += s * 0.9;
    }
    // 双层低频 boom
    add_swept(
        &mut buf,
        0.0,
        0.5,
        |t| 70.0 - t * 90.0,
        sine,
        Adsr::percussive(0.45),
        0.95,
    );
    add_swept(
        &mut buf,
        0.02,
        0.35,
        |t| 130.0 - t * 200.0,
        triangle,
        Adsr::percussive(0.30),
        0.5,
    );
    encode_wav(&buf, 0.92)
}

// —— 玩家受伤：短促"嗡"下行 ——————————————————————————————

pub fn hurt() -> Vec<u8> {
    let dur = 0.35;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    add_swept(
        &mut buf,
        0.0,
        dur,
        |t| 320.0 - t * 600.0,
        |p| square(p, 0.5),
        Adsr {
            attack: 0.005,
            decay: 0.08,
            sustain: 0.5,
            release: 0.20,
        },
        0.6,
    );
    add_swept(
        &mut buf,
        0.0,
        dur,
        |t| 161.0 - t * 280.0,
        triangle,
        Adsr::percussive(0.30),
        0.4,
    );
    encode_wav(&buf, 0.9)
}

// —— 升级：上行琶音 ——————————————————————————————————————————

pub fn levelup() -> Vec<u8> {
    let dur = 0.55;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    let notes = [60.0, 64.0, 67.0, 72.0]; // C E G C
    for (i, &n) in notes.iter().enumerate() {
        let t0 = i as f32 * 0.07;
        add_note(
            &mut buf,
            t0,
            0.30,
            midi_hz(n),
            triangle,
            Adsr {
                attack: 0.002,
                decay: 0.12,
                sustain: 0.4,
                release: 0.18,
            },
            0.45,
        );
        add_note(
            &mut buf,
            t0,
            0.30,
            midi_hz(n + 12.0),
            sine,
            Adsr::pluck(0.20),
            0.18,
        );
    }
    encode_wav(&buf, 0.85)
}

// —— 拾取/Buff：上行扫频 + 闪光 —————————————————————————————

pub fn powerup() -> Vec<u8> {
    let dur = 0.30;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    add_swept(
        &mut buf,
        0.0,
        dur,
        |t| 600.0 + t * 2400.0,
        triangle,
        Adsr {
            attack: 0.002,
            decay: 0.05,
            sustain: 0.5,
            release: 0.20,
        },
        0.45,
    );
    // 高音闪光
    add_note(
        &mut buf,
        0.05,
        0.18,
        midi_hz(84.0),
        sine,
        Adsr::pluck(0.15),
        0.30,
    );
    encode_wav(&buf, 0.9)
}

// —— Super：大爆炸 + 低频冲击 ————————————————————————————————

pub fn super_bomb() -> Vec<u8> {
    let dur = 1.10;
    let n = (dur * SR_F) as usize;
    let mut buf = vec![0.0_f32; n];

    let mut rng = NoiseRng::new(0xBADD_CAFE);
    for i in 0..n {
        let t = i as f32 / SR_F;
        let cutoff = 7000.0 * (1.0 - t / dur).max(0.0).powf(1.0) + 100.0;
        let mut lp = OnePoleLp::new(cutoff);
        let env = (1.0 - t / dur).clamp(0.0, 1.0).powf(0.5);
        buf[i] += lp.process(rng.next()) * env * 0.85;
    }
    // 上行高频"充能"前奏
    add_swept(
        &mut buf,
        0.0,
        0.12,
        |t| 400.0 + t * 6000.0,
        triangle,
        Adsr::pluck(0.10),
        0.45,
    );
    // 低频冲击
    add_swept(
        &mut buf,
        0.10,
        0.7,
        |t| 60.0 - t * 60.0,
        sine,
        Adsr::percussive(0.6),
        1.05,
    );
    encode_wav(&buf, 0.95)
}

// —— UI 点击 —————————————————————————————————————————————

pub fn click() -> Vec<u8> {
    let dur = 0.06;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    add_note(
        &mut buf,
        0.0,
        dur,
        midi_hz(82.0),
        triangle,
        Adsr::percussive(0.05),
        0.5,
    );
    encode_wav(&buf, 0.85)
}

// —— Game Over：下行小三和弦 —————————————————————————————————

pub fn gameover() -> Vec<u8> {
    let dur = 1.6;
    let mut buf = vec![0.0_f32; (dur * SR_F) as usize];
    let notes = [
        (0.00, 64.0), // E
        (0.18, 60.0), // C
        (0.36, 57.0), // A
        (0.54, 53.0), // F
    ];
    for (t0, n) in notes {
        add_note(
            &mut buf,
            t0,
            1.0,
            midi_hz(n),
            triangle,
            Adsr {
                attack: 0.005,
                decay: 0.20,
                sustain: 0.55,
                release: 0.7,
            },
            0.30,
        );
        add_note(
            &mut buf,
            t0,
            0.9,
            midi_hz(n - 12.0),
            |p| square(p, 0.45),
            Adsr {
                attack: 0.005,
                decay: 0.20,
                sustain: 0.6,
                release: 0.6,
            },
            0.18,
        );
    }
    encode_wav(&buf, 0.85)
}

// —— Boss 入场咆哮：警示性低频 + 噪声 ——————————————————

pub fn boss_intro() -> Vec<u8> {
    let dur = 1.2;
    let n = (dur * SR_F) as usize;
    let mut buf = vec![0.0_f32; n];

    // 三声警报短音
    for k in 0..3 {
        let t0 = k as f32 * 0.18;
        add_swept(
            &mut buf,
            t0,
            0.14,
            |t| 880.0 - t * 600.0,
            |p| square(p, 0.5),
            Adsr::percussive(0.13),
            0.45,
        );
    }
    // 低频咆哮
    add_swept(
        &mut buf,
        0.55,
        0.65,
        |t| 80.0 - t * 30.0,
        |p| square(p, 0.5),
        Adsr {
            attack: 0.05,
            decay: 0.10,
            sustain: 0.85,
            release: 0.30,
        },
        0.55,
    );
    // 噪声风声
    let mut rng = NoiseRng::new(0xB055);
    let mut lp = OnePoleLp::new(800.0);
    for i in 0..n {
        let t = i as f32 / SR_F;
        let env = ((t - 0.4) / 0.8).clamp(0.0, 1.0).powf(1.5)
            * (1.0 - (t / dur).clamp(0.0, 1.0)).powf(0.6);
        buf[i] += lp.process(rng.next()) * env * 0.30;
    }
    encode_wav(&buf, 0.92)
}
