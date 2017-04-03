use gb_proc::cartridge::Cartridge;
use gb_proc::handler_holder::GBHandlerHolder;
use gb_proc::apu::NoisePattern;
use gb_proc::cpu::Cpu;

const VOLUME_MAX: i16 = 32000;
const AUDIO_BUFFER_SIZE: usize = 1470;

pub struct Emulator {
    pub cpu: Cpu,
    noise_7_bit: [u8; 127],
    noise_15_bit: [u8; 32767],
    phase: Phase,
    frequency: f64,
}

impl Emulator {
    pub fn from_data(data: &[u8], frequency: f64) -> Result<Emulator, String> {
        let handler = GBHandlerHolder::new(Cartridge::from_data(data));

        Ok(Emulator {
            cpu: Cpu::new(Box::new(handler)),
            frequency: frequency,
            noise_7_bit: generate_noise_7_bit(),
            noise_15_bit: generate_noise_15_bit(),
            phase: Phase {
                channel_1: 0.0,
                channel_2: 0.0,
                channel_3: 0.0,
                channel_4: 0.0,
            },
        })
    }

    pub fn generate_sound(&mut self) -> [i16; AUDIO_BUFFER_SIZE] {
        let audio = self.cpu.handler_holder.get_audio_buffer();

        let mut i = 0;

        let mut channel_1_phase = self.phase.channel_1;
        let channel_1_frequency = 131072 / (2048 - audio.sound_1().frequency());
        let channel_1_phase_inc = channel_1_frequency as f64 / self.frequency;
        let channel_1_volume = (VOLUME_MAX as f64 * audio.sound_1().volume() as f64
                                / 16.0 / 4.0) as i16;

        let mut channel_2_phase = self.phase.channel_2;
        let channel_2_frequency = 131072 / (2048 - audio.sound_2().frequency());
        let channel_2_phase_inc = channel_2_frequency as f64 / self.frequency;
        let channel_2_volume = (VOLUME_MAX as f64 * audio.sound_2().volume() as f64
                                / 16.0 / 4.0) as i16;

        let mut channel_3_phase = self.phase.channel_3;
        let mut channel_3_pattern = [0; 32];
        for i in 0..16 {
            channel_3_pattern[i * 2]     = (audio.sound_3().wave_pattern()[i] & 0xF0) >> 4;
            channel_3_pattern[i * 2 + 1] =  audio.sound_3().wave_pattern()[i] & 0x0F;
        }

        let channel_3_frequency = 65536 / (2048 - audio.sound_3().frequency());
        let channel_3_volume = (VOLUME_MAX as f64 * audio.sound_3().volume().to_volume() as f64
                                / 4.0) as i16;
        let channel_3_phase_inc = channel_3_frequency as f64 / self.frequency;

        let channel_4_pattern = match audio.sound_4().pattern() {
            NoisePattern::C15 => &self.noise_15_bit[..],
            NoisePattern::C7  => &self.noise_7_bit[..],
        };
        let mut channel_4_phase = self.phase.channel_4 % channel_4_pattern.len() as f64;
        let channel_4_phase_inc = audio.sound_4().frequency() as f64 / self.frequency;
        let channel_4_volume = (VOLUME_MAX as f64 * audio.sound_4().volume() as f64
                                / 16.0 / 4.0) as i16;

        let mut out = [0; AUDIO_BUFFER_SIZE];

        while i < out.len() {
            let channel_1 = if channel_1_phase < audio.sound_1().wave_duty() as f64 {
                channel_1_volume
            } else {
                -channel_1_volume
            };

            let channel_2 = if channel_2_phase < audio.sound_2().wave_duty() as f64 {
                channel_2_volume
            } else {
                -channel_2_volume
            };

            let channel_3_index = (channel_3_phase * 32.0) as usize;
            let channel_3 = ((channel_3_pattern[channel_3_index] as f32 - 7.0)
                            / 8.0 * channel_3_volume as f32) as i16;

            let channel_4_index = channel_4_phase as usize;
            let channel_4_bit = channel_4_pattern[channel_4_index];
            let channel_4 = if channel_4_bit == 1 {
                channel_4_volume
            } else {
                -channel_4_volume
            };

            out[i] =
                  ((if audio.sound_1().playing_left() { channel_1 } else { 0 }) +
                (if audio.sound_2().playing_left() { channel_2 } else { 0 }) +
                (if audio.sound_3().playing_left() { channel_3 } else { 0 }) +
                (if audio.sound_4().playing_left() { channel_4 } else { 0 })) / 4;

            out[i + 1] =
               ((if audio.sound_1().playing_right() { channel_1 } else { 0 }) +
                (if audio.sound_2().playing_right() { channel_2 } else { 0 }) +
                (if audio.sound_3().playing_right() { channel_3 } else { 0 }) +
                (if audio.sound_4().playing_right() { channel_3 } else { 0 })) / 4;

            channel_1_phase = (channel_1_phase + channel_1_phase_inc) % 1.0;
            channel_2_phase = (channel_2_phase + channel_2_phase_inc) % 1.0;
            channel_3_phase = (channel_3_phase + channel_3_phase_inc) % 1.0;
            channel_4_phase = (channel_4_phase + channel_4_phase_inc)
                            % channel_4_pattern.len() as f64;

            i += 2;
        }

        self.phase = Phase {
            channel_1: channel_1_phase,
            channel_2: channel_2_phase,
            channel_3: channel_3_phase,
            channel_4: channel_4_phase,
        };

        out
    }
}

#[derive(Clone, Copy)]
pub struct Phase {
    channel_1: f64,
    channel_2: f64,
    channel_3: f64,
    channel_4: f64,
}

fn sound_rng(v: u16, generator: u16) -> u16 {
    let bit = v & 1;
    let next = v >> 1;
    if bit != 0 {
        next ^ generator
    } else {
        next
    }
}

fn generate_noise_array(out: &mut [u8], generator: u16, bits: u32) {
    assert!(bits < 16);

    let mut v = 2u16.pow(bits) - 1;

    for i in 0..out.len() {
        out[i] = (v & 1) as u8;
        v = sound_rng(v, generator);
    }

    // Let's verify that we have reached the end of the cycle
    assert_eq!(2u16.pow(bits) - 1, v);
}

fn generate_noise_7_bit() -> [u8; 127] {
    let mut pattern = [0; 127];
    generate_noise_array(&mut pattern, 0b1000001, 7);
    pattern
}

fn generate_noise_15_bit() -> [u8; 32767] {
    let mut pattern = [0; 32767];
    generate_noise_array(&mut pattern, 0b100000000000001, 15);
    pattern
}
