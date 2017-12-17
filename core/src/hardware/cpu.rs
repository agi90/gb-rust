pub use hardware::opcodes::OpCode;
use hardware::handler_holder::Key;
use hardware::apu::AudioBuffer;
use hardware::timer_controller::TimerController;
use hardware::ppu::ScreenBuffer;

use std::cell::RefCell;

#[cfg(feature = "debugger")]
use std::collections::HashSet;

pub const CYCLES_PER_STEP: usize = 2;

#[derive(Debug, PartialEq, Eq)]
pub enum CpuState {
    Running,
    Halt,
    Stop
}

pub trait Hardware {
    fn get_screen_buffer(&self) -> &ScreenBuffer;
    fn interrupt(&mut self, interrupt: Interrupt);
    fn key_up(&mut self, key: Key);
    fn key_down(&mut self, key: Key);
    fn next(&mut self);
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
    H_reg: u8,
    L_reg: u8,
    SP_reg: u16,
    PC_reg: u16,

    state: CpuState,
    called_set_PC: bool,

    cycles: usize,
    interrupt_handler: InterruptHandler,

    debug: bool,

    pub handler_holder: Box<HandlerHolder>,

    // The watch_addresses feature is very expensive
    // so we only enabled it for platform where we might
    // run a debugger (e.g. desktop) but not on mobile
    // devices.
    #[cfg(feature = "debugger")]
    watch_addresses: HashSet<u16>,
    address_breakpoint: RefCell<bool>,
}


/** Each Handler will handle a specific region of memory
  in write/read access. */
pub trait Handler {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, v: u8);
}

/** This interface is used to decouple memory access
  and the CPU */
pub trait HandlerHolder {
    fn key_down(&mut self, key: Key);
    fn key_up(&mut self, key: Key);
    fn get_screen_buffer(&self) -> &ScreenBuffer;
    fn get_audio_buffer(&self) -> &AudioBuffer;
    fn get_handler_read(&self, address: u16) -> &Handler;
    fn get_handler_write(&mut self, address: u16) -> &mut Handler;
    fn cpu_step(&mut self);
    fn check_interrupts(&mut self) -> Option<Interrupt>;
    fn should_refresh(&mut self) -> bool;
    fn ram(&mut self) -> &mut [u8];
    fn rtc(&mut self) -> Option<&mut u64>;
    fn reset(&mut self);
}

impl Hardware for Cpu {
    fn interrupt(&mut self, interrupt: Interrupt) {
        self.request_interrupt(interrupt);
    }

    fn key_up(&mut self, key: Key) {
        self.handler_holder.key_up(key);
    }

    fn key_down(&mut self, key: Key) {
        self.handler_holder.key_down(key);
    }

    fn next(&mut self) {
        self.next_instruction();
    }

    fn get_screen_buffer(&self) -> &ScreenBuffer {
        self.handler_holder.get_screen_buffer()
    }
}

impl Cpu {
    pub fn new(handler_holder: Box<HandlerHolder>) -> Cpu {
        let cpu = Cpu {
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
            H_reg: 0x01,
            L_reg: 0x4D,
            SP_reg: 0xFFFE,
            PC_reg: 0x0100,

            handler_holder: handler_holder,

            state: CpuState::Running,
            called_set_PC: false,
            cycles: 0,
            interrupt_handler: InterruptHandler::new(),

            debug: false,

            #[cfg(feature = "debugger")]
            watch_addresses: HashSet::new(),
            address_breakpoint: RefCell::new(false),
        };

        cpu
    }

    pub fn reset(&mut self) {
        self.Z_flag = false;
        self.N_flag = false;
        self.H_flag = false;
        self.C_flag = false;

        self.A_reg = 0x01;
        self.B_reg = 0x00;
        self.C_reg = 0x13;
        self.D_reg = 0x00;
        self.E_reg = 0xD8;
        self.H_reg = 0x01;
        self.L_reg = 0x4d;
        self.SP_reg = 0xFFFE;
        self.PC_reg = 0x0100;

        self.state = CpuState::Running;
        self.called_set_PC = false;
        self.cycles = 0;

        self.handler_holder.reset();
    }

    pub fn address_breakpoint(&self) -> bool {
        let value = self.address_breakpoint.borrow().clone();
        *self.address_breakpoint.borrow_mut() = false;

        value
    }

