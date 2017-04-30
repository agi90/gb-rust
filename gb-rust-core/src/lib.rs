#![feature(struct_field_attributes)]

#[allow(dead_code)]
mod bitfield;
mod hardware;
mod emulator;

pub use self::emulator::{
    Emulator,
    AUDIO_BUFFER_SIZE,
};
pub use self::hardware::apu::{
    NoisePattern,
    AudioBuffer,
    AudioLineView,
    Channel1View,
    Channel2View,
    Channel3View,
    Channel4View,
};
pub use self::hardware::cpu::{
    Interrupt,
    Hardware,
    OpCode,
};
pub use self::hardware::handler_holder::Key;
pub use self::hardware::ppu::{
    SCREEN_X,
    SCREEN_Y,
    ScreenBuffer,
    GrayShade,
};

#[cfg(test)]
mod tests;

