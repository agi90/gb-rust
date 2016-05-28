trait Mbc {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, v: u8);
}

struct Mbc3 {
    selected_bank: usize,
    data: Vec<u8>,
    offset: usize,
    ram: [u8; 24576], // Up to 3 x 8KB banks

    ram_offset: usize,
    ram_enabled: bool,
}

impl Mbc3 {
    pub fn new(data: Vec<u8>) -> Mbc3 {
        Mbc3 {
            selected_bank: 1,
            data: data,
            offset: 0,
            ram: [0; 24576],

            ram_offset: 0,
            ram_enabled: false,
        }
    }

    fn write_ram(&mut self, address: u16, v: u8) {
        if !self.ram_enabled {
            panic!("MBC3 RAM has not been enabled.");
        }

        self.ram[(address - 0xA000) as usize + self.ram_offset] = v;
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            panic!("MBC3 RAM has not been enabled.");
        }

        self.ram[(address - 0xA000) as usize + self.ram_offset]
    }
}

impl Mbc for Mbc3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.data[address as usize],
            0x4000 ... 0x7FFF => self.data[address as usize + self.offset],
            0xA000 ... 0xBFFF => self.read_ram(address),
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        // println!("Attempt to write {:02X} to {:04X}", v, address);
        match address {
            0x0000 ... 0x1FFF => {
                match v {
                    0x00 => self.ram_enabled = false,
                    0x0A => self.ram_enabled = true,
                    _    => panic!("Unrecognized value for RAM enablement."),
                }
            },
            0x2000 ... 0x3FFF => {
                // Any write to this area will enable the bank of memory contained in v
                // if 0 is selected we will select 1 because 0 is already available
                // statically in 0 0000 - 3FFF
                if v > 0 {
                    self.offset = ((v - 1) as usize) * 0x4000;
                } else {
                    self.offset = 0x0000;
                }
                // println!("Setting offset to {:06X}", self.offset);
            },
            0x4000 ... 0x5FFF => {
                match v {
                    0x00 => self.ram_offset = 0,
                    0x01 => self.ram_offset = 8192,
                    0x02 => self.ram_offset = 16384,
                    _ => unimplemented!(),
                }
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
    pub fn from_bytes(bytes: Vec<u8>) -> MemoryController {
        let controller = match bytes[0x147] {
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x01 => Box::new(Mbc3::new(bytes)),
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
