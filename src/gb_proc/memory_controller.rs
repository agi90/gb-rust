trait Mbc {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, v: u8);
}

struct Mbc3 {
    selected_bank: usize,
    data: Vec<u8>,
    offset: usize,
    ram: [u8; 8192],
}

impl Mbc3 {
    pub fn new(data: Vec<u8>) -> Mbc3 {
        Mbc3 {
            selected_bank: 1,
            data: data,
            offset: 0,
            ram: [0; 8192],
        }
    }
}

impl Mbc for Mbc3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.data[address as usize],
            0x4000 ... 0x7FFF => self.data[address as usize + self.offset],
            0xA000 ... 0xBFFF => self.ram[(address - 0xA000) as usize],
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x2000 ... 0x3FFF => {
                // Any write to this area will enable the bank of memory contained in v
                // if 0 is selected we will select 1 because 0 is already available
                // statically in 0 0000 - 3FFF
                if v > 0 {
                    self.offset = ((v - 1) as usize) * 0x4000;
                } else {
                    self.offset = 0x0000;
                }
                println!("Setting offset to {:06X}", self.offset);
            },
            0xA000 ... 0xBFFF => {
                self.ram[(address - 0xA000) as usize] = v;
            },
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
