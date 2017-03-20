/** This file is mostly based on http://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware */

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

impl OutputLevel {
    pub fn to_volume(&self) -> f32 {
        match self {
            &OutputLevel::Mute          => 0.0,
            &OutputLevel::WavePattern   => 1.0,
            &OutputLevel::RightShifted2 => 0.5,
            &OutputLevel::RightShifted4 => 0.25,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            &OutputLevel::Mute          => 0,
            &OutputLevel::WavePattern   => 1,
            &OutputLevel::RightShifted2 => 2,
            &OutputLevel::RightShifted4 => 3,
        }
    }

    pub fn from_u8(v: u8) -> OutputLevel {
        match v {
            0 => OutputLevel::Mute,
            1 => OutputLevel::WavePattern,
            2 => OutputLevel::RightShifted2,
            3 => OutputLevel::RightShifted4,
            _ => unreachable!(),
        }
    }
}

u8_enum!{
    SweepDirection {
        Down = 0b0,
        Up = 0b1,
    }
}

u8_enum!{
    ToneSweepDirection {
        Up = 0b0,
        Down = 0b1,
    }
}

impl ToneSweepDirection {
    fn to_sign(self) -> i64 {
        match self {
            ToneSweepDirection::Down => -1,
            ToneSweepDirection::Up   =>  1,
        }
    }
}

u8_enum!{
    NoisePattern {
        C15 = 0b0,
        C7  = 0b1,
    }
}

pub struct Sweep {
    pub counter: i64,
    pub enabled: bool,
    pub shift: i64,
}

pub struct SweepWaveDuty {
    pub volume: u8,
    pub wave_duty: f32,
    pub shadow_frequency: u64,
    pub sweep: Sweep,
}

trait VolumeSweep {
    fn volume(&self) -> u8;
    fn set_volume(&mut self, v: u8);
}

impl VolumeSweep for AudioLine<SweepWaveDuty> {
    fn volume(&self) -> u8 { self.sound.volume }
    fn set_volume(&mut self, v: u8) { self.sound.volume = v; }
}

impl TriggerEvent for AudioLine<SweepWaveDuty> {
    fn trigger_event(&mut self, mapper: &mut LineMapper) {
        self.sound.shadow_frequency = mapper.frequency();

        {
            let sweep = &mut self.sound.sweep;

            let period = mapper.sweep_period() as i64;
            sweep.counter = if period > 0 { period } else { 8 };
            sweep.enabled = period > 0 || sweep.shift > 0;
        }

        if self.sound.sweep.shift > 0 {
            self.update_frequency(false, mapper);
        }
    }
    fn default_length(&self) -> i64 {
        64
    }
}

impl <T> AudioLine<T> {
    fn write_frequency(&mut self, mapper: &mut LineMapper) {
        self.frequency = mapper.frequency();
    }
    fn update_dac(&mut self, mapper: &mut LineMapper) {
        if mapper.dac_off() {
            self.on = false;
        }
    }
}

impl AudioLine<SweepWaveDuty> {
    fn update_frequency(&mut self, update: bool, mapper: &mut LineMapper) {
        let sign = mapper.sweep_direction().to_sign();

        let operand = (self.sound.shadow_frequency >> self.sound.sweep.shift) as i64;
        let new_frequency = (self.sound.shadow_frequency as i64 + sign * operand) as u64;

        if new_frequency >= 2048 {
            self.on = false;
            return;
        }

        if self.sound.sweep.shift > 0 && update {
            self.frequency = new_frequency;
            self.sound.shadow_frequency = new_frequency;
        }
    }
}

pub struct WaveDuty {
    pub volume: u8,
    pub wave_duty: f32,
}

impl TriggerEvent for AudioLine<WaveDuty> {
    fn trigger_event(&mut self, _: &mut LineMapper) {
        // TODO:
    }
    fn default_length(&self) -> i64 {
        64
    }
}

impl VolumeSweep for AudioLine<WaveDuty> {
    fn volume(&self) -> u8 { self.sound.volume }
    fn set_volume(&mut self, v: u8) { self.sound.volume = v; }
}

pub struct Noise {
    pub pattern: NoisePattern,
    pub volume: u8,
}

impl TriggerEvent for AudioLine<Noise> {
    fn trigger_event(&mut self, _: &mut LineMapper) {
        // TODO:
    }
    fn default_length(&self) -> i64 {
        64
    }
}

