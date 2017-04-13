#[allow(dead_code)]
mod bitfield;
mod hardware;
mod emulator;

pub use self::emulator::Emulator;
pub use self::hardware::cpu::Interrupt;
pub use self::hardware::video_controller::GrayShade;
pub use self::hardware::cpu::Hardware;
pub use self::hardware::handler_holder::Key;
pub use self::hardware::video_controller::{SCREEN_X, SCREEN_Y};

#[cfg(test)]
mod tests;

