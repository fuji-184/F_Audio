

use crate::dsp::{
    adsr_envelope, ms_to_samples, BandPass, LowPassBiquad,
    OnePoleHighPass, OnePoleLowPass, Rng, HighShelf, interleave_stereo
};
use crate::notes::{chord_freqs, single_note_freq};
use std::f32::consts::PI;

// ====================================================================
// DRUM
// ====================================================================

pub fn synth_drum(token: &str, sample_rate: u32) -> Vec<f32> {
    match token {
        "bass" | "kick" => synth_kick(sample_rate),
        "snare"         => synth_snare(sample_rate),
        "hihat" | "hat" => synth_hihat(sample_rate),
        "hihat_open" | "hat_open" => synth_hihat_open(sample_rate),
        "tom" | "tom_mid" => synth_tom(sample_rate),
        "tom_low"       => synth_tom_low(sample_rate),
        "crash"         => synth_crash(sample_rate),
        "ride"          => synth_ride(sample_rate),
        _               => synth_kick(sample_rate),
    }
}



// ─── Waveshaping membran ──────────────────────────────────
#[inline]
fn membrane(x: f32) -> f32 {
    x - 0.14 * x * x * x
}
// ─── KICK (bulat, nendang, sub kuat) ─────────────────────
fn synth_kick(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(800, sr);
    let mut out = vec![0.0f32; n];

    // Pitch envelope dalam: overshoot tinggi lalu jatuh ke sub
    let f_start  = 200.0;
    let f_mid    = 80.0;
    let f_end    = 38.0;
    let tau_fast = 0.002;
    let tau_slow = 0.050;

    let mut p1 = 0.0f32; let mut p2 = 0.0f32; let mut p3 = 0.0f32;
    let mut p4 = 0.0f32; let mut p5 = 0.0f32;
    let mut sub_p = 0.0f32;

    let mut rng = Rng::new(0xB00);

    let mut sub_lp     = LowPassBiquad::new(60.0,  0.6, sr);   // lebih rendah
    let mut body_lp    = LowPassBiquad::new(380.0, 0.75, sr);  // lebih rendah & bulat
    let mut beater_bp  = BandPass::new(3200.0, 0.5, sr);
    let mut beater_lp  = LowPassBiquad::new(4800.0, 0.7, sr);
    let mut thud_lp    = LowPassBiquad::new(180.0, 0.55, sr);  // dentuman kayu terasa

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        let freq = f_end
            + (f_start - f_mid) * (-t / tau_fast).exp()
            + (f_mid - f_end)  * (-t / tau_slow).exp();

        p1 += 2.0 * PI * freq * 1.0 / sr as f32;
        p2 += 2.0 * PI * freq * 1.52 / sr as f32;
        p3 += 2.0 * PI * freq * 2.13 / sr as f32;
        p4 += 2.0 * PI * freq * 3.34 / sr as f32;
        p5 += 2.0 * PI * freq * 4.61 / sr as f32;
        sub_p += 2.0 * PI * freq * 0.5 / sr as f32;

        let noise = (rng.next_f32() - 0.5) * 2.0;

        // Beater: klik tinggi + dentuman dominan
        let beater_high = beater_lp.process(beater_bp.process(noise)) * (-t / 0.005).exp() * 0.22;
        let beater_low  = thud_lp.process(beater_bp.process(noise) * 0.5) * (-t / 0.012).exp() * 0.58;

        // Body bulat dengan soft‑clip rendah
        let body_raw =
            p1.sin() * (-t / 0.200).exp() * 0.60
          + p2.sin() * (-t / 0.100).exp() * 0.22
          + p3.sin() * (-t / 0.055).exp() * 0.12
          + p4.sin() * (-t / 0.030).exp() * 0.05
          + p5.sin() * (-t / 0.018).exp() * 0.02;

        let body = body_lp.process((body_raw * 1.3).tanh()) * 0.70;

        // Sub bass tebal
        let sub = sub_lp.process(sub_p.sin()) * (-t / 0.35).exp() * 0.85;

        *slot = (beater_high + beater_low + body + sub) * 0.90;
    }
    out
}

// ─── SNARE (tebal, tegas, tanpa kresek panjang) ──────────
fn synth_snare(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(350, sr);  // sedikit lebih pendek
    let mut out = vec![0.0f32; n];
    
    let mut final_lp = LowPassBiquad::new(8000.0, 0.5, sr);

    let f1 = 175.0; let f2 = f1 * 1.60; let f3 = f1 * 2.15;
    let f4 = f1 * 3.42; let f5 = f1 * 4.73;
    let mut p1 = 0.0f32; let mut p2 = 0.0f32; let mut p3 = 0.0f32;
    let mut p4 = 0.0f32; let mut p5 = 0.0f32;

    let mut rng = Rng::new(0x5BADE5);

    let mut body_lp   = LowPassBiquad::new(1100.0, 0.7, sr);
    let mut stick_bp   = BandPass::new(4500.0, 0.5, sr);
    let mut stick_lp   = LowPassBiquad::new(8000.0, 0.7, sr);
    let mut crack_bp   = BandPass::new(3200.0, 0.6, sr);
    let mut crack_lp   = LowPassBiquad::new(5500.0, 0.7, sr);
    let mut shell_bp   = BandPass::new(185.0, 1.5, sr);   // resonansi cangkang lebih rendah

    let mut wire_bp1 = BandPass::new(4200.0, 0.8, sr);
    let mut wire_bp2 = BandPass::new(7500.0, 0.8, sr);
    let mut wire_lp  = LowPassBiquad::new(9500.0, 0.7, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let noise = (rng.next_f32() - 0.5) * 2.0;

        p1 += 2.0 * PI * f1 / sr as f32;
        p2 += 2.0 * PI * f2 / sr as f32;
        p3 += 2.0 * PI * f3 / sr as f32;
        p4 += 2.0 * PI * f4 / sr as f32;
        p5 += 2.0 * PI * f5 / sr as f32;

        // Body lebih panjang & dominan
        let body_raw = p1.sin() * (-t / 0.090).exp() * 0.50
                     + p2.sin() * (-t / 0.045).exp() * 0.24
                     + p3.sin() * (-t / 0.022).exp() * 0.14
                     + p4.sin() * (-t / 0.014).exp() * 0.06
                     + p5.sin() * (-t / 0.009).exp() * 0.03;
        let body = body_lp.process(membrane(body_raw * 1.2)) * 0.45;

        // Shell tebal
        let shell = shell_bp.process(body_raw * 0.75) * (-t / 0.07).exp() * 0.40;

        // Stick pendek dan bersih
        let stick = stick_lp.process(stick_bp.process(noise)) * (-t / 0.004).exp() * 0.50;

        // Crack (rimshot) sedikit dikurangi
        let crack_raw = crack_bp.process(noise) * 0.65 + p3.sin() * 0.20;
        let crack = crack_lp.process(membrane(crack_raw * 2.0)) * (-t / 0.020).exp() * 0.45;

        // Wire dipotong lebih awal dan volume kecil – tidak ada kresek panjang
        let wire_lfo = (t * 15.0 * PI).sin() * 0.15 + 0.85;
        let wire_env = (-t / 0.065).exp() * wire_lfo;   // decay lebih cepat
        let wire = wire_lp.process(wire_bp1.process(noise) * 0.4 + wire_bp2.process(noise) * 0.4) 
                    * wire_env * 0.28;

        let raw = (body + shell + stick + crack + wire) * 0.90;
        
        *slot = final_lp.process(raw); 
    }
    out
}



fn synth_hihat(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(120, sr);
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(0xC0FFEE);

    let mut nbp1 = BandPass::new(7200.0,  0.9, sr);
    let mut nbp2 = BandPass::new(10500.0, 0.8, sr);
    let mut n_rolloff = LowPassBiquad::new(14500.0, 0.7, sr);

    let mut c: Vec<f32> = vec![0.0; 5];
    let cf = [3900.0, 5100.0, 6300.0, 7800.0, 9200.0];
    let mut clang_lp = LowPassBiquad::new(11500.0, 0.7, sr);

    let mut stick_bp = BandPass::new(5600.0, 0.5, sr);
    let mut stick_lp = LowPassBiquad::new(9500.0, 0.7, sr);

    // 🔧 Tambahan: low-pass filter akhir untuk memotong frekuensi > 16kHz
    let mut final_lp = LowPassBiquad::new(16000.0, 0.5, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let noise = (rng.next_f32() - 0.5) * 2.0;

        for (k, freq) in cf.iter().enumerate() {
            c[k] += 2.0 * PI * freq / sr as f32;
        }

        // Kurangi gain noise pada sizzle (0.45 -> 0.35)
        let n_env = (-t / 0.002).exp() * 0.20 + (-t / 0.025).exp() * 0.80;
        let n_raw = nbp1.process(noise) * 0.35 + nbp2.process(noise) * 0.35; // dikurangi
        let noise_out = n_rolloff.process(n_raw) * n_env * 0.38;

        // Kurangi gain attack noise (0.60 -> 0.40)
        let attack_noise = nbp1.process(noise) * (-t / 0.0025).exp() * 0.40;
        let clang_sum: f32 = c.iter().enumerate().map(|(k, &phase)| {
            let env = [0.012, 0.010, 0.014, 0.008, 0.006][k];
            phase.sin() * (-t / env).exp() * [0.28, 0.22, 0.18, 0.12, 0.08][k]
        }).sum();
        let clang = clang_lp.process(membrane((clang_sum + attack_noise) * 1.8)) * 0.48;

        // Stick click sangat pendek (tetap)
        let stick = stick_lp.process(stick_bp.process(noise)) * (-t / 0.002).exp() * 0.25;

        let choke = if t > 0.070 { (-(t - 0.070) / 0.012).exp().max(0.0) } else { 1.0 };

        let raw = (noise_out + clang + stick) * 0.85 * choke;
        *slot = final_lp.process(raw); // 🔧 potong frekuensi tinggi
    }
    out
}

// ─── CRASH (natural & tegas) ─────────────────────────────
fn synth_crash(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(1800, sr);
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(0xCCA55);
    
    let mut final_lp = LowPassBiquad::new(16000.0, 0.5, sr); 

    let mut nbp1 = BandPass::new(3600.0, 0.8, sr);
    let mut nbp2 = BandPass::new(6500.0, 0.7, sr);
    let mut nbp3 = BandPass::new(9200.0, 0.6, sr);
    let mut n_rolloff = LowPassBiquad::new(13500.0, 0.7, sr);

    // Osilator dikurangi, lebih rendah & alami
    let mut c = vec![0.0f32; 8];
    let cf = [2500.0, 3200.0, 4100.0, 5200.0, 6400.0, 7700.0, 9100.0, 10700.0];
    let mut clang_lp = LowPassBiquad::new(10000.0, 0.7, sr);

    let mut sp1 = 0.0f32; let mut sp2 = 0.0f32; let mut sp3 = 0.0f32;
    let sf1 = 3100.0; let sf2 = 4750.0; let sf3 = 7300.0;
    let mut shim_lp = LowPassBiquad::new(11500.0, 0.7, sr);

    let mut stick_bp = BandPass::new(4800.0, 0.5, sr);
    let mut stick_lp = LowPassBiquad::new(8500.0, 0.7, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let noise = (rng.next_f32() - 0.5) * 2.0;

        for (k, freq) in cf.iter().enumerate() {
            c[k] += 2.0 * PI * freq / sr as f32;
        }
        sp1 += 2.0 * PI * sf1 / sr as f32;
        sp2 += 2.0 * PI * sf2 / sr as f32;
        sp3 += 2.0 * PI * sf3 / sr as f32;

        let n_env = (-t / 0.03).exp() * 0.20 + (-t / 0.28).exp() * 0.45 + (-t / 1.15).exp() * 0.35;
        let n_raw = nbp1.process(noise)*0.25 + nbp2.process(noise)*0.35 + nbp3.process(noise)*0.25;
        let noise_out = n_rolloff.process(n_raw) * n_env * 0.55;

        let attack_noise = nbp2.process(noise) * (-t / 0.010).exp() * 0.50;
        let clang_sum: f32 = c.iter().enumerate().map(|(k, &phase)| {
            let env = [0.030, 0.024, 0.035, 0.020, 0.018, 0.014, 0.011, 0.008][k];
            let amp = [0.22, 0.20, 0.16, 0.13, 0.10, 0.08, 0.06, 0.04][k];
            phase.sin() * (-t / env).exp() * amp
        }).sum();
        let clang = clang_lp.process(membrane((clang_sum + attack_noise) * 1.5)) * 0.50;

        let shimmer = shim_lp.process(
            sp1.sin() * (-t / 0.70).exp() * 0.12
          + sp2.sin() * (-t / 0.50).exp() * 0.09
          + sp3.sin() * (-t / 0.35).exp() * 0.07
        );
        let stick_out = stick_lp.process(stick_bp.process(noise)) * (-t / 0.003).exp() * 0.14;

        let raw = (noise_out + clang + shimmer + stick_out) * 0.84;
        *slot = final_lp.process(raw); 
    }
    
    for s in out.iter_mut() {
        if s.abs() < 1e-25 { *s = 0.0; }
    }
    
    out
}

// ─── HIHAT OPEN ──────────────────────────────────────────
fn synth_hihat_open(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(650, sr);
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(0xC0FFEF);
    
    let mut final_lp = LowPassBiquad::new(16000.0, 0.5, sr); 

    let mut nbp1 = BandPass::new(6800.0, 0.9, sr);
    let mut nbp2 = BandPass::new(9800.0, 0.8, sr);
    let mut nbp3 = BandPass::new(12200.0, 0.7, sr);
    let mut n_rolloff = LowPassBiquad::new(13800.0, 0.7, sr);

    let mut c: Vec<f32> = vec![0.0; 8];
    let cf = [3750.0, 4650.0, 5850.0, 7100.0, 8800.0, 10100.0, 11500.0, 13200.0];
    let mut clang_lp = LowPassBiquad::new(11500.0, 0.7, sr);

    let mut sp1 = 0.0f32; let mut sp2 = 0.0f32;
    let sf1 = 6100.0; let sf2 = 8300.0;
    let mut shim_lp = LowPassBiquad::new(12500.0, 0.7, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let noise = (rng.next_f32() - 0.5) * 2.0;

        for (k, freq) in cf.iter().enumerate() {
            c[k] += 2.0 * PI * freq / sr as f32;
        }
        sp1 += 2.0 * PI * sf1 / sr as f32;
        sp2 += 2.0 * PI * sf2 / sr as f32;

        let n_env = (-t / 0.005).exp() * 0.20 + (-t / 0.16).exp() * 0.80;
        let n_raw = nbp1.process(noise)*0.3 + nbp2.process(noise)*0.4 + nbp3.process(noise)*0.3;
        let noise_out = n_rolloff.process(n_raw) * n_env * 0.42;

        let attack_noise = nbp1.process(noise) * (-t / 0.005).exp() * 0.55;
        let clang_sum: f32 = c.iter().enumerate().map(|(k, &phase)| {
            let env = [0.045, 0.038, 0.048, 0.028, 0.022, 0.018, 0.014, 0.010][k];
            phase.sin() * (-t / env).exp() * [0.30, 0.25, 0.20, 0.15, 0.10, 0.08, 0.06, 0.04][k]
        }).sum();
        let clang = clang_lp.process(membrane((clang_sum + attack_noise) * 2.4)) * 0.48;

        let shimmer = shim_lp.process(
            sp1.sin() * (-t / 0.28).exp() * 0.12
          + sp2.sin() * (-t / 0.20).exp() * 0.08
        );

        let raw = (noise_out + clang + shimmer) * 0.78;
        *slot = final_lp.process(raw); 
    }
    
    for s in out.iter_mut() {
        if s.abs() < 1e-25 { *s = 0.0; }
    }
    
    out
}

// ─── TOM ────────────────────────────────────────────────
fn synth_tom(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(440, sr);
    let mut out = vec![0.0f32; n];

    let f_start   = 195.0;
    let f_end     = 88.0;
    let pitch_tau = 0.055;

    let mut p1 = 0.0f32; let mut p2 = 0.0f32; let mut p3 = 0.0f32;
    let mut p4 = 0.0f32; let mut p5 = 0.0f32; let mut sub_p = 0.0f32;

    let mut rng = Rng::new(0x7AAA);
    let mut body_lp   = LowPassBiquad::new(600.0, 0.8, sr);
    let mut sub_lp    = LowPassBiquad::new(120.0, 0.6, sr);
    let mut stick_bp  = BandPass::new(2600.0, 0.6, sr);
    let mut stick_lp  = LowPassBiquad::new(5000.0, 0.7, sr);
    let mut attack_lp = LowPassBiquad::new(400.0, 0.8, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let freq = f_end + (f_start - f_end) * (-t / pitch_tau).exp();
        let noise = (rng.next_f32() - 0.5) * 2.0;

        p1 += 2.0 * PI * freq * 1.0 / sr as f32;
        p2 += 2.0 * PI * freq * 1.60 / sr as f32;
        p3 += 2.0 * PI * freq * 2.15 / sr as f32;
        p4 += 2.0 * PI * freq * 3.42 / sr as f32;
        p5 += 2.0 * PI * freq * 4.73 / sr as f32;
        sub_p += 2.0 * PI * freq * 0.5 / sr as f32;

        let attack = attack_lp.process(p1.sin()) * (-t / 0.010).exp() * 0.55;

        let body_raw = p1.sin() * (-t / 0.20).exp() * 0.50
                     + p2.sin() * (-t / 0.07).exp() * 0.25
                     + p3.sin() * (-t / 0.03).exp() * 0.15
                     + p4.sin() * (-t / 0.018).exp() * 0.07
                     + p5.sin() * (-t / 0.010).exp() * 0.03;
        let body = body_lp.process(membrane(body_raw * 1.3)) * 0.55;

        let sub = sub_lp.process(sub_p.sin()) * (-t / 0.28).exp() * 0.35;
        let stick = stick_lp.process(stick_bp.process(noise)) * (-t / 0.009).exp() * 0.18;

        *slot = (attack + body + sub + stick) * 0.85;
    }
    out
}