impl VolumeSweep for AudioLine<Noise> {
    fn volume(&self) -> u8 { self.sound.volume }
    fn set_volume(&mut self, v: u8) { self.sound.volume = v; }
}

pub struct Wave {
    pub wave_pattern: [u8; 16],
    pub volume: OutputLevel,
}

impl TriggerEvent for AudioLine<Wave> {
    fn trigger_event(&mut self, _: &mut LineMapper) {
        // TODO:
    }
    fn default_length(&self) -> i64 {
        256
    }
}

impl VolumeSweep for AudioLine<Wave> {
    fn volume(&self) -> u8 { self.sound.volume.to_u8() }
    fn set_volume(&mut self, v: u8) {
        self.sound.volume = OutputLevel::from_u8(v);
    }
}

pub struct AudioLine<T> {
    pub id: usize,
    pub frequency: u64,
    pub playing_left: bool,
    pub playing_right: bool,

    // Volume sweep stuff
    pub on: bool,
    pub counter: i64,
    pub envelope_counter: i64,

    pub sound: T,
}

impl<T> AudioLine<T> {
    pub fn new(id: usize, sound: T) -> AudioLine<T> {
        AudioLine {
            id: id,
            frequency: 0,
            playing_left: false,
            playing_right: false,
            on: false,
            counter: 0,
            envelope_counter: 0,
            sound: sound,
        }
    }
}

trait TriggerEvent {
    fn trigger_event(&mut self, line_mapper: &mut LineMapper);
    fn default_length(&self) -> i64;
}

pub struct AudioBuffer {
    pub sound_1: AudioLine<SweepWaveDuty>,
    pub sound_2: AudioLine<WaveDuty>,
    pub sound_3: AudioLine<Wave>,
    pub sound_4: AudioLine<Noise>,
}

pub struct SoundController {
    mapper: SoundMemoryMapper,
    buffer: AudioBuffer,
    frame_sequencer: FrameSequencer,
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

trait LineMapper {
    fn envelope_sweep(&self) -> u8;
    fn direction(&self) -> SweepDirection;
    fn initial_volume(&self) -> u8;
    fn frequency(&self) -> u64;
    fn sweep_period(&self) -> u8;
    fn sweep_direction(&self) -> ToneSweepDirection;
    fn consecutive(&self) -> bool;
    fn dac_off(&self) -> bool;
    fn write(&mut self, address: u16, v: u8);
}

struct Line1Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

struct Line2Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

struct Line3Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

struct Line4Mapper<'a> {
    mapper: &'a mut SoundMemoryMapper,
}

impl<'a> LineMapper for Line1Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        self.mapper.sound_1_envelope_sweep()
    }
    fn direction(&self) -> SweepDirection {
        self.mapper.sound_1_direction()
    }
    fn initial_volume(&self) -> u8 {
        self.mapper.sound_1_volume()
    }
    fn frequency(&self) -> u64 {
        self.mapper.sound_1_frequency_low as u64 +
            ((self.mapper.sound_1_frequency_high() as u64) << 8)
    }
    fn sweep_period(&self) -> u8 {
        self.mapper.sound_1_sweep_period()
    }
    fn sweep_direction(&self) -> ToneSweepDirection {
        self.mapper.sound_1_sweep_direction()
    }
    fn consecutive(&self) -> bool {
        self.mapper.sound_1_consecutive() > 0
    }
    fn dac_off(&self) -> bool {
        self.mapper.sound_1_volume() == 0
            && self.mapper.sound_1_direction() == SweepDirection::Down
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
}

impl<'a> LineMapper for Line3Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        // This line does not support sweep
        0
    }
    fn direction(&self) -> SweepDirection {
        panic!();
    }
    fn initial_volume(&self) -> u8 {
        self.mapper.sound_3_output_level().to_u8()
    }
    fn frequency(&self) -> u64 {
        self.mapper.sound_3_frequency_low as u64 +
            ((self.mapper.sound_3_frequency_high() as u64) << 8)
    }
    fn sweep_period(&self) -> u8 {
        panic!();
    }
    fn sweep_direction(&self) -> ToneSweepDirection {
        panic!();
    }
    fn consecutive(&self) -> bool {
        self.mapper.sound_3_consecutive() > 0
    }
    fn dac_off(&self) -> bool {
        self.mapper.sound_3_on() == SoundStatus::SoundOff
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
}

