use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};

trait Mbc {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, v: u8);
}

struct Mbc0 {
    data: Vec<u8>,
    ram: [u8; 8192],
}

impl Mbc0 {
    pub fn new(data: Vec<u8>) -> Mbc0 {
        Mbc0 {
            data: data,
            ram: [0; 8192],
        }
    }
}

impl Mbc for Mbc0 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x7FFF => self.data[address as usize],
            0xA000 ... 0xBFFF => self.ram[(address as usize) - 0xA000],
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, _: u16, _: u8) {
        // Theoretically not supposed to write to the Mbc0 ROM, but some games do
        // it anyway.
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
enum MemoryMode {
    C4_32,
    C16_8,
}

enum MbcMode {
    Mbc3,
    Mbc1,
}

#[allow(dead_code)]
struct Mbc13 {
    selected_bank: usize,
    data: Vec<u8>,
    offset: usize,
    ram: RefCell<File>,

    ram_offset: usize,
    ram_enabled: bool,

    mode: MbcMode,
}

impl Mbc13 {
    pub fn new(data: Vec<u8>, mut disk: File, mode: MbcMode) -> Mbc13 {
        let mut ram = vec![];
        disk.read_to_end(&mut ram).unwrap();

        if ram.len() < 32768 {
            println!("Warning: invalid file size, blanking ram.");
            disk.seek(SeekFrom::Start(0)).unwrap();
            disk.write_all(&[0; 32768]).unwrap();
        }

        Mbc13 {
            selected_bank: 1,
            data: data,
            offset: 0,
            ram: RefCell::new(disk),

            ram_offset: 0,
            ram_enabled: false,

            mode: mode,
        }
    }

    fn write_ram(&mut self, address: u16, v: u8) {
        if !self.ram_enabled {
            panic!("MBC3 RAM has not been enabled.");
        }

        let address = (address - 0xA000) as usize + self.ram_offset;
        self.ram.borrow_mut().seek(SeekFrom::Start(address as u64)).unwrap();
        self.ram.borrow_mut().write_all(&[v]).unwrap();
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            panic!("MBC3 RAM has not been enabled.");
        }

        let address = (address - 0xA000) as usize + self.ram_offset;
        self.ram.borrow_mut().seek(SeekFrom::Start(address as u64)).unwrap();
        let mut buffer = [0; 1];
        self.ram.borrow_mut().read_exact(&mut buffer).unwrap();

        buffer[0]
    }

    fn switch_bank_mbc3(&mut self, v: u8) {
        // Any write to this area will enable the bank of memory contained in v
        // if 0 is selected we will select 1 because 0 is already available
        // statically in 0 0000 - 3FFF
        if v > 0 {
            self.offset = ((v - 1) as usize) * 0x4000;
        } else {
            self.offset = 0x0000;
        }
    }

    fn switch_bank_mbc1(&mut self, v: u8) {
        // There's a bug in MBC1 that makes it so that for v = 0x00, 0x20, 0x40, 0x60
        // the bank selected is actually the following one (v+1)
        let v_bug = match v & 0b11111 {
            0x00 | 0x20 | 0x40 | 0x60 => v + 1,
            _ => v,
        };

        self.offset = ((v_bug - 1) as usize) * 0x4000;
    }
}

impl Mbc for Mbc13 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.data[address as usize],
            0x4000 ... 0x7FFF => self.data[address as usize + self.offset],
            0xA000 ... 0xBFFF => self.read_ram(address),
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x0000 ... 0x1FFF => {
                match v {
                    0x00 => self.ram_enabled = false,
                    0x0A => self.ram_enabled = true,
                    _    => panic!("Unrecognized value for RAM enablement."),
                }
            },
            0x2000 ... 0x3FFF => {
                match self.mode {
                    MbcMode::Mbc1 => self.switch_bank_mbc1(v),
                    MbcMode::Mbc3 => self.switch_bank_mbc3(v),
                }
            },
            0x4000 ... 0x5FFF => {
                // Writing to this area will cause the controller to switch
                // banks of in-cartridge RAM.
                // Only the lowest two bits are relevant for this register.
                match v & 0b11 {
                    0x00 => self.ram_offset = 0,
                    0x01 => self.ram_offset = 8192,
                    0x02 => self.ram_offset = 16384,
                    0x03 => self.ram_offset = 24576,
                    _ => unimplemented!(),
                }
            },
            0x6000 ... 0x7FFF => {
                // TODO
            },
            0xA000 ... 0xBFFF => { self.write_ram(address, v); },
            _ => unimplemented!(),
        }
    }
}

pub struct MemoryController {
    controller: Box<Mbc>,
}

impl MemoryController {
    pub fn from_bytes(bytes: Vec<u8>, save_file: File) -> MemoryController {
        let controller = match bytes[0x147] {
            0x00 =>
                Box::new(Mbc0::new(bytes)) as Box<Mbc>,
            0x01 | 0x02 | 0x03 =>
                Box::new(Mbc13::new(bytes, save_file, MbcMode::Mbc1)) as Box<Mbc>,
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 =>
                Box::new(Mbc13::new(bytes, save_file, MbcMode::Mbc3)) as Box<Mbc>,
            _ => {
                println!("Unrecognized type {:02X}", bytes[0x147]);
                panic!();
            }
        };

        MemoryController {
            controller: controller,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.controller.read(address)
    }

    pub fn write(&mut self, address: u16, v: u8) {
        self.controller.write(address, v);
    }
}
