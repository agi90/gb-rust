/** This file is mostly based on http://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware */
use bitfield::Bitfield;
use hardware::cpu;
use hardware::cpu::Handler;
use std::convert::From;

u8_enum! {
    SoundStatus {
        SoundOff = 0b0,
        SoundOn = 0b1,
    }
}

u8_enum! {
    WavePattern {
        C12 = 0b00,
        C25 = 0b01,
        C50 = 0b10,
        C75 = 0b11,
    }
}

u8_enum! {
    OutputLevel {
        Mute          = 0b00,
        WavePattern   = 0b01,
        RightShifted2 = 0b10,
        RightShifted4 = 0b11,
    }
}

macro_rules! channel_1 {
    { $self: ident, $name: path } => {
        $name(
            &mut Line1Mapper{ mapper: &mut $self.mapper },
            &mut $self.buffer.sound_1);
    }
}

macro_rules! channel_2 {
    { $self: ident, $name: path } => {
        $name(
            &mut Line2Mapper{ mapper: &mut $self.mapper },
            &mut $self.buffer.sound_2);
    }
}

macro_rules! channel_3 {
    { $self: ident, $name: path } => {
        $name(
            &mut Line3Mapper{ mapper: &mut $self.mapper },
            &mut $self.buffer.sound_3);
    }
}

macro_rules! channel_4 {
    { $self: ident, $name: path } => {
        $name(
            &mut Line4Mapper{ mapper: &mut $self.mapper },
            &mut $self.buffer.sound_4);
    }
}

macro_rules! all_channels {
    { $self: ident, $name: path } => {
        channel_1!($self, $name);
        channel_2!($self, $name);
        channel_3!($self, $name);
        channel_4!($self, $name);
    }
}

