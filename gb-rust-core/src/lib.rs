#[allow(dead_code)]
mod bitfield;
mod gb_proc;
mod emulator;

pub use self::emulator::Emulator;
pub use self::gb_proc::cpu::Interrupt;
pub use self::gb_proc::video_controller::GrayShade;
pub use self::gb_proc::cpu::Hardware;
pub use self::gb_proc::handler_holder::Key;
pub use self::gb_proc::video_controller::{SCREEN_X, SCREEN_Y};

#[cfg(test)]
mod tests;

