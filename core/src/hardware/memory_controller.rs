use std::ops::{Deref, DerefMut};
use std::time::SystemTime;
use std::time;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "imports")]
extern { fn date_now() -> f64; }

pub trait Mbc {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, v: u8);
    fn ram(&mut self) -> &mut [u8];
    fn rtc(&mut self) -> Option<&mut u64>;
}

struct Mbc0 {
    data: Vec<u8>,
    ram: [u8; RAM_BANK_SIZE],
}

impl Mbc0 {
    pub fn new(data: Vec<u8>) -> Mbc0 {
        Mbc0 {
            data: data,
            ram: [0; RAM_BANK_SIZE],
        }
    }
}

impl Mbc for Mbc0 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000 ..= 0x7FFF => self.data[address as usize],
            0xA000 ..= 0xBFFF => self.ram[(address as usize) - 0xA000],
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, _: u16, _: u8) {
        // Theoretically not supposed to write to the Mbc0 ROM, but some games do
        // it anyway.
    }

    fn ram(&mut self) -> &mut [u8] {
        &mut self.ram
    }

    fn rtc(&mut self) -> Option<&mut u64> {
        None
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
    ram: [u8; RAM_BANK_SIZE * 4],

    ram_rtc: RamRtc,
    ram_enabled: bool,

    rtc: u64,
    rtc_register: u8,
    rtc_latch_status: RtcLatchStatus,

    rom_banks: usize,
    mode: MbcMode,
}

const BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum RtcLatchStatus {
    Initial,
    Enabling,
    Enabled,
}

enum RamRtc {
    RamBank(usize),
    RtcRegister(u8),
}

// wasm32-unknown-unknown doesn't support SystemTime::now()
#[cfg(not(target_arch = "wasm32"))]
fn now() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH).unwrap()
        .as_secs()
}

#[cfg(target_arch = "wasm32")]
fn now() -> u64 {
    unsafe { date_now() as u64 }
}

impl Mbc13 {
    pub fn new(data: Vec<u8>, mode: MbcMode) -> Mbc13 {
        if data.len() % BANK_SIZE != 0 {
            panic!(format!("Invalid rom size (must be an integer multiple of {})", BANK_SIZE));
        }

        Mbc13 {
            selected_bank: 1,
            rom_banks: data.len() / BANK_SIZE,
            data: data,
            offset: 0,
            ram: [0; BANK_SIZE * 2],

            ram_rtc: RamRtc::RamBank(0),
            ram_enabled: false,
            rtc: 0,

            rtc_register: 0,
            rtc_latch_status: RtcLatchStatus::Initial,

            mode: mode,
        }
    }

    fn write_ram(&mut self, offset: usize, address: u16, v: u8) {
        if !self.ram_enabled {
            panic!("MBC3 RAM has not been enabled.");
        }

        let address = (address - 0xA000) as usize + offset;
        self.ram[address] = v;
    }

    fn write_ram_mbc1(&mut self, address: u16, v: u8) {
        match self.ram_rtc {
            RamRtc::RamBank(offset) => self.write_ram(offset, address, v),
            // MBC1 does not support RTC
            RamRtc::RtcRegister(_) => unreachable!(),
        }
    }

    fn write_ram_rtc_mbc3(&mut self, address: u16, v: u8) {
        match self.ram_rtc {
            RamRtc::RamBank(offset) => self.write_ram(offset, address, v),
            RamRtc::RtcRegister(reg) => self.write_rtc(reg, v),
        }
    }

    fn read_ram(&self, offset: usize, address: u16) -> u8 {
        if !self.ram_enabled {
            panic!("MBC3 RAM has not been enabled.");
        }

        let address = (address - 0xA000) as usize + offset;
        self.ram[address]
    }

    fn read_rtc(&self, reg: u8) -> u8 {
        let diff = now() - self.rtc;
        let days = diff / 60 / 60 / 24;
        (match reg {
            0x08 => diff % 60,
            0x09 => (diff / 60) % 60,
            0x0A => (diff / 60 / 60) % 24,
            0x0B => (diff / 60 / 60 / 24) % 0xFF,
            0x0C => days >> 8 & 0x01
                + (if days > 511 { 0b10000000 } else { 0 }),
            _ => unreachable!(),
        } as u8)
    }

    fn write_rtc(&mut self, _: u8, _: u8) {
        // TODO
    }

    fn read_ram_mbc1(&self, address: u16) -> u8 {
        match self.ram_rtc {
            RamRtc::RamBank(offset) => self.read_ram(offset, address),
            // MBC1 does not support RTC
            RamRtc::RtcRegister(_) => unreachable!(),
        }
    }

    fn read_ram_rtc_mbc3(&self, address: u16) -> u8 {
        match self.ram_rtc {
            RamRtc::RamBank(offset) => self.read_ram(offset, address),
            RamRtc::RtcRegister(reg) => self.read_rtc(reg),
        }
    }

