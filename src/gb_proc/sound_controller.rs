use std::convert::From;
use gb_proc::cpu::Handler;
use bitfield::Bitfield;

u8_enum!{
    SoundStatus {
        SoundOff = 0b0,
        SoundOn = 0b1,
    }
}

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
    SweepDirection {
        Down = 0b0,
        Up = 0b1,
    }
}

u8_enum!{
    SoundSteps {
        C15 = 0b0,
        C7  = 0b1,
    }
}

pub struct WaveDuty {
    pub wave_duty: f32,
}

pub struct Noise {
    pub rng: fn(u16) -> u16,
}

pub struct AudioLine<T> {
    pub frequency: u64,
    pub playing_left: bool,
    pub playing_right: bool,
    pub volume: f32,

    // Volume sweep stuff
    pub on: bool,
    pub counter: i64,
    pub envelope_counter: i64,
    pub consecutive: bool,

    pub sound: T,
}

impl<T> AudioLine<T> {
    pub fn new(sound: T) -> AudioLine<T> {
        AudioLine {
            frequency: 0,
            playing_left: false,
            playing_right: false,
            volume: 0.0,
            on: true,
            counter: 0,
            envelope_counter: 0,
            consecutive: false,
            sound: sound,
        }
    }
}

pub struct AudioBuffer {
    pub sound_1: AudioLine<WaveDuty>,
    pub sound_2: AudioLine<WaveDuty>,
    pub sound_4: AudioLine<Noise>,
}

pub struct SoundController {
    mapper: SoundMemoryMapper,
    wave_pattern: [u8; 16],
    buffer: AudioBuffer,
}

impl WavePattern {
    pub fn to_wave_duty(&self) -> f32 {
        match self {
            &WavePattern::C12 => 0.125,
            &WavePattern::C25 => 0.25,
            &WavePattern::C50 => 0.50,
            &WavePattern::C75 => 0.75,
        }
    }
}

fn sound_16_bit_rng(v: u16) -> u16 {
    let bit = (v ^ (v >> 1)) & 1;
    (v >> 1) | (bit << 15)
}

trait LineMapper {
    fn envelope_sweep(&self) -> u8;
    fn set_envelope_sweep(&mut self, sweep: u8);
    fn direction(&self) -> SweepDirection;
    fn volume(&self) -> u8;
    fn set_volume(&mut self, volume: u8);
}

struct Line1Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

struct Line2Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

struct Line4Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

impl<'a> LineMapper for Line1Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        self.mapper.sound_1_envelope_sweep()
    }
    fn set_envelope_sweep(&mut self, sweep: u8) {
        self.mapper.set_sound_1_envelope_sweep(sweep);
    }
    fn direction(&self) -> SweepDirection {
        self.mapper.sound_1_direction()
    }
    fn volume(&self) -> u8 {
        self.mapper.sound_1_volume()
    }
    fn set_volume(&mut self, volume: u8) {
        self.mapper.set_sound_1_volume(volume);
    }
}

impl<'a> LineMapper for Line2Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        self.mapper.sound_2_envelope_sweep()
    }
    fn set_envelope_sweep(&mut self, sweep: u8) {
        self.mapper.set_sound_2_envelope_sweep(sweep);
    }
    fn direction(&self) -> SweepDirection {
        self.mapper.sound_2_direction()
    }
    fn volume(&self) -> u8 {
        self.mapper.sound_2_volume()
    }
    fn set_volume(&mut self, volume: u8) {
        self.mapper.set_sound_2_volume(volume);
    }
}

impl<'a> LineMapper for Line4Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        self.mapper.sound_4_envelope_sweep()
    }
    fn set_envelope_sweep(&mut self, sweep: u8) {
        self.mapper.set_sound_4_envelope_sweep(sweep);
    }
    fn direction(&self) -> SweepDirection {
        self.mapper.sound_4_direction()
    }
    fn volume(&self) -> u8 {
        self.mapper.sound_4_volume()
    }
    fn set_volume(&mut self, volume: u8) {
        self.mapper.set_sound_4_volume(volume);
    }
}

fn update_sweep<T>(cycles: usize, line: &mut LineMapper, sound: &mut AudioLine<T>) {
    if sound.consecutive {
        sound.counter -= cycles as i64;

        if sound.counter < 0 {
            sound.consecutive = false;
            sound.counter = 0;
            sound.on = false;
        }
    }

    sound.envelope_counter -= cycles as i64;
    if sound.envelope_counter < 0 && line.envelope_sweep() > 0 {
        let direction = line.direction();
        let volume = line.volume();
        line.set_volume(
            match direction {
                SweepDirection::Up   => if volume == 0xF { 0x0 } else { volume + 1 },
                SweepDirection::Down => if volume == 0x0 { 0xF } else { volume - 1 },
            }
        );

        let envelope_sweep = line.envelope_sweep();
        line.set_envelope_sweep(envelope_sweep - 1);

        if line.envelope_sweep() > 0 {
            sound.envelope_counter += 65536;
        } else {
            sound.envelope_counter = 0;
        }
    }
}

