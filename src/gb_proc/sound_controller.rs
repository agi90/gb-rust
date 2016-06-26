use std::convert::From;
use gb_proc::cpu::Handler;
use bitfield::Bitfield;

#[derive(Clone, Copy, Debug)]

u8_enum!{
    WavePattern {
        C12 = 0b00,
        C25 = 0b01,
        C50 = 0b10,
        C75 = 0b11,
    }
}

u8_enum!{
    OutputLevel {
        Mute          = 0b00,
        WavePattern   = 0b01,
        RightShifted2 = 0b10,
        RightShifted4 = 0b11,
    }
}

u8_enum!{
    SweepTime {
        NoChange = 0b000,
        Ms7800   = 0b001,
        Ms15600  = 0b010,
        Ms23400  = 0b011,
        Ms31300  = 0b100,
        Ms39100  = 0b101,
        Ms46900  = 0b110,
        Ms54700  = 0b111,
    }
}

u8_enum!{
    SweepType {
        Addition = 0b0,
        Subtraction = 0b1,
    }
}

u8_enum!{
    EnvelopeAmplication {
        Attenuate = 0b0,
        Amplify   = 0b1,
    }
}

u8_enum!{
    SoundSteps {
        C15 = 0b0,
        C7  = 0b1,
    }
}

u8_enum!{
    SoundRatio {
        Two     = 0b000,
        One     = 0b001,
        Half    = 0b010,
        Third   = 0b011,
        Fourth  = 0b100,
        Fifth   = 0b101,
        Sixth   = 0b110,
        Seventh = 0b111,
    }
}

pub struct SoundController {
    mapper: SoundMemoryMapper,
    wave_pattern: [u8; 16],
}

impl SoundController {
    pub fn new() -> SoundController {
        SoundController {
            mapper: SoundMemoryMapper::new(),
            wave_pattern: [0; 16],
        }
    }

    pub fn add_cycles(&mut self, _: usize) {
        // TODO:
    }
}

impl Handler for SoundController {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF30 ... 0xFF3F => self.wave_pattern[address as usize - 0xFF30],
            _ => self.mapper.read(address),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF30 ... 0xFF3F => self.wave_pattern[address as usize - 0xFF30] = v,
            _ => self.mapper.write(address, v),
        }
    }
}

memory_mapper!{
    name: SoundMemoryMapper,
    fields: [
        0xFF13, sound_1_frequency_low, 0;
        0xFF18, sound_2_frequency_low, 0;
        0xFF1B, sound_3_length, 0;
        0xFF1D, sound_3_frequency_low, 0;
        0xFF20, sound_4_length, 0;
        0xFF26, sound_status, 0;
        0xFF25, selection_sound, 0xF3;
    ],
    bitfields: {
        getters: [
            0xFF10, sound_1_sweep, 0, [
                // get_012, get_sound_1_sweep_shift, u8;
                // get_3,   get_sound_1_sweep,       SweepType;
                // get_456, get_sound_1_sweep_time,  SweepTime
            ];
            0xFF11, sound_1_wave_pattern, 0, [
                // get_012345, get_sound_1_pattern_length, u8;
                // get_67,     get_sound_1_pattern, WavePattern
            ];
            0xFF12, sound_1_register, 0, [
                // get_012,  get_sound_1_envelope_sweep, u8;
                // get_3,    get_sound_1_envelope_amplification, EnvelopeAmplication;
                // get_4567, get_sound_1_envelope_volume, u8
            ];
            0xFF14, sound_1_frequency_high, 0, [
                // get_6, get_sound_1_consecutive, u8
            ];
            0xFF16, sound_2_wave_pattern, 0, [
                // get_012345, get_sound_2_pattern_length, u8;
                // get_67,     get_sound_2_pattern, WavePattern
            ];
            0xFF17, sound_2_sweep, 0, [
                // get_012, get_sound_2_sweep_shift, u8;
                // get_3,   get_sound_2_sweep,       SweepType;
                // get_456, get_sound_2_sweep_time,  SweepTime
            ];
            0xFF19, sound_2_frequency_hi, 0, [
                // get_6, get_sound_2_consecutive, u8
            ];
            0xFF1A, sound_3_register, 0, [
                // get_6, get_sound_3_on, u8
            ];
            0xFF1C, sound_3_output_level, 0, [
                // get_56, get_sound_3_output_level, OutputLevel
            ];
            0xFF1E, sound_3_frequency_hi, 0, [
                // get_6, get_sound_3_consecutive, u8
            ];
            0xFF21, sound_4_sweep, 0, [
                // get_012, get_sound_4_sweep_shift, u8;
                // get_3,   get_sound_4_sweep,       SweepType;
                // get_456, get_sound_4_sweep_time,  SweepTime
            ];
            0xFF22, sound_4_polynomial, 0, [
                // get_012,  get_sound_4_ratio,       SoundRatio;
                // get_3,    get_sound_4_step,        SoundSteps;
                // get_4567, get_sound_4_shift_clock, u8
            ];
            0xFF23, sound_4_frequency, 0, [
                // get_6, get_sound_4_consecutive, u8
            ];
            0xFF24, sound_control, 0, [
                // get_012, get_sound_1_level, u8;
                // get_3,   get_sound_1_on,    u8;
                // get_456, get_sound_2_level, u8;
                // get_7,   get_sound_2_on,    u8
            ]
        ],
        getter_setters: [
        ],
    },
}