    pub fn deref_debug(&self, address: u16) -> u8 {
        #[cfg(feature = "debugger")]
        {
            if self.watch_addresses.contains(&address) {
                *self.address_breakpoint.borrow_mut() = true;
            }
        }

        match address {
            0xFF04 ... 0xFF07 | 0xFF0F | 0xFFFF => self.interrupt_handler.read(address),
            _ => self.handler_holder.get_handler_read(address)
                     .read(address)
        }
    }

    pub fn deref(&mut self, address: u16) -> u8 {
        #[cfg(feature = "debugger")]
        {
            if self.watch_addresses.contains(&address) {
                *self.address_breakpoint.borrow_mut() = true;
            }
        }

        self.add_cycles(4);
        match address {
            0xFF04 ... 0xFF07 | 0xFF0F | 0xFFFF => self.interrupt_handler.read(address),
            _ => self.handler_holder.get_handler_read(address)
                     .read(address)
        }
    }

    pub fn set_deref_debug(&mut self, address: u16, v: u8) {
        #[cfg(feature = "debugger")]
        {
            if self.watch_addresses.contains(&address) {
                *self.address_breakpoint.borrow_mut() = true;
            }
        }

        match address {
            0xFF46 => self.copy_memory_to_vram(v),
            0xFF04 ... 0xFF07 | 0xFF0F | 0xFFFF => self.interrupt_handler.write(address, v),
            _ => self.handler_holder.get_handler_write(address)
                     .write(address, v),
        }
    }

    pub fn set_deref(&mut self, address: u16, v: u8) {
        #[cfg(feature = "debugger")]
        {
            if self.watch_addresses.contains(&address) {
                *self.address_breakpoint.borrow_mut() = true;
            }
        }

        self.add_cycles(4);
        match address {
            0xFF46 => self.copy_memory_to_vram(v),
            0xFF04 ... 0xFF07 | 0xFF0F | 0xFFFF => self.interrupt_handler.write(address, v),
            _ => self.handler_holder.get_handler_write(address)
                     .write(address, v),
        }
    }

    fn copy_memory_to_vram(&mut self, v: u8) {
        for i in 0..0xA0 {
            let v = self.deref(((v as u16) << 8) + i);
            self.set_deref(0xFE00 + (i as u16), v);
        }
    }

    #[cfg(feature = "debugger")]
    pub fn watch(&mut self, address: u16) {
        self.watch_addresses.insert(address);
    }

    #[cfg(feature = "debugger")]
    pub fn clear_watch(&mut self) {
        self.watch_addresses.clear();
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

    pub fn set_F_reg(&mut self, v: u8) {
        self.Z_flag = (v & 0b10000000) > 0;
        self.N_flag = (v & 0b01000000) > 0;
        self.H_flag = (v & 0b00100000) > 0;
        self.C_flag = (v & 0b00010000) > 0;
    }

    pub fn set_H_reg(&mut self, v: u8) { self.H_reg = v }
    pub fn set_L_reg(&mut self, v: u8) { self.L_reg = v }

    pub fn get_A_reg(&self) -> u8 { self.A_reg }
    pub fn get_B_reg(&self) -> u8 { self.B_reg }
    pub fn get_C_reg(&self) -> u8 { self.C_reg }
    pub fn get_D_reg(&self) -> u8 { self.D_reg }
    pub fn get_E_reg(&self) -> u8 { self.E_reg }

    pub fn get_F_reg(&self) -> u8 {
        (if self.Z_flag { 0b10000000 } else { 0 }) +
        (if self.N_flag { 0b01000000 } else { 0 }) +
        (if self.H_flag { 0b00100000 } else { 0 }) +
        (if self.C_flag { 0b00010000 } else { 0 })
    }

    pub fn get_H_reg(&self) -> u8 { self.H_reg }
    pub fn get_L_reg(&self) -> u8 { self.L_reg }

    pub fn set_state(&mut self, state: CpuState) { self.state = state }
    pub fn get_state(&self) -> &CpuState { &self.state }

    pub fn disable_interrupts(&mut self) {
        self.interrupt_handler.disable();
    }

    pub fn enable_interrupts(&mut self) {
        self.interrupt_handler.enable();
    }

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
    pub fn set_SP(&mut self, v: u16) {
        self.SP_reg = v
    }

    pub fn get_PC(&self) -> u16 { self.PC_reg }
    pub fn inc_PC(&mut self) {
        self.PC_reg += 1;
    }

    pub fn set_PC(&mut self, v: u16) {
        if (v > 0x8000 && v < 0xC000) || (v > 0xE000 && v < 0xFF80) {
            // Likely a bug in the emulator
            panic!("PC outside of valid range.");
        }
        self.PC_reg = v;
        self.called_set_PC = true;
    }

    pub fn cpu_step(&mut self) {
        self.cycles += CYCLES_PER_STEP;

        self.interrupt_handler.cpu_step();
        self.handler_holder.cpu_step();

        self.handler_holder.check_interrupts()
            .map(|i| self.interrupt_handler.add_interrupt(i));
    }

    pub fn add_cycles(&mut self, mut cycles: usize) {
        assert!(cycles % 2 == 0);

        self.cycles += cycles;
        while cycles > 0 {
            cycles -= 2;
            self.cpu_step();
        }
    }

    pub fn get_cycles(&self) -> usize { self.cycles }

    fn interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_handler.disable();
        let next = self.get_PC();

        let h = (next >> 8) as u8;
        self.push_SP(h);

        let l = ((next << 8) >> 8) as u8;
        self.push_SP(l);

        let address = match interrupt {
            Interrupt::VBlank => 0x0040,
            Interrupt::Stat   => 0x0048,
            Interrupt::Timer  => 0x0050,
            Interrupt::Joypad => 0x0060,
        };

        self.set_PC(address);
        self.reset_call_set_PC();
    }