impl OutputLevel {
    pub fn to_volume(&self) -> f32 {
        match self {
            &OutputLevel::Mute => 0.0,
            &OutputLevel::WavePattern => 1.0,
            &OutputLevel::RightShifted2 => 0.5,
            &OutputLevel::RightShifted4 => 0.25,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            &OutputLevel::Mute => 0,
            &OutputLevel::WavePattern => 1,
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

u8_enum! {
    SweepDirection {
        Down = 0b0,
        Up = 0b1,
    }
}

u8_enum! {
    ToneSweepDirection {
        Up = 0b0,
        Down = 0b1,
    }
}

impl ToneSweepDirection {
    fn to_sign(self) -> i64 {
        match self {
            ToneSweepDirection::Down => -1,
            ToneSweepDirection::Up => 1,
        }
    }
}

u8_enum! {
    NoisePattern {
        C15 = 0b0,
        C7  = 0b1,
    }
}

struct Sweep {
    counter: i64,
    enabled: bool,
    shift: i64,
    /** This flags marks whenever a down-sweep has been
     * performed since the Sweep has been enabled from
     * a trigger event.
     *
     * This is useful because in that case, whenever the Sweep
     * direction is changed the sound channel is disabled
     * immediately. */
    down_computed_since_reset: bool,
}

struct SweepWaveDuty {
    volume: u8,
    wave_duty: f32,
    shadow_frequency: u64,
    sweep: Sweep,
}

trait VolumeSweep {
    fn volume(&self) -> u8;
    fn set_volume(&mut self, v: u8);
}

impl VolumeSweep for AudioLine<SweepWaveDuty> {
    fn volume(&self) -> u8 {
        self.sound.volume
    }
    fn set_volume(&mut self, v: u8) {
        self.sound.volume = v;
    }
}

impl TriggerEvent for AudioLine<SweepWaveDuty> {
    fn trigger_event(&mut self, mapper: &mut dyn LineMapper) {
        self.sound.shadow_frequency = mapper.frequency();

        {
            let sweep = &mut self.sound.sweep;

            let period = mapper.sweep_period() as i64;
            sweep.counter = if period > 0 { period } else { 8 };
            sweep.enabled = period > 0 || sweep.shift > 0;
            sweep.down_computed_since_reset = false;
        }

        if self.sound.sweep.shift > 0 {
            self.update_frequency(false, mapper);
        }
    }
    fn default_length(&self) -> i64 {
        64
    }
    fn turn_off(&mut self) {
        self.turn_off_base();
        self.sound.sweep.down_computed_since_reset = false;
    }
}

impl AudioLine<SweepWaveDuty> {
    fn update_frequency(&mut self, update: bool, mapper: &mut dyn LineMapper) {
        if mapper.sweep_direction() == ToneSweepDirection::Down {
            self.sound.sweep.down_computed_since_reset = true;
        }

        let sign = mapper.sweep_direction().to_sign();

        let operand = (self.sound.shadow_frequency >> self.sound.sweep.shift) as i64;
        let new_frequency = (self.sound.shadow_frequency as i64 + sign * operand) as u64;

        if new_frequency >= 2048 {
            self.turn_off();
            return;
        }

        if self.sound.sweep.shift > 0 && update {
            self.frequency = new_frequency;
            self.sound.shadow_frequency = new_frequency;
            mapper.set_frequency(new_frequency);
        }
    }
}

struct WaveDuty {
    volume: u8,
    wave_duty: f32,
}

impl TriggerEvent for AudioLine<WaveDuty> {
    fn trigger_event(&mut self, _: &mut dyn LineMapper) {
        // TODO:
    }
    fn default_length(&self) -> i64 {
        64
    }
    fn turn_off(&mut self) {
        self.turn_off_base();
    }
}

impl VolumeSweep for AudioLine<WaveDuty> {
    fn volume(&self) -> u8 {
        self.sound.volume
    }
    fn set_volume(&mut self, v: u8) {
        self.sound.volume = v;
    }
}

struct Noise {
    pattern: NoisePattern,
    volume: u8,
}

impl TriggerEvent for AudioLine<Noise> {
    fn trigger_event(&mut self, _: &mut dyn LineMapper) {
        // TODO:
    }
    fn default_length(&self) -> i64 {
        64
    }
    fn turn_off(&mut self) {
        self.turn_off_base();
    }
}

impl VolumeSweep for AudioLine<Noise> {
    fn volume(&self) -> u8 {
        self.sound.volume
    }
    fn set_volume(&mut self, v: u8) {
        self.sound.volume = v;
    }
}

struct Wave {
    wave_pattern: [u8; 16],
    volume: OutputLevel,
}

impl TriggerEvent for AudioLine<Wave> {
    fn trigger_event(&mut self, _: &mut dyn LineMapper) {
        // TODO:
    }
    fn default_length(&self) -> i64 {
        256
    }
    fn turn_off(&mut self) {
        self.turn_off_base();
    }
}

impl VolumeSweep for AudioLine<Wave> {
    fn volume(&self) -> u8 {
        self.sound.volume.to_u8()
    }
    fn set_volume(&mut self, v: u8) {
        self.sound.volume = OutputLevel::from_u8(v);
    }
}

struct AudioLine<T> {
    // id is used often enough during debugging
    // that I'd rather leave this here.
    #[allow(dead_code)]
    id: usize,

    frequency: u64,
    playing_left: bool,
    playing_right: bool,

    // Volume sweep stuff
    on: bool,
    counter: i64,
    envelope_counter: i64,

    sound: T,
}

impl<T> AudioLine<T>
where
    AudioLine<T>: TriggerEvent,
{
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
    fn turn_off_base(&mut self) {
        self.on = false;
        self.playing_left = false;
        self.playing_right = false;
    }
}

trait TriggerEvent {
    fn trigger_event(&mut self, line_mapper: &mut dyn LineMapper);
    fn default_length(&self) -> i64;
    fn turn_off(&mut self);
}

struct GbAudioBuffer {
    sound_1: AudioLine<SweepWaveDuty>,
    sound_2: AudioLine<WaveDuty>,
    sound_3: AudioLine<Wave>,
    sound_4: AudioLine<Noise>,
}

pub trait AudioBuffer {
    fn sound_1(&self) -> &dyn Channel1View;
    fn sound_2(&self) -> &dyn Channel2View;
    fn sound_3(&self) -> &dyn Channel3View;
    fn sound_4(&self) -> &dyn Channel4View;
}

/** This view is used to expose the sound state
 * to the outside world. */
pub trait AudioLineView {
    fn playing_left(&self) -> bool;
    fn playing_right(&self) -> bool;
    fn frequency(&self) -> u64;
}

impl<T> AudioLineView for AudioLine<T> {
    fn playing_left(&self) -> bool {
        self.playing_left
    }
    fn playing_right(&self) -> bool {
        self.playing_right
    }
    fn frequency(&self) -> u64 {
        self.frequency
    }
}

pub trait VolumeView {
    fn volume(&self) -> u8;
}

pub trait OutputLevelView {
    fn volume(&self) -> OutputLevel;
}

pub trait WaveDutyView {
    fn wave_duty(&self) -> f32;
}

pub trait WavePatternView {
    fn wave_pattern(&self) -> &[u8];
}

pub trait PatternView {
    fn pattern(&self) -> NoisePattern;
}

pub trait Channel1View: WaveDutyView + VolumeView + AudioLineView {}
impl<T> Channel1View for T where T: WaveDutyView + VolumeView + AudioLineView {}

pub trait Channel2View: WaveDutyView + VolumeView + AudioLineView {}
impl<T> Channel2View for T where T: WaveDutyView + VolumeView + AudioLineView {}

pub trait Channel3View: WavePatternView + OutputLevelView + AudioLineView {}
impl<T> Channel3View for T where T: WavePatternView + OutputLevelView + AudioLineView {}

pub trait Channel4View: PatternView + VolumeView + AudioLineView {}
impl<T> Channel4View for T where T: PatternView + VolumeView + AudioLineView {}

impl VolumeView for AudioLine<WaveDuty> {
    fn volume(&self) -> u8 {
        self.sound.volume
    }
}

impl VolumeView for AudioLine<Noise> {
    fn volume(&self) -> u8 {
        self.sound.volume
    }
}

impl VolumeView for AudioLine<SweepWaveDuty> {
    fn volume(&self) -> u8 {
        self.sound.volume
    }
}

impl OutputLevelView for AudioLine<Wave> {
    fn volume(&self) -> OutputLevel {
        self.sound.volume
    }
}

impl WaveDutyView for AudioLine<WaveDuty> {
    fn wave_duty(&self) -> f32 {
        self.sound.wave_duty
    }
}

impl WaveDutyView for AudioLine<SweepWaveDuty> {
    fn wave_duty(&self) -> f32 {
        self.sound.wave_duty
    }
}

impl WavePatternView for AudioLine<Wave> {
    fn wave_pattern(&self) -> &[u8] {
        &self.sound.wave_pattern
    }
}

impl PatternView for AudioLine<Noise> {
    fn pattern(&self) -> NoisePattern {
        self.sound.pattern
    }
}

impl AudioBuffer for GbAudioBuffer {
    fn sound_1(&self) -> &dyn Channel1View {
        &self.sound_1
    }
    fn sound_2(&self) -> &dyn Channel2View {
        &self.sound_2
    }
    fn sound_3(&self) -> &dyn Channel3View {
        &self.sound_3
    }
    fn sound_4(&self) -> &dyn Channel4View {
        &self.sound_4
    }
}

pub struct SoundController {
    master_status: bool,
    mapper: SoundMemoryMapper,
    buffer: GbAudioBuffer,
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
    fn set_frequency(&mut self, frequency: u64);
    fn sweep_period(&self) -> u8;
    fn sweep_direction(&self) -> ToneSweepDirection;
    fn consecutive(&self) -> bool;
    fn dac_off(&self) -> bool;
    fn write(&mut self, address: u16, v: u8);
    fn playing_left(&self) -> bool;
    fn playing_right(&self) -> bool;
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
        self.mapper.sound_1_frequency_low as u64
            + ((self.mapper.sound_1_frequency_high() as u64) << 8)
    }
    fn set_frequency(&mut self, frequency: u64) {
        self.mapper.sound_1_frequency_low = (frequency & 0xFF) as u8;
        self.mapper
            .set_sound_1_frequency_high((frequency & 0x700 >> 8) as u8);
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
        self.mapper.sound_1_volume() == 0 && self.mapper.sound_1_direction() == SweepDirection::Down
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
    fn playing_left(&self) -> bool {
        self.mapper.sound_1_to_so1() == SoundStatus::SoundOn
    }
    fn playing_right(&self) -> bool {
        self.mapper.sound_1_to_so2() == SoundStatus::SoundOn
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
        self.mapper.sound_3_frequency_low as u64
            + ((self.mapper.sound_3_frequency_high() as u64) << 8)
    }
    fn set_frequency(&mut self, _: u64) {
        panic!();
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
    fn playing_left(&self) -> bool {
        self.mapper.sound_3_to_so1() == SoundStatus::SoundOn
    }
    fn playing_right(&self) -> bool {
        self.mapper.sound_3_to_so2() == SoundStatus::SoundOn
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
        self.mapper.sound_2_frequency_low as u64
            + ((self.mapper.sound_2_frequency_high() as u64) << 8)
    }
    fn set_frequency(&mut self, _: u64) {
        panic!();
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
        self.mapper.sound_2_volume() == 0 && self.mapper.sound_2_direction() == SweepDirection::Down
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
    fn playing_left(&self) -> bool {
        self.mapper.sound_2_to_so1() == SoundStatus::SoundOn
    }
    fn playing_right(&self) -> bool {
        self.mapper.sound_2_to_so2() == SoundStatus::SoundOn
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
    fn set_frequency(&mut self, _: u64) {
        panic!();
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
        self.mapper.sound_4_volume() == 0 && self.mapper.sound_4_direction() == SweepDirection::Down
    }
    fn write(&mut self, address: u16, v: u8) {
        self.mapper.write(address, v);
    }
    fn playing_left(&self) -> bool {
        self.mapper.sound_4_to_so1() == SoundStatus::SoundOn
    }
    fn playing_right(&self) -> bool {
        self.mapper.sound_4_to_so2() == SoundStatus::SoundOn
    }
}

fn update_playing<T>(line: &mut dyn LineMapper, sound: &mut AudioLine<T>) {
    sound.playing_left = sound.on && line.playing_left();
    sound.playing_right = sound.on && line.playing_right();
}

fn update_frequency<T>(mapper: &mut dyn LineMapper, sound: &mut AudioLine<T>) {
    sound.frequency = mapper.frequency();
}

fn update_dac<T>(line: &mut dyn LineMapper, sound: &mut AudioLine<T>)
where
    AudioLine<T>: TriggerEvent,
{
    if line.dac_off() {
        sound.turn_off();
    }
}

fn update_length<T>(line: &mut dyn LineMapper, sound: &mut AudioLine<T>)
where
    AudioLine<T>: TriggerEvent,
{
    if !line.consecutive() {
        return;
    }

    if sound.counter > 0 {
        sound.counter -= 1;
    }

    if sound.counter == 0 {
        sound.turn_off();
    }
}

fn update_volume<T>(line: &mut dyn LineMapper, sound: &mut AudioLine<T>)
where
    AudioLine<T>: VolumeSweep,
{
    let sweep = line.envelope_sweep();
    if sweep == 0 {
        return;
    }

    let volume = sound.volume();
    let direction = line.direction();

    let is_sweep_completed = match direction {
        SweepDirection::Up => volume == 0xF,
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
            SweepDirection::Up => volume + 1,
            SweepDirection::Down => volume - 1,
        };

        sound.set_volume(new_volume);
        sound.envelope_counter = sweep as i64;
    }
}

fn trigger_event<T>(
    sound: &mut AudioLine<T>,
    line_mapper: &mut dyn LineMapper,
    frame_sequencer: &FrameSequencer,
) where
    AudioLine<T>: TriggerEvent + VolumeSweep,
{
    sound.on = true;
    update_playing(line_mapper, sound);

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
        sound.turn_off();
    }
}

fn extra_length_check<T>(sound: &mut AudioLine<T>, frame_sequencer: &FrameSequencer)
where
    AudioLine<T>: TriggerEvent,
{
    let extra_check = sound.counter > 0 && !frame_sequencer.next_step_clocks_length();

    if extra_check {
        sound.counter -= 1;
        if sound.counter == 0 {
            sound.turn_off();
        }
    }
}

/** Behavior for writing to the NRx4 register. */
fn write_nrx4<T>(
    sound: &mut AudioLine<T>,
    mapper: &mut dyn LineMapper,
    frame_sequencer: &FrameSequencer,
    address: u16,
    v: u8,
) where
    AudioLine<T>: TriggerEvent + VolumeSweep,
{
    let turns_length_on = !mapper.consecutive() && v & 0b01000000 > 0;

    mapper.write(address, v);

    if turns_length_on {
        // If we are turning the length counter on, we might
        // have to perform an extra length check.
        extra_length_check(sound, frame_sequencer);
    }

    if v & 0b10000000 > 0 {
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
        let mut s = FrameSequencer { cycles: 0, step: 0 };

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

    #[inline]
    pub fn cpu_step(&mut self) -> Option<SequencerEvent> {
        self.cycles -= cpu::CYCLES_PER_STEP as i64;

        if self.cycles != 0 {
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
            master_status: false,
            frame_sequencer: FrameSequencer::new(),
            mapper: SoundMemoryMapper::new(),
            buffer: GbAudioBuffer {
                sound_1: AudioLine::new(
                    1,
                    SweepWaveDuty {
                        volume: 0,
                        wave_duty: 0.5,
                        shadow_frequency: 0,
                        sweep: Sweep {
                            counter: 0,
                            enabled: false,
                            shift: 0,
                            down_computed_since_reset: false,
                        },
                    },
                ),
                sound_2: AudioLine::new(
                    2,
                    WaveDuty {
                        volume: 0,
                        wave_duty: 0.5,
                    },
                ),
                sound_3: AudioLine::new(
                    3,
                    Wave {
                        volume: OutputLevel::Mute,
                        wave_pattern: [0; 16],
                    },
                ),
                sound_4: AudioLine::new(
                    4,
                    Noise {
                        volume: 0,
                        pattern: NoisePattern::C7,
                    },
                ),
            },
        }
    }

    pub fn get_audio(&self) -> &dyn AudioBuffer {
        &self.buffer
    }

    pub fn cpu_step(&mut self) {
        if !self.master_status {
            return;
        }

        if let Some(ev) = self.frame_sequencer.cpu_step() {
            match ev {
                SequencerEvent::Length => self.update_length(),
                SequencerEvent::LengthSweep => {
                    self.update_length();
                    self.update_sweep();
                }
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

            sound.sweep.counter = if period > 0 { period } else { 8 };

            if !sound.sweep.enabled {
                return;
            }
        }

        if period > 0 {
            let mut line_mapper = Line1Mapper {
                mapper: &mut self.mapper,
            };
            self.buffer.sound_1.update_frequency(true, &mut line_mapper);
            self.buffer
                .sound_1
                .update_frequency(false, &mut line_mapper);
        }
    }

    fn update_length(&mut self) {
        all_channels!(self, update_length);
    }

    fn update_volume(&mut self) {
        channel_1!(self, update_volume);
        channel_2!(self, update_volume);
        channel_4!(self, update_volume);
    }

    fn write_callback(&mut self, address: u16) {
        match address {
            0xFF10 => {
                let sweep = &mut self.buffer.sound_1.sound.sweep;
                sweep.shift = self.mapper.sound_1_sweep_shift() as i64;
            }
            0xFF11 => {
                self.buffer.sound_1.sound.wave_duty = self.mapper.sound_1_pattern().to_wave_duty();
                self.buffer.sound_1.counter = 64 - self.mapper.sound_1_length() as i64;
            }
            0xFF12 => {
                channel_1!(self, update_dac);
            }
            0xFF13 | 0xFF14 => {
                channel_1!(self, update_frequency);
            }
            0xFF16 => {
                self.buffer.sound_2.sound.wave_duty = self.mapper.sound_2_pattern().to_wave_duty();
                self.buffer.sound_2.counter = 64 - self.mapper.sound_2_length() as i64;
            }
            0xFF17 => {
                channel_2!(self, update_dac);
            }
            0xFF18 | 0xFF19 => {
                channel_2!(self, update_frequency);
            }
            0xFF1A => {
                channel_3!(self, update_dac);
            }
            0xFF1B => {
                self.buffer.sound_3.counter = 256 - self.mapper.sound_3_length as i64;
            }
            0xFF1C => {
                self.buffer.sound_3.sound.volume = self.mapper.sound_3_output_level();
            }
            0xFF1D | 0xFF1E => {
                channel_3!(self, update_frequency);
            }
            0xFF20 => {
                self.buffer.sound_4.counter = 64 - self.mapper.sound_4_length() as i64;
            }
            0xFF21 => {
                channel_4!(self, update_dac);
            }
            0xFF22 => {
                channel_4!(self, update_frequency);
                self.buffer.sound_4.sound.pattern = self.mapper.sound_4_step();
            }
            0xFF24 => {
                // TODO: handle all channels
            }
            0xFF25 => {
                all_channels!(self, update_playing);
            }
            0xFF26 => {
                self.master_status = self.mapper.master_status() == SoundStatus::SoundOn;
            }
            _ => {}
        }
    }

    fn set_master_status(&mut self, master_status: SoundStatus) {
        self.master_status = master_status == SoundStatus::SoundOn;
        self.mapper.set_master_status(master_status);
    }

    fn write_nr52(&mut self, v: u8) {
        let new_master_status = if v & 0b10000000 > 0 {
            SoundStatus::SoundOn
        } else {
            SoundStatus::SoundOff
        };

        if (new_master_status == SoundStatus::SoundOn) == self.master_status {
            // Status is not changing so we don't need to do anything here.
            return;
        }

        if new_master_status == SoundStatus::SoundOff {
            self.buffer.sound_1.turn_off();
            self.buffer.sound_2.turn_off();
            self.buffer.sound_3.turn_off();
            self.buffer.sound_4.turn_off();

            for address in 0xFF10..0xFF30 {
                match address {
                    // NR52 and length are not affected by power in the DMG
                    0xFF26 | 0xFF1B | 0xFF20 => {}
                    0xFF11 => *self.mapper.sound_1_wave_pattern &= 0b00111111,
                    0xFF16 => *self.mapper.sound_2_wave_pattern &= 0b00111111,
                    _ => self.write(address, 0x00),
                }
            }
        } else {
            // We're turning the APU on so we need to reset the Sequencer
            self.frame_sequencer.reset();
        }

        self.set_master_status(new_master_status);
    }
}

impl Handler for SoundController {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF26 => {
                let v = (if self.master_status { 0b10000000 } else { 0 })
                    + (if self.buffer.sound_1.on {
                        0b00000001
                    } else {
                        0
                    })
                    + (if self.buffer.sound_2.on {
                        0b00000010
                    } else {
                        0
                    })
                    + (if self.buffer.sound_3.on {
                        0b00000100
                    } else {
                        0
                    })
                    + (if self.buffer.sound_4.on {
                        0b00001000
                    } else {
                        0
                    });

                v | 0b01110000
            }
            0xFF30..=0xFF3F => self.buffer.sound_3.sound.wave_pattern[address as usize - 0xFF30],
            _ => self.mapper.read(address),
        }
    }

    fn write(&mut self, address: u16, mut v: u8) {
        if !self.master_status && address != 0xFF26 {
            let mask = match address {
                // Only writes to NR52 and sound registers are allowed in the DMG
                // while the APU is powered down.
                0xFF26 => 0b10000000,
                0xFF11 => 0b00111111,
                0xFF16 => 0b00111111,
                0xFF1B => 0b11111111,
                0xFF20 => 0b11111111,
                _ => return,
            };

            v &= mask;
        }

        match address {
            0xFF10 => {
                if v & 0b00001000 == 0
                    && self.buffer.sound_1.sound.sweep.down_computed_since_reset
                    && self.mapper.sound_1_sweep_direction() == ToneSweepDirection::Down
                {
                    // The sweep direction is changing, and we have computed at least one
                    // sweep since it was enabled, in this case the sound channel is disabled
                    // immediately.
                    self.buffer.sound_1.turn_off();
                }
                self.mapper.write(address, v);
            }
            0xFF14 => {
                write_nrx4(
                    &mut self.buffer.sound_1,
                    &mut Line1Mapper {
                        mapper: &mut self.mapper,
                    },
                    &self.frame_sequencer,
                    address,
                    v,
                );
            }
            0xFF19 => {
                write_nrx4(
                    &mut self.buffer.sound_2,
                    &mut Line2Mapper {
                        mapper: &mut self.mapper,
                    },
                    &self.frame_sequencer,
                    address,
                    v,
                );
            }
            0xFF1E => {
                write_nrx4(
                    &mut self.buffer.sound_3,
                    &mut Line3Mapper {
                        mapper: &mut self.mapper,
                    },
                    &self.frame_sequencer,
                    address,
                    v,
                );
            }
            0xFF23 => {
                write_nrx4(
                    &mut self.buffer.sound_4,
                    &mut Line4Mapper {
                        mapper: &mut self.mapper,
                    },
                    &self.frame_sequencer,
                    address,
                    v,
                );
            }
            0xFF26 => self.write_nr52(v),
            0xFF30..=0xFF3F => {
                self.buffer.sound_3.sound.wave_pattern[address as usize - 0xFF30] = v
            }
            _ => {
                self.mapper.write(address, v);
            }
        }
        self.write_callback(address);
    }
}

memory_mapper! {
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