// ─── TOM LOW ──────────────────────────────────────────────
fn synth_tom_low(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(600, sr);
    let mut out = vec![0.0f32; n];

    let f_start   = 135.0;
    let f_end     = 54.0;
    let pitch_tau = 0.075;

    let mut p1 = 0.0f32; let mut p2 = 0.0f32; let mut p3 = 0.0f32;
    let mut p4 = 0.0f32; let mut p5 = 0.0f32; let mut sub_p = 0.0f32;

    let mut rng = Rng::new(0x7AAB);
    let mut body_lp   = LowPassBiquad::new(450.0, 0.8, sr);
    let mut sub_lp    = LowPassBiquad::new(90.0, 0.6, sr);
    let mut stick_bp  = BandPass::new(2200.0, 0.6, sr);
    let mut stick_lp  = LowPassBiquad::new(4200.0, 0.7, sr);
    let mut attack_lp = LowPassBiquad::new(320.0, 0.8, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let freq = f_end + (f_start - f_end) * (-t / pitch_tau).exp();
        let noise = (rng.next_f32() - 0.5) * 2.0;

        p1 += 2.0 * PI * freq * 1.0 / sr as f32;
        p2 += 2.0 * PI * freq * 1.60 / sr as f32;
        p3 += 2.0 * PI * freq * 2.15 / sr as f32;
        p4 += 2.0 * PI * freq * 3.42 / sr as f32;
        p5 += 2.0 * PI * freq * 4.73 / sr as f32;
        sub_p += 2.0 * PI * freq * 0.5 / sr as f32;

        let attack = attack_lp.process(p1.sin()) * (-t / 0.012).exp() * 0.55;
        let body_raw = p1.sin() * (-t / 0.25).exp() * 0.50
                     + p2.sin() * (-t / 0.09).exp() * 0.25
                     + p3.sin() * (-t / 0.04).exp() * 0.15
                     + p4.sin() * (-t / 0.022).exp() * 0.07
                     + p5.sin() * (-t / 0.012).exp() * 0.03;
        let body = body_lp.process(membrane(body_raw * 1.3)) * 0.55;
        let sub = sub_lp.process(sub_p.sin()) * (-t / 0.35).exp() * 0.40;
        let stick = stick_lp.process(stick_bp.process(noise)) * (-t / 0.011).exp() * 0.16;

        *slot = (attack + body + sub + stick) * 0.85;
    }
    out
}


// ─── RIDE ─────────────────────────────────────────────────
fn synth_ride(sr: u32) -> Vec<f32> {
    let n = ms_to_samples(900, sr);
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(0xA1DE);

    let mut bp1 = 0.0f32; let mut bp2 = 0.0f32; let mut bp3 = 0.0f32;
    let bf1 = 2020.0; let bf2 = 3400.0; let bf3 = 4850.0;
    let mut bell_lp = LowPassBiquad::new(7800.0, 0.7, sr);

    let mut c = vec![0.0f32; 8];
    let cf = [3450.0, 4150.0, 5500.0, 6800.0, 7900.0, 9100.0, 10500.0, 11800.0];
    let mut clang_lp = LowPassBiquad::new(8800.0, 0.7, sr);

    let mut wbp1 = BandPass::new(4800.0, 0.9, sr);
    let mut wbp2 = BandPass::new(7800.0, 0.7, sr);
    let mut w_rolloff = LowPassBiquad::new(12500.0, 0.7, sr);

    let mut stick_bp = BandPass::new(5800.0, 0.5, sr);
    let mut stick_lp = LowPassBiquad::new(9200.0, 0.7, sr);

    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        let noise = (rng.next_f32() - 0.5) * 2.0;

        bp1 += 2.0 * PI * bf1 / sr as f32;
        bp2 += 2.0 * PI * bf2 / sr as f32;
        bp3 += 2.0 * PI * bf3 / sr as f32;
        for (k, freq) in cf.iter().enumerate() {
            c[k] += 2.0 * PI * freq / sr as f32;
        }

        let bell = bell_lp.process(membrane(
            (bp1.sin() * (-t / 0.42).exp() * 0.40
           + bp2.sin() * (-t / 0.26).exp() * 0.25
           + bp3.sin() * (-t / 0.15).exp() * 0.15) * 1.4
        )) * 0.32;

        let attack_noise = wbp1.process(noise) * (-t / 0.008).exp() * 0.42;
        let clang_sum: f32 = c.iter().enumerate().map(|(k, &phase)| {
            let env = [0.014, 0.011, 0.016, 0.013, 0.010, 0.008, 0.007, 0.006][k];
            let amp = [0.30, 0.24, 0.16, 0.12, 0.09, 0.06, 0.04, 0.03][k];
            phase.sin() * (-t / env).exp() * amp
        }).sum();
        let clang = clang_lp.process(membrane((clang_sum + attack_noise) * 2.0)) * 0.28;

        let w_env = (-t / 0.04).exp() * 0.18 + (-t / 0.42).exp() * 0.82;
        let w_raw = wbp1.process(noise)*0.40 + wbp2.process(noise)*0.40;
        let wash = w_rolloff.process(w_raw) * w_env * 0.24;

        let stick_out = stick_lp.process(stick_bp.process(noise)) * (-t / 0.003).exp() * 0.14;

        *slot = (bell + clang + wash + stick_out) * 0.82;
    }
    out
}



// ====================================================================
// PIANO — Additive murni, zero noise, HIFI Cinematic High-Dynamic
// ====================================================================

pub fn synth_piano_chord(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    // ====================================================================
    // PARSER NOT ANGKA (1-8) -> SOLMISASI DO-RE-MI-FA-SOL-LA-SI-DO TINGGI
    // ====================================================================
    // Mendukung single note ("1") maupun chord ("1_3_5")
    let mapped_token = token.split('_')
        .map(|part| match part.trim() {
            "1" => "C4",  // Do
            "2" => "D4",  // Re
            "3" => "E4",  // Mi
            "4" => "F4",  // Fa
            "5" => "G4",  // Sol
            "6" => "A4",  // La
            "7" => "B4",  // Si
            "8" => "C5",  // Do Tinggi
            other => other, // Jika inputnya sudah "C4" atau "G4", biarkan tetap lolos
        })
        .collect::<Vec<&str>>()
        .join("_");

    // Teruskan token yang sudah diterjemahkan ke fungsi pembaca frekuensi bawaan Anda
    let freqs = chord_freqs(&mapped_token, 3);
    
    let dur   = duration_ms.max(650);
    let n     = ms_to_samples(dur, sample_rate);
    let mut out = vec![0.0f32; n];

    let voice_gain = 1.0 / (freqs.len() as f32).sqrt();

    for (vi, &f0) in freqs.iter().enumerate() {
        piano_string(&mut out, f0, dur, sample_rate, voice_gain, vi);
    }

    let mut hifi_air = HighShelf::new(4500.0, 1.25, sample_rate);
    for s in out.iter_mut() {
        *s = hifi_air.process(*s);
    }

    let max_peak = out.iter().map(|s| s.abs()).fold(0.0f32, |a, b| a.max(b));
    if max_peak > 0.95 {
        let scale = 0.95 / max_peak;
        for s in out.iter_mut() {
            *s *= scale;
        }
    }

    out
}

fn piano_string(out: &mut [f32], f0: f32, duration_ms: u64, sr: u32, gain: f32, vi: usize) {
    let b = 0.00003f32 * (f0 / 110.0).max(0.2).min(5.0); 
    let n = out.len();
    
    let attack_n  = ms_to_samples(10,  sr);  
    let decay_n   = ms_to_samples(150, sr);  
    let sustain   = 0.74f32;
    let release_n = ms_to_samples(450, sr).min(n / 3 + 1);
    let env = adsr_envelope(n, attack_n, decay_n, sustain, release_n);

    let n_harmonics = 14usize;
    let phase_init = vi as f32 * 0.53;

    // Detuning mikro tetap dipertahankan untuk kesan lebar (wide space)
    let detune_left  = 0.99965f32;
    let detune_right = 1.00035f32;

    let base_tau = 5.5f32 * (220.0 / f0).powf(0.25).min(2.0);

    for h in 1..=n_harmonics {
        let stretch = (1.0 + b * (h * h) as f32).sqrt();
        let freq_center = f0 * h as f32 * stretch;
        if freq_center >= sr as f32 * 0.47 { break; }

        let amp = if h == 1 {
            1.0
        } else if h % 2 == 1 {
            0.88 / (h as f32).powf(1.02)
        } else {
            0.48 / (h as f32).powf(1.22)
        };

        let tau = if h == 1 { base_tau } else { base_tau / (h as f32).powf(0.62) };
        let decay_env = -1.0 / tau;

        let dp_c = 2.0 * PI * freq_center / sr as f32;
        let dp_l = 2.0 * PI * (freq_center * detune_left) / sr as f32;
        let dp_r = 2.0 * PI * (freq_center * detune_right) / sr as f32;

        let mut p_c = phase_init + h as f32 * 0.11;
        let mut p_l = p_c + 0.25; 
        let mut p_r = p_c - 0.25;

        for i in 0..n {
            let t = i as f32 / sr as f32;
            p_c += dp_c;
            p_l += dp_l;
            p_r += dp_r;
            
            let decay = (t * decay_env).exp();
            
            // ====================================================================
            // FIX: HIERARKI VOLUME SENAR ACOUSTIC (Total Bobot = 1.0)
            // ====================================================================
            // - p_c (Senar Utama): 65% -> Menjamin core pitch tajam, solid, dan murni
            // - p_l (Sekunder 1):  22% -> Memberikan kehangatan stereo awal
            // - p_r (Sekunder 2):  13% -> Efek gaung mikro inter-string yang halus
            let acoustic_string_wave = (p_c.sin() * 0.65) + (p_l.sin() * 0.22) + (p_r.sin() * 0.13);
            
            out[i] += acoustic_string_wave * amp * decay * env[i] * gain;
        }
    }
}

// ====================================================================
// SAXOPHONE — Reed oscillator presisi, TANPA noise/breath sama sekali
//
// Prinsip:
// - Saxophone = instrumen reed = gelombang "clipped sine" (quasi square)
// - Bukan sawtooth! Saxophone lebih dominan harmonik ganjil dari genap
// - Filter formant memodelkan resonansi tabung logam
// - Vibrato alami yang tumbuh sesaat setelah attack
// - ZERO noise — semua suara dari osilator terkontrol
// ====================================================================

pub fn synth_sax_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 3);
    let dur = duration_ms.max(300);
    let n   = ms_to_samples(dur, sample_rate);
    let mut out = vec![0.0f32; n];

    // ADSR saxophone: attack sedikit lebih lambat dari piano (reed perlu "naik")
    let attack_n  = ms_to_samples(30,  sample_rate);
    let decay_n   = ms_to_samples(60,  sample_rate);
    let sustain   = 0.90f32;  // Sustain tinggi = nada penuh saat dimainkan
    let release_n = ms_to_samples(160, sample_rate).min(n / 3 + 1);
    let env = adsr_envelope(n, attack_n, decay_n, sustain, release_n);

    // Dua formant utama saxophone:
    // F1 ~sekitar 1× f0 (body cavity)
    // F2 ~sekitar 3× f0 (bell flare)
    // Q cukup tinggi agar resonansi terasa, tidak blur
    let mut formant1 = BandPass::new((f0 * 1.8).min(3000.0), 5.0, sample_rate);
    let mut formant2 = BandPass::new((f0 * 3.2).min(5000.0), 4.0, sample_rate);

    // Osilasi reed: campuran harmonik ganjil (menyerupai clarinet/sax)
    // Harmonik: 1, 3, 5, 7, 9, 11 dominan; genap lebih lemah
    const NH: usize = 12;
    let mut phases = [0.0f32; NH];

    for (i, slot) in out.iter_mut().enumerate() {
        let t  = i as f32 / sample_rate as f32;

        // Vibrato: mulai setelah 0.08s, tumbuh ke 0.006 semitone depth
        let vib_depth = 0.006 * (1.0 - (-t / 0.10).exp()).min(1.0);
        let vib       = 1.0 + vib_depth * (2.0 * PI * 5.4 * t).sin();
        let f_inst    = f0 * vib;

        // Bangun sinyal reed dari harmonik
        let mut reed = 0.0f32;
        for h in 1..=NH {
            // Saxophone/clarinet: ganjil jauh lebih kuat dari genap
            let amp = if h % 2 == 1 {
                1.0 / (h as f32).powf(0.80)   // Ganjil: rolloff pelan
            } else {
                0.25 / (h as f32).powf(1.20)  // Genap: sangat redup
            };
            phases[h-1] += 2.0 * PI * f_inst * h as f32 / sample_rate as f32;
            reed += phases[h-1].sin() * amp;
        }
        // Normalisasi reed agar level konsisten
        reed *= 0.28;

        // Lewatkan melalui formant (body resonance)
        let body = formant1.process(reed) * 1.5
                 + formant2.process(reed) * 0.6;

        // Sedikit soft-clip untuk karakter reed yang natural (bukan noise)
        let saturated = body.tanh();

        *slot = saturated * env[i] * 0.85;
    }
    out
}

// ====================================================================
// FLUTE — Sinusoidal murni, breath hanya di 5ms pertama lalu hilang
// ====================================================================

pub fn synth_flute_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0  = single_note_freq(token, 5);
    let dur = duration_ms.max(250);
    let n   = ms_to_samples(dur, sample_rate);
    let mut out = vec![0.0f32; n];

    let attack_n  = ms_to_samples(55, sample_rate);
    let decay_n   = ms_to_samples(45, sample_rate);
    let sustain   = 0.84f32;
    let release_n = ms_to_samples(150, sample_rate).min(n / 3 + 1);
    let env = adsr_envelope(n, attack_n, decay_n, sustain, release_n);

    // Breath hanya untuk 8ms pertama (onset puff), lalu murni nada
    let mut rng   = Rng::new(0xF7137E);
    let mut onset_bp = BandPass::new(f0, 2.5, sample_rate);
    let onset_end = ms_to_samples(8, sample_rate);

    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let mut p3 = 0.0f32;

    for (i, slot) in out.iter_mut().enumerate() {
        let t      = i as f32 / sample_rate as f32;
        let vib_d  = 0.004 * (1.0 - (-t / 0.20).exp()).min(1.0);
        let f_inst = f0 * (1.0 + vib_d * (2.0 * PI * 4.8 * t).sin());

        p1 += 2.0 * PI * f_inst       / sample_rate as f32;
        p2 += 2.0 * PI * f_inst * 2.0 / sample_rate as f32;
        p3 += 2.0 * PI * f_inst * 3.0 / sample_rate as f32;

        // Suara utama: hampir pure sine (flute = sine + sedikit oktaf)
        let tone = p1.sin() * 0.88 + p2.sin() * 0.10 + p3.sin() * 0.02;

        // Breath hanya di onset, lalu nol
        let breath = if i < onset_end {
            let onset_env = (-t / 0.005).exp();
            onset_bp.process(rng.next_f32()) * onset_env * 0.18
        } else {
            0.0
        };

        *slot = (tone + breath) * env[i];
    }
    out
}



// ====================================================================
// BASS GUITAR — True Heavy HiFi Cabinet Edition
// Single KS · Pick Scrape · Cabinet Resonance · Speaker Clip
// ====================================================================