    pub fn get_debug(&self) -> bool { self.debug }
    pub fn set_debug(&mut self, debug: bool) { self.debug = debug }

    pub fn did_call_set_PC(&self) -> bool { self.called_set_PC }
    pub fn reset_call_set_PC(&mut self) { self.called_set_PC = false }

    pub fn get_BC(&self) -> u16 { ((self.B_reg as u16) << 8) + (self.C_reg as u16) }
    pub fn get_DE(&self) -> u16 { ((self.D_reg as u16) << 8) + (self.E_reg as u16) }
    pub fn get_HL(&self) -> u16 { ((self.H_reg as u16) << 8) + (self.L_reg as u16) }

    pub fn deref_PC(&mut self) -> u8 {
        let pc = self.get_PC();
        self.deref(pc)
    }

    pub fn deref_DE(&mut self) -> u8 {
        let de = self.get_DE();
        self.deref(de)
    }

    pub fn deref_HL(&mut self) -> u8 {
        let hl = self.get_HL();
        self.deref(hl)
    }

    pub fn deref_BC(&mut self) -> u8 {
        let bc = self.get_BC();
        self.deref(bc)
    }

    fn deref_SP(&mut self) -> u8 {
        let sp = self.get_SP();
        self.deref(sp)
    }

    pub fn set_deref_HL(&mut self, v: u8) {
        let address = self.get_HL();
        self.set_deref(address, v);
    }

    fn set_deref_SP(&mut self, v: u8) {
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

    fn next_opcode(&mut self) -> OpCode {
        let hex = self.deref_PC();

        if hex == 0xCB {
            self.inc_PC();
            OpCode::from_byte(self.deref_PC(), true)
        } else {
            OpCode::from_byte(hex, false)
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_handler.add_interrupt(interrupt);
    }

    pub fn next_instruction(&mut self) {
        if self.interrupt_handler.has_interrupts() {
            self.state = CpuState::Running;
        }

        let interrupt = self.interrupt_handler.get_interrupt();

        if let Some(int) = interrupt {
            self.interrupt(int);
            return;
        }

        if self.state == CpuState::Running {
            let op = self.next_opcode();

            if self.debug {
                if op.is_prefixed() {
                    println!("[{:04X}] {}", self.get_PC() - 1, op.to_string());
                } else {
                    println!("[{:04X}] {}", self.get_PC(), op.to_string());
                }
            }

            op.execute(self);

            if !self.did_call_set_PC() {
                // No jump happened so we need to increase PC
                self.inc_PC();
            } else {
                self.reset_call_set_PC();
            }
        } else {
            self.cpu_step();
        };
    }
}

#[derive(PartialEq, Eq)]
enum InterruptStatus {
    Disabled,
    Enabling,
    Enabled
}

struct InterruptHandler {
    enabled: InterruptStatus,
    register: InterruptRegister,
    timer_controller: TimerController,
}

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank,
    Timer,
    Joypad,
    Stat,
}

impl InterruptHandler {
    pub fn new() -> InterruptHandler {
        InterruptHandler {
            enabled: InterruptStatus::Enabled,
            register: InterruptRegister::new(),
            timer_controller: TimerController::new(),
        }
    }

