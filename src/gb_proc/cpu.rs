use std::num::Wrapping;
use gb_proc::opcodes::OpCode;

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
    H_reg: u8,
    L_reg: u8,
    SP_reg: u16,
    PC_reg: u16,

    state: CpuState,
    interrupts_enabled: bool,

    called_set_PC: bool,

    cycles: usize,
    timers: Timers,

    debug: bool,

    handler_holder: Box<HandlerHolder>,
}

struct Timers {
    last_v_blank: usize,
    last_divider: usize,
}

impl Timers {
    pub fn new() -> Timers {
        Timers {
            last_v_blank: 0,
            last_divider: 0,
        }
    }
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
    fn get_handler_read(&self, address: u16) -> &Handler;
    fn get_handler_write(&mut self, address: u16) -> &mut Handler;
}

impl Cpu {
    pub fn new(handler_holder: Box<HandlerHolder>) -> Cpu {
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
            H_reg: 0x01,
            L_reg: 0x4D,
            SP_reg: 0xFFFE,
            // Starting address is 0x100
            PC_reg: 0x100,

            handler_holder: handler_holder,

            state: CpuState::Running,
            interrupts_enabled: true,

            called_set_PC: false,
            cycles: 0,
            timers: Timers::new(),

            debug: false,
        };

        cpu
    }

    pub fn deref(&self, address: u16) -> u8 {
        self.handler_holder.get_handler_read(address)
            .read(address)
    }

    pub fn set_deref(&mut self, address: u16, v: u8) {
        self.handler_holder.get_handler_write(address)
            .write(address, v);
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

    pub fn add_cycles(&mut self, cycles: usize) { self.cycles += cycles }
    pub fn get_cycles(&self) -> usize { self.cycles.clone() }

    fn interrupt(&mut self, address: u16) {
        self.interrupts_enabled = false;
        // TODO: set IF and other iterrupt stuff
        self.inc_PC();
        let next = self.get_PC();

        let h = (next >> 8) as u8;
        self.push_SP(h);

        let l = ((next << 8) >> 8) as u8;
        self.push_SP(l);

        self.set_PC(address);
    }

    pub fn get_debug(&self) -> bool { self.debug }
    pub fn set_debug(&mut self, debug: bool) { self.debug = debug }

    pub fn did_call_set_PC(&self) -> bool { self.called_set_PC }
    pub fn reset_call_set_PC(&mut self) { self.called_set_PC = false }

    pub fn get_BC(&self) -> u16 { ((self.B_reg as u16) << 8) + (self.C_reg as u16) }
    pub fn get_DE(&self) -> u16 { ((self.D_reg as u16) << 8) + (self.E_reg as u16) }
    pub fn get_HL(&self) -> u16 { ((self.H_reg as u16) << 8) + (self.L_reg as u16) }

    pub fn deref_PC(&self) -> u8 {
        let pc = self.get_PC();
        self.deref(self.get_PC())
    }

    pub fn deref_BC(&self) -> u8 {
        self.deref(self.get_BC())
    }

    pub fn deref_DE(&self) -> u8 {
        self.deref(self.get_DE())
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

    fn next_opcode(&mut self) -> OpCode {
        let hex = self.deref_PC();

        if hex == 0xCB {
            self.inc_PC();
            OpCode::from_byte(self.deref_PC(), true)
        } else {
            OpCode::from_byte(hex, false)
        }
    }

    pub fn next_instruction(&mut self) {
        let op = self.next_opcode();

        if self.debug {
            println!("[{:04X}] Executing {}", self.get_PC(), op.to_string());
        }

        op.execute(self);

        if !self.did_call_set_PC() {
            // No jump happened so we need to increase PC
            self.inc_PC();
        } else {
            self.reset_call_set_PC();
        }

        self.add_cycles(op.get_cycles());
    }
}

pub fn print_cpu_status(cpu: &Cpu) {
    println!("[Z,N,H,C] = [{},{},{},{}]",
             cpu.get_Z_flag(),
             cpu.get_N_flag(),
             cpu.get_H_flag(),
             cpu.get_C_flag());

    println!("A = ${:02X}",   cpu.get_A_reg());
    println!("B = ${:02X}",   cpu.get_B_reg());
    println!("C = ${:02X}",   cpu.get_C_reg());
    println!("D = ${:02X}",   cpu.get_D_reg());
    println!("E = ${:02X}",   cpu.get_E_reg());
    println!("F = ${:02X}",   cpu.get_F_reg());
    println!("H = ${:02X}",   cpu.get_H_reg());
    println!("L = ${:02X}",   cpu.get_L_reg());
    println!("PC = ${:02X}",  cpu.get_PC());
    println!("SP = ${:02X}",  cpu.get_SP());
    println!("state = {:?}",  cpu.get_state());
    println!("cycles = {:?}", cpu.get_cycles());
    println!("");
}
