//! Modul DSP (Digital Signal Processing) — versi HiFi High Dynamic
//! Semua filter, envelope, reverb, dan utilitas untuk sintesis audio kualitas tinggi.

use std::f32::consts::PI;

pub fn ms_to_samples(ms: u64, sample_rate: u32) -> usize {
    ((ms as f64 / 1000.0) * sample_rate as f64).round() as usize
}

pub fn ms_to_samples_f(ms: f32, sample_rate: u32) -> usize {
    ((ms as f64 / 1000.0) * sample_rate as f64).round() as usize
}

/// PRNG xorshift32 deterministik
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng { state: if seed == 0 { 0x9E37_79B9 } else { seed } }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Filter low-pass satu kutub dengan batasan koefisien aman
#[derive(Clone, Copy)]
pub struct OnePoleLowPass {
    alpha: f32,
    y_prev: f32,
}

impl OnePoleLowPass {
    pub fn new(cutoff_hz: f32, sample_rate: u32) -> Self {
        let dt = 1.0 / sample_rate as f32;
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(1.0));
        let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
        OnePoleLowPass { alpha, y_prev: 0.0 }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let out = self.y_prev + self.alpha * (x - self.y_prev);
        // Anti-denormal protection
        self.y_prev = if out.abs() < 1e-25 { 0.0 } else { out };
        self.y_prev
    }
}

/// Filter high-pass satu kutub
#[derive(Clone, Copy)]
pub struct OnePoleHighPass {
    alpha: f32,
    y_prev: f32,
    x_prev: f32,
}

impl OnePoleHighPass {
    pub fn new(cutoff_hz: f32, sample_rate: u32) -> Self {
        let dt = 1.0 / sample_rate as f32;
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(1.0));
        let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
        OnePoleHighPass { alpha, y_prev: 0.0, x_prev: 0.0 }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.alpha * (self.y_prev + x - self.x_prev);
        self.x_prev = x;
        // Anti-denormal protection
        self.y_prev = if y.abs() < 1e-25 { 0.0 } else { y };
        self.y_prev
    }
}