    pub fn add_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.register.v_blank = true,
            Interrupt::Timer => self.register.timer = true,
            Interrupt::Joypad => self.register.joypad = true,
            Interrupt::Stat => self.register.stat = true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = InterruptStatus::Enabling;
    }

    pub fn disable(&mut self) {
        self.enabled = InterruptStatus::Disabled;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF0F | 0xFFFF => self.register.read(address),
            0xFF04 ... 0xFF07 => self.timer_controller.read(address),
            _ => panic!(),
        }
    }

    pub fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF0F | 0xFFFF => self.register.write(address, v),
            0xFF04 ... 0xFF07 => self.timer_controller.write(address, v),
            _ => panic!(),
        }
    }

    pub fn cpu_step(&mut self) {
        self.timer_controller.cpu_step()
            .map(|i| self.add_interrupt(i));
    }

    /// Checks weather an interrupt is queued up (even if it could be disabled)
    pub fn has_interrupts(&self) -> bool {
        self.register.v_blank
            || self.register.stat
            || self.register.timer
            || self.register.joypad
    }

    pub fn get_interrupt(&mut self) -> Option<Interrupt> {
        let is_enabled = match self.enabled {
            InterruptStatus::Disabled => false,
            InterruptStatus::Enabling => {
                // Interrupts take 1 instruction to be enabled
                self.enabled = InterruptStatus::Enabled;
                false
            },
            InterruptStatus::Enabled => true,
        };

        if !is_enabled { return None; }

        if self.register.v_blank_enabled && self.register.v_blank {
            self.register.v_blank = false;
            return Some(Interrupt::VBlank);
        }

        if self.register.stat_enabled && self.register.stat {
            self.register.stat = false;
            return Some(Interrupt::Stat);
        }

        if self.register.timer_enabled && self.register.timer {
            self.register.timer = false;
            return Some(Interrupt::Timer);
        }

        if self.register.joypad_enabled && self.register.joypad {
            self.register.joypad = false;
            return Some(Interrupt::Joypad);
        }

        None
    }
}

#[derive(Debug)]
struct InterruptRegister {
    v_blank: bool,
    stat: bool,
    timer: bool,
    serial: bool,
    joypad: bool,

    v_blank_enabled: bool,
    stat_enabled: bool,
    timer_enabled: bool,
    serial_enabled: bool,
    joypad_enabled: bool,
}

impl InterruptRegister {
    pub fn new() -> InterruptRegister {
        InterruptRegister {
            v_blank: false,
            stat: false,
            timer: false,
            serial: false,
            joypad: false,

            v_blank_enabled: false,
            stat_enabled: false,
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
        (if self.v_blank_enabled { 0b00000001 } else { 0 }) +
        (if self.stat_enabled    { 0b00000010 } else { 0 }) +
        (if self.timer_enabled   { 0b00000100 } else { 0 }) +
        (if self.serial_enabled  { 0b00001000 } else { 0 }) +
        (if self.joypad_enabled  { 0b00010000 } else { 0 })
    }

    fn write_enabled(&mut self, v: u8) {
        self.v_blank_enabled = (v & 0b00000001) > 0;
        self.stat_enabled    = (v & 0b00000010) > 0;
        self.timer_enabled   = (v & 0b00000100) > 0;
        self.serial_enabled  = (v & 0b00001000) > 0;
        self.joypad_enabled  = (v & 0b00010000) > 0;
    }

    fn read_interrupt(&self) -> u8 {
        (if self.v_blank { 0b00000001 } else { 0 }) +
        (if self.stat    { 0b00000010 } else { 0 }) +
        (if self.timer   { 0b00000100 } else { 0 }) +
        (if self.serial  { 0b00001000 } else { 0 }) +
        (if self.joypad  { 0b00010000 } else { 0 })
    }

    fn write_interrupt(&mut self, v: u8) {
       self.v_blank = (v & 0b00000001) > 0;
       self.stat    = (v & 0b00000010) > 0;
       self.timer   = (v & 0b00000100) > 0;
       self.serial  = (v & 0b00001000) > 0;
       self.joypad  = (v & 0b00010000) > 0;
    }
}
