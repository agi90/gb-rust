use std::str;
use std::fs::File;
use std::io::Read;

use gb_proc::memory_controller::MemoryController;
use gb_proc::cpu::Handler;

pub struct BootRom {
    data: Vec<u8>,
}

impl BootRom {
    pub fn from_file(file: &mut File) -> BootRom {
        let mut s = vec![];
        file.read_to_end(&mut s).unwrap();

        BootRom {
            data: s,
        }
    }
}

impl Handler for BootRom {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, v: u8) {
        panic!("Cannot write to boot rom.");
    }
}

pub struct Cartridge {
    startup_graphic: Vec<u8>,
    game_title: String,
    gb_color_game: bool,
    licence_code: u16,
    super_game_boy: bool,
    memory_controller: MemoryController,
    rom_size: usize,
    ram_size: usize,
    destination_code_jp: bool,
    mask_rom_version_number: u8,
}

fn get_rom_size(byte: u8) -> usize {
    match byte {
        0x00 => 2,
        0x01 => 4,
        0x02 => 8,
        0x03 => 16,
        0x04 => 32,
        0x05 => 64,
        0x06 => 128,
        0x52 => 72,
        0x53 => 80,
        0x54 => 96,
        _ => unreachable!(),
    }
}

fn get_ram_size(byte: u8) -> usize {
    match byte {
        0x00 => 0,
        0x01 => 16,
        0x02 => 64,
        0x03 => 256,
        0x04 => 1024,
        _ => unreachable!(),
    }
}

impl Handler for Cartridge {
    fn read(&self, address: u16) -> u8 {
        self.memory_controller.read(address)
    }

    fn write(&mut self, address: u16, v: u8) {
        self.memory_controller.write(address, v);
    }
}

impl Cartridge {
    pub fn from_file(file: &mut File) -> Cartridge {
        let mut s = vec![];
        file.read_to_end(&mut s).unwrap();

        Cartridge {
            startup_graphic: (&s[0x104..0x133]).to_vec(),
            game_title: String::from_utf8(s[0x134..0x142].to_vec()).unwrap(),
            gb_color_game: (s[0x143] == 0x80),
            licence_code: ((s[0x144] as u16) << 8) + (s[0x145] as u16),
            super_game_boy: s[0x146] == 0x03,
            rom_size: get_rom_size(s[0x148]),
            ram_size: get_ram_size(s[0x149]),
            destination_code_jp: s[0x14A] == 0,
            mask_rom_version_number: s[0x14C],
            memory_controller: MemoryController::from_bytes(s),
        }
    }
}