pub fn synth_bass_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0  = single_note_freq(token, 2);
    let dur = duration_ms.max(300);
    let n   = ms_to_samples(dur, sample_rate);
    let mut out = vec![0.0f32; n];

    // ─── Envelope ───
    let attack_n  = ms_to_samples(1,   sample_rate);
    let decay_n   = ms_to_samples(18,  sample_rate);
    let sustain   = 0.65f32; 
    let release_n = ms_to_samples(5, sample_rate).min(n / 3 + 1);
    let env     = adsr_envelope(n, attack_n, decay_n, sustain, release_n);

    // ================================================================
    // KS DELAY LINE
    // ================================================================
    let dl_len = ((sample_rate as f32 / f0).round() as usize).max(2);
    let mut dl = vec![0.0f32; dl_len];
    
    for i in 0..dl_len {
        let t = i as f32 / sample_rate as f32;

        // String Thump — lembut, tanpa pick scrape inharmonik
        // Dikurangi gain dari 0.9 -> 0.45 agar attack tidak "pluck" keras
        let thump = (
            (2.0 * PI * f0 * t).sin() + 
            0.4 * (4.0 * PI * f0 * t).sin()
        ) * (-t / 0.006).exp() * 0.45;

        // Sustain Seed — sumber nada utama KS loop
        let seed = (
            (2.0 * PI * f0 * t).sin() + 
            0.25 * (4.0 * PI * f0 * t).sin() + 
            0.15 * (6.0 * PI * f0 * t).sin()
        ) * 0.5;

        dl[i] = thump + seed;
    }

    let ks_c   = 0.4992f32;
    let mut prev = 0.0f32;
    let mut idx  = 0usize;

    // ================================================================
    // SUB & OCTAVE UP
    // ================================================================
    let sub_f0  = f0 / 2.0;
    let sub_inc = 2.0 * PI * sub_f0 / sample_rate as f32;
    let mut sub_ph = 0.0f32;

    let up_inc  = 2.0 * PI * f0 * 2.0 / sample_rate as f32;
    let mut up_ph = 0.0f32;

    // ================================================================
    // FILTERS (Tuning untuk KEWARASAN & BERAT)
    // ================================================================
    
    // 1. Cabinet Resonance (Low-pass 120Hz)
    // Memisahkan frekuensi paling bawah untuk di-boost berat
    let mut cab_lp = LowPassBiquad::new(120.0, 0.707, sample_rate);
    
    // 2. Body Low-Pass (Naikkan ke 4500Hz)
    // Biar sifat "twang" pick scrape tetap terbaur di body,
    // tidak terpisah jadi suara tipis di atas.
    let mut body_lp  = LowPassBiquad::new((f0 * 6.0).min(4500.0), 0.88, sample_rate);
    
    // 3. Growl Band-Pass
    let growl_fc = (f0 * 2.0).clamp(120.0, 350.0);
    let mut growl_bp = BandPass::new(growl_fc, 2.5, sample_rate);
    
    // 4. Sub Low-Pass
    let mut sub_lp = LowPassBiquad::new((sub_f0 * 3.5).min(100.0), 0.70, sample_rate);

    // 5. Global High-Pass (35Hz agar sub tetap penuh)
    let hp_a = 1.0 - (-2.0 * PI * 35.0 / sample_rate as f32).exp();
    let mut hp_s = 0.0f32;

    let growl_drive = 3.0f32;  // dikurangi dari 4.5 agar lebih halus
    let body_drive   = 1.6f32; // dikurangi dari 2.0

    let growl_delay_n = ms_to_samples(40, sample_rate);
    let growl_fade_n  = ms_to_samples(150, sample_rate) as f32;

    // ================================================================
    // MAIN LOOP
    // ================================================================
    for i in 0..n {
        
        let delayed  = dl[idx];
        let filtered = (delayed + prev) * ks_c;
        dl[idx] = filtered;
        prev = delayed;
        idx = (idx + 1) % dl_len;

        // ── Sub (Pakai env utama, jangan sub_env, biar ikut nge-bass terus) ──
        let sub_raw = sub_ph.sin();
        sub_ph += sub_inc;
        if sub_ph > 2.0 * PI { sub_ph -= 2.0 * PI; }
        let sub = sub_lp.process(sub_raw) * 0.60 * env[i]; // Naikkan dari 0.45 -> 0.60

        let up_raw = up_ph.sin();
        up_ph += up_inc;
        if up_ph > 2.0 * PI { up_ph -= 2.0 * PI; }
        let oct_hint = up_raw * 0.03 * env[i];

        // ══════════════════════════════════════════
        //  1. CABINET RESONANCE — sub body resonance ringan
        // ══════════════════════════════════════════
        let cab_rumble = cab_lp.process(filtered);
        let resonated_signal = filtered + (cab_rumble * 0.35); // dikurangi dari 0.6

        // ══════════════════════════════════════════
        //  2. BODY PATH — warmth utama
        // ══════════════════════════════════════════
        let body_filt = body_lp.process(resonated_signal);
        let body = (body_filt * body_drive).tanh() / body_drive * 0.80;

        // ══════════════════════════════════════════
        //  3. GROWL PATH — mid karakter, muncul lambat
        // ══════════════════════════════════════════
        let growl_factor = if i > growl_delay_n {
            let t = (i - growl_delay_n) as f32 / growl_fade_n;
            t.min(1.0)
        } else {
            0.0
        };
        let g_raw = growl_bp.process(resonated_signal);
        let g_sat = (g_raw * growl_drive).tanh() / growl_drive;
        let growl = g_sat * 0.30 * growl_factor;

        // ══════════════════════════════════════════
        //  FINAL MIX — tanpa pick path
        // ══════════════════════════════════════════
        let mixed = body + growl + sub + oct_hint;

        hp_s += hp_a * (mixed - hp_s);
        let clean = mixed - hp_s;

        // Soft clip simetris — lebih bersih, tanpa karakter "pick zing"
        let amp_in = clean * env[i] * 1.05; // dikurangi dari 1.35
        out[i] = amp_in.tanh();
    }

    // DC offset removal
    let mean = out.iter().sum::<f32>() / out.len() as f32;
    for s in out.iter_mut() { *s -= mean; }

    // Final Hifi Soft Ceiling (Symetrical di bagian paling luar untuk keamanan)
    for s in out.iter_mut() {
        *s = 0.96 * (*s / 0.96).tanh();
    }

    out
}

pub fn synth_bass_chord(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    synth_bass_note(token, duration_ms, sample_rate)
}

    
// ====================================================================
// MULTI-PRESET GUITAR SYNTHESIZER (Ultra-Realistic Acoustic)
// PERBAIKAN FISIK: Inharmonicity, Dynamic Damping, Pick Transient
// ====================================================================
/*
pub fn synth_guitar_chord(token: &str, duration_ms: u64, sample_rate: u32, preset: &str) -> Vec<f32> {
    let mapped_token = token.split('_')
        .map(|part| match part.trim() {
            "1" => "C3", "2" => "D3", "3" => "E3", "4" => "F3",
            "5" => "G3", "6" => "A3", "7" => "B3", "8" => "C4",
            other => other,
        })
        .collect::<Vec<&str>>()
        .join("_");

    let freqs = chord_freqs(&mapped_token, 3);
    let dur   = duration_ms.max(650);
    let n     = ms_to_samples(dur, sample_rate);
    let mut out = vec![0.0f32; n];

    let n_voices  = freqs.len().min(6);
    let voice_gain = 0.45 / (n_voices as f32).sqrt();

    for (vi, &f0) in freqs.iter().take(n_voices).enumerate() {
        let strum_offset = ms_to_samples(vi as u64 * 5, sample_rate);

        let voice = if preset == "electric 1" {
            engine_electric_string(f0, dur, sample_rate, vi)
        } else {
            engine_pure_acoustic_string(f0, dur, sample_rate, vi)
        };

        crate::dsp::mix_in(&mut out, strum_offset, &voice, voice_gain);
    }

    if preset == "electric 1" {
        apply_electric_body(&mut out, sample_rate);
    } else {
        apply_acoustic_wooden_body(&mut out, sample_rate);
    }

    let max_peak = out.iter().map(|s| s.abs()).fold(0.0f32, |a, b| a.max(b));
    if max_peak > 0.88 {
        let scale = 0.88 / max_peak;
        for s in out.iter_mut() {
            *s *= scale;
        }
    }

    out
}
*/

pub fn synth_guitar_chord_stereo(token: &str, duration_ms: u64, sample_rate: u32, preset: &str) -> Vec<f32> {
    // Mapping token not angka ke note names (sama seperti di synth_guitar_chord)
    let mapped_token = token.split('_')
        .map(|part| match part.trim() {
            "1" => "C3", "2" => "D3", "3" => "E3", "4" => "F3",
            "5" => "G3", "6" => "A3", "7" => "B3", "8" => "C4",
            other => other,
        })
        .collect::<Vec<&str>>()
        .join("_");

    let freqs = chord_freqs(&mapped_token, 3);
    let dur = duration_ms.max(650);
    let n = ms_to_samples(dur, sample_rate);
    let mut left_chord = vec![0.0f32; n];
    let mut right_chord = vec![0.0f32; n];

    let n_voices = freqs.len().min(6);
    let voice_gain = 0.45 / (n_voices as f32).sqrt();

    for (vi, &f0) in freqs.iter().take(n_voices).enumerate() {
        let strum_offset = ms_to_samples(vi as u64 * 5, sample_rate);

        // Kiri: seed_offset = 0
        let voice_l = if preset == "electric 1" {
            engine_electric_string(f0, dur, sample_rate, vi, 0)
        } else {
            engine_pure_acoustic_string(f0, dur, sample_rate, vi, 0)
        };
        crate::dsp::mix_in(&mut left_chord, strum_offset, &voice_l, voice_gain);

        // Kanan: seed_offset = 0xDEAD
        let voice_r = if preset == "electric 1" {
            engine_electric_string(f0, dur, sample_rate, vi, 0xDEAD)
        } else {
            engine_pure_acoustic_string(f0, dur, sample_rate, vi, 0xDEAD)
        };
        crate::dsp::mix_in(&mut right_chord, strum_offset, &voice_r, voice_gain);
    }

    // Terapkan body (kiri dan kanan terpisah)
    if preset == "electric 1" {
        apply_electric_body(&mut left_chord, sample_rate);
        apply_electric_body(&mut right_chord, sample_rate);
    } else {
        apply_acoustic_wooden_body(&mut left_chord, sample_rate);
        apply_acoustic_wooden_body(&mut right_chord, sample_rate);
    }

    interleave_stereo(left_chord, right_chord)
}

// ====================================================================
// HELPERS
// ====================================================================

#[inline]
fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn fade_in_inplace(buf: &mut [f32], fade_len: usize) {
    let fade_len = fade_len.min(buf.len());
    for i in 0..fade_len {
        let g = smootherstep(i as f32 / fade_len as f32);
        buf[i] *= g;
    }
}

// ====================================================================
// ENGINE 1: PURE ACOUSTIC — ULTRA REALISTIC
// ====================================================================
fn engine_pure_acoustic_string(f0: f32, duration_ms: u64, sr: u32, vi: usize, seed_offset: u32) -> Vec<f32> {
    let n = ms_to_samples(duration_ms, sr);
    let mut out = vec![0.0f32; n];
    
    let mut body_lp = LowPassBiquad::new(12000.0, 0.5, sr);

    let delay_len_f = (sr as f32 / f0).max(2.0);
    let delay_len = delay_len_f.ceil() as usize + 1;
    let mut delay_line = vec![0.0f32; delay_len];

    // --- EKSITASI: lebih tajam & punchy ---
    //let mut rng = Rng::new(0xABCDEF + vi as u32 * 0x7F);
    
    let base_seed = 0xABCDEF + vi as u32 * 0x7F + seed_offset;
    let mut rng = Rng::new(base_seed);
    
    let mut smooth = 0.0f32;
    let active_len = delay_len_f.round() as usize;

    for i in 0..delay_line.len() {
        let raw = if i < active_len {
            (rng.next_f32() - 0.5) * 2.0
        } else {
            0.0
        };
        let pos = i as f32 / active_len.max(1) as f32;
        let coeff = 0.30 + 0.42 * pos;   
        smooth += coeff * (raw - smooth);

        let window = (pos * PI).sin().max(0.0);
        delay_line[i] = smooth * window * 1.25;   
    }

    // --- COMB FILTER POSISI PETIK ---
    let pick_pos_ratio = 1.0 / 7.0;
    let pick_delay = ((delay_line.len() as f32) * pick_pos_ratio).round() as usize;
    let pick_delay = pick_delay.max(1).min(delay_line.len() - 1);
    {
        let snapshot = delay_line.clone();
        for i in 0..delay_line.len() {
            let other = snapshot[(i + pick_delay) % snapshot.len()];
            delay_line[i] = snapshot[i] - other * 0.30;
        }
    }

    fade_in_inplace(&mut delay_line, 4);

    // --- LOOP UTAMA DENGAN FISIK SENAR ASLI ---
    let decay_factor = 0.9968f32;
    let mut prev1 = 0.0f32;
    let mut prev2 = 0.0f32;
    let mut dl_idx = 0usize;

    // [BARU] State untuk Inharmonicity (Kekakuan senar via All-Pass Filter)
    // Mensimulasikan harmonik yang sedikit "naik" di frekuensi tinggi.
    let mut ap_state = 0.0f32;
    let ap_coeff = 0.15f32; // Tingkat kekakuan senar (semakin besar semakin "metalik/bright")

    // [BARU] State untuk Pick Transient (Klik jangat/pick di awal)
    let mut pick_noise_env = 1.0f32;

    for i in 0..n {
        let delayed = delay_line[dl_idx];

        // [BARU] Dynamic Damping: Frekuensi tinggi meluruh lebih cepat dari bass
        // Filter 3-tap diubah bobotnya secara dinamis berdasarkan waktu.
        let damp_t = (i as f32 / (sr as f32 * 0.6)).min(1.0); // Luruh dalam 600ms
        let damp_env = smootherstep(damp_t);
        let hf_gain = 1.0 - (0.40 * damp_env); // HF turun hingga 40% seiring waktu
        
        let mut filtered = (delayed * 0.60) 
            + (prev1 * 0.26 * hf_gain) 
            + (prev2 * 0.14 * hf_gain);
        
        // Normalisasi gain agar tidak drop volume
        let norm = 0.60 + 0.26 * hf_gain + 0.14 * hf_gain;
        filtered /= norm;

        // [BARU] Inharmonicity (All-pass dispersion)
        // Membuat pitch harmonik atas sedikit melebar, ciri khas senar steel.
        let ap_out = -ap_coeff * filtered + ap_state;
        ap_state = filtered + ap_coeff * ap_out;
        
        delay_line[dl_idx] = ap_out * decay_factor;

        prev2 = prev1;
        prev1 = delayed;
        dl_idx = (dl_idx + 1) % delay_line.len();

        let tail = if i > n.saturating_sub(300) {
            smootherstep((n - i) as f32 / 300.0)
        } else {
            1.0
        };

        // [BARU] Tambahan Pick Transient (Klik akustik di millisecond pertama)
        let transient = if pick_noise_env > 0.01 {
            let noise = (rng.next_f32() - 0.5) * 2.0;
            let t = noise * pick_noise_env * 0.6;
            pick_noise_env *= 0.85; // Luruh sangat cepat (kira-kira 5-10ms)
            t
        } else {
            0.0
        };

       let raw_out = (delayed + transient) * tail;
       out[i] = body_lp.process(raw_out);
    }

    fade_in_inplace(&mut out, 16);

    out
}

fn apply_acoustic_wooden_body(buffer: &mut [f32], sr: u32) {
    let mut body_cavity = BandPass::new(102.0, 5.0, sr);
    let mut body_board  = BandPass::new(208.0, 4.0, sr);
    let mut body_upper  = BandPass::new(450.0, 3.0, sr);
    let mut presence_peak = BandPass::new(2400.0, 2.5, sr);
    
    // [BARU] Notch untuk menghilangkan suara "kardus/boxy" pada gitar akustik murah
    // Frekuensi 600Hz sering bikin gitar terdengar "ngung" atau mendem.
    let mut boxy_notch = BandPass::new(600.0, 4.0, sr);
    
    let mut string_glow = HighShelf::new(3000.0, 4.5, sr);
    let mut air_shimmer = HighShelf::new(7500.0, 2.8, sr);
    
    // [BARU] DC Blocker untuk mencegah offset yang bikin bass tidak bersih
    let mut hp = OnePoleHighPass::new(82.0, sr);
    let mut dc_x = 0.0f32;
    let mut dc_y = 0.0f32;

    for s in buffer.iter_mut() {
        // 1. DC Blocker manual
        dc_y = *s - dc_x + 0.999 * dc_y;
        dc_x = *s;
        let x = hp.process(dc_y);
        
        // 2. Resonansi Kayu & Body
        let wood_resonance = body_cavity.process(x) * 0.18
            + body_board.process(x) * 0.13
            + body_upper.process(x) * 0.09
            + presence_peak.process(x) * 0.07;
            
        // 3. Pengurangan suara kardus (Notch)
        let anti_boxy = boxy_notch.process(x) * 0.03; // Dikurangi 3% di 600Hz
        
        let processed_tone = x + wood_resonance - anti_boxy;
        
        // 4. High shelves
        let bright = string_glow.process(processed_tone);
        *s = air_shimmer.process(bright);
    }
}

// ====================================================================
// ENGINE 2: PRESET "electric 1"  (tidak berubah sama sekali)
// ====================================================================
fn engine_electric_string(f0: f32, duration_ms: u64, sr: u32, vi: usize, seed_offset: u32) -> Vec<f32> {
    let n = ms_to_samples(duration_ms, sr);
    let mut out = vec![0.0f32; n];
    let detune = 1.0 + (vi as f32 - 1.5) * 0.00015;
    let f0 = f0 * detune;

    let delay_len_f = (sr as f32 / f0).max(2.0);
    let delay_len = delay_len_f.ceil() as usize + 1;
    let mut delay_line = vec![0.0f32; delay_len];

    //let mut rng = Rng::new(0x2026A1 + vi as u32 * 0x3F);
    let base_seed = 0x2026A1 + vi as u32 * 0x3F + seed_offset;
    let mut rng = Rng::new(base_seed);
    
    let mut smooth = 0.0f32;
    let active_len = delay_len_f.round() as usize;

    for i in 0..delay_line.len() {
        let raw_noise = if i < active_len {
            (rng.next_f32() - 0.5) * 2.0
        } else {
            0.0
        };
        let pos = i as f32 / active_len.max(1) as f32;
        let coeff = 0.48 + 0.32 * pos;
        smooth += coeff * (raw_noise - smooth);

        let window = (pos * PI).sin().max(0.0);
        delay_line[i] = smooth * window;
    }
    fade_in_inplace(&mut delay_line, 4);

    let decay_factor = 0.9984f32;
    let mut prev = 0.0f32;
    let mut dl_idx = 0usize;

    for i in 0..n {
        let delayed = delay_line[dl_idx];
        let filtered = 0.5 * (delayed + prev);
        delay_line[dl_idx] = filtered * decay_factor;
        prev = delayed;
        dl_idx = (dl_idx + 1) % delay_line.len();

        let anti_click_tail = if i > n.saturating_sub(200) {
            smootherstep((n - i) as f32 / 200.0)
        } else {
            1.0
        };
        out[i] = delayed * anti_click_tail;
    }

    fade_in_inplace(&mut out, 16);

    out
}