impl<'a> LineMapper for Line2Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        self.mapper.sound_2_envelope_sweep()
    }
    fn direction(&self) -> SweepDirection {
        self.mapper.sound_2_direction()
    }
    fn initial_volume(&self) -> u8 {
        self.mapper.sound_2_volume()
    }
    fn frequency(&self) -> u64 {
        self.mapper.sound_2_frequency_low as u64 +
            ((self.mapper.sound_2_frequency_high() as u64) << 8)
    }
    fn sweep_period(&self) -> u8 {
        panic!();
    }
    fn sweep_direction(&self) -> ToneSweepDirection {
        panic!();
    }
    fn consecutive(&self) -> bool {
        self.mapper.sound_2_consecutive() > 0
    }
    fn dac_off(&self) -> bool {
        self.mapper.sound_2_volume() == 0
            && self.mapper.sound_2_direction() == SweepDirection::Down
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
}

impl<'a> LineMapper for Line4Mapper<'a> {
    fn envelope_sweep(&self) -> u8 {
        self.mapper.sound_4_envelope_sweep()
    }
    fn direction(&self) -> SweepDirection {
        self.mapper.sound_4_direction()
    }
    fn initial_volume(&self) -> u8 {
        self.mapper.sound_4_volume()
    }
    fn frequency(&self) -> u64 {
        let r = if self.mapper.sound_4_ratio() > 0 {
            self.mapper.sound_4_ratio() as f64
        } else {
            0.5
        };

        let s = self.mapper.sound_4_shift_clock() as i32;

        (524288.0 / r / (2.0 as f64).powi(s + 1)) as u64
    }
    fn sweep_period(&self) -> u8 {
        panic!();
    }
    fn sweep_direction(&self) -> ToneSweepDirection {
        panic!();
    }
    fn consecutive(&self) -> bool {
        self.mapper.sound_4_consecutive() > 0
    }
    fn dac_off(&self) -> bool {
        self.mapper.sound_4_volume() == 0
            && self.mapper.sound_4_direction() == SweepDirection::Down
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
}

fn update_length<T>(line: &mut LineMapper, sound: &mut AudioLine<T>) {
    if !line.consecutive() {
        return;
    }

    if sound.counter > 0 {
        sound.counter -= 1;
    }

    if sound.counter == 0 {
        sound.on = false;
    }
}

fn update_volume<T>(line: &mut LineMapper, sound: &mut AudioLine<T>)
    where AudioLine<T> : VolumeSweep {
    let sweep = line.envelope_sweep();
    if sweep == 0 {
        return;
    }

    let volume = sound.volume();
    let direction = line.direction();

    let is_sweep_completed = match direction {
        SweepDirection::Up   => volume == 0xF,
        SweepDirection::Down => volume == 0x0,
    };

    if is_sweep_completed {
        return;
    }

    if sound.envelope_counter > 0 {
        sound.envelope_counter -= 1;
    }

    if sound.envelope_counter == 0 && sweep > 0 && !is_sweep_completed {
        let new_volume = match direction {
            SweepDirection::Up   => volume + 1,
            SweepDirection::Down => volume - 1,
        };

        sound.set_volume(new_volume);
        sound.envelope_counter = sweep as i64;
    }
}

fn trigger_event<T>(sound: &mut AudioLine<T>, line_mapper: &mut LineMapper,
                    frame_sequencer: &FrameSequencer)
    where AudioLine<T>: TriggerEvent + VolumeSweep {
    if sound.counter == 0 {
        sound.counter = sound.default_length();
        if !frame_sequencer.next_step_clocks_length() && line_mapper.consecutive() {
            // When the next step in the sequencer will not fire a Lenght clock
            // the counter is actually initialized to MAX_LENGTH - 1
            sound.counter -= 1;
        }
    }

    sound.set_volume(line_mapper.initial_volume());
    sound.envelope_counter = line_mapper.envelope_sweep() as i64;

    // Channel specific logic
    sound.trigger_event(line_mapper);

    // Even if the Trigger Event fired, if the DAC
    // is off the sound should stay off.
    if line_mapper.dac_off() {
        sound.on = false;
    }
}

fn extra_length_check<T>(sound: &mut AudioLine<T>,
                      frame_sequencer: &FrameSequencer) {
    let extra_check = sound.counter > 0
        && !frame_sequencer.next_step_clocks_length();

    if extra_check {
        sound.counter -= 1;
        if sound.counter == 0 {
            sound.on = false;
        }
    }
}

/** Behavior for writing to the NRx4 register. */
fn write_nrx4<T>(sound: &mut AudioLine<T>, mapper: &mut LineMapper,
              frame_sequencer: &FrameSequencer, address: u16, v: u8)
    where AudioLine<T>: TriggerEvent + VolumeSweep {
    let turns_length_on = !mapper.consecutive() && v & 0b01000000 > 0;

    mapper.write(address, v);

    if turns_length_on {
        // If we are turning the length counter on, we might
        // have to perform an extra length check.
        extra_length_check(sound, frame_sequencer);
    }

    if v & 0b10000000 > 0 {
        sound.on = true;
        trigger_event(sound, mapper, frame_sequencer);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SequencerEvent {
    Length,
    LengthSweep,
    Volume,
}

struct FrameSequencer {
    cycles: i64,
    step: u8,
}


/*
 * From: http://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Frame_Sequencer
 * The frame sequencer generates low frequency clocks for the modulation units.
 * It is clocked by a 512 Hz timer.
 *
 * Step   Length Ctr  Vol Env     Sweep
 * ---------------------------------------
 * 0      Clock       -           -
 * 1      -           -           -
 * 2      Clock       -           Clock
 * 3      -           -           -
 * 4      Clock       -           -
 * 5      -           -           -
 * 6      Clock       -           Clock
 * 7      -           Clock       -
 * ---------------------------------------
 * Rate   256 Hz      64 Hz       128 Hz
 *
 */
impl FrameSequencer {
    pub fn new() -> FrameSequencer {
        let mut s = FrameSequencer {
            cycles: 0,
            step: 0,
        };

        s.reset();
        s
    }

    pub fn reset(&mut self) {
        self.cycles = 8192;
        self.step = 0;
    }

    fn next_event(&self) -> Option<SequencerEvent> {
        match self.step {
            0 => Some(SequencerEvent::Length),
            1 => None,
            2 => Some(SequencerEvent::LengthSweep),
            3 => None,
            4 => Some(SequencerEvent::Length),
            5 => None,
            6 => Some(SequencerEvent::LengthSweep),
            7 => Some(SequencerEvent::Volume),
            _ => panic!("Unexpected frame sequencer step."),
        }
    }

    /** The gb performs an extra length check when the
     * sequencer's next step will not clock length. */
    pub fn next_step_clocks_length(&self) -> bool {
        match self.next_event() {
            Some(SequencerEvent::Length) => true,
            Some(SequencerEvent::LengthSweep) => true,
            _ => false,
        }
    }

    pub fn add_cycles(&mut self, cycles: usize) -> Option<SequencerEvent> {
        self.cycles -= cycles as i64;

        if self.cycles > 0 {
            return None;
        }

        let event = self.next_event();

        self.cycles += 8192;
        self.step = (self.step + 1) % 8;

        event
    }
}

impl SoundController {
    pub fn new() -> SoundController {
        SoundController {
            frame_sequencer: FrameSequencer::new(),
            mapper: SoundMemoryMapper::new(),
            buffer: AudioBuffer {
                sound_1: AudioLine::new(1, SweepWaveDuty {
                    volume: 0,
                    wave_duty: 0.5,
                    shadow_frequency: 0,
                    sweep: Sweep {
                        counter: 0,
                        enabled: false,
                        shift: 0,
                    },
                }),
                sound_2: AudioLine::new(2, WaveDuty {
                    volume: 0,
                    wave_duty: 0.5,
                }),
                sound_3: AudioLine::new(3, Wave {
                    volume: OutputLevel::Mute,
                    wave_pattern: [0; 16],
                }),
                sound_4: AudioLine::new(4, Noise {
                    volume: 0,
                    pattern: NoisePattern::C7,
                }),
            }
        }
    }

    pub fn get_audio(&self) -> &AudioBuffer {
        &self.buffer
    }

    pub fn add_cycles(&mut self, cycles: usize) {
        if let Some(ev) = self.frame_sequencer.add_cycles(cycles) {
            match ev {
                SequencerEvent::Length => self.update_length(),
                SequencerEvent::LengthSweep => {
                    self.update_length();
                    self.update_sweep();
                },
                SequencerEvent::Volume => self.update_volume(),
            }
        }
    }

    fn update_sweep(&mut self) {
        let period = self.mapper.sound_1_sweep_period() as i64;

        {
            let sound = &mut self.buffer.sound_1.sound;
            sound.sweep.counter -= 1;
            if sound.sweep.counter > 0 {
                return;
            }

            sound.sweep.counter = if period > 0 {
                period
            } else {
                8
            };

            if !sound.sweep.enabled {
                return;
            }
        }

        if period > 0 {
            let mut line_mapper = Line1Mapper{ mapper: &mut self.mapper };
            self.buffer.sound_1.update_frequency(true, &mut line_mapper);
            self.buffer.sound_1.update_frequency(false, &mut line_mapper);
        }
    }

    fn update_length(&mut self) {
        update_length(
            &mut Line1Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_1);
        update_length(
            &mut Line2Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_2);
        update_length(
            &mut Line3Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_3);
        update_length(
            &mut Line4Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_4);
    }

    fn update_volume(&mut self) {
        update_volume(
            &mut Line1Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_1);
        update_volume(
            &mut Line2Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_2);
        update_volume(
            &mut Line4Mapper{ mapper: &mut self.mapper },
            &mut self.buffer.sound_4);
    }

    fn write_callback(&mut self, address: u16) {
        match address {
            0xFF10 => {
                let sweep = &mut self.buffer.sound_1.sound.sweep;
                sweep.shift = self.mapper.sound_1_sweep_shift() as i64;
            },
            0xFF11 => {
                self.buffer.sound_1.sound.wave_duty =
                    self.mapper.sound_1_pattern().to_wave_duty();
                self.buffer.sound_1.counter = 64 - self.mapper.sound_1_length() as i64;
            },
            0xFF12 => {
                self.buffer.sound_1.update_dac(&mut Line1Mapper{ mapper: &mut self.mapper });
            },
            0xFF13 | 0xFF14 => {
                self.buffer.sound_1.write_frequency(&mut Line1Mapper{ mapper: &mut self.mapper });
            },
            0xFF16 => {
                self.buffer.sound_2.sound.wave_duty =
                    self.mapper.sound_2_pattern().to_wave_duty();
                self.buffer.sound_2.counter = 64 - self.mapper.sound_2_length() as i64;
            },
            0xFF17 => {
                self.buffer.sound_2.update_dac(&mut Line2Mapper{ mapper: &mut self.mapper });
            },
            0xFF18 | 0xFF19 => {
                self.buffer.sound_2.write_frequency(&mut Line2Mapper{ mapper: &mut self.mapper });
            },
            0xFF1A => {
                self.buffer.sound_3.update_dac(&mut Line3Mapper{ mapper: &mut self.mapper });
            }
            0xFF1B => {
                self.buffer.sound_3.counter = 256 - self.mapper.sound_3_length as i64;
            },
            0xFF1C => {
                self.buffer.sound_3.sound.volume = self.mapper.sound_3_output_level();
            },
            0xFF1D | 0xFF1E => {
                self.buffer.sound_3.write_frequency(&mut Line3Mapper{ mapper: &mut self.mapper });
            }
            0xFF20 => {
                self.buffer.sound_4.counter = 64 - self.mapper.sound_4_length() as i64;
            },
            0xFF21 => {
                self.buffer.sound_4.update_dac(&mut Line4Mapper{ mapper: &mut self.mapper });
            },
            0xFF22 => {
                self.buffer.sound_4.write_frequency(&mut Line4Mapper{ mapper: &mut self.mapper });
                self.buffer.sound_4.sound.pattern = self.mapper.sound_4_step();
            },
            0xFF25 => {
                self.buffer.sound_1.playing_left = self.buffer.sound_1.on &&
                    self.mapper.sound_1_to_so1() == SoundStatus::SoundOn;
                self.buffer.sound_1.playing_right = self.buffer.sound_1.on &&
                    self.mapper.sound_1_to_so2() == SoundStatus::SoundOn;

                self.buffer.sound_2.playing_left = self.buffer.sound_2.on &&
                    self.mapper.sound_2_to_so1() == SoundStatus::SoundOn;
                self.buffer.sound_2.playing_right = self.buffer.sound_2.on &&
                    self.mapper.sound_2_to_so2() == SoundStatus::SoundOn;

                self.buffer.sound_3.playing_left =
                    self.mapper.sound_3_on() == SoundStatus::SoundOn &&
                        self.mapper.sound_3_to_so1() == SoundStatus::SoundOn;
                self.buffer.sound_3.playing_right =
                    self.mapper.sound_3_on() == SoundStatus::SoundOn &&
                        self.mapper.sound_3_to_so2() == SoundStatus::SoundOn;

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
            0xFF26 => {
                let v = (if self.mapper.master_status() == SoundStatus::SoundOn { 0b10000000 } else { 0 }) +
                (if self.buffer.sound_1.on { 0b00000001 } else { 0 }) +
                (if self.buffer.sound_2.on { 0b00000010 } else { 0 }) +
                (if self.buffer.sound_3.on { 0b00000100 } else { 0 }) +
                (if self.buffer.sound_4.on { 0b00001000 } else { 0 });

                v | 0b01110000
            },
            0xFF30 ... 0xFF3F => self.buffer.sound_3.sound.wave_pattern[address as usize - 0xFF30],
            _ => self.mapper.read(address),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        if self.mapper.master_status() == SoundStatus::SoundOff
                && address != 0xFF26 {
            // While the APU is off, the cpu can only write to FF26
            // to turn the APU on.
            return;
        }

        match address {
            0xFF14 => {
                write_nrx4(&mut self.buffer.sound_1,
                           &mut Line1Mapper{ mapper: &mut self.mapper },
                           &self.frame_sequencer,
                           address, v);
            },
            0xFF19 => {
                write_nrx4(&mut self.buffer.sound_2,
                           &mut Line2Mapper{ mapper: &mut self.mapper },
                           &self.frame_sequencer,
                           address, v);
            },
            0xFF1E => {
                write_nrx4(&mut self.buffer.sound_3,
                           &mut Line3Mapper{ mapper: &mut self.mapper },
                           &self.frame_sequencer,
                           address, v);
            },
            0xFF23 => {
                write_nrx4(&mut self.buffer.sound_4,
                           &mut Line4Mapper{ mapper: &mut self.mapper },
                           &self.frame_sequencer,
                           address, v);
            },
            0xFF26 => {
                let new_master_status = if v & 0b10000000 > 0 {
                    SoundStatus::SoundOn
                } else {
                    SoundStatus::SoundOff
                };

                if new_master_status == SoundStatus::SoundOff {
                    self.frame_sequencer.reset();
                    self.buffer.sound_1.on = false;
                    self.buffer.sound_2.on = false;
                    self.buffer.sound_3.on = false;
                    self.buffer.sound_4.on = false;

                    for address in 0xFF10..0xFF30 {
                        if address != 0xFF26 {
                            self.write(address, 0x00);
                        }
                    }
                }

                self.mapper.set_master_status(new_master_status);
            },
            0xFF30 ... 0xFF3F => self.buffer.sound_3.sound.wave_pattern[address as usize - 0xFF30] = v,
            _ => {
                self.mapper.write(address, v);
            }
        }
        self.write_callback(address);
    }
}

memory_mapper!{
    name: SoundMemoryMapper,
    fields: [
        0xFF13, 0xFF, sound_1_frequency_low, 0;
        0xFF15, 0xFF, sound_unknown_01, 0;
        0xFF18, 0xFF, sound_2_frequency_low, 0;
        0xFF1B, 0xFF, sound_3_length, 0;
        0xFF1D, 0xFF, sound_3_frequency_low, 0;
        0xFF1F, 0xFF, sound_unknown_02, 0;
        0xFF27, 0xFF, sound_unknown_03, 0;
        0xFF28, 0xFF, sound_unknown_04, 0;
        0xFF29, 0xFF, sound_unknown_05, 0;
        0xFF2A, 0xFF, sound_unknown_06, 0;
        0xFF2B, 0xFF, sound_unknown_07, 0;
        0xFF2C, 0xFF, sound_unknown_08, 0;
        0xFF2D, 0xFF, sound_unknown_09, 0;
        0xFF2E, 0xFF, sound_unknown_10, 0;
        0xFF2F, 0xFF, sound_unknown_11, 0;
    ],
    bitfields: {
        getters: [
            0xFF11, 0b00111111, sound_1_wave_pattern, 0, [
                get_012345, sound_1_length,  u8;
                get_67,     sound_1_pattern, WavePattern
            ];
            0xFF16, 0b00111111, sound_2_wave_pattern, 0, [
                get_012345, sound_2_length,  u8;
                get_67,     sound_2_pattern, WavePattern
            ];
            0xFF1A, 0b01111111, sound_3_register, 0, [
                get_7, sound_3_on, SoundStatus
            ];
            0xFF1C, 0b10011111, sound_3_output_level, 0, [
                get_56, sound_3_output_level, OutputLevel
            ];
            0xFF1E, 0b10111111, sound_3_frequency_hi, 0, [
                get_6,   sound_3_consecutive, u8;
                get_012, sound_3_frequency_high, u8
            ];
            0xFF20, 0b11111111, sound_4_length, 0, [
                get_012345, sound_4_length, u8
            ];
            0xFF22, 0b00000000, sound_4_polynomial, 0, [
                get_012,  sound_4_ratio,       u8;
                get_3,    sound_4_step,        NoisePattern;
                get_4567, sound_4_shift_clock, u8
            ];
            0xFF23, 0b10111111, sound_4_consecutive, 0xBF, [
                get_6, sound_4_consecutive, u8
            ]
        ],
        getter_setters: [
            0xFF10, 0b10000000, sound_1_sweep, 0, [
                get_012, set_012, sound_1_sweep_shift,     set_sound_1_sweep_shift,     u8;
                get_3,   set_3,   sound_1_sweep_direction, set_sound_1_sweep_direction, ToneSweepDirection;
                get_456, set_456, sound_1_sweep_period,    set_sound_1_sweep_period,    u8
            ];
            0xFF12, 0b00000000, sound_1_volume, 0, [
                get_4567, set_4567, sound_1_volume, set_sound_1_volume, u8;
                get_3,    set_3,    sound_1_direction, set_sound_1_direction, SweepDirection;
                get_012,  set_012,  sound_1_envelope_sweep, set_sound_1_envelope_sweep, u8
            ];
            0xFF14, 0b10111111, sound_1_frequency_high, 0, [
                get_7,   set_7,   sound_1_restart,        set_sound_1_restart,        u8;
                get_6,   set_6,   sound_1_consecutive,    set_sound_1_consecutive,    u8;
                get_012, set_012, sound_1_frequency_high, set_sound_1_frequency_high, u8
            ];
            0xFF17, 0b00000000, sound_2_volume, 0, [
                get_4567, set_4567, sound_2_volume, set_sound_2_volume, u8;
                get_3,    set_3,    sound_2_direction, set_sound_2_direction, SweepDirection;
                get_012,  set_012,  sound_2_envelope_sweep, set_sound_2_envelope_sweep, u8
            ];
            0xFF19, 0b10111111, sound_2_frequency_high, 0, [
                get_7,   set_7,   sound_2_restart,        set_sound_2_restart,        u8;
                get_6,   set_6,   sound_2_consecutive,    set_sound_2_consecutive,    u8;
                get_012, set_012, sound_2_frequency_high, set_sound_2_frequency_high, u8
            ];
            0xFF21, 0b00000000, sound_4_volume, 0, [
                get_4567, set_4567, sound_4_volume, set_sound_4_volume, u8;
                get_3,    set_3,    sound_4_direction, set_sound_4_direction, SweepDirection;
                get_012,  set_012,  sound_4_envelope_sweep, set_sound_4_envelope_sweep, u8
            ];
            0xFF24, 0b00000000, sound_control, 0, [
                get_012, set_012, so1_volume,     set_so1_volume,     u8;
                get_3,   set_3,   so1_vin_status, set_so1_vin_status, SoundStatus;
                get_456, set_456, so2_volume,     set_so2_volume,     u8;
                get_7,   set_7,   so2_vin_status, set_so2_vin_status, SoundStatus
            ];
            0xFF25, 0b00000000, selection_sound, 0xF3, [
                get_7, set_7, sound_4_to_so2, set_sound_4_to_so2, SoundStatus;
                get_6, set_6, sound_3_to_so2, set_sound_3_to_so2, SoundStatus;
                get_5, set_5, sound_2_to_so2, set_sound_2_to_so2, SoundStatus;
                get_4, set_4, sound_1_to_so2, set_sound_1_to_so2, SoundStatus;
                get_3, set_3, sound_4_to_so1, set_sound_4_to_so1, SoundStatus;
                get_2, set_2, sound_3_to_so1, set_sound_3_to_so1, SoundStatus;
                get_1, set_1, sound_2_to_so1, set_sound_2_to_so1, SoundStatus;
                get_0, set_0, sound_1_to_so1, set_sound_1_to_so1, SoundStatus
            ];
            0xFF26, 0b01110000, sound_status_rw, 0, [
                get_7, set_7, master_status, set_master_status, SoundStatus
            ]
        ],
    },
}
