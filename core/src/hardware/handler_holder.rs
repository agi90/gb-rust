use hardware::apu::{AudioBuffer, SoundController};
use hardware::cartridge::Cartridge;
use hardware::cpu;
use hardware::dma::DmaController;
use hardware::ppu::{Ppu, ScreenBuffer};

use bitfield::Bitfield;

pub struct GBHandlerHolder {
    dma: DmaController,
    inner: InnerHandlerHolder,
}

impl GBHandlerHolder {
    pub fn new(cartridge: Cartridge) -> GBHandlerHolder {
        GBHandlerHolder {
            dma: DmaController::new(),
            inner: InnerHandlerHolder {
                cartridge,
                memory_holder: MemoryHolder::new(),
                ppu: Ppu::new(),
                joypad_register: JoypadRegister::new(),
                serial_transfer_controller: SerialTransfer::new(),
                apu: SoundController::new(),
            },
        }
    }
}

pub struct InnerHandlerHolder {
    memory_holder: MemoryHolder,
    cartridge: Cartridge,
    pub ppu: Ppu,
    joypad_register: JoypadRegister,
    serial_transfer_controller: SerialTransfer,
    apu: SoundController,
}

impl cpu::MapperHolder for InnerHandlerHolder {
    fn get_handler_read(&self, address: u16) -> &dyn cpu::Handler {
        match address {
            0x0000..=0x7FFF => &self.cartridge,
            0x8000..=0x9FFF => &self.ppu,
            0xA000..=0xBFFF => &self.cartridge,
            0xC000..=0xDFFF => &self.memory_holder,
            // Accessing this in the real GB will return the internal_ram echoed
            // but it's probably a bug in the emulator, so let's panic
            0xE000..=0xFDFF => panic!("Tried to access echo of internal ram"),
            0xFEA0..=0xFEFF => &self.memory_holder,
            0xFF00 => &self.joypad_register,
            0xFF01..=0xFF02 => &self.serial_transfer_controller,
            0xFF09..=0xFF3F => &self.apu,
            0xFF40..=0xFF45 => &self.ppu,
            0xFF47..=0xFF4B => &self.ppu,
            0xFF4C..=0xFFFE => &self.memory_holder,
            _ => unreachable!(),
        }
    }

    fn get_handler_write(&mut self, address: u16) -> &mut dyn cpu::Handler {
        match address {
            0x0000..=0x7FFF => &mut self.cartridge,
            0x8000..=0x9FFF => &mut self.ppu,
            0xA000..=0xBFFF => &mut self.cartridge,
            0xC000..=0xDFFF => &mut self.memory_holder,
            0xE000..=0xFDFF => &mut self.memory_holder,
            0xFEA0..=0xFEFF => &mut self.memory_holder,
            0xFF00 => &mut self.joypad_register,
            0xFF01..=0xFF02 => &mut self.serial_transfer_controller,
            0xFF09..=0xFF3F => &mut self.apu,
            0xFF40..=0xFF45 => &mut self.ppu,
            0xFF47..=0xFF4B => &mut self.ppu,
            0xFF4C..=0xFFFE => &mut self.memory_holder,
            _ => unimplemented!(),
        }
    }
}

impl InnerHandlerHolder {
    fn get_screen_buffer(&self) -> &ScreenBuffer {
        self.ppu.get_screen()
    }

    fn should_refresh(&mut self) -> bool {
        self.ppu.should_refresh()
    }

    fn get_audio_buffer(&self) -> &dyn AudioBuffer {
        self.apu.get_audio()
    }

    fn key_up(&mut self, key: Key) {
        self.joypad_register.key_up(key);
    }

    fn key_down(&mut self, key: Key) {
        self.joypad_register.key_down(key);
    }

    fn cpu_step(&mut self) {
        self.ppu.cpu_step();
        self.apu.cpu_step();
    }

    fn check_interrupts(&mut self, oam_ram: &[u8]) -> Option<cpu::Interrupt> {
        self.ppu.check_interrupts(oam_ram)
    }

    fn ram(&mut self) -> &mut [u8] {
        self.cartridge.ram()
    }

    fn rtc(&mut self) -> Option<&mut u64> {
        self.cartridge.rtc()
    }

    fn reset(&mut self) {
        self.memory_holder = MemoryHolder::new();
        self.ppu = Ppu::new();
        self.joypad_register = JoypadRegister::new();
        self.serial_transfer_controller = SerialTransfer::new();
        self.apu = SoundController::new();
    }
}

// TODO: move this where the memory is actually used
// e.g. video_ram should be in the Ppu
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

