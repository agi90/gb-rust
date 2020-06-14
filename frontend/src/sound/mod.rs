use gb::{AudioBuffer, Channel1View, Channel2View, Channel3View, Channel4View, NoisePattern};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::ops::DerefMut;

use sdl2;

trait DmgSound {
    fn set_phase_inc(&mut self, f32);
    fn set_phase(&mut self, f32);
    fn set_volume(&mut self, f32);
    fn set_left(&mut self, bool);
    fn set_right(&mut self, bool);
}

struct Sound<T> {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    left: bool,
    right: bool,
    sound: T,
}

impl<T> DmgSound for Sound<T> {
    fn set_phase_inc(&mut self, phase_inc: f32) {
        self.phase_inc = phase_inc;
    }
    fn set_phase(&mut self, phase: f32) {
        self.phase = phase;
    }
    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
    fn set_left(&mut self, left: bool) {
        self.left = left;
    }
    fn set_right(&mut self, right: bool) {
        self.right = right;
    }
}

struct SquareWave {
    wave_duty: f32,
}

impl AudioCallback for Sound<SquareWave> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut i = 0;
        while i < out.len() {
            let value = if self.phase < self.sound.wave_duty {
                self.volume
            } else {
                -self.volume
            };

            // The out array contains both left and right data
            // interleaved [L,R,L,R,L, ...]
            out[i] = if self.left { value } else { 0.0 };
            out[i + 1] = if self.right { value } else { 0.0 };

            self.phase = (self.phase + self.phase_inc) % 1.0;

            i += 2;
        }
    }
}

struct WhiteNoise {
    sound_7_bit: [u8; 127],
    sound_15_bit: [u8; 32767],
    pattern: NoisePattern,
}

fn length(pattern: NoisePattern) -> f32 {
    match pattern {
        NoisePattern::C7 => 127.0,
        NoisePattern::C15 => 32767.0,
    }
}

impl AudioCallback for Sound<WhiteNoise> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut i = 0;
        self.phase = self.phase % length(self.sound.pattern);
        while i < out.len() {
            let index = self.phase as usize;
            let bit = match self.sound.pattern {
                NoisePattern::C15 => self.sound.sound_15_bit[index],
                NoisePattern::C7 => self.sound.sound_7_bit[index],
            };

            let value = if bit == 1 { self.volume } else { -self.volume };

            // The out array contains both left and right data
            // interleaved [L,R,L,R,L, ...]
            out[i] = if self.left { value } else { 0.0 };
            out[i + 1] = if self.right { value } else { 0.0 };

            self.phase = (self.phase + self.phase_inc) % length(self.sound.pattern);

            i += 2;
        }
    }
}

struct WavePattern {
    pattern: [u8; 32],
}

impl AudioCallback for Sound<WavePattern> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut i = 0;
        while i < out.len() {
            let index = (self.phase * 32.0) as usize;
            let value = (self.sound.pattern[index] as f32 - 7.0) / 8.0 * self.volume;
            // The out array contains both left and right data
            // interleaved [L,R,L,R,L, ...]
            out[i] = if self.left { value } else { 0.0 };
            out[i + 1] = if self.right { value } else { 0.0 };

            self.phase = (self.phase + self.phase_inc) % 1.0;

            i += 2;
        }
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

fn sound_rng(v: u16, generator: u16) -> u16 {
    let bit = v & 1;
    let next = v >> 1;
    if bit != 0 {
        next ^ generator
    } else {
        next
    }
}

const FREQUENCY: f32 = 44100.0;

pub struct SDLPlayer {
    device_1: AudioDevice<Sound<SquareWave>>,
    device_2: AudioDevice<Sound<SquareWave>>,
    device_3: AudioDevice<Sound<WavePattern>>,
    device_4: AudioDevice<Sound<WhiteNoise>>,
}

