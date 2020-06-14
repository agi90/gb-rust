#[allow(dead_code)]
mod bitfield;
mod emulator;
mod hardware;

pub use self::emulator::{Emulator, AUDIO_BUFFER_SIZE};
pub use self::hardware::apu::{
    AudioBuffer, AudioLineView, Channel1View, Channel2View, Channel3View, Channel4View,
    NoisePattern,
};
pub use self::hardware::cpu::{Cpu, Hardware, Interrupt, OpCode};
pub use self::hardware::handler_holder::Key;
pub use self::hardware::ppu::{GrayShade, ScreenBuffer, SCREEN_X, SCREEN_Y};

#[cfg(test)]
mod tests;
