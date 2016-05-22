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
    internal_ram: [u8; 8196],
    switchable_ram: [u8; 8196],
    video_ram: [u8; 8196],

    // Cartridge
    cartrdige_switchable_ram: [u8; 16384],
    cartridge_bank: [u8; 16384],

    state: CpuState,
    interrupts_enabled: bool,
}

impl Cpu {
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
    pub fn inc_PC(&mut self) { self.PC_reg += 1 }
    pub fn set_PC(&mut self, v: u16) { self.PC_reg = v }

    pub fn get_BC(&self) -> u16 { ((self.B_reg as u16) << 8) + (self.C_reg as u16) }
    pub fn get_DE(&self) -> u16 { ((self.D_reg as u16) << 8) + (self.E_reg as u16) }
    pub fn get_HL(&self) -> u16 { ((self.H_reg as u16) << 8) + (self.L_reg as u16) }

    pub fn deref(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.cartridge_bank[address as usize],
            _ => unimplemented!(),
        }
    }

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

    fn inc_SP(&mut self) { self.SP_reg += 1; }
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

    pub fn set_deref(&mut self, _: u16, _: u8) {
        unimplemented!();
    }
}