impl SoundController {
    pub fn new() -> SoundController {
        SoundController {
            mapper: SoundMemoryMapper::new(),
            wave_pattern: [0; 16],
            buffer: AudioBuffer {
                sound_1: AudioLine::new(WaveDuty {
                    wave_duty: 0.5,
                }),
                sound_2: AudioLine::new(WaveDuty {
                    wave_duty: 0.5,
                }),
                sound_4: AudioLine::new(Noise {
                    rng: sound_16_bit_rng,
                }),
            }
        }
    }

    pub fn get_audio(&self) -> &AudioBuffer {
        &self.buffer
    }

    pub fn add_cycles(&mut self, cycles: usize) {
        update_sweep(
            cycles,
            &mut Line1Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_1);
        update_sweep(
            cycles,
            &mut Line2Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_2);
        update_sweep(
            cycles,
            &mut Line4Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_4);
    }

    fn write_callback(&mut self, address: u16) {
        match address {
            0xFF11 => {
                self.buffer.sound_1.sound.wave_duty =
                    self.mapper.sound_1_pattern().to_wave_duty();
            },
            0xFF12 => {
                self.buffer.sound_1.volume = self.mapper.sound_1_volume() as f32 / 16.0;
                if self.mapper.sound_1_envelope_sweep() > 0 {
                    self.buffer.sound_1.envelope_counter = 65536;
                }
            },
            0xFF13 | 0xFF14 => {
                let x = self.mapper.sound_1_frequency_low as u64 +
                    ((self.mapper.sound_1_frequency_high() as u64) << 8);
                self.buffer.sound_1.frequency = 131072 / (2048 - x);
            },
            0xFF16 => {
                self.buffer.sound_2.sound.wave_duty =
                    self.mapper.sound_2_pattern().to_wave_duty();
            },
            0xFF17 => {
                self.buffer.sound_2.volume = self.mapper.sound_2_volume() as f32 / 16.0;
                if self.mapper.sound_2_envelope_sweep() > 0 {
                    self.buffer.sound_2.envelope_counter = 65536;
                }
            },
            0xFF18 | 0xFF19 => {
                let x = self.mapper.sound_2_frequency_low as u64 +
                    ((self.mapper.sound_2_frequency_high() as u64) << 8);
                self.buffer.sound_2.frequency = 131072 / (2048 - x);
            },
            0xFF20 => {
                // TODO: probably the register itself needs to be updated to 64 - x
                self.buffer.sound_4.counter = 16384 *
                    (64 - self.mapper.sound_4_length() as i64);
            },
            0xFF21 => {
                self.buffer.sound_4.volume = self.mapper.sound_4_volume() as f32 / 16.0;
                if self.mapper.sound_4_envelope_sweep() > 0 {
                    self.buffer.sound_4.envelope_counter = 65536;
                }
            },
            0xFF22 => {
                let r = if self.mapper.sound_4_ratio() > 0 {
                    self.mapper.sound_4_ratio() as f64
                } else {
                    0.5
                };

                let s = self.mapper.sound_4_shift_clock() as i32;

                self.buffer.sound_4.frequency =
                    (1048576.0 / r / (2.0 as f64).powi(s + 1)) as u64;
            },
            0xFF25 => {
                self.buffer.sound_1.playing_left =
                    self.mapper.sound_1_to_so1() == SoundStatus::SoundOn;
                self.buffer.sound_1.playing_right =
                    self.mapper.sound_1_to_so2() == SoundStatus::SoundOn;

                self.buffer.sound_2.playing_left =
                    self.mapper.sound_2_to_so1() == SoundStatus::SoundOn;
                self.buffer.sound_2.playing_right =
                    self.mapper.sound_2_to_so2() == SoundStatus::SoundOn;

                // TODO: sound_3

                self.buffer.sound_4.playing_left = self.buffer.sound_4.on &&
                    self.mapper.sound_4_to_so1() == SoundStatus::SoundOn;
                self.buffer.sound_4.playing_right = self.buffer.sound_4.on &&
                    self.mapper.sound_4_to_so2() == SoundStatus::SoundOn;
            },
            0xFF24 => {
                // TODO: handle all channels
            },
            _ => {}
        }
    }
}

impl Handler for SoundController {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF23 => if self.buffer.sound_4.consecutive { 0b01000000 } else { 0 },
            0xFF30 ... 0xFF3F => self.wave_pattern[address as usize - 0xFF30],
            _ => self.mapper.read(address),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF23 => {
                self.buffer.sound_4.consecutive = v & 0b01000000 > 0;
                if v & 0b10000000 > 0 {
                    self.buffer.sound_4.on = true;
                }
            },
            0xFF30 ... 0xFF3F => self.wave_pattern[address as usize - 0xFF30] = v,
            _ => {
                self.mapper.write(address, v);
                self.write_callback(address);
            }
        }
    }
}

