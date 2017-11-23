#[macro_use]
extern crate glium;
extern crate sdl2;
extern crate image;

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
use image::ImageBuffer;

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
                println!("{}", result.unwrap_err());
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

const DEFAULT_MAG: u32 = 3;

#[derive(Debug)]
struct Config {
    rom_name: String,
    is_headless: bool,
    timeout: usize,
    mag: u32,
    commands: Vec<String>,
    integ_tests_string_addr: Option<u16>,
    is_debug: bool,
    screenshot_path: Option<String>,
}

impl Config {
    fn from_clap(matches: clap::ArgMatches) -> Result<Config, String> {
        let timeout = matches.value_of("timeout")
            .map(|t| t.parse::<usize>())
            .unwrap_or(Ok(100))
            .unwrap();
        let mag = value_t!(matches.value_of("mag"), u32)
            .unwrap_or(DEFAULT_MAG);
        let commands: Vec<String> = matches.value_of("commands")
            .map(|cc| cc.split(';').map(|c| c.trim().to_string()).collect())
            .unwrap_or(vec![]);
        let string_addr = if let Some(addr) = matches.value_of("string_addr") {
            Some(u16::from_str_radix(addr, 16)
                .map_err(|_| format!("Could not parse address '{}'. \
    Please use format XXXX, where X is 0-F.", addr))?)
        } else {
            None
        };

        Ok(Config {
            rom_name: matches.value_of("ROM").unwrap().to_string(),
            is_headless: matches.occurrences_of("headless") > 0,
            is_debug: matches.occurrences_of("debug") > 0,
            screenshot_path: matches.value_of("screenshot").map(|s| s.to_string()),
            timeout: timeout,
            mag: mag,
            commands: commands,
            integ_tests_string_addr: string_addr,
        })
    }
}

fn save_screenshot(path: &str, screen: &gb::ScreenBuffer) -> Result<(), String> {
    let mut img = ImageBuffer::new(gb::SCREEN_X as u32, gb::SCREEN_Y as u32);
    for i in 0..gb::SCREEN_X {
        for j in 0..gb::SCREEN_Y {
            img.put_pixel(i as u32, j as u32, image::Luma([255 - screen[j][i] as u8 * 64]));
        }
    }

    img.save(path).map_err(|e| e.to_string())
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
        (@arg screenshot: -S --screenshot +takes_value
            "Takes a screenshot at the end of the run. The screenshot will be saved in the file indicated by the argument.")
        (@arg timeout: -t --timeout +takes_value "Timeout when running headless, in millions of cycles. Default 100")
        (@arg string_addr: -s --result +takes_value
            "The emulator will print a null-terminated string preset at this address. For integration tests.")
        (@arg commands: -C --commands +takes_value
            "Semicolon separated commands to run after debugger starts. Assumes --debug.")
    ).get_matches();

    let config = bail!(Config::from_clap(matches));

    let mut save_file = bail!(open_save_file(&config.rom_name));

    let mut controller = if !config.is_headless {
        Some(Controller::new(gb::SCREEN_X as u32 * config.mag,
                             gb::SCREEN_Y as u32 * config.mag))
    } else {
        None
    };

    let mut emulator;
    {
        let mut rom_bytes = vec![];
        let mut rom = bail!(open_rom(&config.rom_name));
        bail!(rom.read_to_end(&mut rom_bytes));

        emulator = Emulator::from_data(&rom_bytes, 44100.0).unwrap();

        // Load save file in ram
        // we don't care if we can't fill the whole buffer, it just
        // means that we don't have a save file
        let _ = save_file.read_exact(emulator.cpu.handler_holder.ram());
    }

    let mut debugger = Debugger::new();
    for c in &config.commands {
        bail!(debugger.exec(c, &mut emulator));
    }

    if config.is_debug || config.commands.len() > 0 {
        debugger.breakpoint(&mut emulator);
    }

    let mut natural_speed = true;
    let mut counter = config.timeout * 1000000;

    while !config.is_headless || counter > 0 {
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

    if let Some(addr) = config.integ_tests_string_addr {
        print!("{}", parse_string_at(addr, &mut emulator.cpu));
    }

    if let Some(screenshot) = config.screenshot_path {
        bail!(save_screenshot(&screenshot,
                              emulator.cpu.handler_holder.get_screen_buffer()));
    }

    bail!(save_file.seek(SeekFrom::Start(0)));
    bail!(save_file.write_all(emulator.cpu.handler_holder.ram()));
}
