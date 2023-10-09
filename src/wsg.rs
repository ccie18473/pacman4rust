#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const WSG_SAMPLE_RATE: u32 = 96000;

#[derive(Copy, Clone)]
pub struct wsg_voice {
    pub frequency: u32,   // 20 bit value
    pub accumulator: u32, // 20 bit value
    pub waveform_no: u8,  // 3 bit value
    pub volume: u8,       // 4 bit value
}

impl wsg_voice {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            accumulator: 0,
            waveform_no: 0,
            volume: 0,
        }
    }
}

pub struct wsg {
    pub voices: [wsg_voice; 3],
    pub sound_rom: [u8; 0x100],
    pub gain: i32,
}

impl wsg {
    pub fn new() -> Self {
        Self {
            voices: [wsg_voice::new(); 3],
            sound_rom: [0; 0x100],
            gain: 0,
        }
    }
}

pub fn wsg_init(w: &mut wsg, sound_rom: [u8; 0x100]) {
    //println!("wsg_init");

    for voice_no in 0..3 {
        w.voices[voice_no].frequency = 0;
        w.voices[voice_no].accumulator = 0;
        w.voices[voice_no].waveform_no = 0;
        w.voices[voice_no].volume = 0;
    }
    w.sound_rom = sound_rom;
    w.gain = 25;
}

pub fn wsg_write(w: &mut wsg, address: u8, value: u8) {
    //println!("wsg_write");

    // waveform 1
    if address == 0x5 {
        w.voices[0].waveform_no = value & 0b111;
    } else if address >= 0x10 && address <= 0x14 {
        let sample_no: u8 = address - 0x10;
        w.voices[0].frequency &= !(0b1111 << (sample_no * 4));
        w.voices[0].frequency |= (value as u32 & 0b1111) << (sample_no * 4);
    } else if address == 0x15 {
        w.voices[0].volume = value & 0xf;
    } else if address >= 0 && address <= 0x4 {
        let sample_no: u8 = address - 0;
        w.voices[0].accumulator &= !(0b1111 << (sample_no * 4));
        w.voices[0].accumulator |= (value as u32 & 0b1111) << (sample_no * 4);
    }

    // waveform 2
    if address == 0xa {
        w.voices[1].waveform_no = value & 0b111;
    } else if address >= 0x16 && address <= 0x19 {
        // voice 2 and 3 cannot set lowest 4 bits of frequency
        let sample_no: u8 = address - 0x16 + 1;
        w.voices[1].frequency &= !(0b1111 << (sample_no * 4));
        w.voices[1].frequency |= (value as u32 & 0b1111) << (sample_no * 4);
    } else if address == 0x1a {
        w.voices[1].volume = value & 0xf;
    } else if address >= 0x6 && address <= 0x9 {
        // voice 2 and 3 cannot set lowest 4 bits of accumulator
        let sample_no: u8 = address - 0x6 + 1;
        w.voices[1].accumulator &= !(0b1111 << (sample_no * 4));
        w.voices[1].accumulator |= (value as u32 & 0b1111) << (sample_no * 4);
    }

    // waveform 3
    if address == 0xf {
        w.voices[2].waveform_no = value & 0b111;
    } else if address >= 0x1b && address <= 0x1e {
        // voice 2 and 3 cannot set lowest 4 bits of frequency
        let sample_no: u8 = address - 0x1b + 1;
        w.voices[2].frequency &= !(0b1111 << (sample_no * 4));
        w.voices[2].frequency |= (value as u32 & 0b1111) << (sample_no * 4);
    } else if address == 0x1f {
        w.voices[2].volume = value & 0xf;
    } else if address >= 0xb && address <= 0xe {
        // voice 2 and 3 cannot set lowest 4 bits of accumulator
        let sample_no: u8 = address - 0xb + 1;
        w.voices[2].accumulator &= !(0b1111 << (sample_no * 4));
        w.voices[2].accumulator |= (value as u32 & 0b1111) << (sample_no * 4);
    }
}

pub fn wsg_play(w: &mut wsg, buffer: &mut Vec<i16>, buffer_len: i32) {
    //println!("wsg_play");

    for i in 0..buffer_len {
        let mut sample: i16 = 0;

        for voice_no in 0..3 {
            let v = &mut w.voices[voice_no];

            if v.frequency == 0 || v.volume == 0 {
                continue;
            }
            v.accumulator = (v.accumulator + v.frequency) & 0xfffff;
            // we use the highest 5 bits of the accumulator (which is
            // a 20 bit value) to select a sample (0-31) from a 32-step
            // waveform (1 step = 1 byte) in sound rom
            let sample_pos: i32 = (v.waveform_no * 32) as i32 + (v.accumulator >> 15) as i32;

            // convert unsigned 8 bit sample to a signed 16 bit sample,
            // and multiply it by the volume of the voice
            let voice_sample: i16 = ((w.sound_rom[sample_pos as usize] as i16) - 8) * (v.volume as i16);
            sample += voice_sample;
        }
        buffer[i as usize] = (sample as i32 * w.gain) as i16;
    }
}
