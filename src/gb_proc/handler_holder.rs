use gb_proc::cartridge::Cartridge;
use gb_proc::video_controller::VideoController;
use gb_proc::sound_controller::SoundController;
use gb_proc::cpu::{Handler, HandlerHolder};

use std::num::Wrapping;

pub struct GBHandlerHolder {
    memory_holder: MemoryHolder,
    cartridge: Cartridge,
    io_registers: IORegisters,
}

impl GBHandlerHolder {
    pub fn new(cartridge: Cartridge) -> GBHandlerHolder {
        GBHandlerHolder {
            memory_holder: MemoryHolder::new(),
            cartridge: cartridge,
            io_registers: IORegisters::new(),
        }
    }
}

// TODO: move this where the memory is actually used
// e.g. video_ram should be in the VideoController
struct MemoryHolder {
    video_ram: [u8; 8196],
    stack: [u8; 256],
    internal_ram: [u8; 8196],
}

impl MemoryHolder {
    pub fn new() -> MemoryHolder {
        MemoryHolder {
            video_ram: [0; 8196],
            stack: [0; 256],
            internal_ram: [0; 8196],
        }
    }
}

impl Handler for MemoryHolder {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000 ... 0x9FFF => self.video_ram[(address - 0x8000) as usize],
            0xC000 ... 0xDFFF => self.internal_ram[(address - 0xC000) as usize],
            0xFF80 ... 0xFFFE => self.stack[(address - 0xFF80) as usize],
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x8000 ... 0x9FFF => self.video_ram[(address - 0x8000) as usize] = v,
            0xC000 ... 0xDFFF => self.internal_ram[(address - 0xC000) as usize] = v,
            0xFF80 ... 0xFFFE => self.stack[(address - 0xFF80) as usize] = v,
            _ => unreachable!(),
        }
    }
}

impl HandlerHolder for GBHandlerHolder {
    fn get_handler_read(&self, address: u16) -> &Handler {
        match address {
            0x0000 ... 0x7FFF => &self.cartridge,
            0x8000 ... 0x9FFF => &self.memory_holder,
            0xA000 ... 0xBFFF => &self.cartridge,
            0xC000 ... 0xDFFF => &self.memory_holder,
            // Accessing this in the real GB will return the internal_ram echoed
            // but it's probably a bug in the emulator, so let's panic
            0xE000 ... 0xFDFF => panic!("Tried to access echo of internal ram"),
            0xFE00 ... 0xFE9F => &self.io_registers,
            0xFEA0 ... 0xFEFF => panic!("Unusable IO ports."),
            0xFF00 ... 0xFF4B => &self.io_registers,
            0xFF4C ... 0xFF7F => panic!("Unusable IO ports."),
            0xFF80 ... 0xFFFE => &self.memory_holder,
            0xFFFF            => &self.io_registers,
            _ => unreachable!(),
        }
    }

    fn get_handler_write(&mut self, address: u16) -> &mut Handler {
        match address {
            0x0000 ... 0x7FFF => &mut self.cartridge,
            0x8000 ... 0x9FFF => &mut self.memory_holder,
            0xA000 ... 0xBFFF => &mut self.cartridge,
            0xC000 ... 0xDFFF => &mut self.memory_holder,
            0xE000 ... 0xFDFF => &mut self.memory_holder,
            0xFE00 ... 0xFE9F => &mut self.io_registers,
            0xFEA0 ... 0xFEFF => panic!("Unusable IO ports."),
            0xFF00 ... 0xFF4B => &mut self.io_registers,
            0xFF4C ... 0xFF7F => panic!("Unusable IO ports."),
            0xFF80 ... 0xFFFE => &mut self.memory_holder,
            0xFFFF            => &mut self.io_registers,
            _ => unimplemented!(),
        }
    }
}

struct IORegisters {
    joypad_register: JoypadRegister,
    video_controller: VideoController,
    serial_transfer_controller: SerialTransferController,
    sound_controller: SoundController,
    divider: Wrapping<u8>,
}

impl Handler for IORegisters {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF00            => self.joypad_register.read(),
            0xFF04            => self.divider.0,
            0xFF01 ... 0xFF02 => self.serial_transfer_controller.read(address),
            0xFF09 ... 0xFF3F => self.sound_controller.read(address),
            0xFF40 ... 0xFF4B => self.video_controller.read(address),
            _ => panic!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF00            => self.joypad_register.write(v),
            0xFF04            => { self.divider.0 = 0 },
            0xFF01 ... 0xFF02 => self.serial_transfer_controller.write(address, v),
            0xFF09 ... 0xFF3F => self.sound_controller.write(address, v),
            0xFF40 ... 0xFF4B => self.video_controller.write(address, v),
            _ => panic!(),
        }
    }
}

impl IORegisters {
    pub fn new() -> IORegisters {
        IORegisters {
            joypad_register: JoypadRegister::new(),
            video_controller: VideoController::new(),
            serial_transfer_controller: SerialTransferController::new(),
            sound_controller: SoundController::new(),
            divider: Wrapping(0),
        }
    }
}

struct SerialTransferController {
    start_transfer: bool,
    shift_clock: bool,
    fast_clock: bool,
}

impl Handler for SerialTransferController {
    fn read(&self, address: u16) -> u8 {
        unimplemented!();
    }

    fn write(&mut self, address:u16, v: u8) {
        match address {
            0xFF01 => self.transfer_data(v),
            0xFF02 => self.set_flags(v),
            _ => panic!(),
        }
    }
}

impl SerialTransferController {
    pub fn new() -> SerialTransferController {
        SerialTransferController {
            fast_clock: false,
            shift_clock: false,
            start_transfer: false,
        }
    }

    fn transfer_data(&mut self, v: u8) {
        // not implemented yet, but
        // code call this for no reason apparently?
    }

    fn set_flags(&mut self, v: u8) {
        self.shift_clock    = (v & 0b00000001) > 0;
        self.fast_clock     = (v & 0b00000010) > 0;
        self.start_transfer = (v & 0b10000000) > 0;
    }
}

struct JoypadRegister {
    P14: bool,
    P15: bool,
}

impl JoypadRegister {
    pub fn new() -> JoypadRegister {
        JoypadRegister { P14: false, P15: false }
    }

    pub fn read(&self) -> u8 {
        // Not really implemented for now
        0
    }

    pub fn write(&mut self, v: u8) {
        self.P14 = (0b00001000 & v) == 0;
        self.P15 = (0b00010000 & v) == 0;
    }
}
