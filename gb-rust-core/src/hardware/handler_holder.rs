use hardware::apu::{SoundController, AudioBuffer};
use hardware::cartridge::Cartridge;
use hardware::cpu::{Handler, HandlerHolder, Interrupt};
use hardware::video_controller::{VideoController, ScreenBuffer};

use bitfield::Bitfield;

pub struct GBHandlerHolder {
    memory_holder: MemoryHolder,
    cartridge: Cartridge,
    pub video_controller: VideoController,
    joypad_register: JoypadRegister,
    serial_transfer_controller: SerialTransferController,
    apu: SoundController,
}

impl GBHandlerHolder {
    pub fn new(cartridge: Cartridge) -> GBHandlerHolder {
        GBHandlerHolder {
            memory_holder: MemoryHolder::new(),
            cartridge: cartridge,
            video_controller: VideoController::new(),
            joypad_register: JoypadRegister::new(),
            serial_transfer_controller: SerialTransferController::new(),
            apu: SoundController::new(),
        }
    }
}

// TODO: move this where the memory is actually used
// e.g. video_ram should be in the VideoController
struct MemoryHolder {
    stack: [u8; 256],
    internal_ram: [u8; 8196],
}

impl MemoryHolder {
    pub fn new() -> MemoryHolder {
        MemoryHolder {
            stack: [0; 256],
            internal_ram: [0; 8196],
        }
    }
}

impl Handler for MemoryHolder {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFEA0 ... 0xFEFF | 0xFF4C ... 0xFF7F => {
                // This area of the memory is not theoretically accessible but
                // some games do try to read from here because of bugs in them.
                // We will just return the default bus value.
                0xFF
            },
            0xC000 ... 0xDFFF => self.internal_ram[(address - 0xC000) as usize],
            0xFF80 ... 0xFFFE => self.stack[(address - 0xFF80) as usize],
            _ => panic!(format!("Address not supported {:04X}", address)),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFEA0 ... 0xFEFF | 0xFF4C ... 0xFF7F => {
                // This area is not mapped to anything in the game boy hardware,
                // so writes have no effect.
            }
            0xC000 ... 0xDFFF => self.internal_ram[(address - 0xC000) as usize] = v,
            0xFF80 ... 0xFFFE => self.stack[(address - 0xFF80) as usize] = v,
            _ => panic!(format!("Address not supported {:04X}", address)),
        }
    }
}

impl HandlerHolder for GBHandlerHolder {
    fn get_screen_buffer(&self) -> &ScreenBuffer {
        self.video_controller.get_screen()
    }

    fn should_refresh(&mut self) -> bool {
        self.video_controller.should_refresh()
    }

    fn get_audio_buffer(&self) -> &AudioBuffer {
        self.apu.get_audio()
    }

    fn key_up(&mut self, key: Key) {
        self.joypad_register.key_up(key);
    }

    fn key_down(&mut self, key: Key) {
        self.joypad_register.key_down(key);
    }

    fn get_handler_read(&self, address: u16) -> &Handler {
        match address {
            0x0000 ... 0x7FFF => &self.cartridge,
            0x8000 ... 0x9FFF => &self.video_controller,
            0xA000 ... 0xBFFF => &self.cartridge,
            0xC000 ... 0xDFFF => &self.memory_holder,
            // Accessing this in the real GB will return the internal_ram echoed
            // but it's probably a bug in the emulator, so let's panic
            0xE000 ... 0xFDFF => panic!("Tried to access echo of internal ram"),
            0xFE00 ... 0xFE9F => &self.video_controller,
            0xFEA0 ... 0xFEFF => &self.memory_holder,
            0xFF00            => &self.joypad_register,
            0xFF01 ... 0xFF02 => &self.serial_transfer_controller,
            0xFF09 ... 0xFF3F => &self.apu,
            0xFF40 ... 0xFF4B => &self.video_controller,
            0xFF4C ... 0xFFFE => &self.memory_holder,
            _ => unreachable!(),
        }
    }

