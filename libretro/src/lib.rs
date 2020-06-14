extern crate gb;

#[macro_use]
extern crate libretro_backend;

#[cfg(feature = "profiler")]
extern crate cpuprofiler;

#[cfg(feature = "profiler")]
extern crate uuid;

#[cfg(feature = "profiler")]
use cpuprofiler::PROFILER;

#[cfg(feature = "profiler")]
use self::uuid::Uuid;

use std::ops::{Deref, DerefMut};

use gb::{Emulator, GrayShade, Hardware, Interrupt, Key};

use libretro_backend::{
    AudioVideoInfo, CoreInfo, GameData, JoypadButton, LoadGameResult, PixelFormat, Region,
    RuntimeHandle, Variables,
};

const FREQUENCY: f64 = 44100.0; // Hz

struct EmulatorWrapper {
    emulator: Option<Emulator>,
    game_data: Option<GameData>,
    frame: [u8; gb::SCREEN_X * gb::SCREEN_Y * 4],
    palette: Palette,
    init_variables: bool,
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
            palette: GB_POCKET_PALETTE,
            init_variables: false,
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), String> {
        self.emulator = Some(Emulator::from_data(data, FREQUENCY)?);
        Ok(())
    }

    pub fn update_button(
        &mut self,
        handle: &mut RuntimeHandle,
        button: JoypadButton,
        gb_button: Key,
    ) {
        if handle.is_joypad_button_pressed(0, button) {
            self.cpu.key_down(gb_button);
        } else {
            self.cpu.key_up(gb_button);
        }
    }

    pub fn update_variables(&mut self, handle: &mut RuntimeHandle) {
        if let Some(palette) = handle.get_variable("palette") {
            self.palette = match palette.as_str() {
                "dmg" => DMG_PALETTE,
                "gb_pocket" => GB_POCKET_PALETTE,
                _ => GB_POCKET_PALETTE,
            };
        }

        self.init_variables = true;
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

struct Palette {
    c01: Color,
    c10: Color,
    c11: Color,
    c00: Color,
}

impl Palette {
    #[inline]
    fn color(&self, c: GrayShade) -> &Color {
        match c {
            GrayShade::C00 => &self.c00,
            GrayShade::C01 => &self.c01,
            GrayShade::C10 => &self.c10,
            GrayShade::C11 => &self.c11,
            GrayShade::Transparent => unreachable!(),
        }
    }
}

const GB_POCKET_PALETTE: Palette = Palette {
    c11: Color {
        a: 0xFF,
        r: 0x6C,
        g: 0x6C,
        b: 0x4E,
    },
    c10: Color {
        a: 0xFF,
        r: 0x8E,
        g: 0x8B,
        b: 0x61,
    },
    c01: Color {
        a: 0xFF,
        r: 0xC3,
        g: 0xC4,
        b: 0xA5,
    },
    c00: Color {
        a: 0xFF,
        r: 0xE3,
        g: 0xE6,
        b: 0xC9,
    },
};

const DMG_PALETTE: Palette = Palette {
    c11: Color {
        a: 0xFF,
        r: 0x2A,
        g: 0x45,
        b: 0x3B,
    },
    c10: Color {
        a: 0xFF,
        r: 0x36,
        g: 0x5D,
        b: 0x48,
    },
    c01: Color {
        a: 0xFF,
        r: 0x57,
        g: 0x7C,
        b: 0x45,
    },
    c00: Color {
        a: 0xFF,
        r: 0x7F,
        g: 0x86,
        b: 0x0F,
    },
};

impl libretro_backend::Core for EmulatorWrapper {
    fn info() -> CoreInfo {
        CoreInfo::new("gb-rust", env!("CARGO_PKG_VERSION"))
            .supports_roms_with_extension("gb")
            .supports_roms_with_extension("gbc")
    }

    fn variables() -> Variables {
        Variables::new().variable("palette", &["dmg", "gb_pocket"], "Palette")
    }

    fn save_memory(&mut self) -> Option<&mut [u8]> {
        Some(self.cpu.handler_holder.ram())
    }

    fn rtc_memory(&mut self) -> Option<&mut [u8]> {
        self.cpu.handler_holder.rtc().map(|v| unsafe {
            std::slice::from_raw_parts_mut(v as *mut _ as *mut u8, std::mem::size_of::<i64>())
        })
    }

    fn on_reset(&mut self) {
        self.reset();
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        #[cfg(feature = "profiler")]
        PROFILER
            .lock()
            .unwrap()
            .start(format!("./gb-rust-{}.profile", Uuid::new_v4()))
            .unwrap();

        if game_data.data().is_none() {
            return LoadGameResult::Failed(game_data);
        }

        if self.load_rom(game_data.data().unwrap()).is_err() {
            return LoadGameResult::Failed(game_data);
        }

        let av = AudioVideoInfo::new()
            .video(
                gb::SCREEN_X as u32,
                gb::SCREEN_Y as u32,
                60.0,
                PixelFormat::ARGB8888,
            )
            .audio(FREQUENCY)
            .region(Region::NTSC);

        self.game_data = Some(game_data);
        return LoadGameResult::Success(av);
    }

    fn on_unload_game(&mut self) -> GameData {
        #[cfg(feature = "profiler")]
        PROFILER.lock().unwrap().stop().unwrap();

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

        if !self.init_variables || handle.did_variables_update() {
            self.update_variables(handle);
        }

        let mut screen = [[GrayShade::C00; gb::SCREEN_X]; gb::SCREEN_Y];
        {
            let buffer = self.cpu.handler_holder.get_screen_buffer();
            screen.copy_from_slice(&buffer[..]);
        }

        for i in 0..gb::SCREEN_Y {
            for j in 0..gb::SCREEN_X {
                let color = self.palette.color(screen[i][j]);

                let index = i * gb::SCREEN_X * 4 + j * 4;
                color.write(&mut self.frame[index..index + 4]);
            }
        }

        handle.upload_video_frame(&self.frame);
        handle.upload_audio_frame(&self.generate_sound()[..]);
    }
}

libretro_core!(EmulatorWrapper);
