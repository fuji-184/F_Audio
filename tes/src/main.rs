#![recursion_limit = "4096"]
use f_audio::*;

// ============================================================
// LANCARAN KEBO GIRO — Pelog Barang
// Mapping pelog: 2=D, 3=E, 5=G, 6=A, 7=B
//
// Struktur lancaran: 4 gatra per gongan, 4 keteg per gatra
// Kempul di akhir gatra 1,2,3 | Kenong+Kempul di akhir gatra 4
// Bonang panerus: imbal 2× kecepatan balungan
// Tempo: ~126 bpm (lancaran pernikahan = cukup cepat, gegap gempita)
// ============================================================

fn main() {
    // Satu keteg = 238ms (≈126 bpm), satu gatra = 4 keteg = 952ms
    // Lancaran: 4 gatra = satu gongan = 3808ms
    // Kita main 3 putaran (gongan) agar terasa penuh

    audio!(song, {

        // ── DEMUNG: balungan utama (satu nada per keteg) ────────────
        demung {
            // Baris 1: 6 5 3 2 | 3 2 6 5 | 6 5 3 2 | 3 2 6 (5)
            baris1 => { volume: 88, loop {
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
                "E3", long(220), gap(18),
                "D3", long(220), gap(18),

                "E3", long(220), gap(18),
                "D3", long(220), gap(18),
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),

                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
                "E3", long(220), gap(18),
                "D3", long(220), gap(18),

                "E3", long(220), gap(18),
                "D3", long(220), gap(18),
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
            }},
            // Baris 2: 6 5 6 7 | 6 7 6 5 | 6 5 6 7 | 6 7 6 (5)
            baris2 => { volume: 88, loop {
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
                "A3", long(220), gap(18),
                "B3", long(220), gap(18),

                "A3", long(220), gap(18),
                "B3", long(220), gap(18),
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),

                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
                "A3", long(220), gap(18),
                "B3", long(220), gap(18),

                "A3", long(220), gap(18),
                "B3", long(220), gap(18),
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
            }},
            // Baris 3: 7 6 3 2 | 3 2 6 (5)
            baris3 => { volume: 88, loop {
                "B3", long(220), gap(18),
                "A3", long(220), gap(18),
                "E3", long(220), gap(18),
                "D3", long(220), gap(18),

                "E3", long(220), gap(18),
                "D3", long(220), gap(18),
                "A3", long(220), gap(18),
                "G3", long(220), gap(18),
            }},
        },

        // ── SARON BARUNG: balungan (sama oktaf lebih tinggi) ────────
        saron {
            baris1 => { volume: 82, loop {
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
                "E4", long(210), gap(28),
                "D4", long(210), gap(28),

                "E4", long(210), gap(28),
                "D4", long(210), gap(28),
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),

                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
                "E4", long(210), gap(28),
                "D4", long(210), gap(28),

                "E4", long(210), gap(28),
                "D4", long(210), gap(28),
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
            }},
            baris2 => { volume: 82, loop {
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
                "A4", long(210), gap(28),
                "B4", long(210), gap(28),

                "A4", long(210), gap(28),
                "B4", long(210), gap(28),
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),

                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
                "A4", long(210), gap(28),
                "B4", long(210), gap(28),

                "A4", long(210), gap(28),
                "B4", long(210), gap(28),
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
            }},
            baris3 => { volume: 82, loop {
                "B4", long(210), gap(28),
                "A4", long(210), gap(28),
                "E4", long(210), gap(28),
                "D4", long(210), gap(28),

                "E4", long(210), gap(28),
                "D4", long(210), gap(28),
                "A4", long(210), gap(28),
                "G4", long(210), gap(28),
            }},
        },

        // ── BONANG BARUNG: imbal — mengisi selang antar balungan ────
        // Bonang barung: keteg yang sama tapi ditambah selipan nada
        bonang {
            baris1 => { volume: 72, loop {
                // Setiap keteg balungan diisi bonang 2× lipat
                // Pola: nada berikutnya dulu, lalu nada sekarang
                "G4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "E4", long(100), gap(14),
                "G4", long(100), gap(14),
                "G4", long(100), gap(14),
                "E4", long(100), gap(14),
                "D4", long(100), gap(14),
                "E4", long(100), gap(14),
                "E4", long(100), gap(14),
                "D4", long(100), gap(14),
                "A4", long(100), gap(14),
                "D4", long(100), gap(14),
                "D4", long(100), gap(14),
                "A4", long(100), gap(14),
                // gatra 3-4 sama
                "G4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "E4", long(100), gap(14),
                "G4", long(100), gap(14),
                "G4", long(100), gap(14),
                "E4", long(100), gap(14),
                "D4", long(100), gap(14),
                "E4", long(100), gap(14),
                "E4", long(100), gap(14),
                "D4", long(100), gap(14),
                "A4", long(100), gap(14),
                "D4", long(100), gap(14),
                "D4", long(100), gap(14),
                "A4", long(100), gap(14),
            }},
            baris2 => { volume: 72, loop {
                "G4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "B4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "G4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "B4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "G4", long(100), gap(14),
            }},
            baris3 => { volume: 72, loop {
                "A4", long(100), gap(14),
                "B4", long(100), gap(14),
                "B4", long(100), gap(14),
                "A4", long(100), gap(14),
                "E4", long(100), gap(14),
                "A4", long(100), gap(14),
                "A4", long(100), gap(14),
                "E4", long(100), gap(14),
                "D4", long(100), gap(14),
                "E4", long(100), gap(14),
                "E4", long(100), gap(14),
                "D4", long(100), gap(14),
                "A4", long(100), gap(14),
                "D4", long(100), gap(14),
                "D4", long(100), gap(14),
                "A4", long(100), gap(14),
            }},
        },

        // ── KEMPUL: ketukan di akhir gatra 1,2,3 ────────────────────
        // Dalam lancaran: kempul di setiap seleh kecuali gong
        // Nada kempul mengikuti nada seleh gatra (nada terakhir tiap gatra)
        kempul {
            baris1 => { volume: 85, loop {
                gap(952),              // gatra 1 selesai
                "G3", long(700), gap(252),  // seleh nada 5=G, gatra 2 mulai
                gap(952),              // gatra 3 selesai
                "G3", long(700), gap(252),  // seleh gatra 4 = nada 5=G
            }},
            baris2 => { volume: 85, loop {
                gap(952),
                "G3", long(700), gap(252),
                gap(952),
                "G3", long(700), gap(252),
            }},
            baris3 => { volume: 85, loop {
                gap(952),
                "G3", long(700), gap(252),
            }},
        },

        // ── KENONG: seleh akhir tiap gongan ─────────────────────────
        // Lancaran: kenong di akhir setiap gatra (lebih sering dari gendhing besar)
        kenong {
            baris1 => { volume: 88, loop {
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(1200), gap(0),  // gong tone panjang
            }},
            baris2 => { volume: 88, loop {
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(1200), gap(0),
            }},
            baris3 => { volume: 88, loop {
                gap(952),
                "G3", long(800), gap(152),
                gap(952),
                "G3", long(1200), gap(0),  // akhir baris 3 = balik ke baris 1
            }},
        },

        // ── URUTAN PLAYBACK: Buka → Baris1 → Baris2 → Baris3 → ulang
        // Buka dilewati (tidak ada kendang, langsung masuk ompak)
        // Baris 1 (2 gongan)
        start(demung  => baris1),
        start(saron   => baris1),
        start(bonang  => baris1),
        start(kempul  => baris1),
        start(kenong  => baris1),
        gap(3808),   // 1 gongan baris 1

        // Baris 2 dimulai setelah 1 putaran baris 1
        stop(demung  => baris1),
        stop(saron   => baris1),
        stop(bonang  => baris1),
        stop(kempul  => baris1),
        stop(kenong  => baris1),
        start(demung  => baris2),
        start(saron   => baris2),
        start(bonang  => baris2),
        start(kempul  => baris2),
        start(kenong  => baris2),
        gap(3808),   // 1 gongan baris 2

        // Baris 3 (setengah gongan, lalu balik baris 1)
        stop(demung  => baris2),
        stop(saron   => baris2),
        stop(bonang  => baris2),
        stop(kempul  => baris2),
        stop(kenong  => baris2),
        start(demung  => baris3),
        start(saron   => baris3),
        start(bonang  => baris3),
        start(kempul  => baris3),
        start(kenong  => baris3),
        gap(1904),   // setengah gongan (baris 3 hanya 2 gatra)

        // Kembali baris 1 — ulang 2× lagi
        stop(demung  => baris3),
        stop(saron   => baris3),
        stop(bonang  => baris3),
        stop(kempul  => baris3),
        stop(kenong  => baris3),
        start(demung  => baris1),
        start(saron   => baris1),
        start(bonang  => baris1),
        start(kempul  => baris1),
        start(kenong  => baris1),
        gap(7616),   // 2 gongan baris 1

        stop(demung  => baris1),
        stop(saron   => baris1),
        stop(bonang  => baris1),
        stop(kempul  => baris1),
        stop(kenong  => baris1),
        gap(2000),
    });

    f_audio::save(&song, "output.wav");
    println!("Saved: output.wav — Lancaran Kebo Giro, Pelog Barang");
}