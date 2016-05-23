use gb_proc::cartridge::Cartridge;
use gb_proc::video_controller::VideoController;
use gb_proc::timer_controller::TimerController;
use gb_proc::sound_controller::SoundController;

#[derive(Debug, PartialEq, Eq)]
pub enum CpuState {
    Running,
    Halt,
    Stop
}

pub struct Cpu {
    // Flags
    Z_flag: bool,
    N_flag: bool,
    H_flag: bool,
    C_flag: bool,

    // Registers
    A_reg: u8,
    B_reg: u8,
    C_reg: u8,
    D_reg: u8,
    E_reg: u8,
    F_reg: u8,
    H_reg: u8,
    L_reg: u8,
    SP_reg: u16,
    PC_reg: u16,

    // RAM
    stack: [u8; 256],
    internal_ram: [u8; 8196],
    switchable_ram: [u8; 8196],
    video_ram: [u8; 8196],

    // Cartridge
    cartridge: Cartridge,

    state: CpuState,
    interrupts_enabled: bool,

    called_set_PC: bool,
    io_registers: IORegisters,
}

impl Cpu {
    pub fn new(cartridge: Cartridge) -> Cpu {
        let mut cpu = Cpu {
            // Flags
            Z_flag: false,
            N_flag: false,
            H_flag: false,
            C_flag: false,

            // Registers
            A_reg: 0x01,
            B_reg: 0x00,
            C_reg: 0x13,
            D_reg: 0x00,
            E_reg: 0xD8,
            F_reg: 0xB0,
            H_reg: 0x01,
            L_reg: 0x4D,
            SP_reg: 0xFFFE,
            // Starting address is 0x100
            PC_reg: 0x100,

            // RAM
            stack: [0; 256],
            internal_ram: [0; 8196],
            switchable_ram: [0; 8196],
            video_ram: [0; 8196],

            cartridge: cartridge,

            state: CpuState::Running,
            interrupts_enabled: true,

            called_set_PC: false,
            io_registers: IORegisters::new(),
        };

        cpu
    }

