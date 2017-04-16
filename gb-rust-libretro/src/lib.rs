extern crate gb;

#[macro_use]
extern crate libretro_backend;

use std::ops::{Deref, DerefMut};

use gb::{
    Emulator,
    Interrupt,
    GrayShade,
    Hardware,
    Key,
};

use libretro_backend::{
    GameData,
    LoadGameResult,
    RuntimeHandle,
    AudioVideoInfo,
    PixelFormat,
    Region,
    JoypadButton,
    CoreInfo,
    MemoryType
};

const FREQUENCY: f64 = 44100.0; // Hz

struct EmulatorWrapper {
    emulator: Option<Emulator>,
    game_data: Option<GameData>,
    frame: [u8; gb::SCREEN_X * gb::SCREEN_Y * 4],
}

impl Default for EmulatorWrapper {
    fn default() -> Self {
        EmulatorWrapper::new()
    }
}

impl EmulatorWrapper {
    pub fn new() -> EmulatorWrapper {
        EmulatorWrapper {
            emulator: None,
            game_data: None,
            frame: [0xFF; gb::SCREEN_X * gb::SCREEN_Y * 4],
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), String> {
        self.emulator = Some(Emulator::from_data(data, FREQUENCY)?);
        Ok(())
    }

    pub fn update_button(&mut self, handle: &mut RuntimeHandle,
                         button: JoypadButton, gb_button: Key) {
        if handle.is_joypad_button_pressed(0, button) {
            self.cpu.key_down(gb_button);
        } else {
            self.cpu.key_up(gb_button);
        }
    }
}

impl Deref for EmulatorWrapper {
    type Target = Emulator;

    fn deref(&self) -> &Emulator {
        self.emulator.as_ref().unwrap()
    }
}

impl DerefMut for EmulatorWrapper {
    fn deref_mut(&mut self) -> &mut Emulator {
        self.emulator.as_mut().unwrap()
    }
}


impl libretro_backend::Core for EmulatorWrapper {
    fn info() -> CoreInfo {
        CoreInfo::new("gb-rust", env!("CARGO_PKG_VERSION"))
            .supports_roms_with_extension("gb")
            .supports_roms_with_extension("gbc")
    }

    fn memory_data(&mut self, memory_type: MemoryType) -> Option<&mut [u8]> {
        match memory_type {
            MemoryType::SaveRam => Some(self.cpu.handler_holder.ram()),
            MemoryType::Rtc => self.cpu.handler_holder.rtc().map(
                |v| unsafe {
                    std::slice::from_raw_parts_mut(
                        v as *mut _ as *mut u8,
                        std::mem::size_of::<i64>())
                }),
            _ => None,
        }
    }

    fn on_reset(&mut self) {
        self.reset();
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if game_data.data().is_none() {
            return LoadGameResult::Failed(game_data);
        }

        if self.load_rom(game_data.data().unwrap()).is_err() {
            return LoadGameResult::Failed(game_data);
        }

        let av = AudioVideoInfo::new()
            .video(gb::SCREEN_X as u32, gb::SCREEN_Y as u32, 60.0, PixelFormat::ARGB8888)
            .audio(FREQUENCY)
            .region(Region::NTSC);

        self.game_data = Some(game_data);
        return LoadGameResult::Success(av);
    }

    fn on_unload_game(&mut self) -> GameData {
        self.emulator = None;
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        self.update_button(handle, JoypadButton::A, Key::A);
        self.update_button(handle, JoypadButton::B, Key::B);
        self.update_button(handle, JoypadButton::Select, Key::Select);
        self.update_button(handle, JoypadButton::Start, Key::Start);
        self.update_button(handle, JoypadButton::Up, Key::Up);
        self.update_button(handle, JoypadButton::Down, Key::Down);
        self.update_button(handle, JoypadButton::Left, Key::Left);
        self.update_button(handle, JoypadButton::Right, Key::Right);
		self.cpu.interrupt(Interrupt::Joypad);

        loop {
            self.cpu.next_instruction();

            if self.cpu.handler_holder.should_refresh() {
                break;
            }
        }

        let mut screen = [[GrayShade::C00; gb::SCREEN_X]; gb::SCREEN_Y];
        {
            let buffer = self.cpu.handler_holder.get_screen_buffer();
            screen.copy_from_slice(&buffer[..]);
        }

        for i in 0 .. gb::SCREEN_Y {
            for j in 0 .. gb::SCREEN_X {
                self.frame[i * gb::SCREEN_X * 4 + j * 4 + 0] = (3 - screen[i][j] as u8) * 85;
                self.frame[i * gb::SCREEN_X * 4 + j * 4 + 1] = (3 - screen[i][j] as u8) * 85;
                self.frame[i * gb::SCREEN_X * 4 + j * 4 + 2] = (3 - screen[i][j] as u8) * 85;
                self.frame[i * gb::SCREEN_X * 4 + j * 4 + 3] = (3 - screen[i][j] as u8) * 85;
            }
        }

        handle.upload_video_frame(&self.frame);
        handle.upload_audio_frame(&self.generate_sound()[..]);
    }
}

libretro_core!(EmulatorWrapper);
