#[macro_use]
extern crate glium;
extern crate sdl2;
extern crate rand;

#[macro_use]
extern crate clap;

pub mod gb_proc;
pub mod gpu;
pub mod controller;
mod sound;

#[allow(dead_code)]
mod bitfield;
mod debugger;

use self::controller::{Event, Controller};
use self::debugger::Debugger;
use self::gb_proc::cartridge::Cartridge;
use self::gb_proc::cpu::Cpu;
use self::gb_proc::handler_holder::GBHandlerHolder;
use self::gb_proc::video_controller::{SCREEN_X, SCREEN_Y};
use self::gpu::renderer::Renderer;
use std::fs::{File, OpenOptions};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

fn open_save_file(rom_name: &str) -> Result<File, String> {
    let mut v: Vec<&str> = rom_name.split('.').collect();
    // If the file does not end with .gb we might be reading garbage,
    // let's bail.
    if v.pop() != Some("gb") {
        return Err(format!("Invalid ROM name: '{}' filename must end with '.gb'.", rom_name));
    }

    // Replace extension with "sav"
    v.push("sav");

    let save_name = v.join(".");

    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(save_name)
        .unwrap())
}

fn open_rom(rom_name: &str) -> Result<File, String> {
    let f = File::open(rom_name);
    f.map_err(|_| format!("Error: ROM '{}' not found.", rom_name))
}

// Similar to try! but for the main function
macro_rules! bail {
    ($expr : expr) => {
        {
            if $expr.is_err() {
                println!("{}", $expr.unwrap_err());
                return;
            }

            $expr.unwrap()
        }
    }
}

pub fn main() {
    let matches = clap_app!(gbrust =>
        (version: "0.1b")
        (author: "Agi Sferro <agi.novanta@gmail.com>")
        (about: "Yet another DMG emulator written in Rust.")
        (@arg ROM: +required "Selects the ROM to run.")
        (@arg mag: -m --magnification +takes_value "Number of times the screen should be magnified. Default is '3'.") 
        (@arg sound: -s --sound "Enables sound emulation (experimental).")
        (@arg debug: -d --debug "Starts in debug mode.")
    ).get_matches();

    let rom_name = matches.value_of("ROM").unwrap();
    let mut rom = bail!(open_rom(&rom_name));
    let save_file = bail!(open_save_file(&rom_name));

    let cartridge = Cartridge::from_file(&mut rom, save_file);

    let mag = value_t!(matches.value_of("mag"), u32).unwrap_or_else(|e| {
        println!("Invalid magnification: {}", e);
        3
    });

    let mut controller = Controller::new(SCREEN_X as u32 * mag,
                                         SCREEN_Y as u32 * mag);

    let handler = GBHandlerHolder::new(Box::new(cartridge));
    let mut cpu = Cpu::new(Box::new(handler));

    let mut last_update = Instant::now();

    let mut debugger = Debugger::new();

    if matches.occurrences_of("debug") > 0 {
        debugger.breakpoint(&mut cpu);
    }

    let sound_enabled = matches.occurrences_of("sound") > 0;

    let mut natural_speed = true;

    // This target seems to match the actual Game Boy in terms of speed
    let target = Duration::new(0, (1e9/63.0) as u32);
    loop {
        debugger.check_breakpoints(&mut cpu);

        cpu.next_instruction();

        if cpu.handler_holder.should_refresh() {
            match controller.check_events(&mut cpu) {
                Event::Quit => break,
                Event::Break => {
                    debugger.breakpoint(&mut cpu);
                },
                Event::ToggleSpeed => { natural_speed = !natural_speed },
                Event::Continue => {},
            }

            let screen = cpu.handler_holder.get_screen_buffer();
            controller.refresh(screen);

            if sound_enabled {
                let audio = cpu.handler_holder.get_audio_buffer();
                controller.refresh_sound(audio);
            }

            let diff = Instant::now() - last_update;

            if target > diff && natural_speed {
                thread::sleep(target - diff);
            }

            last_update = Instant::now();
        }

        debugger.next_instruction(&mut cpu);
    }
}
