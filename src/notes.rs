

/*
//! Modul DSP (Digital Signal Processing) — Versi HiFi High Dynamic
//! Penerjemah token notasi musik → frekuensi Hz (A4 = 440 Hz, equal temperament)


fn base_semitone(letter: char) -> Option<i32> {
    match letter.to_ascii_uppercase() {
        'C' => Some(0),
        'D' => Some(2),
        'E' => Some(4),
        'F' => Some(5),
        'G' => Some(7),
        'A' => Some(9),
        'B' => Some(11),
        _ => None,
    }
}

struct ParsedToken {
    pitch_class: i32,
    is_minor: bool,
    octave: i32,
}

fn parse_token(token: &str, default_octave: i32) -> Option<ParsedToken> {
    let chars: Vec<char> = token.chars().collect();
    if chars.is_empty() { return None; }
    
    let mut pitch_class = base_semitone(chars[0])?;
    let mut idx = 1;
    
    // 1. Deteksi Accidental (Sharp / Flat)
    // '#' = standar, 's' = alias sharp (misal: "Cs4" = C#4, "Gs4" = G#4)
    if idx < chars.len() && (chars[idx] == '#' || chars[idx] == 's') {
        pitch_class += 1; 
        idx += 1;
    } else if idx < chars.len() && chars[idx] == 'b' {
        pitch_class -= 1; 
        idx += 1;
    }
    
    // 2. Deteksi Minor Chord
    let is_minor = idx < chars.len() && chars[idx] == 'm';
    if is_minor { 
        idx += 1; 
    }
    
    // 3. FIX: Deteksi Oktaf Eksplisit dari Token (misal: "C4", "F#3", "Am5")
    let mut octave = default_octave;
    if idx < chars.len() && chars[idx].is_ascii_digit() {
        if let Some(digit) = chars[idx].to_digit(10) {
            octave = digit as i32;
        }
    }
    
    // Amankan pitch class di dalam rentang 12 nada (chromatic scale)
    pitch_class = ((pitch_class % 12) + 12) % 12;
    
    Some(ParsedToken { pitch_class, is_minor, octave })
}

/// Konversi MIDI note ke Frekuensi dengan pembatasan batas dengar manusia (HiFi Safe)
fn midi_to_freq(midi: i32) -> f32 {
    // Batasi rentang MIDI antara 0 (8.18Hz) sampai 127 (12543.85Hz) demi mencegah aliasing digital
    let clapped_midi = midi.clamp(0, 127);
    440.0 * 2f32.powf((clapped_midi as f32 - 69.0) / 12.0)
}

pub fn single_note_freq(token: &str, default_octave: i32) -> f32 {
    match parse_token(token, default_octave) {
        // Jika sukses, hitung frekuensi berbasis oktaf yang terdeteksi
        Some(p) => midi_to_freq((p.octave + 1) * 12 + p.pitch_class),
        // FIX: Kembalikan 0.0 (Senyap) jika token corrupt, menjauhkan mix dari distorsi frekuensi liar
        None => 0.0, 
    }
}

pub fn chord_freqs(token: &str, default_octave: i32) -> Vec<f32> {
    match parse_token(token, default_octave) {
        Some(p) => {
            let root_midi = (p.octave + 1) * 12 + p.pitch_class;
            let third_interval = if p.is_minor { 3 } else { 4 };
            
            // Mengembalikan susunan harmoni chord piano standar (Triad + 1 Bass Low Root)
            vec![
                midi_to_freq(root_midi - 12), // Bass Note (Memperkaya tekstur low-end)
                midi_to_freq(root_midi),      // Root Note
                midi_to_freq(root_midi + third_interval), // Terz (Penentu Mayor/Minor)
                midi_to_freq(root_midi + 7),  // Kwin (Perfect 5th - Penjaga kestabilan harmoni)
            ]
        }
        // Jaga kestabilan output dengan mengosongkan vektor jika gagal parse
        None => vec![0.0],
    }
}
*/

    
fn base_semitone(letter: char) -> Option<i32> {
    match letter.to_ascii_uppercase() {
        'C' => Some(0),
        'D' => Some(2),
        'E' => Some(4),
        'F' => Some(5),
        'G' => Some(7),
        'A' => Some(9),
        'B' => Some(11),
        _ => None,
    }
}

struct ParsedToken {
    pitch_class: i32,
    is_minor: bool,
    octave: i32,
}

fn parse_token(token: &str, default_octave: i32) -> Option<ParsedToken> {
    let chars: Vec<char> = token.chars().collect();
    if chars.is_empty() { return None; }
    
    let mut pitch_class = base_semitone(chars[0])?;
    let mut idx = 1;
    
    // 1. Deteksi Accidental (Sharp / Flat)
    if idx < chars.len() && chars[idx] == '#' {
        pitch_class += 1; 
        idx += 1;
    } else if idx < chars.len() && chars[idx] == 'b' {
        pitch_class -= 1; 
        idx += 1;
    }
    
    // 2. Deteksi Minor Chord
    let is_minor = idx < chars.len() && chars[idx] == 'm';
    if is_minor { 
        idx += 1; 
    }
    
    // 3. FIX: Deteksi Oktaf Eksplisit dari Token (misal: "C4", "F#3", "Am5")
    let mut octave = default_octave;
    if idx < chars.len() && chars[idx].is_ascii_digit() {
        if let Some(digit) = chars[idx].to_digit(10) {
            octave = digit as i32;
        }
    }
    
    // Amankan pitch class di dalam rentang 12 nada (chromatic scale)
    pitch_class = ((pitch_class % 12) + 12) % 12;
    
    Some(ParsedToken { pitch_class, is_minor, octave })
}

/// Konversi MIDI note ke Frekuensi dengan pembatasan batas dengar manusia (HiFi Safe)
fn midi_to_freq(midi: i32) -> f32 {
    // Batasi rentang MIDI antara 0 (8.18Hz) sampai 127 (12543.85Hz) demi mencegah aliasing digital
    let clapped_midi = midi.clamp(0, 127);
    440.0 * 2f32.powf((clapped_midi as f32 - 69.0) / 12.0)
}

pub fn single_note_freq(token: &str, default_octave: i32) -> f32 {
    match parse_token(token, default_octave) {
        // Jika sukses, hitung frekuensi berbasis oktaf yang terdeteksi
        Some(p) => midi_to_freq((p.octave + 1) * 12 + p.pitch_class),
        // FIX: Kembalikan 0.0 (Senyap) jika token corrupt, menjauhkan mix dari distorsi frekuensi liar
        None => 0.0, 
    }
}

pub fn chord_freqs(token: &str, default_octave: i32) -> Vec<f32> {
    match parse_token(token, default_octave) {
        Some(p) => {
            let root_midi = (p.octave + 1) * 12 + p.pitch_class;
            let third_interval = if p.is_minor { 3 } else { 4 };
            
            // Mengembalikan susunan harmoni chord piano standar (Triad + 1 Bass Low Root)
            vec![
                midi_to_freq(root_midi - 12), // Bass Note (Memperkaya tekstur low-end)
                midi_to_freq(root_midi),      // Root Note
                midi_to_freq(root_midi + third_interval), // Terz (Penentu Mayor/Minor)
                midi_to_freq(root_midi + 7),  // Kwin (Perfect 5th - Penjaga kestabilan harmoni)
            ]
        }
        // Jaga kestabilan output dengan mengosongkan vektor jika gagal parse
        None => vec![0.0],
    }
}
