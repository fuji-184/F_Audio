

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

