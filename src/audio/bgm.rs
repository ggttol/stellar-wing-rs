//! 程序化合成的背景音乐。每首是一个无缝循环 buffer，靠 macroquad 的 `looped=true` 反复播放。

#![allow(clippy::needless_range_loop)]

use super::synth::{
    add_note, add_swept, encode_wav, midi_hz, samples_for, sine, square, triangle, Adsr, NoiseRng,
    OnePoleLp, SR_F,
};

// —— 鼓机辅助 ——————————————————————————————————————————

fn kick(buf: &mut Vec<f32>, t0: f32, gain: f32) {
    add_swept(
        buf,
        t0,
        0.18,
        |t| 110.0 - t * 380.0,
        sine,
        Adsr::percussive(0.16),
        gain,
    );
    // 起音"咔"
    add_swept(
        buf,
        t0,
        0.020,
        |_| 1500.0,
        triangle,
        Adsr::percussive(0.020),
        gain * 0.5,
    );
}

fn snare(buf: &mut Vec<f32>, t0: f32, gain: f32) {
    let dur = 0.18;
    let n = samples_for(dur);
    let off = samples_for(t0);
    if buf.len() < off + n {
        buf.resize(off + n, 0.0);
    }
    let mut rng = NoiseRng::new(0xA55_AA55);
    let mut lp = OnePoleLp::new(3500.0);
    for i in 0..n {
        let t = i as f32 / SR_F;
        let env = (1.0 - t / dur).clamp(0.0, 1.0).powf(1.4);
        buf[off + i] += lp.process(rng.next()) * env * gain;
    }
    add_swept(
        buf,
        t0,
        0.05,
        |t| 220.0 - t * 200.0,
        triangle,
        Adsr::percussive(0.05),
        gain * 0.4,
    );
}

fn hat(buf: &mut Vec<f32>, t0: f32, gain: f32, open: bool) {
    let dur = if open { 0.10 } else { 0.04 };
    let n = samples_for(dur);
    let off = samples_for(t0);
    if buf.len() < off + n {
        buf.resize(off + n, 0.0);
    }
    let mut rng = NoiseRng::new(0xCAFE ^ (off as u32 | 1));
    for i in 0..n {
        let t = i as f32 / SR_F;
        let env = (1.0 - t / dur).clamp(0.0, 1.0).powf(2.0);
        // 高通：减去低频近似（把噪声 - 低通）
        let mut lp = OnePoleLp::new(6000.0);
        let s = rng.next();
        let hp = s - lp.process(s);
        buf[off + i] += hp * env * gain * 0.6;
    }
}

// —— 在缓冲尾部做一段淡出，避免无缝循环边界处的咔声 ——

fn fade_edges(buf: &mut [f32]) {
    let n = buf.len();
    let fade = (SR_F * 0.04) as usize; // 40ms
    for i in 0..fade.min(n) {
        let g = i as f32 / fade as f32;
        buf[i] *= g;
        buf[n - 1 - i] *= g;
    }
}

// —— 菜单 BGM：缓慢的四度环境 ————————————————————————

pub fn menu() -> Vec<u8> {
    // 4 bars at 80 BPM = 4 * 4 * 0.75s = 12s
    let bpm = 80.0;
    let beat = 60.0 / bpm;
    let total = beat * 16.0;
    let mut buf = vec![0.0_f32; samples_for(total)];

    // 大三和弦缓慢上行序列：Am  F  C  G
    let progression = [
        [57.0, 60.0, 64.0], // A C E -> Am
        [53.0, 57.0, 60.0], // F A C
        [48.0, 52.0, 55.0], // C E G
        [55.0, 59.0, 62.0], // G B D
    ];
    for (bar, chord) in progression.iter().enumerate() {
        let t0 = bar as f32 * beat * 4.0;
        for &note in chord {
            // 长 pad，sine + triangle 微合成
            add_note(
                &mut buf,
                t0,
                beat * 4.0,
                midi_hz(note),
                sine,
                Adsr::pad(0.4, 0.6),
                0.10,
            );
            add_note(
                &mut buf,
                t0,
                beat * 4.0,
                midi_hz(note + 12.0),
                triangle,
                Adsr::pad(0.5, 0.6),
                0.05,
            );
        }
        // 慢 arp：四个八分上行
        for step in 0..8 {
            let t = t0 + step as f32 * beat * 0.5;
            let n = chord[step % 3] + 12.0;
            add_note(
                &mut buf,
                t,
                beat * 0.45,
                midi_hz(n),
                triangle,
                Adsr::pluck(beat * 0.4),
                0.08,
            );
        }
    }
    fade_edges(&mut buf);
    encode_wav(&buf, 0.85)
}

// —— 战斗 BGM：推进式 8-bit ——————————————————————