    fn get_handler_write(&mut self, address: u16) -> &mut Handler {
        match address {
            0x0000 ... 0x7FFF => &mut self.cartridge,
            0x8000 ... 0x9FFF => &mut self.video_controller,
            0xA000 ... 0xBFFF => &mut self.cartridge,
            0xC000 ... 0xDFFF => &mut self.memory_holder,
            0xE000 ... 0xFDFF => &mut self.memory_holder,
            0xFE00 ... 0xFE9F => &mut self.video_controller,
            0xFEA0 ... 0xFEFF => &mut self.memory_holder,
            0xFF00            => &mut self.joypad_register,
            0xFF01 ... 0xFF02 => &mut self.serial_transfer_controller,
            0xFF09 ... 0xFF3F => &mut self.apu,
            0xFF40 ... 0xFF4B => &mut self.video_controller,
            0xFF4C ... 0xFFFE => &mut self.memory_holder,
            _ => unimplemented!(),
        }
    }

    fn add_cycles(&mut self, cycles: usize) {
        self.video_controller.add_cycles(cycles);
        self.apu.add_cycles(cycles);
    }

    fn check_interrupts(&mut self) -> Vec<Interrupt> {
        self.video_controller.check_interrupts()
    }

    fn ram(&mut self) -> &mut [u8] {
        self.cartridge.ram()
    }
}

memory_mapper!{
    name: SerialTransferController,
    fields: [
        0xFF01, 0b00000000, transfer_data, 0;
    ],
    bitfields: {
        getters: [
            0xFF02, 0b01111110, flags, 0, [
                get_0, shift_clock, u8;
                get_1, fast_clock, u8;
                get_7, start_transfer, u8
            ]
        ],
        getter_setters: [
        ],
    },
}

#[derive(Debug)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Select,
    Start,
}

struct JoypadRegister {
    P14: bool,
    P15: bool,

    // Button status
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    select: bool,
    start: bool,
}

impl JoypadRegister {
    pub fn new() -> JoypadRegister {
        JoypadRegister {
            P14: false,
            P15: false,
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            select: false,
            start: false,
        }
    }

    pub fn key_up(&mut self, key: Key) {
        match key {
            Key::Up => { self.up = false },
            Key::Down => { self.down = false },
            Key::Left => { self.left = false },
            Key::Right => { self.right = false },
            Key::A => { self.a = false },
            Key::B => { self.b = false },
            Key::Select => { self.select = false },
            Key::Start => { self.start = false },
        }
    }

    pub fn key_down(&mut self, key: Key) {
        match key {
            Key::Up => { self.up = true },
            Key::Down => { self.down = true },
            Key::Left => { self.left = true },
            Key::Right => { self.right = true },
            Key::A => { self.a = true },
            Key::B => { self.b = true },
            Key::Select => { self.select = true },
            Key::Start => { self.start = true },
        }
    }
}

impl Handler for JoypadRegister {
    fn read(&self, address: u16) -> u8 {
        if address != 0xFF00 {
            unimplemented!();
        }

        // 0 means that the button is pressed
        let mut r = 0b00001111;
        if self.P14 {
            if self.right { r &= 0b00001110 };
            if self.left  { r &= 0b00001101 };
            if self.up    { r &= 0b00001011 };
            if self.down  { r &= 0b00000111 };
        } else if self.P15 {
            if self.a      { r &= 0b00001110 };
            if self.b      { r &= 0b00001101 };
            if self.select { r &= 0b00001011 };
            if self.start  { r &= 0b00000111 };
        } else {
            // Can this ever happen?
            unimplemented!();
        }

        r
    }

    fn write(&mut self, address: u16, v: u8) {
        if address != 0xFF00 {
            unimplemented!();
        }

        // I'm not totally sure about this, but it seems to work
        self.P15 = (0b00010000 & v) > 0;
        self.P14 = (0b00100000 & v) > 0;
    }
}