memory_mapper!{
    name: SoundMemoryMapper,
    fields: [
        0xFF13, sound_1_frequency_low, 0;
        0xFF15, sound_unknown_01, 0;
        0xFF18, sound_2_frequency_low, 0;
        0xFF1B, sound_3_length, 0;
        0xFF1D, sound_3_frequency_low, 0;
        0xFF1F, sound_unknown_02, 0;
        0xFF27, sound_unknown_03, 0;
        0xFF28, sound_unknown_04, 0;
        0xFF29, sound_unknown_05, 0;
        0xFF2A, sound_unknown_06, 0;
        0xFF2B, sound_unknown_07, 0;
        0xFF2C, sound_unknown_08, 0;
        0xFF2D, sound_unknown_09, 0;
        0xFF2E, sound_unknown_10, 0;
        0xFF2F, sound_unknown_11, 0;
    ],
    bitfields: {
        getters: [
            0xFF10, sound_1_sweep, 0, [
                // get_012, sound_1_sweep_shift, u8;
                // get_3,   sound_1_sweep,       SweepType;
                // get_456, sound_1_sweep_time,  SweepTime
            ];
            0xFF11, sound_1_wave_pattern, 0, [
                get_012345, sound_1_pattern_length, u8;
                get_67,     sound_1_pattern, WavePattern
            ];
            0xFF14, sound_1_frequency_high, 0, [
                get_7,   sound_1_restart, u8;
                get_6,   sound_1_consecutive, u8;
                get_012, sound_1_frequency_high, u8
            ];
            0xFF16, sound_2_wave_pattern, 0, [
                get_012345, sound_2_pattern_length, u8;
                get_67,     sound_2_pattern, WavePattern
            ];
            0xFF19, sound_2_frequency_hi, 0, [
                get_7,   sound_2_restart, u8;
                get_6,   sound_2_consecutive, u8;
                get_012, sound_2_frequency_high, u8
            ];
            0xFF1A, sound_3_register, 0, [
                // get_6, sound_3_on, u8
            ];
            0xFF1C, sound_3_output_level, 0, [
                // get_56, sound_3_output_level, OutputLevel
            ];
            0xFF1E, sound_3_frequency_hi, 0, [
                // get_6, sound_3_consecutive, u8
            ];
            0xFF20, sound_4_length, 0, [
                get_012345, sound_4_length, u8
            ];
            0xFF22, sound_4_polynomial, 0, [
                get_012,  sound_4_ratio,       u8;
                get_3,    sound_4_step,        SoundSteps;
                get_4567, sound_4_shift_clock, u8
            ]
        ],
        getter_setters: [
            0xFF12, sound_1_volume, 0, [
                get_4567, set_4567, sound_1_volume, set_sound_1_volume, u8;
                get_3,    set_3,    sound_1_direction, set_sound_1_direction, SweepDirection;
                get_012,  set_012,  sound_1_envelope_sweep, set_sound_1_envelope_sweep, u8
            ];
            0xFF17, sound_2_volume, 0, [
                get_4567, set_4567, sound_2_volume, set_sound_2_volume, u8;
                get_3,    set_3,    sound_2_direction, set_sound_2_direction, SweepDirection;
                get_012,  set_012,  sound_2_envelope_sweep, set_sound_2_envelope_sweep, u8
            ];
            0xFF21, sound_4_volume, 0, [
                get_4567, set_4567, sound_4_volume, set_sound_4_volume, u8;
                get_3,    set_3,    sound_4_direction, set_sound_4_direction, SweepDirection;
                get_012,  set_012,  sound_4_envelope_sweep, set_sound_4_envelope_sweep, u8
            ];
            0xFF24, sound_control, 0, [
                get_012, set_012, so1_volume,     set_so1_volume,     u8;
                get_3,   set_3,   so1_vin_status, set_so1_vin_status, SoundStatus;
                get_456, set_456, so2_volume,     set_so2_volume,     u8;
                get_7,   set_7,   so2_vin_status, set_so2_vin_status, SoundStatus
            ];
            0xFF25, selection_sound, 0xF3, [
                get_7, set_7, sound_4_to_so2, set_sound_4_to_so2, SoundStatus;
                get_6, set_6, sound_3_to_so2, set_sound_3_to_so2, SoundStatus;
                get_5, set_5, sound_2_to_so2, set_sound_2_to_so2, SoundStatus;
                get_4, set_4, sound_1_to_so2, set_sound_1_to_so2, SoundStatus;
                get_3, set_3, sound_4_to_so1, set_sound_4_to_so1, SoundStatus;
                get_2, set_2, sound_3_to_so1, set_sound_3_to_so1, SoundStatus;
                get_1, set_1, sound_2_to_so1, set_sound_2_to_so1, SoundStatus;
                get_0, set_0, sound_1_to_so1, set_sound_1_to_so1, SoundStatus
            ];
            0xFF26, sound_status_rw, 0, [
                get_7, set_7, master_status, set_master_status, SoundStatus
            ]
        ],
    },
}