    pub fn deref(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x7FFF => self.cartridge.read(address),
            0x8000 ... 0x9FFF => self.video_ram[(address - 0x8000) as usize],
            0xA000 ... 0xBFFF => self.cartridge.read(address),
            0xC000 ... 0xDFFF => self.internal_ram[(address - 0xC000) as usize],
            // Accessing this in the real GB will return the internal_ram echoed
            // but it's probably a bug in the emulator, so let's panic
            0xE000 ... 0xFDFF => panic!("Tried to access echo of internal ram"),
            0xFE00 ... 0xFE9F => self.io_registers.read(address),
            0xFEA0 ... 0xFEFF => panic!("Unusable IO ports."),
            0xFF00 ... 0xFF4B => self.io_registers.read(address),
            0xFF4C ... 0xFF7F => panic!("Unusable IO ports."),
            0xFF80 ... 0xFFFE => self.stack[(address - 0xFF80) as usize],
            0xFFFF            => self.io_registers.read(address),
            _ => unreachable!(),
        }
    }

    pub fn set_deref(&mut self, address: u16, v: u8) {
        match address {
            0x0000 ... 0x7FFF => self.cartridge.write(address, v),
            0x8000 ... 0x9FFF => self.video_ram[(address - 0x8000) as usize] = v,
            0xA000 ... 0xBFFF => self.cartridge.write(address, v),
            0xC000 ... 0xDFFF => self.internal_ram[(address - 0xC000) as usize] = v,
            0xE000 ... 0xFDFF => self.internal_ram[(address - 0xE000) as usize] = v,
            0xFE00 ... 0xFE9F => self.io_registers.write(address, v),
            0xFEA0 ... 0xFEFF => panic!("Unusable IO ports."),
            0xFF00 ... 0xFF4B => self.io_registers.write(address, v),
            0xFF4C ... 0xFF7F => panic!("Unusable IO ports."),
            0xFF80 ... 0xFFFE => self.stack[(address - 0xFF80) as usize] = v,
            0xFFFF            => self.io_registers.write(address, v),
            _ => unimplemented!(),
        }
    }

    pub fn set_Z_flag(&mut self) { self.Z_flag = true }
    pub fn set_N_flag(&mut self) { self.N_flag = true }
    pub fn set_H_flag(&mut self) { self.H_flag = true }
    pub fn set_C_flag(&mut self) { self.C_flag = true }

    pub fn reset_Z(&mut self) { self.Z_flag = false }
    pub fn reset_N(&mut self) { self.N_flag = false }
    pub fn reset_H(&mut self) { self.H_flag = false }
    pub fn reset_C(&mut self) { self.C_flag = false }

    pub fn get_Z_flag(&self) -> bool { self.Z_flag }
    pub fn get_N_flag(&self) -> bool { self.N_flag }
    pub fn get_H_flag(&self) -> bool { self.H_flag }
    pub fn get_C_flag(&self) -> bool { self.C_flag }

    pub fn set_A_reg(&mut self, v: u8) { self.A_reg = v }
    pub fn set_B_reg(&mut self, v: u8) { self.B_reg = v }
    pub fn set_C_reg(&mut self, v: u8) { self.C_reg = v }
    pub fn set_D_reg(&mut self, v: u8) { self.D_reg = v }
    pub fn set_E_reg(&mut self, v: u8) { self.E_reg = v }
    pub fn set_F_reg(&mut self, v: u8) { self.F_reg = v }
    pub fn set_H_reg(&mut self, v: u8) { self.H_reg = v }
    pub fn set_L_reg(&mut self, v: u8) { self.L_reg = v }

    pub fn get_A_reg(&self) -> u8 { self.A_reg }
    pub fn get_B_reg(&self) -> u8 { self.B_reg }
    pub fn get_C_reg(&self) -> u8 { self.C_reg }
    pub fn get_D_reg(&self) -> u8 { self.D_reg }
    pub fn get_E_reg(&self) -> u8 { self.E_reg }
    pub fn get_F_reg(&self) -> u8 { self.F_reg }
    pub fn get_H_reg(&self) -> u8 { self.H_reg }
    pub fn get_L_reg(&self) -> u8 { self.L_reg }

    pub fn set_state(&mut self, state: CpuState) { self.state = state }
    pub fn get_state(&self) -> &CpuState { &self.state }

    pub fn disable_interrupts(&mut self) { self.interrupts_enabled = false }
    pub fn enable_interrupts(&mut self) { self.interrupts_enabled = true }

    pub fn set_BC(&mut self, v: u16) {
        self.B_reg = (v >> 8) as u8;
        self.C_reg = v as u8;
    }

    pub fn set_DE(&mut self, v: u16) {
        self.D_reg = (v >> 8) as u8;
        self.E_reg = v as u8;
    }

    pub fn set_HL(&mut self, v: u16) {
        self.H_reg = (v >> 8) as u8;
        self.L_reg = v as u8;
    }

    pub fn get_SP(&self) -> u16 { self.SP_reg }
    pub fn set_SP(&mut self, v: u16) { self.SP_reg = v }

    pub fn get_PC(&self) -> u16 { self.PC_reg }
    pub fn inc_PC(&mut self) {
        self.PC_reg += 1;
    }

    pub fn set_PC(&mut self, v: u16) {
        if v > 0x8000 {
            // Likely a bug in the emulator
            panic!("PC outside of ROM range.");
        }
        self.PC_reg = v;
        self.called_set_PC = true;
    }

    pub fn did_call_set_PC(&self) -> bool { self.called_set_PC }
    pub fn reset_call_set_PC(&mut self) { self.called_set_PC = false }

    pub fn get_BC(&self) -> u16 { ((self.B_reg as u16) << 8) + (self.C_reg as u16) }
    pub fn get_DE(&self) -> u16 { ((self.D_reg as u16) << 8) + (self.E_reg as u16) }
    pub fn get_HL(&self) -> u16 { ((self.H_reg as u16) << 8) + (self.L_reg as u16) }

    pub fn deref_PC(&self) -> u8 {
        self.deref(self.get_PC())
    }

    pub fn deref_BC(&self) -> u8 {
        self.deref(self.get_BC())
    }

    pub fn deref_DE(&self) -> u8 {
        self.deref(self.get_BC())
    }

    pub fn deref_HL(&self) -> u8 {
        self.deref(self.get_HL())
    }

    pub fn deref_SP(&self) -> u8 {
        self.deref(self.get_SP())
    }

    pub fn set_deref_BC(&mut self, v: u8) {
        let address = self.get_BC();
        self.set_deref(address, v);
    }

    pub fn set_deref_DE(&mut self, v: u8) {
        let address = self.get_DE();
        self.set_deref(address, v);
    }

    pub fn set_deref_HL(&mut self, v: u8) {
        let address = self.get_HL();
        self.set_deref(address, v);
    }

    pub fn set_deref_SP(&mut self, v: u8) {
        let address = self.get_SP();
        self.set_deref(address, v);
    }

    pub fn inc_SP(&mut self) { self.SP_reg += 1; }
    fn dec_SP(&mut self) { self.SP_reg -= 1; }

    pub fn push_SP(&mut self, v: u8) {
        self.dec_SP();
        self.set_deref_SP(v);
    }

    pub fn pop_SP(&mut self) -> u8 {
        let v = self.deref_SP();
        self.inc_SP();
        v
    }
}