fn apply_electric_body(buffer: &mut [f32], sr: u32) {
    let mut body_wood = BandPass::new(260.0, 3.5, sr);
    let mut hp = OnePoleHighPass::new(95.0, sr);
    let mut glassy_shine = HighShelf::new(4800.0, 1.8, sr);

    for s in buffer.iter_mut() {
        let x = hp.process(*s);
        let resonance = body_wood.process(x) * 0.04;
        *s = glassy_shine.process(x + resonance);
    }
}



// Tidak dipakai tapi tetap di-export untuk kompatibilitas


// ============================================================
// KODE TAMBAHAN: GITAR AKUSTIK PETIKAN SATU PER SATU
// Paste ke file masing-masing sesuai panduan di bawah
// ============================================================


// ============================================================
// [FILE 1] src/synth.rs
// Paste di bagian PALING BAWAH file (setelah baris terakhir)
// ============================================================

/// Gitar akustik petikan satu senar — Karplus-Strong dengan karakter
/// petikan jari/plektrum, sustain natural, dan resonansi body kayu.
/// Menghasilkan output stereo interleaved (sama seperti synth_guitar_chord_stereo).
pub fn synth_guitar_single_note_stereo(
    token: &str,
    duration_ms: u64,
    sample_rate: u32,
) -> Vec<f32> {
    // Mapping angka 1-8 ke not dalam oktaf 3 (range gitar akustik standar)
    let mapped = match token.trim() {
        "1" => "C3",
        "2" => "D3",
        "3" => "E3",
        "4" => "F3",
        "5" => "G3",
        "6" => "A3",
        "7" => "B3",
        "8" => "C4",
        other => other,
    };

    let f0 = crate::notes::single_note_freq(mapped, 3);
    let dur = duration_ms.max(300);

    // Kiri dan kanan pakai seed berbeda untuk efek stereo alami
    let mut left  = engine_pure_acoustic_string(f0, dur, sample_rate, 0, 0x00000000);
    let mut right = engine_pure_acoustic_string(f0, dur, sample_rate, 0, 0x0000DEAD);

    // Body resonansi kayu pada kedua channel
    apply_acoustic_wooden_body(&mut left,  sample_rate);
    apply_acoustic_wooden_body(&mut right, sample_rate);

    // Normalisasi peak agar tidak clipping
    let peak = left.iter().chain(right.iter())
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    if peak > 0.88 {
        let scale = 0.88 / peak;
        for s in left.iter_mut()  { *s *= scale; }
        for s in right.iter_mut() { *s *= scale; }
    }

    crate::dsp::interleave_stereo(left, right)
}



// ====================================================================
// VIOLIN — Luxury Bowed String Synthesizer
//
// Arsitektur:
//   1. Helmholtz Slip-Stick oscillator (bow-string physics)
//   2. 18 partials additive + inharmonicity stretch
//   3. Dual-layer body resonance (Stradivari-style formants)
//   4. Expressive vibrato dengan attack curve organik
//   5. True stereo output (L/R dengan seed berbeda)
//   6. Rosin scrape & bow change transient di onset
// ====================================================================

/// Satu suara violin monophonic dengan model fisik bowed string.
/// Output: stereo interleaved (sama dengan guitar/drum).
use crate::dsp;
pub fn synth_violin_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0  = single_note_freq(token, 5);
    if f0 < 1.0 { return vec![0.0; dsp::ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = violin_mono_voice(f0, duration_ms, sample_rate, 0x5A7E_B0ED);
    let mut right = violin_mono_voice(f0, duration_ms, sample_rate, 0xDEAD_CAFE);

    // Chorus stereoiser: kanan di-detune sangat halus untuk kesan ruang
    let detune = 1.0 + 0.00018f32; // +0.18 sen
    for (i, s) in right.iter_mut().enumerate() {
        let lfo = (2.0 * PI * 0.31 * i as f32 / sample_rate as f32).sin() * 0.000055;
        *s *= 1.0 + lfo;
        let _ = detune; // digunakan lewat lfo saja
    }

    // Peak normalize bersama agar tidak clipping
    let peak = left.iter().chain(right.iter())
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    let scale = if peak > 0.90 { 0.90 / peak } else { 1.0 };

    use crate::dsp::interleave_stereo;
    let left_scaled:  Vec<f32> = left.into_iter().map(|s| s * scale).collect();
    let right_scaled: Vec<f32> = right.into_iter().map(|s| s * scale).collect();
    interleave_stereo(left_scaled, right_scaled)
}

fn violin_mono_voice(f0: f32, duration_ms: u64, sample_rate: u32, seed: u32) -> Vec<f32> {
    let dur = duration_ms.max(200);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];

    // NEW — Bow Pressure Envelope: sangat tipis → tipis → normal → tipis → sangat tipis
// Ini model fisik gesek busur yang sebenarnya:
// - Busur menyentuh senar pelan (sangat tipis di awal)
// - Tekanan bow bertambah → suara normal
// - Sebelum angkat busur, tekanan berkurang → kembali sangat tipis
let env: Vec<f32> = {
    let swell_ms   = (dur as f32 * 0.18).max(80.0) as u64;  // 18% pertama: bow contact
    let sustain_ms = (dur as f32 * 0.64) as u64;             // 64% tengah: full bow tone
    let taper_ms   = dur - swell_ms - sustain_ms;            // sisa: bow lift

    let swell_n   = ms_to_samples(swell_ms,   sr);
    let sustain_n = ms_to_samples(sustain_ms, sr);
    let taper_n   = ms_to_samples(taper_ms,   sr).max(1);

    let mut e = vec![0.0f32; n];
    for i in 0..n {
        e[i] = if i < swell_n {
            // Bow contact: sangat tipis → naik secara kuadratik
            // pow(3) membuat awal sangat sangat tipis (hampir nol)
            let t = i as f32 / swell_n.max(1) as f32;
            t * t * t   // 0.0 → 1.0, sangat lambat di awal
        } else if i < swell_n + sustain_n {
            // Full bow pressure: sustain konstan
            1.0f32
        } else {
            // Bow lift: tipis → sangat tipis (linear + squared taper)
            let t = (i - swell_n - sustain_n) as f32 / taper_n as f32;
            let t = t.min(1.0);
            // Kuadrat terbalik: turun lambat dulu, lalu cepat di akhir
            (1.0 - t * t).max(0.0)
        };
    }
    e
};
    
    // ── INHARMONICITY: Senar violin sangat tipis → rendah ──────────────
    let b_coeff = 0.000005f32 * (f0 / 200.0).max(0.5).min(3.0);

    // ── BOWED STRING MODEL (Helmholtz Motion) ──────────────────────────
    // Gelombang Helmholtz ≈ SAWTOOTH murni (1/h rolloff),
    // sangat berbeda dari clarinet/sax yang dominan harmonik GANJIL.
    // Ini yang membedakan gesek vs tiup secara spektral.
    const NH: usize = 24; // lebih banyak harmonik untuk kekayaan gesek
    let mut phases = [0.0f32; NH];

    // ── BOW PRESSURE STATE (Simulasi tekanan busur) ─────────────────────
    // Stick-slip: output asymmetric saat bow "stick", release saat "slip"
    let mut bow_state = 0.0f32;
    let bow_stiffness = 0.35f32; // koefisien gesekan

    // ── ROSIN SCRAPE TRANSIENT ──────────────────────────────────────────
    let mut rng = Rng::new(seed);
    let mut rosin_bp1 = BandPass::new(f0 * 1.5, 4.0, sr);
    let mut rosin_bp2 = BandPass::new(f0 * 3.5, 3.0, sr);
    let rosin_end = ms_to_samples(30, sr); // sedikit lebih panjang

    // ── BODY RESONANCES (Stradivari acoustic modes) ─────────────────────
    // Mode-mode ini sangat penting untuk karakter "nasal" violin
    // yang membedakannya dari flute (yang smooth dan rounded)
    let mut body_a0   = BandPass::new(270.0,  8.0, sr);  // A0: woody bass warmth
    let mut body_air  = BandPass::new(390.0,  6.5, sr);  // Air mode
    let mut body_b1lo = BandPass::new(465.0,  7.0, sr);  // B1-: body richness
    let mut body_b1hi = BandPass::new(550.0,  6.0, sr);  // B1+: Helmholtz coupling
    let mut bridge_lo = BandPass::new(2100.0, 5.5, sr);  // Bridge hill: nasal projection
    let mut bridge_hi = BandPass::new(3200.0, 4.5, sr);  // Bridge upper: brightness
    // Tambahan: "Wolf tone" suppressor & presence enhancer
    let mut upper_presence = BandPass::new(4500.0, 3.5, sr);

    // Low-pass untuk membuang digital artifact di atas 9kHz
    let mut silk_lp = LowPassBiquad::new(8800.0, 0.60, sr);

    // High-shelf tipis untuk "air" konser
    //let mut air_shelf = HighShelf::new(4800.0, 3.5, sr);
    let mut air_shelf = HighShelf::new(5500.0, 3.5, sr); // boost sangat kecil, frekuensi lebih tinggi

    // Sedikit warmth boost di bawah
    let mut warmth_lp = LowPassBiquad::new(800.0, 0.7, sr);

    // ── BOW CHANGE (perubahan arah bow) ────────────────────────────────
    let bow_change_pos = ms_to_samples((dur / 2).min(800), sr);
    let bow_change_dur = ms_to_samples(12, sr).max(1);

    // ── MAIN LOOP ────────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // Vibrato: karakteristik pemain violin klasik — lambat berkembang
        let vib_grow  = (1.0 - (-t / 0.22).exp()).powf(1.8); // tumbuh lebih lambat
        let vib_amt   = 0.010 * vib_grow; // sedikit lebih dalam dari sebelumnya
        let vib = 1.0 + vib_amt * (2.0 * PI * 5.2 * t).sin()
                      + vib_amt * 0.08 * (2.0 * PI * 10.4 * t).sin(); // overtone vibrato

        // Micro-intonation drift (manusiawi)
        let drift = 1.0 + 0.00035 * (2.0 * PI * 0.13 * t).sin();
        let f_inst = f0 * vib * drift;

        // ── HELMHOLTZ SAWTOOTH SYNTHESIS ─────────────────────────────
        // Bowed string = sawtooth: amplitudo 1/h MERATA untuk semua harmonik
        // (BERBEDA dari sax/flute yang dominan harmonik ganjil!)
        let mut saw = 0.0f32;
        for h in 1..=NH {
            let stretch = (1.0 + b_coeff * (h * h) as f32).sqrt();
            let f_h = f_inst * h as f32 * stretch;
            if f_h >= sr as f32 * 0.46 { break; }

            // TRUE SAWTOOTH rolloff: 1/h untuk SEMUA harmonik (genap & ganjil sama)
            // Ini ciri khas alat gesek vs alat tiup
            let amp = match h {
                1 => 1.00,
                2 => 0.50,
                3 => 0.33,
                4 => 0.25,
                5 => 0.20,
                6 => 0.17,
                7 => 0.14,
                8 => 0.12,
                _ => 1.0 / h as f32,
            };

            // Bow speed modulation: harmonik tinggi sedikit lebih responsif
            let bow_mod = 1.0 + 0.04 * (h as f32 / NH as f32);

            phases[h - 1] += 2.0 * PI * f_h / sr as f32;
            saw += phases[h - 1].sin() * amp * bow_mod;
        }
        saw *= 0.09; // level dasar — lebih tipis agar saturasi tidak terjadi

        // ── STICK-SLIP NONLINEARITY (kunci suara gesek!) ─────────────
        // Model Helmholtz: bow "sticks" ke senar lalu "slips"
        // Menghasilkan asymmetric clipping yang khas alat gesek
        // (BERBEDA dari tanh simetris yang terdengar seperti reed/tiup)
        let bow_vel = saw - bow_state * bow_stiffness;

        // Slip condition: asymmetric nonlinearity
        let bowed = if bow_vel.abs() < 0.4 {
            // STICK phase: gesekan kuat, output mengikuti bow
            bow_state = bow_state * 0.96 + bow_vel * 0.55;
            bow_vel * 1.8 + bow_vel * bow_vel * bow_vel * 0.3 // slight asymmetry
        } else {
            // SLIP phase: senar meluncur bebas
            bow_state = bow_state * 0.88;
            // Hard asymmetric clip — inilah yang membuat violin terdengar "gesek"
            let s = saw * 1.6;
            if s > 0.0 {
                s.min(0.85) * 1.0 + s * 0.05 // clip lebih keras di atas
            } else {
                s.max(-0.72) * 1.0 + s * 0.08 // lebih lembut di bawah (asymmetry!)
            }
        };

        // ── ROSIN TRANSIENT ───────────────────────────────────────────
        let rosin = if i < rosin_end {
            let env_r = (-t / 0.015).exp();
            let n1 = rng.next_f32();
            let n2 = rng.next_f32();
            (rosin_bp1.process(n1) * 0.65 + rosin_bp2.process(n2) * 0.40) * env_r * 0.22
        } else {
            0.0
        };

        // ── BOW DIRECTION CHANGE ──────────────────────────────────────
        let bow_click = if i >= bow_change_pos && i < bow_change_pos + bow_change_dur {
            let t_click = (i - bow_change_pos) as f32 / bow_change_dur as f32;
            let impulse = (PI * t_click).sin();
            impulse * rng.next_f32() * 0.018
        } else {
            0.0
        };

        let raw = bowed + rosin + bow_click;

        // ── BODY RESONANCE (violin wood modes) ───────────────────────
        // Boost signifikan pada bridge hill (2-3kHz) adalah ciri khas
        // violin yang membedakannya dari flute/sax (yang lebih smooth)
        let warmth = warmth_lp.process(raw) * 0.15; // sedikit warmth
let resonated = raw
    + body_a0.process(raw)       * 0.35
    + body_air.process(raw)      * 0.25
    + body_b1lo.process(raw)     * 0.32
    + body_b1hi.process(raw)     * 0.28
    + bridge_lo.process(raw)     * 0.52   // bridge lebih kuat = nasal violin tone
    + bridge_hi.process(raw)     * 0.38   // brightness khas violin
    + upper_presence.process(raw)* 0.20   // presence 4.5kHz
    + warmth;

        // Silk low-pass (potong artifak digital)
        let silked = silk_lp.process(resonated);

        // Air shelf
        let aired = air_shelf.process(silked);

        // Final output: TIDAK pakai tanh simetris murni!
        // Gunakan soft asymmetric saturation untuk menjaga karakter gesek
        let driven = aired * 0.65; // drive kecil — cegah saturasi yang bikin suara tebal
        let saturated = if driven > 0.0 {
            1.0 - (-driven).exp()         // eksponen asimetris (lebih natural)
        } else {
            -1.0 + (driven).exp()
        };

        *slot = saturated * env[i] * 0.92;  // naikkan gain karena envelope baru lebih konservatif
    }

    out
}


// ====================================================================
// VIOLA — Bowed String, lebih gelap & warm dari violin
//
// Perbedaan fisik dari violin:
//   - Senar lebih tebal → suara lebih grainy, dark, kurang bright
//   - Body lebih besar → resonansi di frekuensi lebih rendah
//   - Range: C3–A5 (satu kuintal lebih rendah dari violin)
//   - Vibrato lebih lambat & lebih dalam
//   - Karakter: "nasal warm" bukan "nasal bright" seperti violin
// ====================================================================

