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

use gb::{Emulator, Cpu};

use self::controller::{Event, Controller};
use self::debugger::Debugger;

fn open_save_file(rom_name: &str) -> Result<File, String> {
    let mut v: Vec<&str> = rom_name.split('.').collect();
    // If the file does not end with .gb or .gbc we might be reading garbage,
    // let's bail.
    let ext = v.pop().unwrap();
    if ext != "gb" && ext != "gbc" {
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
            let result = $expr;
            if result.is_err() {
                println!("{}", $expr.unwrap_err());
                return;
            }

            result.unwrap()
        }
    }
}

fn parse_string_at(mut address: u16, cpu: &mut Cpu) -> String {
    let mut result = String::from("");
    let mut c = 0xFF;
    while c != 0 {
        c = cpu.deref(address);
        address += 1;
        if c != 0 {
            result.push(c as char);
        }
    }
    result
}

pub fn main() {
    let matches = clap_app!(gbrust =>
        (version: "0.1b")
        (author: "Agi Sferro <agi.novanta@gmail.com>")
        (about: "Yet another DMG emulator written in Rust.")
        (@arg ROM: +required "Selects the ROM to run.")
        (@arg mag: -m --magnification +takes_value "Number of times the screen should be magnified. Default is '3'.") 
        (@arg debug: -d --debug "Starts in debug mode.")
        (@arg headless: -H --headless "Run in headless mode. For integration tests.")
        (@arg timeout: -t --timeout +takes_value "Timeout when running headless, in millions of cycles. Default 100")
        (@arg string_addr: -s --result +takes_value
            "The emulator will print a null-terminated string preset at this address. For integration tests.")
        (@arg commands: -C --commands +takes_value
            "Semicolon separated commands to run after debugger starts. Assumes --debug.")
    ).get_matches();

    let rom_name = matches.value_of("ROM").unwrap();
    let mut save_file = bail!(open_save_file(&rom_name));

    let headless = matches.occurrences_of("headless") > 0;
    let timeout = matches.value_of("timeout")
        .map(|t| t.parse::<usize>())
        .unwrap_or(Ok(100))
        .unwrap();
    let mag = value_t!(matches.value_of("mag"), u32)
        .unwrap_or(3);

    let mut controller = if !headless {
        Some(Controller::new(gb::SCREEN_X as u32 * mag,
                             gb::SCREEN_Y as u32 * mag))
    } else {
        None
    };

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
    let commands: Vec<&str> = matches.value_of("commands")
        .map(|cc| cc.split(';').map(|c| c.trim()).collect())
        .unwrap_or(vec![]);

    for c in &commands {
        bail!(debugger.exec(c, &mut emulator));
    }

    if matches.occurrences_of("debug") > 0 || commands.len() > 0 {
        debugger.breakpoint(&mut emulator);
    }

    let mut natural_speed = true;
    let mut counter = timeout * 1000000;

    while !headless || counter > 0 {
        debugger.check_breakpoints(&mut emulator);

        emulator.cpu.next_instruction();

        if let Some(ref mut c) = controller {
            if emulator.cpu.handler_holder.should_refresh() {
                match c.check_events(&mut emulator) {
                    Event::Quit => break,
                    Event::Break => {
                        debugger.breakpoint(&mut emulator);
                    },
                    Event::ToggleSpeed => { natural_speed = !natural_speed },
                    Event::Continue => {},
                }

                c.refresh(&mut emulator);
            }
        }

        debugger.next_instruction(&mut emulator);

        counter -= 1;
    }

    if let Some(string_address) = matches.value_of("string_addr") {
        let address = bail!(u16::from_str_radix(string_address, 16)
            .map_err(|_| format!("Could not parse address '{}'. \
Please use format XXXX, where X is 0-F.", string_address)));
        print!("{}", parse_string_at(address, &mut emulator.cpu));
    }

    bail!(save_file.seek(SeekFrom::Start(0)));
    bail!(save_file.write_all(emulator.cpu.handler_holder.ram()));
}