impl cpu::Handler for MemoryHolder {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFEA0..=0xFEFF | 0xFF4C..=0xFF7F => {
                // This area of the memory is not theoretically accessible but
                // some games do try to read from here because of bugs in them.
                // We will just return open bus.
                0xFF
            }
            0xC000..=0xDFFF => self.internal_ram[(address - 0xC000) as usize],
            0xFF80..=0xFFFE => self.stack[(address - 0xFF80) as usize],
            _ => panic!(format!("Address not supported {:04X}", address)),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFEA0..=0xFEFF | 0xFF4C..=0xFF7F => {
                // This area is not mapped to anything in the game boy hardware,
                // so writes have no effect.
            }
            0xC000..=0xDFFF => self.internal_ram[(address - 0xC000) as usize] = v,
            0xFF80..=0xFFFE => self.stack[(address - 0xFF80) as usize] = v,
            _ => panic!(format!("Address not supported {:04X}", address)),
        }
    }
}

impl cpu::MapperHolder for GBHandlerHolder {
    fn get_handler_read(&self, address: u16) -> &dyn cpu::Handler {
        match address {
            0xFE00..=0xFE9F => &self.dma,
            0xFF46 => &self.dma,
            _ => self.inner.get_handler_read(address),
        }
    }

    fn get_handler_write(&mut self, address: u16) -> &mut dyn cpu::Handler {
        match address {
            0xFE00..=0xFE9F => &mut self.dma,
            0xFF46 => &mut self.dma,
            _ => self.inner.get_handler_write(address),
        }
    }
}

impl cpu::HandlerHolder for GBHandlerHolder {
    fn get_screen_buffer(&self) -> &ScreenBuffer {
        self.inner.get_screen_buffer()
    }

    fn should_refresh(&mut self) -> bool {
        self.inner.should_refresh()
    }

    fn get_audio_buffer(&self) -> &dyn AudioBuffer {
        self.inner.get_audio_buffer()
    }

    fn key_up(&mut self, key: Key) {
        self.inner.key_up(key);
    }

    fn key_down(&mut self, key: Key) {
        self.inner.key_down(key);
    }

    fn cpu_step(&mut self) {
        self.inner.cpu_step();
        self.dma.cpu_step(&mut self.inner);
    }

    fn check_interrupts(&mut self) -> Option<cpu::Interrupt> {
        self.inner.check_interrupts(&self.dma.oam_ram)
    }

    fn ram(&mut self) -> &mut [u8] {
        self.inner.ram()
    }

    fn rtc(&mut self) -> Option<&mut u64> {
        self.inner.rtc()
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.dma = DmaController::new();
    }
}

memory_mapper! {
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

struct SerialTransfer {
    debug_data: Vec<u8>,
    mapper: SerialTransferController,
}

impl SerialTransfer {
    pub fn new() -> SerialTransfer {
        SerialTransfer {
            debug_data: vec![],
            mapper: SerialTransferController::new(),
        }
    }
}

impl cpu::Handler for SerialTransfer {
    fn read(&self, address: u16) -> u8 {
        self.mapper.read(address)
    }

    fn write(&mut self, address: u16, v: u8) {
        if address == 0xFF01 {
            print!("{}", v as char);
            // Blargg's test roms use this address to print debug information
            self.debug_data.push(v);
        }
        self.mapper.write(address, v);
    }
}

#[derive(Debug, Clone, Copy)]
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
            Key::Up => self.up = false,
            Key::Down => self.down = false,
            Key::Left => self.left = false,
            Key::Right => self.right = false,
            Key::A => self.a = false,
            Key::B => self.b = false,
            Key::Select => self.select = false,
            Key::Start => self.start = false,
        }
    }

    pub fn key_down(&mut self, key: Key) {
        match key {
            Key::Up => self.up = true,
            Key::Down => self.down = true,
            Key::Left => self.left = true,
            Key::Right => self.right = true,
            Key::A => self.a = true,
            Key::B => self.b = true,
            Key::Select => self.select = true,
            Key::Start => self.start = true,
        }
    }
}

impl cpu::Handler for JoypadRegister {
    fn read(&self, address: u16) -> u8 {
        if address != 0xFF00 {
            unimplemented!();
        }

        // 0 means that the button is pressed
        let mut r = 0b00001111;
        if self.P14 {
            if self.right {
                r &= 0b00001110
            };
            if self.left {
                r &= 0b00001101
            };
            if self.up {
                r &= 0b00001011
            };
            if self.down {
                r &= 0b00000111
            };
        } else if self.P15 {
            if self.a {
                r &= 0b00001110
            };
            if self.b {
                r &= 0b00001101
            };
            if self.select {
                r &= 0b00001011
            };
            if self.start {
                r &= 0b00000111
            };
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