pub fn synth_viola_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    // Viola default oktaf 4 → tapi mapping not-nya di lib.rs pakai oktaf lebih rendah
    let f0 = single_note_freq(token, 4); // oktaf 4 = range viola (C3–A5 tergantung token)
    if f0 < 1.0 { return vec![0.0; dsp::ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = viola_mono_voice(f0, duration_ms, sample_rate, 0x7A3C_D1F2);
    let mut right = viola_mono_voice(f0, duration_ms, sample_rate, 0xBEEF_A5C0);

    // Stereo chorus lebih lebar dari violin (body viola lebih besar)
    for (i, s) in right.iter_mut().enumerate() {
        let lfo = (2.0 * PI * 0.27 * i as f32 / sample_rate as f32).sin() * 0.000075;
        *s *= 1.0 + lfo;
    }

    let peak = left.iter().chain(right.iter())
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    let scale = if peak > 0.90 { 0.90 / peak } else { 1.0 };

    let left_scaled:  Vec<f32> = left.into_iter().map(|s| s * scale).collect();
    let right_scaled: Vec<f32> = right.into_iter().map(|s| s * scale).collect();
    interleave_stereo(left_scaled, right_scaled)
}

fn viola_mono_voice(f0: f32, duration_ms: u64, sample_rate: u32, seed: u32) -> Vec<f32> {
    let dur = duration_ms.max(200);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];

    // ── BOW PRESSURE ENVELOPE (sama dengan violin) ──────────────────
    let env: Vec<f32> = {
        let swell_ms   = (dur as f32 * 0.18).max(80.0) as u64;
        let sustain_ms = (dur as f32 * 0.64) as u64;
        let taper_ms   = dur - swell_ms - sustain_ms;
        let swell_n   = ms_to_samples(swell_ms,   sr);
        let sustain_n = ms_to_samples(sustain_ms, sr);
        let taper_n   = ms_to_samples(taper_ms,   sr).max(1);
        let mut e = vec![0.0f32; n];
        for i in 0..n {
            e[i] = if i < swell_n {
                let t = i as f32 / swell_n.max(1) as f32;
                t * t * t
            } else if i < swell_n + sustain_n {
                1.0f32
            } else {
                let t = (i - swell_n - sustain_n) as f32 / taper_n as f32;
                (1.0 - t.min(1.0) * t.min(1.0)).max(0.0)
            };
        }
        e
    };

    // ── INHARMONICITY: senar viola lebih tebal → sedikit lebih tinggi ──
    let b_coeff = 0.000012f32 * (f0 / 150.0).max(0.5).min(3.0);

    // ── HELMHOLTZ SAWTOOTH (sama, tapi saw level lebih hangat) ─────────
    const NH: usize = 20; // sedikit lebih sedikit harmonik tinggi (senar tebal = rolloff lebih cepat)
    let mut phases = [0.0f32; NH];

    // ── BOW PRESSURE STATE ───────────────────────────────────────────
    let mut bow_state = 0.0f32;
    let bow_stiffness = 0.28f32; // lebih rendah dari violin (senar lebih tebal = grip berbeda)

    // ── ROSIN SCRAPE TRANSIENT ────────────────────────────────────────
    let mut rng = Rng::new(seed);
    // Frekuensi rosin lebih rendah dari violin (senar lebih tebal)
    let mut rosin_bp1 = BandPass::new(f0 * 1.2, 3.5, sr);
    let mut rosin_bp2 = BandPass::new(f0 * 2.8, 2.8, sr);
    let rosin_end = ms_to_samples(35, sr);

    // ── BODY RESONANCES — VIOLA lebih gelap & warm ────────────────────
    // Frekuensi lebih rendah ~15-20% dari violin karena body lebih besar
    let mut body_a0   = BandPass::new(220.0,  8.0, sr);  // A0 lebih rendah
    let mut body_air  = BandPass::new(320.0,  6.0, sr);  // Air mode
    let mut body_b1lo = BandPass::new(390.0,  7.5, sr);  // B1-
    let mut body_b1hi = BandPass::new(460.0,  6.5, sr);  // B1+
    // Bridge hill viola: lebih rendah & lebih "nasal warm" dari violin
    let mut bridge_lo = BandPass::new(1600.0, 5.0, sr);  // bridge hill lebih rendah
    let mut bridge_hi = BandPass::new(2400.0, 4.0, sr);  // brightness lebih redup
    let mut upper_presence = BandPass::new(3500.0, 3.0, sr); // presence lebih rendah

    // Low-pass lebih agresif dari violin → potong brightness tinggi
    let mut silk_lp = LowPassBiquad::new(7000.0, 0.65, sr);

    // Air shelf lebih rendah & boost kecil → viola tidak "shimmery"
    let mut air_shelf = HighShelf::new(4000.0, 0.8, sr);

    // Warmth boost sedikit lebih besar dari violin (body lebih besar = lebih warm)
    let mut warmth_lp = LowPassBiquad::new(600.0, 0.80, sr);

    // ── BOW CHANGE ───────────────────────────────────────────────────
    let bow_change_pos = ms_to_samples((dur / 2).min(800), sr);
    let bow_change_dur = ms_to_samples(14, sr).max(1);

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // Vibrato viola: lebih lambat & sedikit lebih dalam dari violin
        let vib_grow  = (1.0 - (-t / 0.28).exp()).powf(2.0); // tumbuh lebih lambat
        let vib_amt   = 0.013 * vib_grow; // lebih dalam dari violin (0.010)
        let vib = 1.0 + vib_amt * (2.0 * PI * 4.8 * t).sin() // lebih lambat (violin 5.2Hz)
                      + vib_amt * 0.06 * (2.0 * PI * 9.6 * t).sin();

        let drift = 1.0 + 0.00045 * (2.0 * PI * 0.11 * t).sin(); // drift sedikit lebih besar
        let f_inst = f0 * vib * drift;

        // ── HELMHOLTZ SAWTOOTH ─────────────────────────────────────
        let mut saw = 0.0f32;
        for h in 1..=NH {
            let stretch = (1.0 + b_coeff * (h * h) as f32).sqrt();
            let f_h = f_inst * h as f32 * stretch;
            if f_h >= sr as f32 * 0.46 { break; }

            // Sawtooth 1/h rolloff — sama dengan violin, tapi rolloff lebih cepat di HF
            // (senar tebal = harmonik tinggi lebih redup)
            let amp = match h {
                1 => 1.00,
                2 => 0.50,
                3 => 0.33,
                4 => 0.25,
                5 => 0.19,
                6 => 0.15,
                7 => 0.12,
                8 => 0.09,  // viola lebih redup dari h=8 ke atas
                _ => 0.85 / h as f32, // rolloff lebih cepat dari violin
            };

            let bow_mod = 1.0 + 0.03 * (h as f32 / NH as f32);
            phases[h - 1] += 2.0 * PI * f_h / sr as f32;
            saw += phases[h - 1].sin() * amp * bow_mod;
        }
        saw *= 0.09; // sama dengan violin yang sudah difix

        // ── STICK-SLIP (sama dengan violin) ──────────────────────────
        let bow_vel = saw - bow_state * bow_stiffness;
        let bowed = if bow_vel.abs() < 0.4 {
            bow_state = bow_state * 0.96 + bow_vel * 0.55;
            bow_vel * 1.8 + bow_vel * bow_vel * bow_vel * 0.3
        } else {
            bow_state = bow_state * 0.88;
            let s = saw * 1.6;
            if s > 0.0 {
                s.min(0.85) * 1.0 + s * 0.05
            } else {
                s.max(-0.72) * 1.0 + s * 0.08
            }
        };

        // ── ROSIN TRANSIENT ───────────────────────────────────────────
        let rosin = if i < rosin_end {
            let env_r = (-t / 0.018).exp();
            let n1 = rng.next_f32();
            let n2 = rng.next_f32();
            (rosin_bp1.process(n1) * 0.55 + rosin_bp2.process(n2) * 0.35) * env_r * 0.20
        } else {
            0.0
        };

        // ── BOW DIRECTION CHANGE ──────────────────────────────────────
        let bow_click = if i >= bow_change_pos && i < bow_change_pos + bow_change_dur {
            let t_click = (i - bow_change_pos) as f32 / bow_change_dur as f32;
            let impulse = (PI * t_click).sin();
            impulse * rng.next_f32() * 0.015
        } else {
            0.0
        };

        let raw = bowed + rosin + bow_click;

        // ── BODY RESONANCE (viola = lebih warm, kurang bright) ────────
        let warmth = warmth_lp.process(raw) * 0.06; // sedikit lebih besar dari violin
        let resonated = raw
            + body_a0.process(raw)        * 0.08
            + body_air.process(raw)       * 0.10
            + body_b1lo.process(raw)      * 0.09
            + body_b1hi.process(raw)      * 0.11
            + bridge_lo.process(raw)      * 0.28   // bridge lebih redup dari violin
            + bridge_hi.process(raw)      * 0.18   // brightness lebih kecil dari violin
            + upper_presence.process(raw) * 0.10   // presence lebih kecil
            + warmth;

        let silked  = silk_lp.process(resonated);
        let aired   = air_shelf.process(silked);

        // Saturasi asimetris sama seperti violin
        let driven = aired * 0.65;
        let saturated = if driven > 0.0 {
            1.0 - (-driven).exp()
        } else {
            -1.0 + driven.exp()
        };

        *slot = saturated * env[i] * 1.35;
    }

    out
}

// ====================================================================
// ORGAN — Hammond B3 + Leslie Rotary Speaker Simulation
//
// Arsitektur:
//   1. Tonewheels: 9 drawbar (16', 5⅓', 8', 4', 2⅔', 2', 1⅗', 1⅓', 1')
//   2. Key click: transient attack khas Hammond
//   3. Chorus/Vibrato scanner (pre-Leslie)
//   4. Overdrive tube (pre-amp distortion ringan)
//   5. Leslie cabinet: horn (Doppler pitch + tremolo) + bass rotor
//   6. Cabinet IR simulation: wooden box resonance
// ====================================================================

pub fn synth_organ_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 4);
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    // Preset drawbar: "Full Organ" — 888000000 style jazz/rock
    // [16', 5⅓', 8', 4', 2⅔', 2', 1⅗', 1⅓', 1']
    let drawbars: [f32; 9] = [0.8, 0.8, 1.0, 0.7, 0.5, 0.4, 0.2, 0.1, 0.1];

    let left  = organ_mono_voice(f0, duration_ms, sample_rate, &drawbars, 0x4E5A_C0FF, false);
    let right = organ_mono_voice(f0, duration_ms, sample_rate, &drawbars, 0xBEEF_1234, true);

    // Normalize
    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    let left  = left.into_iter().map(|s| s * scale);
    let right = right.into_iter().map(|s| s * scale);
    left.zip(right).flat_map(|(l, r)| [l, r]).collect()
}

fn organ_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    drawbars: &[f32; 9],
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(100);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── DRAWBAR HARMONICS (tonewheel ratios) ─────────────────────────
    // Ratio relatif terhadap 8' (fundamental):
    //  16'=0.5, 5⅓'=1.5, 8'=1.0, 4'=2.0, 2⅔'=3.0, 2'=4.0, 1⅗'=5.0, 1⅓'=6.0, 1'=8.0
    let tonewheel_ratios: [f32; 9] = [0.5, 1.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0];
    let mut phases = [0.0f32; 9];

    // Tonewheel "leakage" — setiap tonewheel sedikit detuned (magnetic crosstalk)
    let detuning: [f32; 9] = {
        let mut d = [0.0f32; 9];
        for i in 0..9 {
            // ±0.03% random per tonewheel, seeded — konstan selama playback
            let r = (seed.wrapping_mul(0x9E377 + i as u32 * 0x137) >> 8) as f32 / 16777216.0;
            d[i] = 1.0 + (r - 0.5) * 0.0006;
        }
        d
    };

    // ── KEY CLICK — transient attack khas Hammond ─────────────────────
    // Saat key ditekan, relay mekanik membuat "click" ~5ms
    let click_dur = ms_to_samples(6, sr);
    let mut click_bp = BandPass::new(f0 * 8.0, 1.8, sr); // click di frekuensi tinggi
    let mut click_lp = LowPassBiquad::new(3500.0, 0.7, sr);

    // ── SCANNER VIBRATO — efek chorus/vibrato Hammond ─────────────────
    // Delay line untuk scanner: modulasi frekuensi melalui kapasitor scan
    let vib_depth_hz = 6.0_f32; // scanner rate (V1 mode = ~7Hz, C3 = chorus)
    let chorus_delay_samples = ms_to_samples(1, sr).max(1); // ~1ms delay untuk chorus
    let mut delay_buf = vec![0.0f32; chorus_delay_samples * 4];
    let mut delay_idx = 0usize;

    // ── OVERDRIVE — tube pre-amp (ringan untuk jazz, lebih keras untuk rock) ─
    let drive = 1.8f32; // 1.0 = clean, 3.0 = dirty

    // ── LESLIE CABINET ────────────────────────────────────────────────
    // Horn (treble) rotor: ~6.7Hz fast, ~0.7Hz slow
    // Bass rotor: ~1.0Hz fast, ~0.4Hz slow
    // Mode: fast (Tremolo) untuk suara classic Hammond
    let horn_rate  = 6.7_f32;  // Hz
    let bass_rate  = 0.8_f32;  // Hz (lebih lambat dari horn)

    // Leslie Doppler: horn berputar → pitch modulation ±0.6 semitone
    let horn_depth = 0.006_f32;  // depth pitch mod horn
    let bass_depth = 0.003_f32;  // depth pitch mod bass rotor

    // Leslie amplitude modulation (tremolo dari speaker berputar)
    let horn_trem  = 0.45_f32;   // 0=tidak ada, 1=penuh (horn = lebih dalam)
    let bass_trem  = 0.18_f32;   // bass rotor tremolo lebih kecil

    // Phase offset antar L/R untuk efek stereo rotary yang benar
    let leslie_phase_offset = if is_right { PI * 0.5 } else { 0.0 };

    // ── CABINET RESONANCE — wooden Leslie box ─────────────────────────
    let mut cabinet_lp = LowPassBiquad::new(8500.0, 0.72, sr); // HF rolloff kayu
    let mut cabinet_bp = BandPass::new(380.0, 2.5, sr); // wooden box resonance
    let mut horn_bp    = BandPass::new(2800.0, 1.8, sr); // horn driver resonance

    // ── SUSTAIN + RELEASE ENVELOPE ────────────────────────────────────
    // Organ: attack instan (tonewheel langsung on), release pendek
    let release_n = ms_to_samples(80, sr).min(n / 4 + 1);
    let body_n    = n.saturating_sub(release_n);

    // ── MAIN SYNTHESIS LOOP ───────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── LESLIE MODULATION PHASES ─────────────────────────────────
        let horn_lfo_phase  = 2.0 * PI * horn_rate * t + leslie_phase_offset;
        let bass_lfo_phase  = 2.0 * PI * bass_rate * t + leslie_phase_offset * 0.6;

        // Doppler pitch shift (horn berputar mendekati/menjauhi mic)
        let horn_pitch_mod  = 1.0 + horn_depth * horn_lfo_phase.sin();
        let bass_pitch_mod  = 1.0 + bass_depth * bass_lfo_phase.sin();

        // ── SCANNER VIBRATO LFO ───────────────────────────────────────
        // C3 mode (chorus): mix dry + delayed, bukan pure vibrato
        let scanner_lfo = (2.0 * PI * vib_depth_hz * t).sin();

        // ── TONEWHEEL GENERATION ──────────────────────────────────────
        let mut tonewheel_sum = 0.0f32;
        let mut total_weight  = 0.0f32;

        for (k, &ratio) in tonewheel_ratios.iter().enumerate() {
            let db = drawbars[k];
            if db < 0.01 { continue; }

            // Effective frequency: ratio × f0 × detuning × Leslie pitch mod
            // Bass drawbars (16', 8') affected by bass rotor
            // Treble drawbars (4' and above) affected by horn
            let leslie_mod = if ratio <= 1.0 { bass_pitch_mod } else { horn_pitch_mod };

            let f_tw = f0 * ratio * detuning[k] * leslie_mod;
            if f_tw >= sr as f32 * 0.47 { continue; }

            let inc = 2.0 * PI * f_tw / sr as f32;
            phases[k] = (phases[k] + inc) % (2.0 * PI);

            // Pure sine (tonewheel = sine generator, bukan sawtooth)
            // Sedikit 2nd harmonic dari tonewheel imperfection
            let tw_out = phases[k].sin() + phases[k].sin().powi(2) * 0.04;
            tonewheel_sum += tw_out * db;
            total_weight  += db;
        }

        // Normalize drawbar mix
        let raw = if total_weight > 0.01 {
            tonewheel_sum / total_weight.sqrt()
        } else {
            0.0
        };

        // ── KEY CLICK ─────────────────────────────────────────────────
        let click = if i < click_dur {
            let click_env = (1.0 - i as f32 / click_dur as f32).powi(2);
            let noise = rng.next_f32() * 2.0 - 1.0;
            let bp_noise = click_bp.process(noise);
            click_lp.process(bp_noise) * click_env * 0.35
        } else {
            0.0
        };

        let pre_overdrive = raw + click;

        // ── SCANNER CHORUS (C3 mode) ──────────────────────────────────
        // Mix dry + slightly delayed (modulated) → chorus effect
        delay_buf[delay_idx] = pre_overdrive;
        let delayed_idx = (delay_idx + delay_buf.len()
            - (chorus_delay_samples as f32 * (0.5 + 0.5 * scanner_lfo)) as usize)
            % delay_buf.len();
        let delayed = delay_buf[delayed_idx];
        delay_idx = (delay_idx + 1) % delay_buf.len();
        // Chorus: 70% dry + 30% delayed modulated
        let chorused = pre_overdrive * 0.70 + delayed * 0.30;

        // ── TUBE OVERDRIVE ────────────────────────────────────────────
        // Soft asymmetric clip — tube glow, bukan hard clip
        let driven = chorused * drive;
        let overdriven = if driven > 0.0 {
            1.0 - (-driven * 1.2).exp()
        } else {
            -(1.0 - (driven * 0.9).exp())
        };

        // ── LESLIE AMPLITUDE MODULATION (tremolo) ────────────────────
        // Split signal ke horn (HF) dan bass (LF) path, apply leslie per-band

        // Horn path: BPF di ~2.5kHz + amplitude mod dari horn rotor
        let horn_signal = horn_bp.process(overdriven);
        let horn_trem_factor = 1.0 - horn_trem + horn_trem * (0.5 + 0.5 * horn_lfo_phase.cos());
        let horn_out = horn_signal * horn_trem_factor;

        // Bass path: full signal minus horn + bass rotor mod
        let bass_signal = overdriven - horn_signal * 0.6;
        let bass_trem_factor = 1.0 - bass_trem + bass_trem * (0.5 + 0.5 * bass_lfo_phase.cos());
        let bass_out = bass_signal * bass_trem_factor;

        // Recombine horn + bass
        let leslie_out = horn_out * 0.55 + bass_out * 0.72;

        // ── CABINET COLORING ──────────────────────────────────────────
        let cabinet_res = cabinet_bp.process(leslie_out) * 0.08; // wooden box resonance
        let with_cab_res = leslie_out + cabinet_res;
        let cabinet_out = cabinet_lp.process(with_cab_res);

        // ── ENVELOPE (sustain flat, release turun) ────────────────────
        let env_gain = if i < body_n {
            1.0_f32
        } else {
            let t_rel = (i - body_n) as f32 / release_n.max(1) as f32;
            (1.0 - t_rel).max(0.0).powi(2) // kuadrat = lebih smooth
        };

        *slot = cabinet_out * env_gain * 0.78;
    }

    out
}