/// Filter band-pass biquad resonan
#[derive(Clone, Copy)]
pub struct BandPass {
    b0: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl BandPass {
    pub fn new(center_hz: f32, q: f32, sample_rate: u32) -> Self {
        let center_hz = center_hz.clamp(20.0, sample_rate as f32 * 0.45);
        let w0 = 2.0 * PI * center_hz / sample_rate as f32;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();
        let a0 = 1.0 + alpha;
        
        BandPass {
            b0: alpha / a0,
            b2: -alpha / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        
        self.x2 = self.x1; self.x1 = x;
        // Bersihkan denormal floats untuk menjaga kestabilan feedback loop
        self.y2 = self.y1; self.y1 = if y.abs() < 1e-25 { 0.0 } else { y };
        self.y1
    }
}

/// Filter low-pass biquad (Butterworth 2nd order)
#[derive(Clone, Copy)]
pub struct LowPassBiquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl LowPassBiquad {
    pub fn new(cutoff_hz: f32, q: f32, sample_rate: u32) -> Self {
        let cutoff_hz = cutoff_hz.clamp(10.0, sample_rate as f32 * 0.45);
        let w0 = 2.0 * PI * cutoff_hz / sample_rate as f32;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();
        let a0 = 1.0 + alpha;
        
        LowPassBiquad {
            b0: ((1.0 - cos_w0) / 2.0) / a0,
            b1: (1.0 - cos_w0) / a0,
            b2: ((1.0 - cos_w0) / 2.0) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0*x + self.b1*self.x1 + self.b2*self.x2 - self.a1*self.y1 - self.a2*self.y2;
        self.x2 = self.x1; self.x1 = x;
        self.y2 = self.y1; self.y1 = if y.abs() < 1e-25 { 0.0 } else { y };
        self.y1
    }
}

/// High-shelf filter — boost frekuensi tinggi (presence/air)
#[derive(Clone, Copy)]
pub struct HighShelf {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl HighShelf {
    pub fn new(freq_hz: f32, gain_db: f32, sample_rate: u32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq_hz / sample_rate as f32;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / 2.0 * ((a + 1.0/a) * (1.0/0.9 - 1.0) + 2.0).sqrt();
        
        let a0 = (a+1.0) - (a-1.0)*cos_w0 + 2.0*a.sqrt()*alpha;
        let b0 = a * ((a+1.0) + (a-1.0)*cos_w0 + 2.0*a.sqrt()*alpha);
        let b1 = -2.0 * a * ((a-1.0) + (a+1.0)*cos_w0);
        let b2 = a * ((a+1.0) + (a-1.0)*cos_w0 - 2.0*a.sqrt()*alpha);
        let a1 = 2.0 * ((a-1.0) - (a+1.0)*cos_w0);
        let a2 = (a+1.0) - (a-1.0)*cos_w0 - 2.0*a.sqrt()*alpha;
        
        HighShelf {
            b0: b0/a0, b1: b1/a0, b2: b2/a0,
            a1: a1/a0, a2: a2/a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0*x + self.b1*self.x1 + self.b2*self.x2 - self.a1*self.y1 - self.a2*self.y2;
        self.x2 = self.x1; self.x1 = x;
        self.y2 = self.y1; self.y1 = if y.abs() < 1e-25 { 0.0 } else { y };
        self.y1
    }
}

/// Amplop ADSR Presisi Tinggi (Bebas dari letupan transisi/Micro-clicks)
pub fn adsr_envelope(
    n_samples: usize,
    attack: usize,
    decay: usize,
    sustain_level: f32,
    release: usize,
) -> Vec<f32> {
    let attack = attack.min(n_samples);
    let decay = decay.min(n_samples.saturating_sub(attack));
    let release = release.min(n_samples.saturating_sub(attack + decay));
    let sustain_start = attack + decay;
    let release_start = n_samples.saturating_sub(release);
    
    let mut env = vec![0.0f32; n_samples];

    // Pra calculate explonential factor to land on correct target
    let att_factor = 1.0 - (-5.0f32).exp();
    let dec_factor = 1.0 - (-3.0f32).exp();
    let rel_factor = (-4.0f32).exp();

    for (i, slot) in env.iter_mut().enumerate() {
        *slot = if i < attack {
            let t = i as f32 / attack.max(1) as f32;
            (1.0 - (-5.0 * t).exp()) / att_factor
        } else if i < sustain_start {
            let t = (i - attack) as f32 / decay.max(1) as f32;
            1.0 + (sustain_level - 1.0) * (1.0 - (-3.0 * t).exp()) / dec_factor
        } else if i < release_start {
            sustain_level
        } else if release > 0 {
            let t = (i - release_start) as f32 / release as f32;
            // Diturunkan hingga benar-benar menyentuh angka 0.0 di akhir sampel
            sustain_level * ((-4.0 * t).exp() - rel_factor) / (1.0 - rel_factor)
        } else {
            0.0
        };
    }
    env
}

/// Soft clipper tanh murni
pub fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

/// Hard/soft clipper analog tabung vakum (Gunakan secara eksplisit untuk efek)
pub fn tube_saturate(x: f32, drive: f32) -> f32 {
    let driven = x * drive;
    if driven >= 0.0 {
        1.0 - (-driven).exp()
    } else {
        -1.0 + (driven).exp()
    }
}

/// Normalisasi HiFi Sejati Linier Berbasis Headroom 
pub fn normalize(buffer: &mut [f32]) {
    let peak = buffer.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
    if peak < 1e-6 { return; }
    
    let target = 0.95; // Standar headroom industri audio digital
    
    // FIX: Skala linier murni tanpa soft_clip paksaan. 
    // Menjaga keaslian sinyal dan ruang dinamika (transparansi 100%).
    if peak > target {
        let scale = target / peak;
        for s in buffer.iter_mut() {
            *s *= scale;
        }
    }
}

// ─── STEREO MIXING ────────────────────────────────────────────

/// Campurkan voice mono ke buffer stereo interleaved (L R L R ...)
/// `offset` dalam **sample frame** (bukan sample tunggal)
/// `pan`: 0.0 = full left, 1.0 = full right, 0.5 = center (constant power)
/// `width`: 0.0 = mono (kedua kanal identik), 1.0 = stereo penuh
/*pub fn mix_in_stereo(buffer: &mut [f32], offset_frame: usize, voice: &[f32], pan: f32, width: f32, gain: f32) {
    if voice.is_empty() { return; }
    let half = buffer.len() / 2;
    let max_frames = half.saturating_sub(offset_frame);
    let mix_len = voice.len().min(max_frames);

    if mix_len == 0 { return; }

    let pan = pan.clamp(0.0, 1.0);
    let width = width.clamp(0.0, 1.0);

    let left_gain = gain * (1.0 - pan).sqrt();
    let right_gain = gain * pan.sqrt();

    // Stereo widening: kita buat salinan kiri & kanan dari voice dengan perbedaan halus
    // menggunakan Haas delay dan filter low-pass berbeda.
    // Agar tetap efisien, proses dilakukan per sample.
    for i in 0..mix_len {
        let sample = voice[i];

        // Haas delay kecil untuk memberi lebar (max 1.5 ms)
        let haas_offset = (1.5 * width * 44.1) as usize; // 1.5ms pada 44.1kHz ~ 66 samples
        let idx_left = i;
        let idx_right = i.saturating_sub(haas_offset);
        let sample_right = if idx_right < voice.len() { voice[idx_right] } else { 0.0 };

        // Filter lowpass asimetris untuk meniru HRTF sederhana
        // Tidak disimpan state, cukup berbeda gain antara sample kiri & kanan
        let l_sample = sample;
        let r_sample = sample_right * (0.7 + 0.3 * width); // makin lebar makin berbeda

        let buf_idx = 2 * (offset_frame + i);
        if buf_idx + 1 < buffer.len() {
            buffer[buf_idx] += l_sample * left_gain;
            buffer[buf_idx + 1] += r_sample * right_gain;
        }
    }
}*/


pub fn mix_in_stereo(buffer: &mut [f32], offset_frame: usize, voice: &[f32], pan: f32, width: f32, gain: f32) {
    if voice.is_empty() { return; }
    let half = buffer.len() / 2;
    let max_frames = half.saturating_sub(offset_frame);
    let mix_len = voice.len().min(max_frames);
    if mix_len == 0 { return; }

    let width = width.clamp(0.0, 1.0);
    // Center pan: equal power untuk kiri dan kanan
    let center_gain = gain * 0.5f32.sqrt(); // ≈ 0.7071 * gain

    // Haas delay maksimum dalam sample (1.5 ms pada 44.1 kHz ≈ 66 sample)
    let haas_max = (1.5 * 44.1) as usize;
    // Semakin besar width, semakin besar delay difference
    let haas_offset = (haas_max as f32 * width) as usize;

    for i in 0..mix_len {
        let sample_center = voice[i];

        // Untuk kiri: gunakan sample saat ini
        let left = sample_center;

        // Untuk kanan: gunakan sample yang di-delay oleh haas_offset (meniru perbedaan waktu antar telinga)
        let right = if haas_offset > 0 {
            let idx_right = i.saturating_sub(haas_offset);
            if idx_right < voice.len() { voice[idx_right] } else { 0.0 }
        } else {
            sample_center // width 0 = mono
        };

        // Campurkan ke buffer stereo dengan gain center
        let buf_idx = 2 * (offset_frame + i);
        if buf_idx + 1 < buffer.len() {
            buffer[buf_idx] += left * center_gain;
            buffer[buf_idx + 1] += right * center_gain;
        }
    }
}

// ─── TRUE STEREO REVERB (Industrial Grade) ────────────────────


// ─────────────────────────────────────────────
// MONO MIXING (untuk internal synth seperti chord)
// ─────────────────────────────────────────────
pub fn mix_in(buffer: &mut [f32], offset: usize, voice: &[f32], gain: f32) {
    if offset >= buffer.len() { return; }
    let mix_len = voice.len().min(buffer.len() - offset);
    for i in 0..mix_len {
        buffer[offset + i] += voice[i] * gain;
    }
}

// ─────────────────────────────────────────────
// STEREO MIXING (untuk output akhir)
// ─────────────────────────────────────────────

/// Campur voice **mono** ke buffer stereo interleaved.
/// `offset_frame` dalam sample frame (bukan sample index).
/// `_pan` diabaikan (center), `width` mengontrol penyebaran stereo (0 = mono, 1 = lebar maks).
pub fn mix_in_stereo_mono(buffer: &mut [f32], offset_frame: usize, voice: &[f32], _pan: f32, width: f32, gain: f32) {
    if voice.is_empty() { return; }
    let half = buffer.len() / 2;
    let max_frames = half.saturating_sub(offset_frame);
    let mix_len = voice.len().min(max_frames);
    if mix_len == 0 { return; }

    let width = width.clamp(0.0, 1.0);
    let center_gain = gain * 0.5f32.sqrt(); // equal power center
    let haas_max = (1.5 * 44.1) as usize;
    let haas_offset = (haas_max as f32 * width) as usize;

    for i in 0..mix_len {
        let sample_center = voice[i];
        let left = sample_center;
        let right = if haas_offset > 0 {
            let idx = i.saturating_sub(haas_offset);
            if idx < voice.len() { voice[idx] } else { 0.0 }
        } else {
            sample_center
        };

        let buf_idx = 2 * (offset_frame + i);
        if buf_idx + 1 < buffer.len() {
            buffer[buf_idx] += left * center_gain;
            buffer[buf_idx + 1] += right * center_gain;
        }
    }
}

/// Campur voice **stereo interleaved** ke buffer stereo interleaved.
/// Width diabaikan (dianggap sudah tertanam di sampel). Gain diterapkan langsung.
pub fn mix_in_stereo_stereo(buffer: &mut [f32], offset_frame: usize, voice: &[f32], gain: f32) {
    if voice.len() < 2 { return; }
    let voice_frames = voice.len() / 2;
    let half = buffer.len() / 2;
    let max_frames = half.saturating_sub(offset_frame);
    let mix_frames = voice_frames.min(max_frames);
    for i in 0..mix_frames {
        let buf_idx = 2 * (offset_frame + i);
        let v_idx = 2 * i;
        buffer[buf_idx] += voice[v_idx] * gain;
        buffer[buf_idx + 1] += voice[v_idx + 1] * gain;
    }
}

pub fn apply_true_stereo_reverb(buffer: &mut [f32], sample_rate: u32) {
    if buffer.len() < 2 { return; }
    let n_frames = buffer.len() / 2;

    let mut left = vec![0.0f32; n_frames];
    let mut right = vec![0.0f32; n_frames];
    for i in 0..n_frames {
        left[i] = buffer[2 * i];
        right[i] = buffer[2 * i + 1];
    }

    let num_lines = 8;
    let delays_ms: [f32; 8] = [29.7, 37.1, 41.3, 47.9, 53.1, 59.3, 67.1, 71.9];
    let decays:    [f32; 8] = [0.80, 0.79, 0.78, 0.77, 0.76, 0.75, 0.74, 0.73];
    let mut comb_l = vec![];
    let mut comb_r = vec![];
    let mut idx_l = vec![0usize; num_lines];
    let mut idx_r = vec![0usize; num_lines];
    for &d_ms in &delays_ms {
        let d = ((d_ms / 1000.0) * sample_rate as f32).round().max(1.0) as usize;
        comb_l.push(vec![0.0f32; d]);
        comb_r.push(vec![0.0f32; d]);
    }
    let comb_norm = 1.0 / (num_lines as f32).sqrt();

    let mut wet_l = vec![0.0f32; n_frames];
    let mut wet_r = vec![0.0f32; n_frames];

    for i in 0..n_frames {
        let mut out_l = 0.0;
        let mut out_r = 0.0;
        for line in 0..num_lines {
            let mut d_l = comb_l[line][idx_l[line]];
            let mut d_r = comb_r[line][idx_r[line]];

            // Anti‑denormal: bersihkan jika sangat kecil
            if d_l.abs() < 1e-25 { d_l = 0.0; }
            if d_r.abs() < 1e-25 { d_r = 0.0; }

            out_l += d_l * comb_norm;
            out_r += d_r * comb_norm;

            // Cross‑feed aman (dikurangi sedikit)
            let in_l = left[i] + d_l * decays[line] + d_r * decays[line] * 0.15;
            let in_r = right[i] + d_r * decays[line] + d_l * decays[line] * 0.15;

            // Clamp ringan untuk mencegah ledakan tak terduga
            comb_l[line][idx_l[line]] = in_l.clamp(-2.0, 2.0);
            comb_r[line][idx_r[line]] = in_r.clamp(-2.0, 2.0);

            idx_l[line] = (idx_l[line] + 1) % comb_l[line].len();
            idx_r[line] = (idx_r[line] + 1) % comb_r[line].len();
        }
        wet_l[i] = out_l;
        wet_r[i] = out_r;
    }

    // All‑pass diffuser (diperbaiki, setiap stage pakai gain yang benar)
    let ap_delays_ms = [10.0, 13.7, 15.2, 17.8];
    let ap_gains = [0.65, 0.70, 0.60, 0.72];
    for stage in 0..4 {
        let d = ((ap_delays_ms[stage] / 1000.0) * sample_rate as f32).round().max(1.0) as usize;
        let g = ap_gains[stage];    // <-- kini setiap stage punya g sendiri
        let mut buf_l = vec![0.0f32; d];
        let mut buf_r = vec![0.0f32; d];
        let mut i_l = 0usize;
        let mut i_r = 0usize;
        for f in 0..n_frames {
            let bl = buf_l[i_l];
            let nl = wet_l[f] + bl * g;
            buf_l[i_l] = nl;
            wet_l[f] = bl - g * nl;
            i_l = (i_l + 1) % d;

            let br = buf_r[i_r];
            let nr = wet_r[f] + br * g;
            buf_r[i_r] = nr;
            wet_r[f] = br - g * nr;
            i_r = (i_r + 1) % d;
        }
    }

    let pre_l = ms_to_samples(7, sample_rate);
    let pre_r = ms_to_samples(11, sample_rate);
    let wet_mix = 0.16;
    for i in 0..n_frames {
        let wl = if i >= pre_l { wet_l[i - pre_l] } else { 0.0 };
        let wr = if i >= pre_r { wet_r[i - pre_r] } else { 0.0 };
        buffer[2 * i] = left[i] + wl * wet_mix;
        buffer[2 * i + 1] = right[i] + wr * wet_mix;
    }

    // Terakhir, sapu denormal dari seluruh buffer
    for s in buffer.iter_mut() {
        if s.abs() < 1e-30 { *s = 0.0; }
    }
}

/// Noise gate yang memotong sampel di bawah threshold absolut.
/// Digunakan untuk membersihkan denormal dan reverb tail yang tidak perlu.
pub fn noise_gate(buffer: &mut [f32], threshold: f32) {
    for s in buffer.iter_mut() {
        if s.abs() < threshold {
            *s = 0.0;
        }
    }
}

pub fn clean_denormal(samples: &mut [f32]) {
    for s in samples.iter_mut() {
        if s.abs() < 1e-30 {
            *s = 0.0;
        }
    }
}


pub fn make_stereo_voice(mono: &[f32], width: f32, sample_rate: u32) -> Vec<f32> {
    let len = mono.len();
    let mut stereo = vec![0.0f32; len * 2];
    let width = width.clamp(0.0, 1.0);

    // Haas delay maksimum ~1.5 ms pada sample rate
    let haas_max_samples = (1.5 * sample_rate as f32 / 1000.0) as usize;
    let haas_offset = (haas_max_samples as f32 * width) as usize;

    // Low‑pass filter biquad untuk jalur delayed (cutoff 8 kHz, Q 0.5)
    let mut lp = LowPassBiquad::new(8000.0, 0.5, sample_rate);

    // Gain agar energi tetap
    let gain = 0.5f32.sqrt();

    for i in 0..len {
        let center = mono[i];

        // Kiri: sinyal asli
        let left = center;

        // Kanan: sample yang di‑delay (dari masa lalu)
        let right = if haas_offset > 0 && i >= haas_offset {
            mono[i - haas_offset]
        } else {
            center   // saat offset belum cukup, gunakan sample sekarang
        };

        // Filter low‑pass pada kanal kanan
        let right_filtered = lp.process(right);

        stereo[i * 2] = left * gain;
        stereo[i * 2 + 1] = right_filtered * gain;
    }

    stereo
}

/// Gabungkan dua buffer mono (left & right) menjadi interleaved stereo.
pub fn interleave_stereo(left: Vec<f32>, right: Vec<f32>) -> Vec<f32> {
    let n = left.len().min(right.len());
    let mut out = Vec::with_capacity(n * 2);
    for i in 0..n {
        out.push(left[i]);
        out.push(right[i]);
    }
    out
}

fn mono_to_fake_stereo(mono: Vec<f32>) -> Vec<f32> {
    let n = mono.len();
    let mut out = Vec::with_capacity(n * 2);
    for s in mono {
        out.push(s);
        out.push(s);
    }
    out
}

/// Atur lebar stereo buffer interleaved.
/// width: 0.0 = mono (center), 1.0 = stereo penuh (asli), >1.0 = hiper-lebar (hindari).
pub fn adjust_stereo_width(buffer: &mut [f32], width: f32) {
    if buffer.len() < 2 { return; }
    let width = width.clamp(0.0, 1.0);
    let side_gain = width;
    let mid_gain = (1.0 - width * 0.5).sqrt();

    // Hanya proses pasangan penuh (genap) – hindari sisa elemen ganjil
    let limit = buffer.len() & !1; // bulatkan ke bawah ke kelipatan 2
    for i in (0..limit).step_by(2) {
        let left = buffer[i];
        let right = buffer[i + 1]; // aman karena i+1 < limit ≤ buffer.len()
        let mid = (left + right) * 0.5;
        let side = (left - right) * 0.5;
        buffer[i]     = mid * mid_gain + side * side_gain;
        buffer[i + 1] = mid * mid_gain - side * side_gain;
    }
}