fn refresh_line<T>(
    device: &mut AudioDevice<Sound<T>>,
    playing_left: bool,
    playing_right: bool,
    frequency: f32,
) where
    Sound<T>: AudioCallback,
{
    if !playing_left && !playing_right {
        device.pause();
    } else {
        {
            let mut old_buffer = device.lock();
            old_buffer.deref_mut().set_left(playing_left);
            old_buffer.deref_mut().set_right(playing_right);
            old_buffer.deref_mut().set_phase_inc(frequency / FREQUENCY);
        }

        device.resume();
    }
}

fn init_device<T>(audio_subsystem: &sdl2::AudioSubsystem, sound: T) -> AudioDevice<Sound<T>>
where
    Sound<T>: AudioCallback,
{
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2), // mono
        samples: Some(40), // default sample size
    };

    audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            Sound {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
                left: false,
                right: false,
                sound: sound,
            }
        })
        .unwrap()
}

fn refresh_device_1(channel: &dyn Channel1View, b: &mut Sound<SquareWave>) -> f32 {
    b.sound.wave_duty = channel.wave_duty();
    b.volume = channel.volume() as f32 / 16.0 / 4.0;
    131072.0 / (2048.0 - channel.frequency() as f32)
}

fn refresh_device_2(channel: &dyn Channel2View, b: &mut Sound<SquareWave>) -> f32 {
    b.sound.wave_duty = channel.wave_duty();
    b.volume = channel.volume() as f32 / 16.0 / 4.0;
    131072.0 / (2048.0 - channel.frequency() as f32)
}

fn refresh_device_3(channel: &dyn Channel3View, b: &mut Sound<WavePattern>) -> f32 {
    let mut pattern = [0; 32];
    for i in 0..16 {
        pattern[i * 2] = (channel.wave_pattern()[i] & 0b11110000) >> 4;
        pattern[i * 2 + 1] = channel.wave_pattern()[i] & 0b00001111;
    }
    b.sound.pattern = pattern;
    b.volume = channel.volume().to_volume() / 4.0;
    65536.0 / (2048.0 - channel.frequency() as f32)
}

fn refresh_device_4(channel: &dyn Channel4View, b: &mut Sound<WhiteNoise>) -> f32 {
    b.sound.pattern = channel.pattern();
    b.volume = channel.volume() as f32 / 16.0 / 4.0;
    channel.frequency() as f32
}

impl SDLPlayer {
    pub fn new() -> SDLPlayer {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        SDLPlayer {
            device_1: init_device(&audio_subsystem, SquareWave { wave_duty: 0.5 }),
            device_2: init_device(&audio_subsystem, SquareWave { wave_duty: 0.5 }),
            device_3: init_device(&audio_subsystem, WavePattern { pattern: [0; 32] }),
            device_4: init_device(
                &audio_subsystem,
                WhiteNoise {
                    sound_7_bit: generate_noise_7_bit(),
                    sound_15_bit: generate_noise_15_bit(),
                    pattern: NoisePattern::C7,
                },
            ),
        }
    }

    pub fn refresh(&mut self, audio_buffer: &dyn AudioBuffer) {
        {
            let freq = refresh_device_1(audio_buffer.sound_1(), self.device_1.lock().deref_mut());
            refresh_line(
                &mut self.device_1,
                audio_buffer.sound_1().playing_left(),
                audio_buffer.sound_1().playing_right(),
                freq,
            );
        }

        {
            let freq = refresh_device_2(audio_buffer.sound_2(), self.device_2.lock().deref_mut());
            refresh_line(
                &mut self.device_2,
                audio_buffer.sound_2().playing_left(),
                audio_buffer.sound_2().playing_right(),
                freq,
            );
        }

        {
            let freq = refresh_device_3(audio_buffer.sound_3(), self.device_3.lock().deref_mut());
            refresh_line(
                &mut self.device_3,
                audio_buffer.sound_3().playing_left(),
                audio_buffer.sound_3().playing_right(),
                freq,
            );
        }

        {
            let freq = refresh_device_4(audio_buffer.sound_4(), self.device_4.lock().deref_mut());
            refresh_line(
                &mut self.device_4,
                audio_buffer.sound_4().playing_left(),
                audio_buffer.sound_4().playing_right(),
                freq,
            );
        }
    }
}