// ====================================================================
// TRUMPET — Ska / Brass Section Style
//
// Karakteristik ska trumpet:
//   - Attack sangat tajam & bright (lip buzz langsung penuh)
//   - Harmonik kaya: genap DAN ganjil kuat (beda dari sax/clarinet)
//   - Mouthpiece "buzz" transient di awal (~8ms)
//   - Formant bell flare: boost kuat di 1-3kHz (nasal + cutting)
//   - Slight "fall" (pitch turun) di akhir note — ska style
//   - Vibrato hampir nol (ska = straight tone, bukan jazz vibrato)
//   - Overdrive sedikit — lip compression, bukan clean
// ====================================================================

pub fn synth_trumpet_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 4);
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = trumpet_mono_voice(f0, duration_ms, sample_rate, 0x7A1B_C3D4, false);
    let right = trumpet_mono_voice(f0, duration_ms, sample_rate, 0x1234_CAFE, true);

    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    left.into_iter().zip(right.into_iter())
        .flat_map(|(l, r)| [l * scale, r * scale])
        .collect()
}

fn trumpet_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(80);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── ENVELOPE — trumpet ska: attack sangat cepat, sustain flat ─────
    // Beda dari violin (bow swell) — lip langsung buzz penuh
    let attack_n  = ms_to_samples(12, sr);  // sangat cepat, cutting
    let decay_n   = ms_to_samples(18, sr);  // sedikit decay ke sustain
    let sustain   = 0.88f32;
    let release_n = ms_to_samples(55, sr).min(n / 4 + 1);
    let env = adsr_envelope(n, attack_n, decay_n, sustain, release_n);

    // ── SKA "FALL" — pitch turun di 15% akhir note ────────────────────
    // Karakteristik ska: pemain sering "fall off" di akhir phrase
    let fall_start = (n as f32 * 0.85) as usize;
    let fall_semitones = 1.8f32; // turun ~1.8 semitone
    let fall_factor = 2.0_f32.powf(-fall_semitones / 12.0); // < 1.0

    // ── MOUTHPIECE BUZZ TRANSIENT ─────────────────────────────────────
    // Saat lips pertama buzz, ada noise burst ~8ms + lip slap
    let buzz_dur = ms_to_samples(8, sr);
    let mut buzz_bp1 = BandPass::new(f0 * 3.5, 2.5, sr);
    let mut buzz_bp2 = BandPass::new(f0 * 6.0, 2.0, sr);

    // ── LIP BUZZ OSCILLATOR — harmonik trumpet ─────────────────────────
    // Trumpet = cylindrical bore + bell flare → harmonik GENAP + GANJIL kuat
    // Beda dari klarinet (dominan ganjil) dan sax (campuran)
    // Harmonik trumpet khas: 1, 2, 3, 4, 5 semua kuat, lalu rolloff
    const NH: usize = 14;
    let mut phases = [0.0f32; NH];

    // Amplitudo harmonik trumpet (dari analisis FFT recording):
    // H1 fundamental kuat, H2 hampir sama, H3-H5 masih kuat, H6+ rolloff
    let harmonic_amps: [f32; NH] = [
        1.00,  // H1 fundamental
        0.90,  // H2 — trumpet sangat kuat di H2 (beda dari sax)
        0.75,  // H3
        0.55,  // H4
        0.40,  // H5 — bright, cutting
        0.28,  // H6
        0.18,  // H7
        0.12,  // H8
        0.08,  // H9
        0.05,  // H10
        0.03,  // H11
        0.02,  // H12
        0.01,  // H13
        0.01,  // H14
    ];

    // ── BELL FLARE FORMANTS ───────────────────────────────────────────
    // Trumpet bell boost di 1–3kHz (nasal, cutting, "ska-bright")
    let mut bell_low  = BandPass::new((f0 * 2.2).min(1800.0), 3.5, sr);
    let mut bell_mid  = BandPass::new((f0 * 3.8).min(2800.0), 4.0, sr);
    let mut bell_high = BandPass::new((f0 * 6.0).min(4500.0), 3.0, sr);

    // HF rolloff — trumpet bukan cymbal, ada batas kecerahan
    let mut silk_lp = LowPassBiquad::new(9000.0, 0.70, sr);

    // Sedikit presence boost di 3kHz — "cut through the mix" ska style
    let mut presence = HighShelf::new(3200.0, 2.5, sr);

    // ── STEREO: slight detuning L vs R (ensemble feel) ────────────────
    // Pada ska, sering ada 2-3 trumpet — simulasi dengan micro-detune
    let stereo_detune = if is_right { 1.0015 } else { 0.9985 };

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── PITCH: flat sustain + fall di akhir ──────────────────────
        let fall_pitch = if i >= fall_start {
            let t_fall = (i - fall_start) as f32 / (n - fall_start).max(1) as f32;
            // Eksponensial fall — cepat di akhir, bukan linear
            1.0 - (1.0 - fall_factor) * (t_fall * t_fall)
        } else {
            1.0_f32
        };

        // Micro-drift (nafas pemain tidak 100% stabil)
        let drift = 1.0 + 0.00025 * (2.0 * PI * 0.13 * t).sin()
                        + 0.00015 * (2.0 * PI * 0.37 * t).sin();

        // Ska trumpet: hampir NO vibrato (straight, aggressive)
        // Hanya ghost vibrato sangat kecil di note panjang > 300ms
        let vib = if dur > 300 {
            let vib_grow = (1.0 - (-t / 0.25).exp()).min(1.0);
            1.0 + 0.003 * vib_grow * (2.0 * PI * 5.8 * t).sin()
        } else {
            1.0_f32
        };

        let f_inst = f0 * vib * drift * fall_pitch * stereo_detune;

        // ── LIP BUZZ SYNTHESIS ────────────────────────────────────────
        let mut buzz = 0.0f32;
        for h in 1..=NH {
            let f_h = f_inst * h as f32;
            if f_h >= sr as f32 * 0.47 { break; }

            // Lip pressure modulation — sedikit AM dari lip tension
            let lip_mod = 1.0 + 0.02 * ((2.0 * PI * f_inst * 0.5 * t).sin());

            let inc = 2.0 * PI * f_h / sr as f32;
            phases[h - 1] = (phases[h - 1] + inc) % (2.0 * PI);
            buzz += phases[h - 1].sin() * harmonic_amps[h - 1] * lip_mod;
        }
        buzz *= 0.12; // scale down sebelum masuk formant

        // ── MOUTHPIECE BUZZ TRANSIENT ─────────────────────────────────
        let transient = if i < buzz_dur {
            let env_t = (1.0 - i as f32 / buzz_dur as f32).powi(3); // cubic = lebih cepat hilang
            let n1 = rng.next_f32() * 2.0 - 1.0;
            let n2 = rng.next_f32() * 2.0 - 1.0;
            (buzz_bp1.process(n1) * 0.6 + buzz_bp2.process(n2) * 0.4) * env_t * 0.04 // 0.25 → 0.04
        } else {
            0.0
        };

        let raw = buzz + transient;

        // ── BELL RESONANCE ────────────────────────────────────────────
        let bell = bell_low.process(raw)  * 0.55   // warmth + cut
                 + bell_mid.process(raw)  * 0.70   // nasal ska bite
                 + bell_high.process(raw) * 0.30;  // sizzle

        let resonated = raw + bell;

        // ── LIP COMPRESSION (mild overdrive) ─────────────────────────
        // Pemain ska main keras → sedikit compression nonlinear
        let driven = resonated * 1.6;
        let compressed = if driven > 0.0 {
            1.0 - (-driven * 1.1).exp()
        } else {
            -(1.0 - (driven * 0.95).exp())
        };

        // ── SILK + PRESENCE ───────────────────────────────────────────
        let silked   = silk_lp.process(compressed);
        let final_out = presence.process(silked);

        *slot = final_out * env[i] * 1.15;
    }

    out
}

// ====================================================================
// GAMELAN SARON — Bronze Metal Slab, Javanese Tuning
//
// Fisika saron:
//   - Pelat perunggu dipukul pemukul kayu/tanduk → impact transient keras
//   - Inharmonic partials: f1, f2≈2.76×f1, f3≈5.4×f1, f4≈8.9×f1
//     (bukan kelipatan bulat seperti piano/gitar)
//   - Decay: cepat di awal (attack clang), lalu sustain panjang nada utama
//   - "Damping": pemain menahan pelat dengan tangan → note dipotong tiba-tiba
//   - Laras Slendro: 5 nada per oktaf (diapprox dengan token standar)
//   - Resonansi tabung bawah (resonator bambu/logam) → boost fundamental
//   - Shimmer logam: partial tinggi decay sangat cepat
// ====================================================================

pub fn synth_saron_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 4);
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = saron_mono_voice(f0, duration_ms, sample_rate, 0xB41C_0DE0, false);
    let right = saron_mono_voice(f0, duration_ms, sample_rate, 0x7A3E_F1B2, true);

    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    left.into_iter().zip(right.into_iter())
        .flat_map(|(l, r)| [l * scale, r * scale])
        .collect()
}

fn saron_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(150);
    // Saron: note selalu panjang (resonansi alami), dipotong tiba-tiba di akhir
    // Tambah buffer resonansi agar tail terdengar natural
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── INHARMONIC PARTIALS saron perunggu ────────────────────────────
    // Dari riset akustik gamelan (Fletcher & Rossing, Physics of Musical Instruments):
    // Partial ratios untuk saron: 1.0, 2.76, 5.40, 8.93, 13.2
    // Ini jauh dari harmonis (1, 2, 3, 4...) → karakter "metalik khas"
    const NP: usize = 5;
    let partial_ratios: [f32; NP] = [1.000, 2.756, 5.404, 8.933, 13.20];

    // Amplitudo awal tiap partial (dari attack):
    let partial_amp_attack: [f32; NP] = [0.70, 1.00, 0.65, 0.30, 0.12];

    // Decay time tiap partial (detik):
    // - Partial 1 (fundamental): paling panjang (~2-4 detik, tergantung nada)
    // - Partial 2 (2.76×): sedang (~0.8 detik)
    // - Partial 3+: sangat cepat (metalik shimmer)
    let partial_decay: [f32; NP] = [
        (dur as f32 / 1000.0) * 0.85,  // fundamental: hampir sepanjang durasi
        0.55,   // partial 2: decay sedang
        0.18,   // partial 3: cepat
        0.07,   // partial 4: sangat cepat
        0.03,   // partial 5: kilat (attack shimmer only)
    ];

    // Sedikit detune antar L/R untuk stereo image natural
    let stereo_detune = if is_right { 1.0008 } else { 0.9992 };

    // Phase per partial
    let mut phases = [0.0f32; NP];

    // ── MALLET IMPACT TRANSIENT ───────────────────────────────────────
    // Saat pemukul menghantam pelat → broadband impact + clang burst
    // Durasi impact: ~4ms (kayu/tanduk lebih keras dari mallet felt)
    let impact_dur = ms_to_samples(4, sr);
    let mut impact_bp1 = BandPass::new(f0 * 4.5,  2.0, sr); // clang mid
    let mut impact_bp2 = BandPass::new(f0 * 9.0,  1.8, sr); // clang high
    let mut impact_bp3 = BandPass::new(f0 * 0.8,  3.0, sr); // thud low

    // ── RESONATOR TABUNG (bawah pelat) ────────────────────────────────
    // Tabung bambu/logam di bawah pelat → boost fundamental secara resonansi
    // Panjang tabung ≈ λ/4 dari f0 → resonansi di f0 dan 3×f0
    let mut resonator_f1 = BandPass::new(f0,        12.0, sr); // Q tinggi = tabung sempit
    let mut resonator_f3 = BandPass::new(f0 * 3.0,   8.0, sr); // 3rd mode tabung

    // ── BODY COLORING — perunggu gamelan ─────────────────────────────
    // Bronze alloy: bright di awal, warm di sustain
    let mut bronze_lp = LowPassBiquad::new(7000.0, 0.68, sr); // rolloff atas
    let mut bronze_hp = OnePoleHighPass::new(80.0, sr);         // hapus sub-bass

    // ── DAMPING ENVELOPE ──────────────────────────────────────────────
    // Saron: pemain menahan pelat sebelum pukul berikutnya
    // Efek: note dipotong TIBA-TIBA (bukan fade) di akhir durasi
    // Simulasi: sustain penuh → cut tajam di 95% durasi, lalu 5% fade singkat
    let damp_start = (n as f32 * 0.92) as usize;
    let damp_dur   = ms_to_samples(25, sr).min(n.saturating_sub(damp_start) + 1);

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── PARTIAL SYNTHESIS (inharmonic) ───────────────────────────
        let mut partial_sum = 0.0f32;
        for p in 0..NP {
            let f_p = f0 * partial_ratios[p] * stereo_detune;
            if f_p >= sr as f32 * 0.47 { continue; }

            let inc = 2.0 * PI * f_p / sr as f32;
            phases[p] = (phases[p] + inc) % (2.0 * PI);

            // Tiap partial punya decay sendiri (inharmonic metal behavior)
            let partial_env = (-t / partial_decay[p].max(0.001)).exp();
            partial_sum += phases[p].sin() * partial_amp_attack[p] * partial_env;
        }
        partial_sum *= 0.28;

        // ── MALLET IMPACT ─────────────────────────────────────────────
        let impact = if i < impact_dur {
            let env_i = (1.0 - i as f32 / impact_dur as f32).powi(2);
            let noise = rng.next_f32() * 2.0 - 1.0;
            let hi    = rng.next_f32() * 2.0 - 1.0;
            (impact_bp1.process(noise) * 0.50
           + impact_bp2.process(hi)   * 0.35
           + impact_bp3.process(noise) * 0.30) * env_i * 0.55
        } else {
            0.0
        };

        let raw = partial_sum + impact;

        // ── RESONATOR ─────────────────────────────────────────────────
        // Tabung resonator memperkuat fundamental dan third mode
        let resonated = raw
            + resonator_f1.process(raw) * 0.45  // boost fundamental kuat
            + resonator_f3.process(raw) * 0.12; // sedikit boost 3rd

        // ── BRONZE BODY FILTER ────────────────────────────────────────
        let bronzed = bronze_lp.process(resonated);
        let bronzed = bronze_hp.process(bronzed);

        // ── SOFT NONLINEARITY — perunggu tidak linear ─────────────────
        // Bronze plate: sedikit nonlinear saat dipukul keras
        let driven = bronzed * 1.4;
        let soft = if driven > 0.0 {
            1.0 - (-driven * 1.05).exp()
        } else {
            -(1.0 - (driven * 1.05).exp())
        };

        // ── DAMPING (hand stop) ───────────────────────────────────────
        let damp_gain = if i < damp_start {
            1.0_f32
        } else if i < damp_start + damp_dur {
            // Turun sangat cepat — seperti tangan menahan pelat
            let t_d = (i - damp_start) as f32 / damp_dur as f32;
            (1.0 - t_d).powi(4) // sangat steep — bukan fade, tapi "choke"
        } else {
            0.0
        };

        *slot = soft * damp_gain * 1.10;
    }

    out
}

// ====================================================================
// GAMELAN DEMUNG — Bronze Slab, Larger & Lower than Saron
//
// Perbedaan dari saron:
//   - Pelat lebih besar/tebal → inharmonic partials lebih rapat (ratio berbeda)
//   - Decay fundamental jauh lebih panjang (massa besar = sustain panjang)
//   - Impact lebih berat → thud low lebih dominan
//   - Partial tinggi sangat cepat hilang (ukuran besar = damping internal tinggi)
//   - Resonator tabung lebih panjang → Q sangat tinggi, boost fundamental kuat
//   - Karakter: "gong-like", warm, boomy — tulang punggung melodi gamelan
// ====================================================================

pub fn synth_demung_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 3); // demung: satu oktaf di bawah saron
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = demung_mono_voice(f0, duration_ms, sample_rate, 0xDEA0_B33F, false);
    let right = demung_mono_voice(f0, duration_ms, sample_rate, 0x4A7C_1E2D, true);

    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    left.into_iter().zip(right.into_iter())
        .flat_map(|(l, r)| [l * scale, r * scale])
        .collect()
}