pub fn play() -> Vec<u8> {
    // 8 bars at 132 BPM = 8 * 4 * 0.4545 = 14.5s
    let bpm = 132.0;
    let beat = 60.0 / bpm;
    let total = beat * 32.0;
    let mut buf = vec![0.0_f32; samples_for(total)];

    // 进行：Am  F  C  G   Am  F  G  E
    let bass_root = [45.0, 41.0, 36.0, 43.0, 45.0, 41.0, 43.0, 40.0];
    // 每小节的和弦音（用于 lead 取材）
    let chord_tones: [[f32; 3]; 8] = [
        [57.0, 60.0, 64.0],
        [53.0, 57.0, 60.0],
        [48.0, 52.0, 55.0],
        [55.0, 59.0, 62.0],
        [57.0, 60.0, 64.0],
        [53.0, 57.0, 60.0],
        [55.0, 59.0, 62.0],
        [52.0, 56.0, 59.0],
    ];

    for bar in 0..8 {
        let bar_t0 = bar as f32 * beat * 4.0;

        // 鼓：每拍 kick on 1+3, snare on 2+4, hat 八分
        for b in 0..4 {
            let t = bar_t0 + b as f32 * beat;
            if b == 0 || b == 2 {
                kick(&mut buf, t, 0.85);
            }
            if b == 1 || b == 3 {
                snare(&mut buf, t, 0.5);
            }
        }
        for s in 0..8 {
            let t = bar_t0 + s as f32 * beat * 0.5;
            hat(&mut buf, t, 0.18, s % 2 == 1);
        }

        // 贝斯：八分音符，在根音和 root+7 上交替
        let root = bass_root[bar];
        for s in 0..8 {
            let t = bar_t0 + s as f32 * beat * 0.5;
            let n = if s % 4 == 2 { root + 7.0 } else { root };
            add_note(
                &mut buf,
                t,
                beat * 0.45,
                midi_hz(n),
                |p| square(p, 0.5),
                Adsr {
                    attack: 0.005,
                    decay: 0.10,
                    sustain: 0.5,
                    release: 0.10,
                },
                0.30,
            );
        }

        // 主旋律：每小节用和弦三音 + 八度的简单 motif
        let lead = chord_tones[bar];
        let pattern = [0, 2, 1, 2, 0, 1, 2, 1];
        for (i, &p) in pattern.iter().enumerate() {
            let t = bar_t0 + i as f32 * beat * 0.5;
            let n = lead[p] + 12.0;
            add_note(
                &mut buf,
                t,
                beat * 0.42,
                midi_hz(n),
                triangle,
                Adsr {
                    attack: 0.003,
                    decay: 0.06,
                    sustain: 0.55,
                    release: 0.10,
                },
                0.20,
            );
        }
    }
    fade_edges(&mut buf);
    encode_wav(&buf, 0.85)
}

// —— Boss BGM：低音急促，半音冲突制造紧张 ———————————

pub fn boss() -> Vec<u8> {
    // 8 bars at 156 BPM = 8 * 4 * 0.3846 = 12.3s
    let bpm = 156.0;
    let beat = 60.0 / bpm;
    let total = beat * 32.0;
    let mut buf = vec![0.0_f32; samples_for(total)];

    // 进行：Dm  Bb  Gm  A7   Dm  F  Gm  A7
    let bass_root = [38.0, 34.0, 31.0, 33.0, 38.0, 41.0, 31.0, 33.0];
    let chord_tones: [[f32; 4]; 8] = [
        [50.0, 53.0, 57.0, 60.0], // Dm + 7th
        [46.0, 50.0, 53.0, 57.0], // Bb
        [43.0, 46.0, 50.0, 53.0], // Gm
        [45.0, 49.0, 52.0, 55.0], // A7
        [50.0, 53.0, 57.0, 60.0],
        [53.0, 57.0, 60.0, 65.0], // F
        [43.0, 46.0, 50.0, 53.0],
        [45.0, 49.0, 52.0, 55.0],
    ];

    for bar in 0..8 {
        let bar_t0 = bar as f32 * beat * 4.0;

        // 鼓：双 kick 起拍 + snare 反拍 + 高密度 hat
        for b in 0..4 {
            let t = bar_t0 + b as f32 * beat;
            kick(&mut buf, t, 0.95);
            if b % 2 == 1 {
                snare(&mut buf, t, 0.65);
            }
        }
        for s in 0..16 {
            let t = bar_t0 + s as f32 * beat * 0.25;
            hat(&mut buf, t, 0.15, false);
        }

        // 贝斯：16 分跑动（root, root, fifth, root）
        let root = bass_root[bar];
        for s in 0..16 {
            let t = bar_t0 + s as f32 * beat * 0.25;
            let n = match s % 4 {
                2 => root + 12.0,
                3 => root + 7.0,
                _ => root,
            };
            add_note(
                &mut buf,
                t,
                beat * 0.22,
                midi_hz(n - 12.0),
                |p| square(p, 0.4),
                Adsr {
                    attack: 0.003,
                    decay: 0.05,
                    sustain: 0.4,
                    release: 0.05,
                },
                0.36,
            );
        }

        // 高音 stab：每小节 3 次琶音上行
        let lead = chord_tones[bar];
        let stabs = [0.0, 1.5, 2.5];
        for &b in &stabs {
            for k in 0..4 {
                let t = bar_t0 + b * beat + k as f32 * beat * 0.08;
                let n = lead[k] + 12.0;
                add_note(
                    &mut buf,
                    t,
                    beat * 0.30,
                    midi_hz(n),
                    |p| square(p, 0.3),
                    Adsr {
                        attack: 0.002,
                        decay: 0.05,
                        sustain: 0.4,
                        release: 0.10,
                    },
                    0.18,
                );
            }
        }

        // 持续低频"嗡"营造紧张
        add_note(
            &mut buf,
            bar_t0,
            beat * 4.0,
            midi_hz(bass_root[bar] - 12.0),
            sine,
            Adsr::pad(0.08, 0.40),
            0.18,
        );
    }
    fade_edges(&mut buf);
    encode_wav(&buf, 0.85)
}
