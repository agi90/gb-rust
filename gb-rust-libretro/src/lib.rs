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


struct Color {
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    #[inline]
    pub fn write(&self, out: &mut [u8]) {
        out[0] = self.b;
        out[1] = self.g;
        out[2] = self.r;
        out[3] = self.a;
    }
}

impl From<GrayShade> for Color {
    fn from(c: GrayShade) -> Self {
        match c {
            GrayShade::C00 => PALETTE_00,
            GrayShade::C01 => PALETTE_01,
            GrayShade::C10 => PALETTE_10,
            GrayShade::C11 => PALETTE_11,
            GrayShade::Transparent => unreachable!(),
        }
    }
}

const PALETTE_11: Color = Color { a: 0xFF, r: 0x0F, g: 0x38, b: 0x0F };
const PALETTE_10: Color = Color { a: 0xFF, r: 0x30, g: 0x62, b: 0x30 };
const PALETTE_01: Color = Color { a: 0xFF, r: 0x8C, g: 0xAD, b: 0x0F };
const PALETTE_00: Color = Color { a: 0xFF, r: 0x9C, g: 0xBD, b: 0x0F };

impl libretro_backend::Core for EmulatorWrapper {
    fn info() -> CoreInfo {
        CoreInfo::new("gb-rust", env!("CARGO_PKG_VERSION"))
            .supports_roms_with_extension("gb")
            .supports_roms_with_extension("gbc")
    }

    fn save_memory(&mut self) -> Option<&mut [u8]> {
        Some(self.cpu.handler_holder.ram())
    }

    fn rtc_memory(&mut self) -> Option<&mut [u8]> {
        self.cpu.handler_holder.rtc().map(
            |v| unsafe {
                std::slice::from_raw_parts_mut(
                    v as *mut _ as *mut u8,
                    std::mem::size_of::<i64>())
            })
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
                let color = Color::from(screen[i][j]);

                let index = i * gb::SCREEN_X * 4 + j * 4;
                color.write(&mut self.frame[index .. index + 4]);
            }
        }

        handle.upload_video_frame(&self.frame);
        handle.upload_audio_frame(&self.generate_sound()[..]);
    }
}

libretro_core!(EmulatorWrapper);