struct IORegisters {
    joypad_register: JoypadRegister,
    interrupt_register: InterruptRegister,
    video_controller: VideoController,
    serial_transfer_controller: SerialTransferController,
    timer_controller: TimerController,
    sound_controller: SoundController,
}

impl IORegisters {
    pub fn new() -> IORegisters {
        IORegisters {
            joypad_register: JoypadRegister::new(),
            interrupt_register: InterruptRegister::new(),
            video_controller: VideoController::new(),
            serial_transfer_controller: SerialTransferController::new(),
            timer_controller: TimerController::new(),
            sound_controller: SoundController::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF00            => self.joypad_register.read(),
            0xFF01 ... 0xFF02 => self.serial_transfer_controller.read(address),
            0xFF04 ... 0xFF07 => self.timer_controller.read(address),
            0xFF0F            => self.interrupt_register.read(address),
            0xFF09 ... 0xFF3F => self.sound_controller.read(address),
            0xFF40 ... 0xFF4B => self.video_controller.read(address),
            0xFFFF            => self.interrupt_register.read(address),
            _ => panic!(),
        }
    }

    pub fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF00            => self.joypad_register.write(v),
            0xFF01 ... 0xFF02 => self.serial_transfer_controller.write(address, v),
            0xFF04 ... 0xFF07 => self.timer_controller.write(address, v),
            0xFF0F            => self.interrupt_register.write(address, v),
            0xFF09 ... 0xFF3F => self.sound_controller.write(address, v),
            0xFF40 ... 0xFF4B => self.video_controller.write(address, v),
            0xFFFF            => self.interrupt_register.write(address, v),
            _ => panic!(),
        }
    }
}

struct SerialTransferController {
    start_transfer: bool,
    shift_clock: bool,
    fast_clock: bool,
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

    pub fn read(&self, address: u16) -> u8 {
        unimplemented!();
    }

    pub fn write(&mut self, address:u16, v: u8) {
        match address {
            0xFF01 => self.transfer_data(v),
            0xFF02 => self.set_flags(v),
            _ => panic!(),
        }
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

struct InterruptRegister {
    v_blank: bool,
    lcd_stat: bool,
    timer: bool,
    serial: bool,
    joypad: bool,

    v_blank_enabled: bool,
    lcd_stat_enabled: bool,
    timer_enabled: bool,
    serial_enabled: bool,
    joypad_enabled: bool,
}

impl InterruptRegister {
    pub fn new() -> InterruptRegister {
        InterruptRegister {
            v_blank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,

            v_blank_enabled: false,
            lcd_stat_enabled: false,
            timer_enabled: false,
            serial_enabled: false,
            joypad_enabled: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF0F => self.read_interrupt(),
            0xFFFF => self.read_enabled(),
            _      => panic!(),
        }
    }

    pub fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF0F => self.write_interrupt(v),
            0xFFFF => self.write_enabled(v),
            _      => panic!(),
        }
    }

    fn read_enabled(&self) -> u8 {
        (if self.v_blank_enabled  { 0b00000001 } else { 0 }) +
        (if self.lcd_stat_enabled { 0b00000010 } else { 0 }) +
        (if self.timer_enabled    { 0b00000100 } else { 0 }) +
        (if self.serial_enabled   { 0b00001000 } else { 0 }) +
        (if self.joypad_enabled   { 0b00010000 } else { 0 })
    }

    fn write_enabled(&mut self, v: u8) {
        self.v_blank_enabled  = (v & 0b00000001) > 0;
        self.lcd_stat_enabled = (v & 0b00000010) > 0;
        self.timer_enabled    = (v & 0b00000100) > 0;
        self.serial_enabled   = (v & 0b00001000) > 0;
        self.joypad_enabled   = (v & 0b00010000) > 0;
    }

    fn read_interrupt(&self) -> u8 {
        (if self.v_blank && self.v_blank_enabled   { 0b00000001 } else { 0 }) +
        (if self.lcd_stat && self.lcd_stat_enabled { 0b00000010 } else { 0 }) +
        (if self.timer && self.timer_enabled       { 0b00000100 } else { 0 }) +
        (if self.serial && self.serial_enabled     { 0b00001000 } else { 0 }) +
        (if self.joypad && self.joypad_enabled     { 0b00010000 } else { 0 })
    }

    fn write_interrupt(&mut self, v: u8) {
       self.v_blank  = (v & 0b00000001) > 0;
       self.lcd_stat = (v & 0b00000010) > 0;
       self.timer    = (v & 0b00000100) > 0;
       self.serial   = (v & 0b00001000) > 0;
       self.joypad   = (v & 0b00010000) > 0;
    }
}
