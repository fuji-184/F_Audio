#![recursion_limit = "1024"]

use f_audio::*;

fn main() {



    audio!(my_audio, {
    
        k = read("./sample/kick.wav"),
        s = read("./sample/snare.wav"),
        h = read("./sample/hihat.wav"),
    
        drum {
            song2_drum => {
                volume: 98,
                loop {
                    k, h, long(370),
                    k, gap(190), s, h, gap(250), k, gap(250), k, h, long(370),
                    k, gap(190), s, h, gap(250), k, gap(250), k, h, long(370),
                    k, gap(190), s, h, gap(250), k, gap(250), k, h, long(370),
                    k, gap(190), s, h, gap(250), k, gap(250),
                }
            }
        },
        guitar {
            song2_guitar => {
                volume: 90,
                preset: "overdriven_guitar",
                loop {
                    F, long(444), F, gap(444),       
                    Gs, long(444), Gs, long(444),     
                    As, long(444), As, long(444),     
                    Cs, long(222), C, long(222),      
                    gap(444)
                }
            }
        },
        bass {
            song2_bass => {
                volume: 95,
                loop {     
                    F, long(222), F, long(222), F, long(222), F, long(222),
                    Gs, long(222), Gs, long(222), Gs, long(222), Gs, long(222),
                    As, long(222), As, long(222), As, long(222), As, long(222),
                    Cs, long(222), C, long(222), gap(444)
                }
            }
        },
        piano {
            song2_synth => {
                volume: 75,
                preset: "poly_synth",
                loop {
                    F, long(1776), Gs, long(1776), As, long(1776), Cs, long(888), C, long(888)
                }
            }
        },
        
        sax {
            song2_woo_hoo => {
                volume: 95,
                loop {
                    // Bagian "Woo-Hoo!" setelah riff gitar utama selesai berputar
                    gap(3552), 
                    C, long(222), Gs, long(444), // "Woo-Hoo!" pitch bend/jump
                    gap(3108)
                }
            }
        },

        // === TIMING MANAGEMENT ===
        gap(1000),

        start(
            drum => song2_drum,
            guitar => song2_guitar,
       /*     bass => song2_bass,
            piano => song2_synth,
            sax => song2_woo_hoo*/
        ),

        // Mainkan selama kemauanmu (misal: 90 detik)
        gap(90000),

        stop(
            drum => song2_drum,
            guitar => song2_guitar,
          /*  bass => song2_bass,
            piano => song2_synth,
            sax => song2_woo_hoo*/
        )
    });

    println!("Saving Blur - Song 2 to my_audio.wav...");
    save(&my_audio, "my_audio.wav");
    play(&my_audio);
}