fn demung_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(200);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── INHARMONIC PARTIALS demung ─────────────────────────────────────
    // Pelat lebih besar → partial ratio berbeda dari saron:
    // Ratio demung lebih "compact" — partial 2 lebih dekat ke fundamental
    // karena thickness ratio berbeda (thicker plate = lower partial ratios)
    const NP: usize = 5;
    let partial_ratios: [f32; NP] = [1.000, 2.620, 4.980, 8.120, 11.85];

    // Amplitudo: fundamental JAUH lebih dominan dari saron
    // Pelat besar = energi tersimpan di fundamental, bukan shimmer tinggi
    let partial_amp: [f32; NP] = [1.00, 0.55, 0.22, 0.08, 0.03];

    // Decay demung: fundamental sangat panjang, partial tinggi sangat pendek
    let partial_decay: [f32; NP] = [
        (dur as f32 / 1000.0) * 0.92, // fundamental: hampir full durasi
        0.80,   // partial 2: lebih panjang dari saron (massa lebih besar)
        0.22,   // partial 3
        0.06,   // partial 4: kilat
        0.025,  // partial 5: hampir hanya di impact
    ];

    // Micro-detune stereo
    let stereo_detune = if is_right { 1.0006 } else { 0.9994 };

    let mut phases = [0.0f32; NP];

    // ── MALLET IMPACT — lebih berat dari saron ────────────────────────
    // Pemukul demung lebih besar → impact lebih lama (~7ms) dan lebih low
    let impact_dur = ms_to_samples(7, sr);
    let mut impact_thud = BandPass::new(f0 * 0.7,  2.5, sr); // thud sub-low
    let mut impact_body = BandPass::new(f0 * 2.8,  2.0, sr); // body clang
    let mut impact_hi   = BandPass::new(f0 * 6.5,  1.8, sr); // shimmer (kecil)
    let mut impact_lp   = LowPassBiquad::new(4000.0, 0.7, sr); // potong HF impact

    // ── RESONATOR TABUNG — lebih panjang dari saron ───────────────────
    // Tabung demung lebih panjang → resonansi f0 lebih sharp (Q lebih tinggi)
    let mut resonator_f1 = BandPass::new(f0,       16.0, sr); // Q sangat tinggi
    let mut resonator_f3 = BandPass::new(f0 * 2.8,  9.0, sr); // 3rd mode tabung

    // ── BRONZE BODY — pelat lebih tebal = lebih warm ─────────────────
    let mut bronze_lp = LowPassBiquad::new(5500.0, 0.65, sr); // rolloff lebih rendah dari saron
    let mut bronze_hp = OnePoleHighPass::new(55.0, sr);         // buang sub-bass

    // Warmth boost: demung lebih "gong-like" → sedikit mid-low boost
    let mut warmth_bp = BandPass::new(f0 * 1.5, 4.0, sr);

    // ── DAMPING ENVELOPE ──────────────────────────────────────────────
    // Demung juga di-damp tangan, tapi kadang dibiarkan lebih panjang
    // Choke di 94% durasi (lebih panjang dari saron 92%)
    let damp_start = (n as f32 * 0.94) as usize;
    let damp_dur   = ms_to_samples(35, sr).min(n.saturating_sub(damp_start) + 1);

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── PARTIAL SYNTHESIS ─────────────────────────────────────────
        let mut partial_sum = 0.0f32;
        for p in 0..NP {
            let f_p = f0 * partial_ratios[p] * stereo_detune;
            if f_p >= sr as f32 * 0.47 { continue; }

            let inc = 2.0 * PI * f_p / sr as f32;
            phases[p] = (phases[p] + inc) % (2.0 * PI);

            let partial_env = (-t / partial_decay[p].max(0.001)).exp();
            partial_sum += phases[p].sin() * partial_amp[p] * partial_env;
        }
        partial_sum *= 0.32;

        // ── MALLET IMPACT ─────────────────────────────────────────────
        let impact = if i < impact_dur {
            let env_i = (1.0 - i as f32 / impact_dur as f32).powi(2);
            let noise = rng.next_f32() * 2.0 - 1.0;
            let lo    = rng.next_f32() * 2.0 - 1.0;
            let raw_impact = impact_thud.process(lo)   * 0.65  // thud dominan
                           + impact_body.process(noise) * 0.40
                           + impact_hi.process(noise)   * 0.15; // shimmer minimal
            impact_lp.process(raw_impact) * env_i * 0.70 // lebih keras dari saron
        } else {
            0.0
        };

        let raw = partial_sum + impact;

        // ── RESONATOR ─────────────────────────────────────────────────
        let warmth = warmth_bp.process(raw) * 0.18; // gong-like warmth
        let resonated = raw
            + resonator_f1.process(raw) * 0.60  // fundamental sangat kuat
            + resonator_f3.process(raw) * 0.15
            + warmth;

        // ── BRONZE FILTER ─────────────────────────────────────────────
        let bronzed = bronze_lp.process(resonated);
        let bronzed = bronze_hp.process(bronzed);

        // ── NONLINEARITY — pelat tebal, pukul keras ───────────────────
        // Demung dipukul lebih keras → nonlinearity lebih terasa
        let driven = bronzed * 1.65;
        let soft = if driven > 0.0 {
            1.0 - (-driven * 1.08).exp()
        } else {
            -(1.0 - (driven * 1.08).exp())
        };

        // ── DAMPING ───────────────────────────────────────────────────
        let damp_gain = if i < damp_start {
            1.0_f32
        } else if i < damp_start + damp_dur {
            let t_d = (i - damp_start) as f32 / damp_dur as f32;
            (1.0 - t_d).powi(4)
        } else {
            0.0
        };

        *slot = soft * damp_gain * 1.15;
    }

    out
}

// ====================================================================
// GAMELAN BONANG — Bronze Pot-Gong (Pencu/Knob Gong)
//
// Perbedaan fundamental dari saron/demung (pelat datar):
//   - Bentuk pot dengan pencu (knob) di tengah → fisika getaran berbeda total
//   - Partial inharmonik: f1, f2≈3.01×f1, f3≈4.85×f1 (berbeda dari slab)
//   - Pencu mode: ada "bonang knob mode" di ~1.5×f1 yang khas
//   - Sustain sangat panjang (gong shape = self-reinforcing resonance)
//   - Beating antara partial → suara "bergoyang" khas bonang
//   - Dimainkan dengan tabuh berlapis kulit → impact lebih soft dari saron
//   - Tidak di-damp tangan (berbeda dari saron) → sustain penuh
//   - Resonator tabung: Q tinggi, lebih panjang dari saron
// ====================================================================

pub fn synth_bonang_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 4);
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = bonang_mono_voice(f0, duration_ms, sample_rate, 0xB0E0_4321, false);
    let right = bonang_mono_voice(f0, duration_ms, sample_rate, 0xC0FF_A1B2, true);

    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    left.into_iter().zip(right.into_iter())
        .flat_map(|(l, r)| [l * scale, r * scale])
        .collect()
}

fn bonang_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(300);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── INHARMONIC PARTIALS bonang (pot-gong shape) ───────────────────
    // Dari riset: Rahn (1996), Rossing & Hampton "Acoustics of Gamelan"
    // Pot-gong partials sangat berbeda dari slab (saron):
    //   Mode 1 (fundamental): pencu rocking mode
    //   Mode 2 (pencu knob): ~1.52× — khas bonang, tidak ada di saron
    //   Mode 3: ~3.01× — ring mode pertama
    //   Mode 4: ~4.85× — ring mode kedua
    //   Mode 5: ~7.20× — shimmer
    const NP: usize = 5;
    let partial_ratios: [f32; NP] = [1.000, 1.520, 3.010, 4.850, 7.200];

    // Amplitudo: mode pencu (1.52×) sangat kuat — ini yang bikin bonang unik
    let partial_amp: [f32; NP] = [1.00, 0.82, 0.45, 0.20, 0.06];

    // Decay bonang: JAUH lebih panjang dari saron (gong shape = no damping)
    // Fundamental bisa 4-8 detik, tapi kita clamp ke durasi
    let base_decay = (dur as f32 / 1000.0).min(6.0);
    let partial_decay: [f32; NP] = [
        base_decay * 0.95,   // fundamental: hampir full
        base_decay * 0.80,   // pencu mode: sedikit lebih pendek
        base_decay * 0.45,   // ring mode 1
        base_decay * 0.20,   // ring mode 2
        0.08,                // shimmer: sangat cepat
    ];

    // ── BEATING — khas bonang "bergoyang" ─────────────────────────────
    // Bonang asli punya slight detuning antara dua "sisi" pot
    // Ini menciptakan beating (amplitudo modulation) ~2-5 Hz
    // Setiap partial punya dua fase: satu normal, satu sedikit detuned
    let beat_rate: [f32; NP] = [2.8, 3.4, 4.1, 5.2, 6.0]; // Hz beating per partial
    let beat_depth: [f32; NP] = [0.12, 0.18, 0.10, 0.07, 0.04]; // depth AM

    // Stereo: L/R phase offset di beating → bonang "berputar" di stereo field
    let beat_phase_offset = if is_right { PI * 0.6 } else { 0.0 };

    // Micro-detune antar L/R
    let stereo_detune = if is_right { 1.0004 } else { 0.9996 };

    let mut phases = [0.0f32; NP];
    // Phase kedua untuk beating simulation
    let mut phases_b = [0.0f32; NP]; // sedikit detuned

    // ── TABUH IMPACT — berlapis kulit, lebih soft dari saron ──────────
    // Tabuh bonang dibungkus kulit/karet → impact lebih bulat, kurang transient
    let impact_dur = ms_to_samples(9, sr); // lebih panjang dari saron (tabuh lebih soft)
    let mut impact_knob = BandPass::new(f0 * 1.5,  3.0, sr); // pencu mode impact
    let mut impact_body = BandPass::new(f0 * 3.0,  2.5, sr); // body resonance
    let mut impact_lp   = LowPassBiquad::new(3500.0, 0.72, sr); // kulit meredam HF

    // ── RESONATOR TABUNG bonang ────────────────────────────────────────
    // Tabung bonang: diameter lebih kecil dari saron → Q lebih tinggi
    let mut resonator_f1   = BandPass::new(f0,        18.0, sr); // Q sangat tinggi
    let mut resonator_knob = BandPass::new(f0 * 1.52, 12.0, sr); // pencu mode boost
    let mut resonator_ring = BandPass::new(f0 * 3.01,  8.0, sr); // ring mode

    // ── BODY FILTER — perunggu pot, bukan slab ────────────────────────
    // Pot shape → lebih warm di mid, kurang brightness dari saron
    let mut body_lp  = LowPassBiquad::new(6500.0, 0.68, sr);
    let mut body_hp  = OnePoleHighPass::new(45.0, sr);
    let mut body_mid = BandPass::new(f0 * 2.2, 3.5, sr); // pot cavity resonance

    // ── TIDAK ADA DAMPING — bonang tidak di-damp tangan ──────────────
    // Berbeda dari saron/demung, bonang dibiarkan sustain penuh
    // Hanya envelope release di akhir durasi (pemain pukul berikutnya)
    let release_n = ms_to_samples(120, sr).min(n / 5 + 1);
    let body_n    = n.saturating_sub(release_n);

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── PARTIAL SYNTHESIS dengan BEATING ─────────────────────────
        let mut partial_sum = 0.0f32;
        for p in 0..NP {
            let f_p  = f0 * partial_ratios[p] * stereo_detune;
            // Frekuensi kedua: sedikit lebih tinggi untuk beating
            let f_p2 = f_p * (1.0 + beat_rate[p] / (f_p + 0.001));
            if f_p >= sr as f32 * 0.47 { continue; }

            let inc  = 2.0 * PI * f_p  / sr as f32;
            let inc2 = 2.0 * PI * f_p2 / sr as f32;
            phases[p]   = (phases[p]   + inc)  % (2.0 * PI);
            phases_b[p] = (phases_b[p] + inc2) % (2.0 * PI);

            let partial_env = (-t / partial_decay[p].max(0.001)).exp();

            // Beating: mix dua osilator sedikit detuned
            // + phase offset untuk stereo rotation
            let osc_a = phases[p].sin();
            let osc_b = (phases_b[p] + beat_phase_offset).sin();
            let beat_mix = osc_a * (1.0 - beat_depth[p] * 0.5)
                         + osc_b * beat_depth[p] * 0.5;

            partial_sum += beat_mix * partial_amp[p] * partial_env;
        }
        partial_sum *= 0.30;

        // ── TABUH IMPACT ──────────────────────────────────────────────
        let impact = if i < impact_dur {
            // Kulit tabuh: envelope lebih lambat naik dari saron (softer attack)
            let env_i = {
                let t_i = i as f32 / impact_dur as f32;
                // Bell curve — bukan langsung langsung drop
                let up   = (t_i * 3.0).min(1.0);
                let down = 1.0 - ((t_i - 0.3).max(0.0) / 0.7).powi(2);
                up * down
            };
            let noise = rng.next_f32() * 2.0 - 1.0;
            let mid   = rng.next_f32() * 2.0 - 1.0;
            let raw_impact = impact_knob.process(mid)   * 0.55  // pencu impact dominan
                           + impact_body.process(noise) * 0.35
                           + partial_sum * 0.40; // partial juga aktif di impact
            impact_lp.process(raw_impact) * env_i * 0.65
        } else {
            0.0
        };

        // Blend: di impact, gabungkan; setelah impact, hanya partial
        let raw = if i < impact_dur {
            partial_sum * 0.6 + impact
        } else {
            partial_sum
        };

        // ── RESONATOR ─────────────────────────────────────────────────
        let resonated = raw
            + resonator_f1.process(raw)   * 0.50  // fundamental boost kuat
            + resonator_knob.process(raw) * 0.40  // pencu mode — khas bonang
            + resonator_ring.process(raw) * 0.18; // ring mode

        // ── POT CAVITY MID BOOST ──────────────────────────────────────
        let cavity = body_mid.process(resonated) * 0.15;
        let with_cavity = resonated + cavity;

        // ── BODY FILTER ───────────────────────────────────────────────
        let filtered = body_lp.process(with_cavity);
        let filtered = body_hp.process(filtered);

        // ── SOFT NONLINEARITY — perunggu pot ─────────────────────────
        let driven = filtered * 1.45;
        let soft = if driven > 0.0 {
            1.0 - (-driven * 1.06).exp()
        } else {
            -(1.0 - (driven * 1.06).exp())
        };

        // ── SUSTAIN ENVELOPE (release saja, tidak ada hand-damp) ─────
        let env_gain = if i < body_n {
            1.0_f32
        } else {
            let t_rel = (i - body_n) as f32 / release_n.max(1) as f32;
            (1.0 - t_rel.min(1.0)).powi(3)
        };

        *slot = soft * env_gain * 1.05;
    }

    out
}

// ====================================================================
// GAMELAN KENONG — Horizontal Kettle-Gong on Rope in Wooden Rack
//
// Fisika akurat berdasarkan konstruksi nyata:
//   - Pot gong duduk HORIZONTAL di atas tali dalam rak kayu (rancak)
//   - Rim terbuka menghadap ke BAWAH → ruang udara dalam pot = Helmholtz
//     cavity resonator internal (berbeda dari bonang yang rim menghadap atas)
//   - Tali meredam getaran rim bawah → sustain fundamental lebih bersih
//     (rim tidak berkontribusi banyak ke decay, hanya pencu yang dominan)
//   - Boss/pencu dipukul tabuh kayu berlapis tipis (lebih sharp dari bonang)
//   - Ukuran besar (20-40cm, ~7kg) → fundamental lebih rendah, sustain sangat panjang
//   - Cavity Helmholtz boost: ruang udara dalam pot resonansi di f0
//   - Peran struktural: penanda kenongan — dipukul kuat, bobot penuh
//   - Damping: kadang disentuh ringan tangan setelah dipukul (opsional)
// ====================================================================

pub fn synth_kenong_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 3); // kenong: lebih rendah dari bonang
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = kenong_mono_voice(f0, duration_ms, sample_rate, 0xCE00_A1D3, false);
    let right = kenong_mono_voice(f0, duration_ms, sample_rate, 0x5F7B_E29C, true);

    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    left.into_iter().zip(right.into_iter())
        .flat_map(|(l, r)| [l * scale, r * scale])
        .collect()
}

