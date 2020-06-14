use hardware::cpu::Handler;
use hardware::memory_controller::MemoryController;
use std::fmt;
use std::fmt::Debug;

#[allow(dead_code)]
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

impl Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Cartridge {{\n")?;
        write!(f, "\t game_title: {}\n", self.game_title)?;
        write!(f, "\t gb_color_game: {}\n", self.gb_color_game)?;
        write!(f, "\t licence_code: {}\n", self.licence_code)?;
        write!(f, "\t super_game_boy: {}\n", self.super_game_boy)?;
        write!(f, "\t rom_size: {}\n", self.rom_size)?;
        write!(f, "\t ram_size: {}\n", self.ram_size)?;
        write!(f, "}}\n")
    }
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
    pub fn from_data(data: &[u8]) -> Cartridge {
        let mut copy = vec![0; data.len()];
        copy.clone_from_slice(data);

        Cartridge {
            startup_graphic: (&copy[0x104..0x133]).to_vec(),
            game_title: String::from_utf8(copy[0x134..0x142].to_vec())
                .unwrap_or("UNKNOWN".to_string()),
            gb_color_game: (copy[0x143] == 0x80),
            licence_code: ((copy[0x144] as u16) << 8) + (copy[0x145] as u16),
            super_game_boy: copy[0x146] == 0x03,
            rom_size: get_rom_size(copy[0x148]),
            ram_size: get_ram_size(copy[0x149]),
            destination_code_jp: copy[0x14A] == 0,
            mask_rom_version_number: copy[0x14C],
            memory_controller: MemoryController::from_bytes(copy),
        }
    }

    pub fn ram(&mut self) -> &mut [u8] {
        self.memory_controller.ram()
    }

    pub fn rtc(&mut self) -> Option<&mut u64> {
        self.memory_controller.rtc()
    }
}
