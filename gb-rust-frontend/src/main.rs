#[macro_use]
extern crate glium;
extern crate sdl2;

#[macro_use]
extern crate clap;

pub mod gpu;
pub mod controller;
mod sound;

#[allow(dead_code)]
mod debugger;

extern crate gb;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

use gb::Emulator;

use self::controller::{Event, Controller};
use self::debugger::Debugger;

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
    v.push("srm");

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
        (@arg debug: -d --debug "Starts in debug mode.")
    ).get_matches();

    let rom_name = matches.value_of("ROM").unwrap();
    let mut save_file = bail!(open_save_file(&rom_name));

    let mag = value_t!(matches.value_of("mag"), u32).unwrap_or_else(|e| {
        println!("Invalid magnification: {}", e);
        3
    });

    let mut controller = Controller::new(gb::SCREEN_X as u32 * mag,
                                         gb::SCREEN_Y as u32 * mag);

    let mut emulator;
    {
        let mut rom_bytes = vec![];
        let mut rom = bail!(open_rom(&rom_name));
        bail!(rom.read_to_end(&mut rom_bytes));

        emulator = Emulator::from_data(&rom_bytes, 44100.0).unwrap();

        // Load save file in ram
        // we don't care if we can't fill the whole buffer, it just
        // means that we don't have a save file
        let _ = save_file.read_exact(emulator.cpu.handler_holder.ram());
    }

    let mut debugger = Debugger::new();

    if matches.occurrences_of("debug") > 0 {
        debugger.breakpoint(&mut emulator);
    }

    let mut natural_speed = true;

    loop {
        debugger.check_breakpoints(&mut emulator);

        emulator.cpu.next_instruction();

        if emulator.cpu.handler_holder.should_refresh() {
            match controller.check_events(&mut emulator) {
                Event::Quit => break,
                Event::Break => {
                    debugger.breakpoint(&mut emulator);
                },
                Event::ToggleSpeed => { natural_speed = !natural_speed },
                Event::Continue => {},
            }

            controller.refresh(&mut emulator);
        }

        debugger.next_instruction(&mut emulator);
    }

    bail!(save_file.seek(SeekFrom::Start(0)));
    bail!(save_file.write_all(emulator.cpu.handler_holder.ram()));
}