fn kenong_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(600);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── INHARMONIC PARTIALS kenong ─────────────────────────────────────
    // Pot-gong horizontal, rim bawah diredam tali → pencu mode paling dominan
    // Cavity Helmholtz internal boost fundamental lebih dari bonang
    // Partial ratios kenong (lebih dekat ke gong ageng daripada bonang):
    const NP: usize = 6;
    let partial_ratios: [f32; NP] = [
        1.000,  // fundamental — rim mode, dikuatkan Helmholtz cavity
        1.465,  // pencu mode — sedikit lebih rendah dari bonang (pot lebih dalam)
        2.880,  // ring mode 1
        4.550,  // ring mode 2
        6.720,  // shimmer
        9.100,  // ultra-shimmer (hilang sangat cepat)
    ];

    // Kenong: fundamental & pencu mode SANGAT dominan
    // Rim diredam tali → energy di ring mode lebih kecil dari bonang
    let partial_amp: [f32; NP] = [1.00, 0.88, 0.32, 0.12, 0.04, 0.01];

    // Decay: fundamental sangat panjang (tali meredam rim, bukan pencu)
    let base_decay = (dur as f32 / 1000.0).min(7.0);
    let partial_decay: [f32; NP] = [
        base_decay * 0.96,  // fundamental: hampir full durasi
        base_decay * 0.85,  // pencu: sedikit lebih pendek
        base_decay * 0.35,  // ring mode 1: sedang
        base_decay * 0.14,  // ring mode 2: cepat
        0.06,               // shimmer: kilat
        0.02,               // ultra-shimmer: hanya di impact
    ];

    // ── BEATING — lebih lambat dari bonang (ukuran lebih besar) ────────
    let beat_rate:  [f32; NP] = [1.6, 2.1, 2.8, 3.5, 4.5, 5.5];
    let beat_depth: [f32; NP] = [0.09, 0.14, 0.08, 0.04, 0.02, 0.01];
    let beat_phase_offset = if is_right { PI * 0.55 } else { 0.0 };
    let stereo_detune = if is_right { 1.0003 } else { 0.9997 };

    let mut phases   = [0.0f32; NP];
    let mut phases_b = [0.0f32; NP];

    // ── TABUH IMPACT — kayu berlapis tipis, lebih sharp dari bonang ───
    // Berlapis tipis (bukan tebal seperti bonang) → attack lebih defined
    // tapi tetap lebih soft dari saron/demung (masih ada lapisan kulit)
    let impact_dur  = ms_to_samples(6, sr);
    let mut impact_pencu = BandPass::new(f0 * 1.47, 3.5, sr); // pencu mode impact
    let mut impact_body  = BandPass::new(f0 * 2.88, 2.8, sr); // body impact
    let mut impact_sharp = BandPass::new(f0 * 5.5,  2.0, sr); // tabuh tip (tipis = ada sedikit HF)
    let mut impact_lp    = LowPassBiquad::new(4500.0, 0.70, sr);

    // ── HELMHOLTZ CAVITY RESONATOR (ruang udara dalam pot) ────────────
    // Rim menghadap bawah → udara terperangkap dalam pot
    // Helmholtz resonance ≈ f0 (kontraktor desain pot memang untuk ini)
    // Q tinggi karena leher "cavity" sempit (hanya celah antara rim & lantai rak)
    let mut helmholtz = BandPass::new(f0, 14.0, sr);       // cavity boost fundamental
    let mut helm_2nd  = BandPass::new(f0 * 1.47, 10.0, sr); // pencu mode juga di-boost cavity

    // ── TALI ROPE COUPLING — meredam rim, filter selektif ─────────────
    // Tali menyerap energi di sekitar titik kontak (rim bawah)
    // Efek: sedikit high-pass pada sinyal (bass rim diredam, pencu tetap)
    let mut rope_hp  = OnePoleHighPass::new(f0 * 0.4, sr);

    // ── BODY FILTER — perunggu pot besar ─────────────────────────────
    let mut body_lp  = LowPassBiquad::new(5800.0, 0.66, sr);
    let mut body_hp  = OnePoleHighPass::new(40.0, sr); // buang sub yang tidak perlu
    let mut body_mid = BandPass::new(f0 * 1.8, 4.5, sr); // pot cavity mid resonance

    // ── SUSTAIN + OPTIONAL HAND-DAMP ──────────────────────────────────
    // Kenong kadang di-damp tangan, kadang tidak → kita simulasikan
    // sebagai release smooth di akhir (tanpa choke keras seperti saron)
    let release_n = ms_to_samples(180, sr).min(n / 4 + 1);
    let body_n    = n.saturating_sub(release_n);

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── PARTIAL SYNTHESIS dengan BEATING ─────────────────────────
        let mut partial_sum = 0.0f32;
        for p in 0..NP {
            let f_p  = f0 * partial_ratios[p] * stereo_detune;
            let f_p2 = f_p * (1.0 + beat_rate[p] / (f_p + 0.001));
            if f_p >= sr as f32 * 0.47 { continue; }

            let inc  = 2.0 * PI * f_p  / sr as f32;
            let inc2 = 2.0 * PI * f_p2 / sr as f32;
            phases[p]   = (phases[p]   + inc)  % (2.0 * PI);
            phases_b[p] = (phases_b[p] + inc2) % (2.0 * PI);

            let partial_env = (-t / partial_decay[p].max(0.001)).exp();

            let osc_a   = phases[p].sin();
            let osc_b   = (phases_b[p] + beat_phase_offset).sin();
            let beat_mix = osc_a * (1.0 - beat_depth[p] * 0.5)
                         + osc_b *  beat_depth[p] * 0.5;

            partial_sum += beat_mix * partial_amp[p] * partial_env;
        }
        partial_sum *= 0.32;

        // ── TABUH IMPACT ──────────────────────────────────────────────
        let impact = if i < impact_dur {
            // Bell-curve envelope: naik cepat lalu turun
            let t_i  = i as f32 / impact_dur as f32;
            let up   = (t_i * 4.0).min(1.0);
            let down = 1.0 - ((t_i - 0.25).max(0.0) / 0.75).powi(2);
            let env_i = up * down;

            let noise = rng.next_f32() * 2.0 - 1.0;
            let hi    = rng.next_f32() * 2.0 - 1.0;
            let raw_impact =
                impact_pencu.process(noise) * 0.60  // pencu paling dominan
              + impact_body.process(noise)  * 0.35
              + impact_sharp.process(hi)    * 0.20  // sedikit HF dari lapisan tipis
              + partial_sum * 0.35;                 // partial langsung aktif
            impact_lp.process(raw_impact) * env_i * 0.72
        } else {
            0.0
        };

        let raw = if i < impact_dur {
            partial_sum * 0.65 + impact
        } else {
            partial_sum
        };

        // ── ROPE COUPLING: filter selektif rim ────────────────────────
        let rope_filtered = rope_hp.process(raw);

        // ── HELMHOLTZ CAVITY BOOST ────────────────────────────────────
        // Ruang udara dalam pot memperkuat fundamental & pencu mode
        let cavity_boost =
            helmholtz.process(rope_filtered) * 0.55   // fundamental sangat kuat
          + helm_2nd.process(rope_filtered)  * 0.38;  // pencu mode boost
        let with_cavity = rope_filtered + cavity_boost;

        // ── POT MID RESONANCE ─────────────────────────────────────────
        let mid_res  = body_mid.process(with_cavity) * 0.12;
        let resonated = with_cavity + mid_res;

        // ── BRONZE BODY FILTER ────────────────────────────────────────
        let filtered = body_lp.process(resonated);
        let filtered = body_hp.process(filtered);

        // ── NONLINEARITY — kenong dipukul kuat (penanda struktural) ───
        let driven = filtered * 1.55;
        let soft = if driven > 0.0 {
            1.0 - (-driven * 1.07).exp()
        } else {
            -(1.0 - (driven * 1.07).exp())
        };

        // ── ENVELOPE — sustain penuh, release smooth (hand-damp ringan) ─
        let env_gain = if i < body_n {
            1.0_f32
        } else {
            let t_rel = (i - body_n) as f32 / release_n.max(1) as f32;
            // Lebih smooth dari saron (choke), lebih cepat dari bonang
            (1.0 - t_rel.min(1.0)).powi(2)
        };

        *slot = soft * env_gain * 1.08;
    }

    out
}

// ====================================================================
// GAMELAN KEMPUL — Hanging Vertical Gong (Small-Medium)
//
// Fisika akurat berdasarkan konstruksi nyata:
//   - DIGANTUNG VERTIKAL dari rak kayu via tali di flange — bebas berayun
//   - Diameter 19-25cm, lebih kecil dari kenong → pitched lebih tinggi
//   - Tabuh berlapis SANGAT TEBAL → attack paling soft di antara semua gong
//   - Partial HAMPIR HARMONIS (IEEE research: 1×, 2×, 3×, 4× fundamental)
//     — berbeda dari kenong/bonang yang sangat inharmonis
//   - Swing mode: gong berayun saat dipukul → micro-pitch wobble ~0.5Hz awal
//   - Digantung bebas → tidak ada rope damping di rim (berbeda dari kenong)
//     → sustain sangat panjang, decay natural exponential
//   - Kadang di-damp tangan antara pukulan (colotomic role)
//   - Fungsi: penanda tengah kenongan — lebih sering dari kenong
// ====================================================================

pub fn synth_kempul_note(token: &str, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
    let f0 = single_note_freq(token, 3);
    if f0 < 1.0 { return vec![0.0; ms_to_samples(duration_ms, sample_rate) * 2]; }

    let left  = kempul_mono_voice(f0, duration_ms, sample_rate, 0xF1A3_C05E, false);
    let right = kempul_mono_voice(f0, duration_ms, sample_rate, 0x2B7D_E49A, true);

    let peak = left.iter().chain(right.iter()).map(|s| s.abs()).fold(0.0f32, f32::max);
    let scale = if peak > 0.88 { 0.88 / peak } else { 1.0 };

    left.into_iter().zip(right.into_iter())
        .flat_map(|(l, r)| [l * scale, r * scale])
        .collect()
}

fn kempul_mono_voice(
    f0: f32,
    duration_ms: u64,
    sample_rate: u32,
    seed: u32,
    is_right: bool,
) -> Vec<f32> {
    let dur = duration_ms.max(400);
    let n   = ms_to_samples(dur, sample_rate);
    let sr  = sample_rate;
    let mut out = vec![0.0f32; n];
    let mut rng = Rng::new(seed);

    // ── PARTIALS KEMPUL — hampir harmonis (IEEE research finding) ─────
    // Berbeda dari semua gamelan sebelumnya yang sangat inharmonis
    // Gong vertikal kecil → partial mendekati integer multiple
    // Dari pengukuran akustik: 1×, 2×, 3×, 4× dengan sedikit stretch
    const NP: usize = 6;
    let partial_ratios: [f32; NP] = [
        1.000,   // fundamental
        1.998,   // ≈ 2× (sedikit flat dari oktaf sempurna)
        2.996,   // ≈ 3× (hampir perfect fifth dari partial 2)
        3.995,   // ≈ 4×
        5.420,   // partial 5: mulai inharmonis di sini
        7.180,   // partial 6: shimmer
    ];

    // Amplitudo: fundamental & partial 2 dominan (hampir harmonis = partial genap kuat)
    let partial_amp: [f32; NP] = [1.00, 0.72, 0.45, 0.28, 0.10, 0.04];

    // Decay: hampir harmonis → energi lebih merata, fundamental panjang
    let base_decay = (dur as f32 / 1000.0).min(6.0);
    let partial_decay: [f32; NP] = [
        base_decay * 0.95,  // fundamental: sangat panjang
        base_decay * 0.80,  // partial 2
        base_decay * 0.58,  // partial 3
        base_decay * 0.35,  // partial 4
        0.12,               // partial 5: cepat
        0.05,               // partial 6: kilat
    ];

    // ── BEATING — kempul punya beating dari partial hampir-harmonis ───
    // Karena partial tidak persis integer, ada beating antar partial
    // Beat rate: selisih frekuensi antara partial dan integer sempurna
    // Partial 2 di 1.998× → beating dengan 2.000× = 0.002 × f0 Hz
    let beat_hz: [f32; NP] = [
        0.0,                    // fundamental: tidak ada beating
        0.002 * f0,             // partial 2: beating sangat lambat
        0.004 * f0,             // partial 3
        0.005 * f0,             // partial 4
        0.42 * f0,              // partial 5: inharmonis, beat cepat
        0.18 * f0,              // partial 6
    ];
    let beat_depth: [f32; NP] = [0.0, 0.08, 0.10, 0.12, 0.06, 0.03];

    // ── SWING MODE — gong berayun setelah dipukul ─────────────────────
    // Kempul digantung bebas → berayun ~0.3-0.8 Hz setelah impact
    // Efek: micro pitch-wobble di awal (50-200ms pertama), lalu settle
    // Berbeda dari vibrato — ini fisika pendulum, bukan lip/bow
    let swing_rate = 0.5_f32; // Hz — pendulum gong ~50cm tali
    let swing_depth = 0.0018_f32; // sangat kecil, hanya terasa di awal
    let swing_decay = 0.8_f32; // detik — pendulum redaman cepat

    // Stereo: L/R swing phase berbeda → efek gong "bergerak" di stereo
    let swing_phase_offset = if is_right { PI * 0.4 } else { 0.0 };
    let stereo_detune = if is_right { 1.0005 } else { 0.9995 };

    let mut phases   = [0.0f32; NP];
    let mut phases_b = [0.0f32; NP]; // untuk beating

    // ── TABUH IMPACT — paling soft dari semua gamelan ─────────────────
    // "Heavily padded wooden stick" → impact sangat lembut
    // Hampir tidak ada transient HF — hanya body low-mid
    let impact_dur  = ms_to_samples(12, sr); // lebih lama dari kenong (6ms) karena padding tebal
    let mut impact_boss = BandPass::new(f0 * 1.5,  4.0, sr); // boss/pencu resonance
    let mut impact_body = BandPass::new(f0 * 0.8,  3.0, sr); // body thud
    let mut impact_lp   = LowPassBiquad::new(2800.0, 0.68, sr); // potong semua HF (heavily padded)

    // ── FREE-HANGING RESONANCE — tidak ada rope damping di rim ────────
    // Kempul digantung via flange, bukan rim — getaran bebas
    // Tidak perlu rope_hp seperti kenong
    // Tapi perlu model "flange suspension": sedikit notch di f0×0.6
    // (titik suspensi mempengaruhi mode tertentu)
    let mut flange_notch = BandPass::new(f0 * 0.62, 6.0, sr); // mode yang diredam suspensi

    // ── RESONATOR: tidak ada Helmholtz cavity (rim menghadap atas) ────
    // Kempul digantung vertikal → rim menghadap ke SAMPING, bukan bawah
    // Tidak ada Helmholtz cavity seperti kenong
    // Tapi ada "air column" di dalam pot yang mengikuti mode partial
    let mut air_col_f1 = BandPass::new(f0,        10.0, sr);
    let mut air_col_f2 = BandPass::new(f0 * 2.0,   7.0, sr);

    // ── BODY FILTER — perunggu gong vertikal ─────────────────────────
    let mut body_lp = LowPassBiquad::new(6200.0, 0.67, sr);
    let mut body_hp = OnePoleHighPass::new(50.0, sr);

    // ── ENVELOPE: attack SANGAT soft (heavily padded tabuh) ───────────
    // Berbeda dari saron/demung/kenong — attack terasa seperti "bloom"
    // Rise time ~20ms sebelum mencapai peak (tabuh padding menyerap impact)
    let bloom_n   = ms_to_samples(20, sr); // rise time
    let release_n = ms_to_samples(200, sr).min(n / 4 + 1);
    let body_n    = n.saturating_sub(release_n);

    // ── MAIN LOOP ─────────────────────────────────────────────────────
    for (i, slot) in out.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;

        // ── SWING MODE pitch modulation ───────────────────────────────
        let swing_env = (-t / swing_decay).exp();
        let swing = swing_depth * swing_env
            * (2.0 * PI * swing_rate * t + swing_phase_offset).sin();

        // ── PARTIAL SYNTHESIS dengan beating ─────────────────────────
        let mut partial_sum = 0.0f32;
        for p in 0..NP {
            let f_p  = f0 * partial_ratios[p] * stereo_detune * (1.0 + swing);
            let f_p2 = f_p + beat_hz[p]; // frekuensi kedua untuk beating
            if f_p >= sr as f32 * 0.47 { continue; }

            let inc  = 2.0 * PI * f_p  / sr as f32;
            let inc2 = 2.0 * PI * f_p2 / sr as f32;
            phases[p]   = (phases[p]   + inc)  % (2.0 * PI);
            phases_b[p] = (phases_b[p] + inc2) % (2.0 * PI);

            let partial_env = (-t / partial_decay[p].max(0.001)).exp();

            let osc_a    = phases[p].sin();
            let osc_b    = phases_b[p].sin();
            let beat_mix = osc_a * (1.0 - beat_depth[p] * 0.5)
                         + osc_b *  beat_depth[p] * 0.5;

            partial_sum += beat_mix * partial_amp[p] * partial_env;
        }
        partial_sum *= 0.30;

        // ── TABUH IMPACT ──────────────────────────────────────────────
        // Sangat soft — hampir blur ke dalam tone
        let impact = if i < impact_dur {
            let t_i  = i as f32 / impact_dur as f32;
            // Sangat lambat naik (heavy padding) — bukan bell curve tapi smooth rise
            let env_i = (t_i * 2.0 * PI * 0.5).sin().max(0.0) * (1.0 - t_i);
            let noise = rng.next_f32() * 2.0 - 1.0;
            let raw_impact =
                impact_boss.process(noise) * 0.45
              + impact_body.process(noise) * 0.55; // body lebih dominan dari boss
            impact_lp.process(raw_impact) * env_i * 0.55 // lebih kecil dari kenong
        } else {
            0.0
        };

        // Soft blend impact ke partial — tidak ada "click", langsung menyatu
        let raw = if i < impact_dur {
            partial_sum * 0.7 + impact
        } else {
            partial_sum
        };

        // ── FLANGE SUSPENSION NOTCH ───────────────────────────────────
        // Mode yang sedikit diredam oleh titik suspensi di flange
        let notch_signal = flange_notch.process(raw) * 0.06;
        let with_notch   = raw - notch_signal; // subtract = notch filter behavior

        // ── AIR COLUMN RESONANCE ──────────────────────────────────────
        let air_boost = air_col_f1.process(with_notch) * 0.35
                      + air_col_f2.process(with_notch) * 0.20;
        let resonated = with_notch + air_boost;

        // ── BODY FILTER ───────────────────────────────────────────────
        let filtered = body_lp.process(resonated);
        let filtered = body_hp.process(filtered);

        // ── NONLINEARITY — lebih ringan dari kenong ───────────────────
        let driven = filtered * 1.40;
        let soft = if driven > 0.0 {
            1.0 - (-driven * 1.05).exp()
        } else {
            -(1.0 - (driven * 1.05).exp())
        };

        // ── BLOOM ENVELOPE + SUSTAIN + RELEASE ───────────────────────
        // Attack bloom: terasa seperti "membesar" bukan langsung penuh
        let bloom_gain = if i < bloom_n {
            let t_b = i as f32 / bloom_n as f32;
            // Smooth S-curve bloom — tabuh heavily padded
            t_b * t_b * (3.0 - 2.0 * t_b)
        } else if i < body_n {
            1.0_f32
        } else {
            let t_rel = (i - body_n) as f32 / release_n.max(1) as f32;
            (1.0 - t_rel.min(1.0)).powi(2)
        };

        *slot = soft * bloom_gain * 1.10;
    }

    out
}