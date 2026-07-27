//! f_audio — DSL deklaratif untuk aransemen multi-instrumen
//! Versi 2.0: + Bass Guitar, + Electric Heavy Distortion Guitar
//!            + Dukungan file audio eksternal (WAV), HiFi audio quality

pub mod dsp;
pub mod notes;
pub mod synth;

use std::collections::HashMap;

// ========================================================================
// Sumber suara: sintetis atau file audio eksternal
// ========================================================================

/// Data sampel yang sudah di-load (dari sintesis atau file WAV)
#[derive(Debug, Clone)]
pub struct AudioSample {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Registry file audio eksternal yang sudah di-load
pub struct SampleRegistry {
    samples: HashMap<String, AudioSample>,
}

impl SampleRegistry {
    pub fn new() -> Self {
        SampleRegistry { samples: HashMap::new() }
    }

    /// Load file WAV dan simpan dengan nama alias
    pub fn load(&mut self, alias: &str, path: &str) {
        match load_wav(path) {
            Ok(audio) => {
                println!("[f_audio] Sample '{}' loaded dari '{}'", alias, path);
                self.samples.insert(alias.to_string(), audio);
            }
            Err(e) => {
                eprintln!("[f_audio] PERINGATAN: Gagal load '{}' dari '{}': {}. Menggunakan sintesis fallback.", alias, path, e);
            }
        }
    }