    fn switch_bank_mbc3(&mut self, v: u8) {
        // Any write to this area will enable the bank of memory contained in v
        // if 0 is selected we will select 1 because 0 is already available
        // statically in 0 0000 - 3FFF
        // Moreover, if the requested bank is out of range, the DMG wraps around
        // and picks a valid bank regardless of the value.
        let bank = v as usize % self.rom_banks;
        if bank > 0 {
            self.offset = (bank - 1) * BANK_SIZE;
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

        // The rom bank selected will wrap around if the value is out of range.
        let bank = v_bug as usize % self.rom_banks;

        self.offset = (bank - 1) * BANK_SIZE;
    }

    fn switch_ram_bank_mbc1(&mut self, v: u8) {
        // Writing to this area will cause the controller to switch
        // banks of in-cartridge RAM.
        // Only the lowest two bits are relevant for this register.
        self.ram_rtc = RamRtc::RamBank(RAM_BANK_SIZE * (v & 0b11) as usize);
    }

    fn switch_ram_bank_mbc3(&mut self, v: u8) {
        if v < 0x04 {
            self.ram_rtc = RamRtc::RamBank(RAM_BANK_SIZE * (v & 0b11) as usize);
        } else if v >= 0x08 && v <= 0x0C {
            self.ram_rtc = RamRtc::RtcRegister(v);
        } else {
            // TODO: Looking at the town map in Pokemon Red/Blue triggers this,
            // not sure what is supposed to happen but doing nothing seems good
            // enough for now.
        }
    }

    fn latch_clock_data(&mut self, v: u8) {
        if v == 0x00 {
            self.rtc_latch_status = RtcLatchStatus::Enabling;
        } else if v == 0x01 && self.rtc_latch_status == RtcLatchStatus::Enabling {
            self.rtc_latch_status = RtcLatchStatus::Enabled;
            // TODO write registers
        }
    }
}

impl Mbc for Mbc13 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000 ..= 0x3FFF => self.data[address as usize],
            0x4000 ..= 0x7FFF => self.data[address as usize + self.offset],
            0xA000 ..= 0xBFFF => {
                match self.mode {
                    MbcMode::Mbc1 => self.read_ram_mbc1(address),
                    MbcMode::Mbc3 => self.read_ram_rtc_mbc3(address),
                }
            }
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x0000 ..= 0x1FFF => {
                match v & 0x0A {
                    0x00 => self.ram_enabled = false,
                    0x0A => self.ram_enabled = true,
                    _    => {
                        // Not too sure about this, but some games write
                        // to this area other values which don't seem to
                        // do anything. TODO: test on real hardware
                    }
                }
            },
            0x2000 ..= 0x3FFF => {
                match self.mode {
                    MbcMode::Mbc1 => self.switch_bank_mbc1(v),
                    MbcMode::Mbc3 => self.switch_bank_mbc3(v),
                }
            },
            0x4000 ..= 0x5FFF => {
                match self.mode {
                    MbcMode::Mbc1 => self.switch_ram_bank_mbc1(v),
                    MbcMode::Mbc3 => self.switch_ram_bank_mbc3(v),
                }
            },
            0x6000 ..= 0x7FFF => {
                match self.mode {
                    MbcMode::Mbc1 => unimplemented!(),
                    MbcMode::Mbc3 => self.latch_clock_data(v),
                }
            },
            0xA000 ..= 0xBFFF => {
                match self.mode {
                    MbcMode::Mbc1 => self.write_ram_mbc1(address, v),
                    MbcMode::Mbc3 => self.write_ram_rtc_mbc3(address, v),
                }
            },
            _ => unimplemented!(),
        }
    }

    fn ram(&mut self) -> &mut [u8] {
        &mut self.ram
    }

    fn rtc(&mut self) -> Option<&mut u64> {
        Some(&mut self.rtc)
    }
}

pub struct MemoryController {
    controller: Box<dyn Mbc>,
}

impl MemoryController {
    pub fn from_bytes(bytes: Vec<u8>) -> MemoryController {
        let controller = match bytes[0x147] {
            0x00 =>
                Box::new(Mbc0::new(bytes)) as Box<dyn Mbc>,
            0x01 | 0x02 | 0x03 =>
                Box::new(Mbc13::new(bytes, MbcMode::Mbc1)) as Box<dyn Mbc>,
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 =>
                Box::new(Mbc13::new(bytes, MbcMode::Mbc3)) as Box<dyn Mbc>,
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

impl Deref for MemoryController {
    type Target = dyn Mbc;
    fn deref(&self) -> &Self::Target {
        &*self.controller
    }
}

impl DerefMut for MemoryController {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.controller
    }
}