    pub fn get(&self, alias: &str) -> Option<&AudioSample> {
        self.samples.get(alias)
    }
}

/// Load file WAV menggunakan hound, normalkan ke f32 [-1.0, 1.0]
pub fn load_wav(path: &str) -> Result<AudioSample, String> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| format!("Tidak bisa buka '{}': {}", path, e))?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let orig_channels = spec.channels;

    let samples_f32: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>()
                .map(|s| s.map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Error baca float sample: {}", e))?
        }
        hound::SampleFormat::Int => {
            let max_val = (1i64 << (spec.bits_per_sample - 1)) as f32;
            match spec.bits_per_sample {
                8 => reader.samples::<i8>()
                    .map(|s| s.map(|v| v as f32 / 128.0).map_err(|e| e.to_string()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Error: {}", e))?,
                16 => reader.samples::<i16>()
                    .map(|s| s.map(|v| v as f32 / 32768.0).map_err(|e| e.to_string()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Error: {}", e))?,
                24 | 32 => reader.samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / max_val).map_err(|e| e.to_string()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Error: {}", e))?,
                _ => return Err(format!("Bit depth tidak didukung: {}", spec.bits_per_sample)),
            }
        }
    };

    // Ubah ke stereo interleaved: jika mono -> duplikasi, jika stereo -> biarkan
    let stereo = if orig_channels == 1 {
        let mut out = Vec::with_capacity(samples_f32.len() * 2);
        for s in samples_f32 {
            out.push(s);
            out.push(s);
        }
        out
    } else {
        // sudah stereo atau lebih, kita hanya ambil 2 channel pertama jika >2
        if orig_channels > 2 {
            samples_f32
                .chunks(orig_channels as usize)
                .flat_map(|ch| [ch[0], ch[1]])
                .collect()
        } else {
            samples_f32
        }
    };

    Ok(AudioSample {
        samples: stereo,
        sample_rate,
        channels: 2, // selalu 2 setelah konversi
    })
}


// ========================================================================
// Tipe data inti
// ========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    Drum,
    Piano,
    Sax,
    Flute,
    Bass,    
    Guitar,  
}

impl Instrument {
    pub fn from_str(s: &str) -> Instrument {
        match s {
            "drum" | "drums" => Instrument::Drum,
            "piano" | "keys" => Instrument::Piano,
            "sax" | "saxophone" => Instrument::Sax,
            "flute" => Instrument::Flute,
            "bass" | "bass_guitar" | "bassguitar" => Instrument::Bass,
            "guitar" | "elec_guitar" | "distortion" | "heavy_guitar" => Instrument::Guitar,
            other => panic!("[f_audio] Instrumen tidak dikenal: '{}'\nInstrumen yang tersedia: drum, piano, sax, flute, bass, guitar", other),
        }
    }
    
    /*
    pub fn stereo_params(&self) -> (f32, f32) {
        match self {
            Instrument::Drum  => (0.50, 0.15),   // drum cenderung di tengah
            Instrument::Piano => (0.38, 0.70),   // piano lebar
            Instrument::Sax   => (0.65, 0.55),
            Instrument::Flute => (0.60, 0.50),
            Instrument::Bass  => (0.50, 0.00),   // bass mono
            Instrument::Guitar => (0.40, 0.65),
        }
    }
    */
    
    pub fn stereo_params(&self) -> (f32, f32) {
        match self {
            Instrument::Drum  => (0.50, 0.15),   // drum sempit, dekat center
            Instrument::Piano => (0.50, 0.80),   // piano lebar, mengisi ruang
            Instrument::Sax   => (0.50, 0.55),   // sax agak lebar
            Instrument::Flute => (0.50, 0.50),   // flute setengah lebar
            Instrument::Bass  => (0.50, 0.00),   // bass mono total
            Instrument::Guitar => (0.50, 0.65),  // gitar cukup lebar
        }
    }
    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    Loop,
    Once,
}

impl LoopMode {
    pub fn from_str(s: &str) -> LoopMode {
        match s {
            "loop" => LoopMode::Loop,
            "once" => LoopMode::Once,
            other => panic!("[f_audio] Mode tidak dikenal: '{}' (gunakan 'loop' atau 'once')", other),
        }
    }
}
/// Satu event dalam urutan pola
#[derive(Debug, Clone)]
pub enum Event {
    /// Token not/chord/pukulan (bisa juga alias sample eksternal)
    Note(String),
    /// FIX: Jeda diam murni tanpa memperpanjang suara not sebelumnya (milidetik)
    Gap(u64),
    /// FIX: Menekan/menahan not agar berbunyi lebih lama (milidetik)
    Long(u64),
}

fn compute_slot_durations(events: &[Event]) -> Vec<u64> {
    let n = events.len();
    let mut slots = vec![0u64; n];
    for i in 0..n {
        if let Event::Note(_) = events[i] {
            let mut sounding_dur = 0u64;
            let mut j = i + 1;
            while j < n {
                match &events[j] {
                    // Hanya token long() yang mengakumulasi panjang bunyinya not
                    Event::Long(ms) => { 
                        sounding_dur += *ms; 
                        j += 1; 
                    }
                    // Jika ketemu gap() atau Note baru, berhenti menghitung durasi bunyinya
                    _ => break, 
                }
            }
            // Jika not langsung diikuti gap() tanpa long(), beri durasi default pendek (misal 200ms)
            // sehingga suara not akan mati sendiri sebelum jeda kosong berakhir (staccato/detached)
            slots[i] = if sounding_dur == 0 { 200 } else { sounding_dur };
        }
    }
    slots
}

struct Pattern {
    instrument: Instrument,
    preset: String,
    volume: f32,
    mode: LoopMode,
    events: Vec<Event>,
    note_slot_ms: Vec<u64>,
}



#[derive(Clone, Copy)]
enum TimelineKind { Start, Stop }

struct TimelineEntry {
    inst: String,
    pat: String,
    time_ms: u64,
    kind: TimelineKind,
}

/// Proyek audio utama
pub struct Project {
    sample_rate: u32,
    patterns: HashMap<(String, String), Pattern>,
    timeline: Vec<TimelineEntry>,
    current_time_ms: u64,
    pub samples: SampleRegistry,
}

impl Project {
    pub fn new(sample_rate: u32) -> Self {
        Project {
            sample_rate,
            patterns: HashMap::new(),
            timeline: Vec::new(),
            current_time_ms: 0,
            samples: SampleRegistry::new(),
        }
    }

    pub fn sample_rate(&self) -> u32 { self.sample_rate }

    pub fn load_sample(&mut self, alias: &str, path: &str) {
        self.samples.load(alias, path);
    }

    pub fn add_pattern(
        &mut self,
        inst_name: &str,
        pattern_name: &str,
        instrument: Instrument,
        preset: &str, 
        volume: f32,
        mode: LoopMode,
        events: Vec<Event>,
    ) {
        let note_slot_ms = compute_slot_durations(&events);
        self.patterns.insert(
            (inst_name.to_string(), pattern_name.to_string()),
            Pattern { 
                instrument, 
                preset: preset.to_string(), // Simpan preset string ke struct
                volume, 
                mode, 
                events, 
                note_slot_ms 
            },
        );
    }


    pub fn queue_start(&mut self, inst_name: &str, pattern_name: &str) {
        self.timeline.push(TimelineEntry {
            inst: inst_name.to_string(),
            pat: pattern_name.to_string(),
            time_ms: self.current_time_ms,
            kind: TimelineKind::Start,
        });
    }

    pub fn queue_stop(&mut self, inst_name: &str, pattern_name: &str) {
        self.timeline.push(TimelineEntry {
            inst: inst_name.to_string(),
            pat: pattern_name.to_string(),
            time_ms: self.current_time_ms,
            kind: TimelineKind::Stop,
        });
    }

    pub fn advance_time(&mut self, ms: u64) { self.current_time_ms += ms; }

    pub fn render(&self) -> Vec<f32> {
    let total_ms = self.current_time_ms.max(1);
    let total_frames = dsp::ms_to_samples(total_ms, self.sample_rate);
    let mut buffer = vec![0.0f32; total_frames * 2];

    for (key, pattern) in self.patterns.iter() {
        let (pan, width) = pattern.instrument.stereo_params();
        for (start_ms, stop_ms) in self.intervals_for(key) {
            self.render_pattern_interval(pattern, start_ms, stop_ms, pan, width, &mut buffer);
        }
    }

    dsp::apply_true_stereo_reverb(&mut buffer, self.sample_rate);
    dsp::noise_gate(&mut buffer, 5e-4); // ← tambahan ini
    dsp::normalize(&mut buffer);
    buffer
}

    fn intervals_for(&self, key: &(String, String)) -> Vec<(u64, u64)> {
        let mut starts: Vec<u64> = Vec::new();
        let mut intervals = Vec::new();
        for entry in &self.timeline {
            if entry.inst == key.0 && entry.pat == key.1 {
                match entry.kind {
                    TimelineKind::Start => starts.push(entry.time_ms),
                    TimelineKind::Stop => {
                        if let Some(start) = starts.pop() {
                            intervals.push((start, entry.time_ms));
                        }
                    }
                }
            }
        }
        for start in starts {
            intervals.push((start, self.current_time_ms));
        }
        intervals
    }

    fn render_pattern_interval(
        &self,
        pattern: &Pattern,
        start_ms: u64,
        stop_ms: u64,
        pan: f32,
        width: f32,
        buffer: &mut [f32],
    ) {
        if pattern.events.is_empty() || stop_ms <= start_ms { return; }
        let gain = (pattern.volume / 100.0).clamp(0.0, 2.0);
        let mut cursor_ms = start_ms;

        loop {
            let mut time_advanced = false;
            for (idx, event) in pattern.events.iter().enumerate() {
                if cursor_ms >= stop_ms { return; }
                match event {
                    Event::Gap(ms) => { cursor_ms += ms; if *ms > 0 { time_advanced = true; } }
                    Event::Long(ms) => { cursor_ms += ms; if *ms > 0 { time_advanced = true; } }
                   
                   /* Event::Note(token) => {
                        let slot_ms = pattern.note_slot_ms[idx];
                        let (mut voice, channels) = self.synth_voice(pattern.instrument, &pattern.preset, token, slot_ms);
                        
                        dsp::clean_denormal(&mut voice);
                        
                        let offset_frame = dsp::ms_to_samples(cursor_ms, self.sample_rate);
                        if channels == 1 {
                            dsp::mix_in_stereo_mono(buffer, offset_frame, &voice, pan, width, gain);
                        } else {
                            dsp::mix_in_stereo_stereo(buffer, offset_frame, &voice, gain);
                        }
                    } */
                    
                    Event::Note(token) => {
    let slot_ms = pattern.note_slot_ms[idx];
    let (voice, channels) = self.synth_voice(pattern.instrument, &pattern.preset, token, slot_ms);
    let offset_frame = dsp::ms_to_samples(cursor_ms, self.sample_rate);
    
    if channels == 2 {
        // Voice sudah stereo (drum & gitar hasil true stereo synthesis)
        let mut stereo_voice = voice;
        dsp::adjust_stereo_width(&mut stereo_voice, width); // terapkan width
        dsp::mix_in_stereo_stereo(buffer, offset_frame, &stereo_voice, gain);
    } else {
        // Voice mono (piano, sax, flute, bass)
        let stereo_voice = dsp::make_stereo_voice(&voice, width, self.sample_rate);
        dsp::mix_in_stereo_stereo(buffer, offset_frame, &stereo_voice, gain);
    }
}
                    
                    
                }
            }
            if pattern.mode == LoopMode::Once { return; }
            if !time_advanced { return; }
        }
    }

    fn synth_voice(&self, instrument: Instrument, preset: &str, token: &str, duration_ms: u64) -> (Vec<f32>, u16) {
        let sr = self.sample_rate;

        let release_ms = match instrument {
            Instrument::Drum => 40,
            Instrument::Piano => 1500,
            Instrument::Guitar => 600,
            Instrument::Bass => 300,
            _ => 200,
        };

        let total_ms = duration_ms + release_ms;

        // Jika sample eksternal, ambil dari registry
        if let Some(audio) = self.samples.get(token) {
            let resampled = resample_voice(&audio.samples, audio.sample_rate, sr, total_ms, audio.channels);
            return (resampled, audio.channels);
        }

        // Sintesis mono
        let (audio, channels) = match instrument {
        Instrument::Drum => (synth::synth_drum(token, sr), 2),  // synth_drum sekarang mengembalikan stereo
        Instrument::Piano => (synth::synth_piano_chord(token, total_ms, sr), 1),
        Instrument::Sax => (synth::synth_sax_note(token, total_ms, sr), 1),
        Instrument::Flute => (synth::synth_flute_note(token, total_ms, sr), 1),
        Instrument::Bass => (synth::synth_bass_note(token, total_ms, sr), 1),
        
        Instrument::Guitar if preset == "acoustic_single" => {
            (synth::synth_guitar_single_note_stereo(token, total_ms, sr), 2)
        },
        Instrument::Guitar => (synth::synth_guitar_chord_stereo(token, total_ms, sr, preset), 2),

    };

        // Fade out
        
        let mut voice = audio;
    let body_samples = dsp::ms_to_samples(duration_ms, sr);
    if channels == 2 {
        // Fade out pada buffer stereo interleaved
        let total_frames = voice.len() / 2;
        if total_frames > body_samples {
            let release_frames = total_frames - body_samples;
            for i in 0..release_frames {
                let idx = (body_samples + i) * 2;
                let progress = i as f32 / release_frames as f32;
                let factor = (-5.0 * progress).exp() * (1.0 - progress);
                voice[idx] *= factor;
                voice[idx + 1] *= factor;
            }
        } else {
            let fade_len = (sr as usize / 100).min(total_frames);
            for i in (total_frames - fade_len)..total_frames {
                let idx = i * 2;
                let gain = 1.0 - ((i - (total_frames - fade_len)) as f32 / fade_len as f32);
                voice[idx] *= gain;
                voice[idx + 1] *= gain;
            }
        }
    } else {
       
        
        if voice.len() > body_samples {
            let release_samples = voice.len() - body_samples;
            for i in 0..release_samples {
                let idx = body_samples + i;
                let progress = i as f32 / release_samples as f32;
                let factor = (-5.0 * progress).exp() * (1.0 - progress);
                voice[idx] *= factor;
            }
        } else {
            let fade_len = (sr as usize / 100).min(voice.len());
            let len = voice.len();
            for (i, s) in voice[len.saturating_sub(fade_len)..].iter_mut().enumerate() {
                *s *= 1.0 - (i as f32 / fade_len as f32);
            }
        }
        }

        (voice, channels)
    
}
}

fn resample_voice(samples: &[f32], src_rate: u32, dst_rate: u32, max_ms: u64, channels: u16) -> Vec<f32> {
    let frames_in = samples.len() / channels as usize;
    let max_frames = dsp::ms_to_samples(max_ms, dst_rate);
    if src_rate == dst_rate {
        let out_frames = frames_in.min(max_frames);
        let mut out = samples[..(out_frames * channels as usize)].to_vec();
        // fade out
        let fade_len = (dst_rate as usize / 100).min(out_frames);
        for ch in 0..channels {
            let offset = ch as usize;
            for i in (out_frames - fade_len)..out_frames {
                let idx = i * channels as usize + offset;
                let gain = 1.0 - ((i - (out_frames - fade_len)) as f32 / fade_len as f32);
                out[idx] *= gain;
            }
        }
        return out;
    }

    let ratio = src_rate as f32 / dst_rate as f32;
    let out_frames = ((frames_in as f32 / ratio) as usize).min(max_frames);
    let mut out = vec![0.0f32; out_frames * channels as usize];

    for ch in 0..channels {
        let offset = ch as usize;
        for i in 0..out_frames {
            let src_pos = i as f32 * ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f32;

            let im1 = if src_idx > 0 { src_idx - 1 } else { 0 };
            let i0 = src_idx;
            let i1 = (src_idx + 1).min(frames_in - 1);
            let i2 = (src_idx + 2).min(frames_in - 1);

            let get = |idx| samples[idx * channels as usize + offset];
            let y0 = get(im1);
            let y1 = get(i0);
            let y2 = get(i1);
            let y3 = get(i2);

            let c0 = y1;
            let c1 = 0.5 * (y2 - y0);
            let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
            let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

            let val = ((c3 * frac + c2) * frac + c1) * frac + c0;
            out[i * channels as usize + offset] = val;
        }
    }

    // fade out akhir
    let fade_len = (dst_rate as usize / 100).min(out_frames);
    for ch in 0..channels {
        let offset = ch as usize;
        for i in (out_frames - fade_len)..out_frames {
            let idx = i * channels as usize + offset;
            let gain = 1.0 - ((i - (out_frames - fade_len)) as f32 / fade_len as f32);
            out[idx] *= gain;
        }
    }
    out
}

// ========================================================================
// save & play (stereo WAV)
// ========================================================================
pub fn save(project: &Project, filename: &str) {
    let samples = project.render();
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: project.sample_rate(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(filename, spec)
        .expect("[f_audio] Gagal membuat file WAV");
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(v).expect("[f_audio] Gagal menulis sample");
    }
    writer.finalize().expect("[f_audio] Gagal finalize WAV");
    println!("[INFO] Berhasil menyimpan audio stereo ke '{}'", filename);
}

pub fn play(project: &Project) {
    use std::process::Command;
    let tmp = "_f_audio_play_tmp.wav";
    save(project, tmp);

    let players: &[(&str, &[&str])] = &[
        ("aplay",  &[tmp]),
        ("paplay", &[tmp]),
        ("ffplay", &["-nodisp", "-autoexit", tmp]),
        ("afplay", &[tmp]),
        ("cvlc",   &["--play-and-exit", tmp]),
        ("mpg123", &[tmp]),
        ("sox",    &[tmp, "-d"]),
    ];

    let mut played = false;
    for (player, args) in players {
        match Command::new(player).args(*args).status() {
            Ok(status) if status.success() => {
                played = true;
                break;
            }
            _ => continue,
        }
    }
    let _ = std::fs::remove_file(tmp);
    if !played {
        println!("[INFO] Audio player tidak ditemukan. Putar manual:");
        println!("[INFO]   aplay my_audio.wav");
    }
}

// ========================================================================
// MACRO DSL (FIXED FOR LITERALS AND SHARPS)
// ========================================================================

#[macro_export]
macro_rules! read {
    ($proj:ident, $alias:ident, $path:literal) => {
        $proj.load_sample(stringify!($alias), $path);
    };
}

#[macro_export]
macro_rules! audio {
    ($name:ident, { $($body:tt)* }) => {
        let mut $name = $crate::Project::new(44100);
        $crate::audio_items!($name; $($body)*);
    };
}

#[macro_export]
macro_rules! audio_items {

    ($name:ident; $sample_name:ident = read($path:literal) , $($rest:tt)*) => {
       /* $name.load_sample(stringify!($sample_name), $path); audio_impl!($name $($rest)*); */
       $name.load_sample(stringify!($sample_name), $path);
       $crate::audio_items!($name; $($rest)*);
    };

    ($name:ident; $inst:ident { $($patterns:tt)* } , $($rest:tt)*) => {
        $crate::audio_patterns!($name, $inst; $($patterns)*);
        $crate::audio_items!($name; $($rest)*);
    };
    ($name:ident; $inst:ident { $($patterns:tt)* }) => {
        $crate::audio_patterns!($name, $inst; $($patterns)*);
    };
    ($name:ident; start( $($links:tt)* ) , $($rest:tt)*) => {
        $crate::audio_links!($name, start; $($links)*);
        $crate::audio_items!($name; $($rest)*);
    };
    ($name:ident; start( $($links:tt)* )) => {
        $crate::audio_links!($name, start; $($links)*);
    };
    ($name:ident; stop( $($links:tt)* ) , $($rest:tt)*) => {
        $crate::audio_links!($name, stop; $($links)*);
        $crate::audio_items!($name; $($rest)*);
    };
    ($name:ident; stop( $($links:tt)* )) => {
        $crate::audio_links!($name, stop; $($links)*);
    };
    ($name:ident; gap($ms:literal) , $($rest:tt)*) => {
        $name.advance_time($ms);
        $crate::audio_items!($name; $($rest)*);
    };
    ($name:ident; gap($ms:literal)) => {
        $name.advance_time($ms);
    };
    ($name:ident;) => {};
}

#[macro_export]
macro_rules! audio_patterns {
    // Pattern dengan kustomisasi preset (contoh: preset: "electric 1")
    ($name:ident, $inst:ident; $pname:ident => { volume: $vol:literal, preset: $preset:literal, $mode:ident { $($events:tt)* } } , $($rest:tt)*) => {
        $crate::audio_make_pattern!($name, $inst, $pname, $vol, $preset, $mode; $($events)*);
        $crate::audio_patterns!($name, $inst; $($rest)*);
    };
    ($name:ident, $inst:ident; $pname:ident => { volume: $vol:literal, preset: $preset:literal, $mode:ident { $($events:tt)* } }) => {
        $crate::audio_make_pattern!($name, $inst, $pname, $vol, $preset, $mode; $($events)*);
    };

    // Fallback pattern normal (Otomatis memakai preset default: "acoustic")
    ($name:ident, $inst:ident; $pname:ident => { volume: $vol:literal, $mode:ident { $($events:tt)* } } , $($rest:tt)*) => {
        $crate::audio_make_pattern!($name, $inst, $pname, $vol, "acoustic", $mode; $($events)*);
        $crate::audio_patterns!($name, $inst; $($rest)*);
    };
    ($name:ident, $inst:ident; $pname:ident => { volume: $vol:literal, $mode:ident { $($events:tt)* } }) => {
        $crate::audio_make_pattern!($name, $inst, $pname, $vol, "acoustic", $mode; $($events)*);
    };
    
    ($name:ident, $inst:ident;) => {};
}

#[macro_export]
macro_rules! audio_make_pattern {
    ($name:ident, $inst:ident, $pname:ident, $vol:literal, $preset:literal, $mode:ident; $($events:tt)*) => {
        {
            let mut __f_audio_events: Vec<$crate::Event> = Vec::new();
            $crate::audio_events!(__f_audio_events; $($events)*);
            $name.add_pattern(
                stringify!($inst),
                stringify!($pname),
                $crate::Instrument::from_str(stringify!($inst)),
                $preset, // Meneruskan preset literal ke sistem internal
                $vol as f32,
                $crate::LoopMode::from_str(stringify!($mode)),
                __f_audio_events,
            );
        }
    };
}

#[macro_export]
macro_rules! audio_events {
    ($events:ident; gap($ms:literal) , $($rest:tt)*) => {
        $events.push($crate::Event::Gap($ms));
        $crate::audio_events!($events; $($rest)*);
    };
    ($events:ident; gap($ms:literal)) => {
        $events.push($crate::Event::Gap($ms));
    };
    // FIX: Daftarkan token baru long(ms) ke generator AST macro
    ($events:ident; long($ms:literal) , $($rest:tt)*) => {
        $events.push($crate::Event::Long($ms));
        $crate::audio_events!($events; $($rest)*);
    };
    ($events:ident; long($ms:literal)) => {
        $events.push($crate::Event::Long($ms));
    };
    ($events:ident; $note:literal , $($rest:tt)*) => {
        $events.push($crate::Event::Note($note.to_string()));
        $crate::audio_events!($events; $($rest)*);
    };
    ($events:ident; $note:literal) => {
        $events.push($crate::Event::Note($note.to_string()));
    };
    ($events:ident; $note:ident , $($rest:tt)*) => {
        $events.push($crate::Event::Note(stringify!($note).to_string()));
        $crate::audio_events!($events; $($rest)*);
    };
    ($events:ident; $note:ident) => {
        $events.push($crate::Event::Note(stringify!($note).to_string()));
    };
    ($events:ident;) => {};
}

#[macro_export]
macro_rules! audio_links {
    ($name:ident, start; $inst:ident => $pat:ident , $($rest:tt)*) => {
        $name.queue_start(stringify!($inst), stringify!($pat));
        $crate::audio_links!($name, start; $($rest)*);
    };
    ($name:ident, start; $inst:ident => $pat:ident) => {
        $name.queue_start(stringify!($inst), stringify!($pat));
    };
    ($name:ident, stop; $inst:ident => $pat:ident , $($rest:tt)*) => {
        $name.queue_stop(stringify!($inst), stringify!($pat));
        $crate::audio_links!($name, stop; $($rest)*);
    };
    ($name:ident, stop; $inst:ident => $pat:ident) => {
        $name.queue_stop(stringify!($inst), stringify!($pat));
    };
    ($name:ident, $mode:ident;) => {};
}