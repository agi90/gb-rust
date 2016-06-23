use gb_proc::cpu::{Cpu, CpuState};

macro_rules! op_codes {
    // First the unprefixed op codes
    ($($element: ident: ($tostring: expr,
                         $hex: expr,
                         $func: path)),+;
    // Then the 0XCB prefixed op codes
     $($cb_element: ident: ($cb_tostring: expr,
                            $cb_hex: expr,
                            $cb_func: path)),+) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum OpCode {
            $($element),+,
            $($cb_element),+
        }

        impl OpCode {
            pub fn to_byte(&self) -> u8 {
                match self {
                    $(&OpCode::$element => $hex),*,
                    $(&OpCode::$cb_element => $cb_hex),*,
                }
            }

            pub fn from_byte(hex: u8, prefixed: bool) -> OpCode {
                if prefixed {
                    match hex {
                        $($cb_hex => OpCode::$cb_element),*,
                        _ => panic!(format!("Op code not implemented 0xCB 0x{:x}", hex))
                    }
                } else {
                    match hex {
                        $($hex => OpCode::$element),*,
                        _ => panic!(format!("Op code not implemented 0x{:x}", hex))
                    }
                }
            }

            pub fn is_prefixed(&self) -> bool {
                match self {
                    $(&OpCode::$cb_element => true),*,
                    $(&OpCode::$element => false),*,
                }
            }

            pub fn to_string(&self) -> &'static str {
                match self {
                    $(&OpCode::$element => $tostring),*,
                    $(&OpCode::$cb_element => $cb_tostring),*,
                }
            }

            pub fn execute(&self, cpu: &mut Cpu) {
                match self {
                    $(&OpCode::$element => $func(cpu)),*,
                    $(&OpCode::$cb_element => $cb_func(cpu)),*,
                }
            }
        }
    }
}

fn no_op(_: &mut Cpu) {}

/* Get the value (8-bit) pointed by the Program Counter (PC) */
fn next_value(cpu: &mut Cpu) -> u8 {
    cpu.add_cycles(4);
    cpu.inc_PC();
    let v = cpu.deref_PC();
    if cpu.get_debug() {
        println!("v = ${:02X}", v);
    }
    v
}

/* Get the pointer (16-bit value) pointed  by the Program Counter (PC) */
fn next_pointer(cpu: &mut Cpu) -> u16 {
    cpu.add_cycles(4);
    cpu.inc_PC();
    let l = cpu.deref_PC();

    cpu.add_cycles(4);
    cpu.inc_PC();
    let h = cpu.deref_PC();

    let v = ((h as u16) << 8) + (l as u16);
    if cpu.get_debug() {
        println!("nn = ${:04X}", v);
    }
    v
}

fn deref(cpu: &mut Cpu, address: u16) -> u8 {
    cpu.add_cycles(4);
    cpu.deref(address)
}

fn deref_HL(cpu: &mut Cpu) -> u8 {
    let hl = cpu.get_HL();
    deref(cpu, hl)
}

fn set_deref(cpu: &mut Cpu, address: u16, v: u8) {
    cpu.add_cycles(4);
    cpu.set_deref(address, v);
}

fn set_deref_HL(cpu: &mut Cpu, v: u8) {
    let hl = cpu.get_HL();
    set_deref(cpu, hl, v);
}

fn push_SP(cpu: &mut Cpu, v: u8) {
    cpu.add_cycles(4);
    cpu.push_SP(v);
}

fn pop_SP(cpu: &mut Cpu) -> u8 {
    cpu.add_cycles(4);
    cpu.pop_SP()
}

fn ld_B_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_B_reg(v);
}

fn ld_C_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_C_reg(v);
}

fn ld_D_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_D_reg(v);
}

fn ld_E_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_E_reg(v);
}

fn ld_H_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_H_reg(v);
}

fn ld_L_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_L_reg(v);
}

fn ld_A_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_A_reg(v);
}

fn ld_A_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_A_reg(v);
}

fn ld_A_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_A_reg(v);
}

fn ld_A_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_A_reg(v);
}

fn ld_A_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_A_reg(v);
}

fn ld_A_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_A_reg(v);
}

fn ld_A_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_A_reg(v);
}

fn ld_B_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    cpu.set_B_reg(v);
}

fn ld_B_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_B_reg(v);
}

fn ld_B_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_B_reg(v);
}

fn ld_B_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_B_reg(v);
}

fn ld_B_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_B_reg(v);
}

fn ld_B_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_B_reg(v);
}

fn ld_B_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_B_reg(v);
}

fn ld_C_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    cpu.set_C_reg(v);
}

fn ld_C_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_C_reg(v);
}

fn ld_C_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_C_reg(v);
}

fn ld_C_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_C_reg(v);
}

fn ld_C_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_C_reg(v);
}

fn ld_C_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_C_reg(v);
}

fn ld_C_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_C_reg(v);
}

fn ld_D_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    cpu.set_D_reg(v);
}

fn ld_D_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_D_reg(v);
}

fn ld_D_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_D_reg(v);
}

fn ld_D_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_D_reg(v);
}

fn ld_D_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_D_reg(v);
}

fn ld_D_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_D_reg(v);
}

fn ld_D_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_D_reg(v);
}

fn ld_E_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    cpu.set_E_reg(v);
}

fn ld_E_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_E_reg(v);
}

fn ld_E_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_E_reg(v);
}

fn ld_E_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_E_reg(v);
}

fn ld_E_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_E_reg(v);
}

fn ld_E_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_E_reg(v);
}

fn ld_E_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_E_reg(v);
}

fn ld_H_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    cpu.set_H_reg(v);
}
fn ld_H_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_H_reg(v);
}

fn ld_H_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_H_reg(v);
}

fn ld_H_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_H_reg(v);
}

fn ld_H_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_H_reg(v);
}

fn ld_H_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_H_reg(v);
}

fn ld_H_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_H_reg(v);
}

fn ld_L_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    cpu.set_L_reg(v);
}
fn ld_L_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_L_reg(v);
}

fn ld_L_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_L_reg(v);
}

fn ld_L_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_L_reg(v);
}

fn ld_L_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_L_reg(v);
}

fn ld_L_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_L_reg(v);
}

fn ld_L_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    cpu.set_L_reg(v);
}

fn ld_HL_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    set_deref_HL(cpu, v);
}

fn ld_HL_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    set_deref_HL(cpu, v);
}

fn ld_HL_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    set_deref_HL(cpu, v);
}

fn ld_HL_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    set_deref_HL(cpu, v);
}

fn ld_HL_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    set_deref_HL(cpu, v);
}

fn ld_HL_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    set_deref_HL(cpu, v);
}

fn ld_HL_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    set_deref_HL(cpu, v);
}

fn op_A_X(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, y: u8, cpu: &mut Cpu) {
    let x = cpu.get_A_reg();
    let result = func(x, y, cpu);
    cpu.set_A_reg(result);
}

fn op_A_A(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_A_reg();
    op_A_X(func, y, cpu);
}

fn op_A_B(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_B_reg();
    op_A_X(func, y, cpu);
}

fn op_A_C(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_C_reg();
    op_A_X(func, y, cpu);
}

fn op_A_D(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_D_reg();
    op_A_X(func, y, cpu);
}

fn op_A_E(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_E_reg();
    op_A_X(func, y, cpu);
}

fn op_A_H(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_H_reg();
    op_A_X(func, y, cpu);
}

fn op_A_L(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.get_L_reg();
    op_A_X(func, y, cpu);
}

fn ld_A_BC(cpu: &mut Cpu) {
    let bc = cpu.get_BC();
    let y = deref(cpu, bc);
    cpu.set_A_reg(y);
}

fn ld_A_DE(cpu: &mut Cpu) {
    let de = cpu.get_DE();
    let y = deref(cpu, de);
    cpu.set_A_reg(y);
}

fn op_A_HL(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = deref_HL(cpu);
    op_A_X(func, y, cpu);
}

fn ld_A_nn(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    let y = deref(cpu, address);
    cpu.set_A_reg(y);
}

fn op_A_x(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = next_value(cpu);
    op_A_X(func, y, cpu);
}

fn ld_BC_A(cpu: &mut Cpu) {
    let bc = cpu.get_BC();
    let result = cpu.get_A_reg();
    set_deref(cpu, bc, result);
}

fn ld_DE_A(cpu: &mut Cpu) {
    let de = cpu.get_DE();
    let result = cpu.get_A_reg();
    set_deref(cpu, de, result);
}

fn ld_HL_A(cpu: &mut Cpu) {
    let result = cpu.get_A_reg();
    set_deref_HL(cpu, result);
}

fn ld_nn_A(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    let result = cpu.get_A_reg();
    set_deref(cpu, address, result);
}

fn ld(_: u8, y: u8, _: &mut Cpu) -> u8 { y }

fn ld_A_x(cpu: &mut Cpu)  {  op_A_x(ld, cpu) }

fn ldd(_: u8, y: u8, cpu: &mut Cpu) -> u8 {
    deref(cpu, 0xFF00 + y as u16)
}

fn ld_A_FFC(cpu: &mut Cpu) { op_A_C(ldd, cpu); }

fn ld_FFC_A(cpu: &mut Cpu) {
    let address = 0xFF00 + cpu.get_C_reg() as u16;
    let v = cpu.get_A_reg();
    set_deref(cpu, address, v);
}

fn ldd_A_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    let hl = cpu.get_HL();
    cpu.set_HL(hl - 1);
    cpu.set_A_reg(v);
}

fn ldd_HL_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    let hl = cpu.get_HL();
    set_deref_HL(cpu, v);
    cpu.set_HL(hl - 1);
}

fn ldi_A_HL(cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    let hl = cpu.get_HL();
    cpu.set_HL(hl + 1);
    if cpu.get_debug() {
        println!("v = {:02X}",v);
    }
    cpu.set_A_reg(v);
}

fn ldi_HL_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    let hl = cpu.get_HL();
    set_deref_HL(cpu, v);
    cpu.set_HL(hl + 1);
}

fn ldh_n_A(cpu: &mut Cpu) {
    let address = 0xFF00 + next_value(cpu) as u16;

    // Dereferencing memory
    let x = cpu.get_A_reg();
    cpu.add_cycles(4);
    cpu.set_deref(address, x);
}

fn ldh_A_n(cpu: &mut Cpu) {
    let address = 0xFF00 + next_value(cpu) as u16;

    // Dereferencing memory
    cpu.add_cycles(4);
    let x = cpu.deref(address);
    cpu.set_A_reg(x);
}

fn ld_BC_nn(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    cpu.set_BC(address);
}

fn ld_DE_nn(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    cpu.set_DE(address);
}

fn ld_HL_nn(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    cpu.set_HL(address);
}

fn ld_SP_nn(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    cpu.set_SP(address);
}

fn ld_SP_HL(cpu: &mut Cpu) {
    cpu.add_cycles(4);
    let address = cpu.get_HL();
    cpu.set_SP(address);
}

fn ldhl_SP_n(cpu: &mut Cpu) {
    let sp = cpu.get_SP();
    let n = next_value(cpu);

    cpu.add_cycles(4);
    let result = add_16_8(sp, n, cpu);
    cpu.set_HL(result);
}

fn ld_nn_SP(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    let l = (cpu.get_SP() & 0xFF) as u8;
    let h = ((cpu.get_SP() & 0xFF00) >> 8) as u8;

    set_deref(cpu, address,     l);
    set_deref(cpu, address + 1, h);
}

fn push_AF(cpu: &mut Cpu) {
    cpu.add_cycles(4);

    let a = cpu.get_A_reg();
    push_SP(cpu, a);

    let f = cpu.get_F_reg();
    push_SP(cpu, f);
}

fn push_BC(cpu: &mut Cpu) {
    cpu.add_cycles(4);

    let b = cpu.get_B_reg();
    push_SP(cpu, b);

    let c = cpu.get_C_reg();
    push_SP(cpu, c);
}

fn push_DE(cpu: &mut Cpu) {
    cpu.add_cycles(4);

    let d = cpu.get_D_reg();
    push_SP(cpu, d);

    let e = cpu.get_E_reg();
    push_SP(cpu, e);
}

fn push_HL(cpu: &mut Cpu) {
    cpu.add_cycles(4);

    let h = cpu.get_H_reg();
    push_SP(cpu, h);

    let l = cpu.get_L_reg();
    push_SP(cpu, l);
}

fn pop_AF(cpu: &mut Cpu) {
    let f = pop_SP(cpu);
    cpu.set_F_reg(f);

    let a = pop_SP(cpu);
    cpu.set_A_reg(a);
}

fn pop_BC(cpu: &mut Cpu) {
    let c = pop_SP(cpu);
    cpu.set_C_reg(c);

    let b = pop_SP(cpu);
    cpu.set_B_reg(b);
}

fn pop_DE(cpu: &mut Cpu) {
    let e = pop_SP(cpu);
    cpu.set_E_reg(e);

    let d = pop_SP(cpu);
    cpu.set_D_reg(d);
}

fn pop_HL(cpu: &mut Cpu) {
    let l = pop_SP(cpu);
    cpu.set_L_reg(l);

    let h = pop_SP(cpu);
    cpu.set_H_reg(h);
}

/* Add two values and set relevant flags */
fn add(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    let result = x as u16 + y as u16;

    cpu.reset_N();

    if result == 0x100 || result == 0x00 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    if result > 0xFF {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    if (x & 0x0F) + (y & 0x0F) > 0xF {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    result as u8
}

fn add_A_A(cpu: &mut Cpu)  {  op_A_A(add, cpu); }
fn add_A_B(cpu: &mut Cpu)  {  op_A_B(add, cpu); }
fn add_A_C(cpu: &mut Cpu)  {  op_A_C(add, cpu); }
fn add_A_D(cpu: &mut Cpu)  {  op_A_D(add, cpu); }
fn add_A_E(cpu: &mut Cpu)  {  op_A_E(add, cpu); }
fn add_A_H(cpu: &mut Cpu)  {  op_A_H(add, cpu); }
fn add_A_L(cpu: &mut Cpu)  {  op_A_L(add, cpu); }
fn add_A_HL(cpu: &mut Cpu) { op_A_HL(add, cpu); }
fn add_A_x(cpu: &mut Cpu)  {  op_A_x(add, cpu); }

fn adc(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    if cpu.get_C_flag() {
        let result = x as u16 + y as u16 + 1;

        cpu.reset_N();

        if result == 0x100 {
            cpu.set_Z_flag();
        } else {
            cpu.reset_Z();
        }

        if result > 0xFF {
            cpu.set_C_flag();
        } else {
            cpu.reset_C();
        }

        if (x & 0x0F) + (y & 0x0F) + 1 > 0xF {
            cpu.set_H_flag();
        } else {
            cpu.reset_H();
        }

        result as u8
    } else {
        add(x, y, cpu)
    }
}

fn adc_A_A(cpu: &mut Cpu)  {  op_A_A(adc, cpu); }
fn adc_A_B(cpu: &mut Cpu)  {  op_A_B(adc, cpu); }
fn adc_A_C(cpu: &mut Cpu)  {  op_A_C(adc, cpu); }
fn adc_A_D(cpu: &mut Cpu)  {  op_A_D(adc, cpu); }
fn adc_A_E(cpu: &mut Cpu)  {  op_A_E(adc, cpu); }
fn adc_A_H(cpu: &mut Cpu)  {  op_A_H(adc, cpu); }
fn adc_A_L(cpu: &mut Cpu)  {  op_A_L(adc, cpu); }
fn adc_A_HL(cpu: &mut Cpu) { op_A_HL(adc, cpu); }
fn adc_A_x(cpu: &mut Cpu)  {  op_A_x(adc, cpu); }

/* Subract two values and set relevant flags */
fn sub(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    cpu.set_N_flag();

    if x == y {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    if x < y {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    if (x & 0xF) < (y & 0xF) {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    (x as i16 - y as i16) as u8
}

fn sub_A_A(cpu: &mut Cpu)  {  op_A_A(sub, cpu); }
fn sub_A_B(cpu: &mut Cpu)  {  op_A_B(sub, cpu); }
fn sub_A_C(cpu: &mut Cpu)  {  op_A_C(sub, cpu); }
fn sub_A_D(cpu: &mut Cpu)  {  op_A_D(sub, cpu); }
fn sub_A_E(cpu: &mut Cpu)  {  op_A_E(sub, cpu); }
fn sub_A_H(cpu: &mut Cpu)  {  op_A_H(sub, cpu); }
fn sub_A_L(cpu: &mut Cpu)  {  op_A_L(sub, cpu); }
fn sub_A_HL(cpu: &mut Cpu) { op_A_HL(sub, cpu); }
fn sub_A_x(cpu: &mut Cpu)  {  op_A_x(sub, cpu); }

fn sbc(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    if cpu.get_C_flag() {
        let result = x as i16 - y as i16 - 1;

        cpu.set_N_flag();

        if (result as u8) == 0 {
            cpu.set_Z_flag();
        } else {
            cpu.reset_Z();
        }

        if result < 0 {
            cpu.set_C_flag();
        } else {
            cpu.reset_C();
        }

        if (x & 0xF) as i8 - (y & 0xF) as i8 - 1 < 0 {
            cpu.set_H_flag();
        } else {
            cpu.reset_H();
        }

        (x as i16 - y as i16 - 1) as u8
    } else {
        sub(x, y, cpu)
    }
}

fn sbc_A_A(cpu: &mut Cpu)  {  op_A_A(sbc, cpu); }
fn sbc_A_B(cpu: &mut Cpu)  {  op_A_B(sbc, cpu); }
fn sbc_A_C(cpu: &mut Cpu)  {  op_A_C(sbc, cpu); }
fn sbc_A_D(cpu: &mut Cpu)  {  op_A_D(sbc, cpu); }
fn sbc_A_E(cpu: &mut Cpu)  {  op_A_E(sbc, cpu); }
fn sbc_A_H(cpu: &mut Cpu)  {  op_A_H(sbc, cpu); }
fn sbc_A_L(cpu: &mut Cpu)  {  op_A_L(sbc, cpu); }
fn sbc_A_HL(cpu: &mut Cpu) { op_A_HL(sbc, cpu); }
fn sbc_A_x(cpu: &mut Cpu)  {  op_A_x(sbc, cpu); }

/* Performs x & y and sets the relevant flags. */
fn and(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    let result = x & y;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.set_H_flag();
    cpu.reset_C();

    result
}

fn and_A_A(cpu: &mut Cpu)  {  op_A_A(and, cpu); }
fn and_A_B(cpu: &mut Cpu)  {  op_A_B(and, cpu); }
fn and_A_C(cpu: &mut Cpu)  {  op_A_C(and, cpu); }
fn and_A_D(cpu: &mut Cpu)  {  op_A_D(and, cpu); }
fn and_A_E(cpu: &mut Cpu)  {  op_A_E(and, cpu); }
fn and_A_H(cpu: &mut Cpu)  {  op_A_H(and, cpu); }
fn and_A_L(cpu: &mut Cpu)  {  op_A_L(and, cpu); }
fn and_A_HL(cpu: &mut Cpu) { op_A_HL(and, cpu); }
fn and_A_x(cpu: &mut Cpu)  {  op_A_x(and, cpu); }

/* Performs x | y and sets the relevant flags */
fn or(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    let result = x | y;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();
    cpu.reset_C();

    result
}

fn or_A_A(cpu: &mut Cpu)  {  op_A_A(or, cpu); }
fn or_A_B(cpu: &mut Cpu)  {  op_A_B(or, cpu); }
fn or_A_C(cpu: &mut Cpu)  {  op_A_C(or, cpu); }
fn or_A_D(cpu: &mut Cpu)  {  op_A_D(or, cpu); }
fn or_A_E(cpu: &mut Cpu)  {  op_A_E(or, cpu); }
fn or_A_H(cpu: &mut Cpu)  {  op_A_H(or, cpu); }
fn or_A_L(cpu: &mut Cpu)  {  op_A_L(or, cpu); }
fn or_A_HL(cpu: &mut Cpu) { op_A_HL(or, cpu); }
fn or_A_x(cpu: &mut Cpu)  {  op_A_x(or, cpu); }

/* Performs x ^ y and sets the relevant flags */
fn xor(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    let result = x ^ y;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();
    cpu.reset_C();

    result
}

fn xor_A_A(cpu: &mut Cpu)  {  op_A_A(xor, cpu); }
fn xor_A_B(cpu: &mut Cpu)  {  op_A_B(xor, cpu); }
fn xor_A_C(cpu: &mut Cpu)  {  op_A_C(xor, cpu); }
fn xor_A_D(cpu: &mut Cpu)  {  op_A_D(xor, cpu); }
fn xor_A_E(cpu: &mut Cpu)  {  op_A_E(xor, cpu); }
fn xor_A_H(cpu: &mut Cpu)  {  op_A_H(xor, cpu); }
fn xor_A_L(cpu: &mut Cpu)  {  op_A_L(xor, cpu); }
fn xor_A_HL(cpu: &mut Cpu) { op_A_HL(xor, cpu); }
fn xor_A_x(cpu: &mut Cpu)  {  op_A_x(xor, cpu); }

fn cp(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    sub(x, y, cpu);
    x
}

fn cp_A_A(cpu: &mut Cpu)  {  op_A_A(cp, cpu); }
fn cp_A_B(cpu: &mut Cpu)  {  op_A_B(cp, cpu); }
fn cp_A_C(cpu: &mut Cpu)  {  op_A_C(cp, cpu); }
fn cp_A_D(cpu: &mut Cpu)  {  op_A_D(cp, cpu); }
fn cp_A_E(cpu: &mut Cpu)  {  op_A_E(cp, cpu); }
fn cp_A_H(cpu: &mut Cpu)  {  op_A_H(cp, cpu); }
fn cp_A_L(cpu: &mut Cpu)  {  op_A_L(cp, cpu); }
fn cp_A_HL(cpu: &mut Cpu) { op_A_HL(cp, cpu); }
fn cp_A_x(cpu: &mut Cpu)  {  op_A_x(cp, cpu); }

fn inc(v: u8, cpu: &mut Cpu) -> u8 {
    let result;

    if v == 0xFF {
        cpu.set_Z_flag();
        result = 0x00;
    } else {
        cpu.reset_Z();
        result = v + 1;
    }

    if (v & 0x0F) == 0x0F {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    cpu.reset_N();

    result
}

fn op_A(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    let result = func(v, cpu);
    cpu.set_A_reg(result);
}

fn op_B(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    let result = func(v, cpu);
    cpu.set_B_reg(result);
}

fn op_C(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    let result = func(v, cpu);
    cpu.set_C_reg(result);
}

fn op_D(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    let result = func(v, cpu);
    cpu.set_D_reg(result);
}

fn op_E(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    let result = func(v, cpu);
    cpu.set_E_reg(result);
}

fn op_H(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    let result = func(v, cpu);
    cpu.set_H_reg(result);
}

fn op_L(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    let result = func(v, cpu);
    cpu.set_L_reg(result);
}

fn op_HLp(func: fn(v: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let v = deref_HL(cpu);
    let result = func(v, cpu);
    set_deref_HL(cpu, result);
}

fn inc_A(cpu: &mut Cpu)  {  op_A(inc, cpu); }
fn inc_B(cpu: &mut Cpu)  {  op_B(inc, cpu); }
fn inc_C(cpu: &mut Cpu)  {  op_C(inc, cpu); }
fn inc_D(cpu: &mut Cpu)  {  op_D(inc, cpu); }
fn inc_E(cpu: &mut Cpu)  {  op_E(inc, cpu); }
fn inc_H(cpu: &mut Cpu)  {  op_H(inc, cpu); }
fn inc_L(cpu: &mut Cpu)  {  op_L(inc, cpu); }
fn inc_HLp(cpu: &mut Cpu) { op_HLp(inc, cpu); }

fn dec(v: u8, cpu: &mut Cpu) -> u8 {
    let result;

    if v == 0x00 {
        result = 0xFF;
    } else {
        result = v - 1;
    }

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    if (v & 0x0F) == 0x00 {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    cpu.set_N_flag();

    result
}

fn dec_A(cpu: &mut Cpu)  {  op_A(dec, cpu); }
fn dec_B(cpu: &mut Cpu)  {  op_B(dec, cpu); }
fn dec_C(cpu: &mut Cpu)  {  op_C(dec, cpu); }
fn dec_D(cpu: &mut Cpu)  {  op_D(dec, cpu); }
fn dec_E(cpu: &mut Cpu)  {  op_E(dec, cpu); }
fn dec_H(cpu: &mut Cpu)  {  op_H(dec, cpu); }
fn dec_L(cpu: &mut Cpu)  {  op_L(dec, cpu); }
fn dec_HLp(cpu: &mut Cpu) { op_HLp(dec, cpu); }

fn op_HL_X(func: fn(x: u16, y: u16, cpu: &mut Cpu) -> u16, y: u16, cpu: &mut Cpu) {
    let x = cpu.get_HL();
    let result = func(x, y, cpu);
    cpu.set_HL(result);
}

fn op_HL_HL(func: fn(x: u16, y: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let y = cpu.get_HL();
    op_HL_X(func, y, cpu);
}

fn op_HL_BC(func: fn(x: u16, y: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let y = cpu.get_BC();
    op_HL_X(func, y, cpu);
}

fn op_HL_DE(func: fn(x: u16, y: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let y = cpu.get_DE();
    op_HL_X(func, y, cpu);
}

fn op_HL_SP(func: fn(x: u16, y: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let y = cpu.get_SP();
    op_HL_X(func, y, cpu);
}

fn add_16(x: u16, y: u16, cpu: &mut Cpu) -> u16 {
    // Internal delay
    cpu.add_cycles(4);

    let result = x as u32 + y as u32;

    if result > 0xFFFF {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    if (x & 0x0FFF) + (y & 0x0FFF) > 0x0FFF {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    cpu.reset_N();

    result as u16
}

fn add_HL_BC(cpu: &mut Cpu) { op_HL_BC(add_16, cpu); }
fn add_HL_DE(cpu: &mut Cpu) { op_HL_DE(add_16, cpu); }
fn add_HL_HL(cpu: &mut Cpu) { op_HL_HL(add_16, cpu); }
fn add_HL_SP(cpu: &mut Cpu) { op_HL_SP(add_16, cpu); }

fn op_SP_x(func: fn(x: u16, y: u8, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let x = cpu.get_SP();
    let y = next_value(cpu);
    let result = func(x, y, cpu);
    cpu.set_SP(result);
}

fn add_16_8(x: u16, y: u8, cpu: &mut Cpu) -> u16 {
    // The C and H Flags are based on the unsigned value of n,
    // rather than the signed value and the lower byte
    // of SP
    if (x & 0x00FF) + y as u16 > 0x00FF {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    if (x & 0x000F) + (y & 0x0F) as u16 > 0x000F {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    cpu.reset_Z();
    cpu.reset_N();

    let y_signed = y as i8;
    (x as i32 + y_signed as i32) as u16
}

fn add_SP_x(cpu: &mut Cpu) {
    // Internal delay
    cpu.add_cycles(8);
    op_SP_x(add_16_8, cpu);
}

fn op_HL(func: fn(v: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let v = cpu.get_HL();
    let result = func(v, cpu);
    cpu.set_HL(result);
}

fn op_BC(func: fn(v: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let v = cpu.get_BC();
    let result = func(v, cpu);
    cpu.set_BC(result);
}

fn op_DE(func: fn(v: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let v = cpu.get_DE();
    let result = func(v, cpu);
    cpu.set_DE(result);
}

fn op_SP(func: fn(v: u16, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let v = cpu.get_SP();
    let result = func(v, cpu);
    cpu.set_SP(result);
}

fn inc_16(x: u16, cpu: &mut Cpu) -> u16 {
    cpu.add_cycles(4);

    if x == 0xFFFF {
        0
    } else {
        x + 1
    }
}

fn inc_HL(cpu: &mut Cpu) { op_HL(inc_16, cpu); }
fn inc_DE(cpu: &mut Cpu) { op_DE(inc_16, cpu); }
fn inc_BC(cpu: &mut Cpu) { op_BC(inc_16, cpu); }
fn inc_SP(cpu: &mut Cpu) { op_SP(inc_16, cpu); }

fn dec_16(x: u16, cpu: &mut Cpu) -> u16 {
    cpu.add_cycles(4);

    if x == 0x0 {
        0xFFFF
    } else {
        x - 1
    }
}

fn dec_HL(cpu: &mut Cpu) { op_HL(dec_16, cpu); }
fn dec_DE(cpu: &mut Cpu) { op_DE(dec_16, cpu); }
fn dec_BC(cpu: &mut Cpu) { op_BC(dec_16, cpu); }
fn dec_SP(cpu: &mut Cpu) { op_SP(dec_16, cpu); }

// This instruction conditionally adjusts the accumulator for BCD addition
// and subtraction operations. For addition (ADD, ADC, INC) or subtraction
// (SUB, SBC, DEC, NEC), the following table indicates the operation performed:
//
// --------------------------------------------------------------------------------
// |           | C Flag  | HEX value in | H Flag | HEX value in | Number  | C flag|
// | Operation | Before  | upper digit  | Before | lower digit  | added   | After |
// |           | DAA     | (bit 7-4)    | DAA    | (bit 3-0)    | to byte | DAA   |
// |------------------------------------------------------------------------------|
// |           |    0    |     0-9      |   0    |     0-9      |   00    |   0   |
// |   ADD     |    0    |     0-8      |   0    |     A-F      |   06    |   0   |
// |           |    0    |     0-9      |   1    |     0-3      |   06    |   0   |
// |   ADC     |    0    |     A-F      |   0    |     0-9      |   60    |   1   |
// |           |    0    |     9-F      |   0    |     A-F      |   66    |   1   |
// |   INC     |    0    |     A-F      |   1    |     0-3      |   66    |   1   |
// |           |    1    |     0-2      |   0    |     0-9      |   60    |   1   |
// |           |    1    |     0-2      |   0    |     A-F      |   66    |   1   |
// |           |    1    |     0-3      |   1    |     0-3      |   66    |   1   |
// |------------------------------------------------------------------------------|
// |   SUB     |    0    |     0-9      |   0    |     0-9      |   00    |   0   |
// |   SBC     |    0    |     0-8      |   1    |     6-F      |   FA    |   0   |
// |   DEC     |    1    |     7-F      |   0    |     0-9      |   A0    |   1   |
// |   NEG     |    1    |     6-F      |   1    |     6-F      |   9A    |   1   |
// --------------------------------------------------------------------------------
fn daa(v: u8, cpu: &mut Cpu) -> u8 {
    let mut result = v as i32;

    if !cpu.get_N_flag() {
        if cpu.get_H_flag() || v & 0xF > 0x9 {
            result += 0x06;
        }

        if cpu.get_C_flag() || result > 0x9F {
            result += 0x60;
        }
    } else {
        if cpu.get_H_flag() {
            result = (result - 6) & 0xFF;
        }

        if cpu.get_C_flag() {
            result -= 0x60;
        }
    }

    cpu.reset_H();
    cpu.reset_Z();

    if (result & 0x100) == 0x100 {
        cpu.set_C_flag();
    }

    result &= 0xFF;

    if result == 0 {
        cpu.set_Z_flag();
    }

    result as u8
}

fn daa_A(cpu: &mut Cpu) { op_A(daa, cpu); }

fn cpl(x: u8, cpu: &mut Cpu) -> u8 {
    cpu.set_N_flag();
    cpu.set_H_flag();
    !x
}

fn cpl_A(cpu: &mut Cpu) { op_A(cpl, cpu); }

fn ccf(cpu: &mut Cpu) {
    if cpu.get_C_flag() {
        cpu.reset_C();
    } else {
        cpu.set_C_flag();
    }

    cpu.reset_N();
    cpu.reset_H();
}

fn scf(cpu: &mut Cpu) {
    cpu.reset_N();
    cpu.reset_H();
    cpu.set_C_flag();
}

fn halt(cpu: &mut Cpu) {
    cpu.set_state(CpuState::Halt);
}

fn stop(cpu: &mut Cpu) {
    cpu.set_state(CpuState::Stop);
}

fn di(cpu: &mut Cpu) {
    cpu.disable_interrupts();
}

fn ei(cpu: &mut Cpu) {
    cpu.enable_interrupts();
}

// We need this because RLCA does not set the Z flag while other RLC operations do
fn rlc_base(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_7 = x >> 7;

    let result = (x << 1) + bit_7;

    if bit_7 == 1 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rlc(x: u8, cpu: &mut Cpu) -> u8 {
    z_if_result_is_zero(x, rlc_base, cpu)
}

fn rlca(cpu: &mut Cpu) {
    op_A(rlc_base, cpu);
    cpu.reset_Z();
}

// We need this because RLA does not set the Z flag while other RL operations do
fn rl_base(x: u8, cpu: &mut Cpu) -> u8 {
    let c_flag = if cpu.get_C_flag() { 1 } else { 0 };

    if (x >> 7) == 1 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = (x << 1) + c_flag;

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn z_if_result_is_zero(x: u8, func: fn(x: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) -> u8 {
    let result = func(x, cpu);

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    result
}

fn rl(x: u8, cpu: &mut Cpu) -> u8 {
    z_if_result_is_zero(x, rl_base, cpu)
}

fn rla(cpu: &mut Cpu) {
    op_A(rl_base, cpu);
    cpu.reset_Z();
}

fn rrc_base(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_7 = x << 7;

    if bit_7 == 0x80 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = (x >> 1) + bit_7;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rrc(x: u8, cpu: &mut Cpu) -> u8 {
    z_if_result_is_zero(x, rrc_base, cpu)
}

fn rrca(cpu: &mut Cpu) {
    op_A(rrc_base, cpu);
    cpu.reset_Z();
}

fn rr_base(x: u8, cpu: &mut Cpu) -> u8 {
    let c_flag = if cpu.get_C_flag() { 0x80 } else { 0 };

    if (x << 7) == 0x80 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = (x >> 1) + c_flag;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rr(x: u8, cpu: &mut Cpu) -> u8 {
    z_if_result_is_zero(x, rr_base, cpu)
}

fn rra(cpu: &mut Cpu) {
    op_A(rr, cpu);
    cpu.reset_Z();
}

fn jp_nn(cpu: &mut Cpu) {
    let nn = next_pointer(cpu);

    cpu.add_cycles(4);
    cpu.set_PC(nn);
}

fn jp_cond_nn(cond: bool, cpu: &mut Cpu) {
    if cond {
        jp_nn(cpu);
    } else {
        // discard operand
        next_pointer(cpu);
    }
}

fn jp_NZ_nn(cpu: &mut Cpu) {
    jp_cond_nn(!cpu.get_Z_flag(), cpu);
}

fn jp_Z_nn(cpu: &mut Cpu) {
    jp_cond_nn(cpu.get_Z_flag(), cpu);
}

fn jp_NC_nn(cpu: &mut Cpu) {
    jp_cond_nn(!cpu.get_C_flag(), cpu);
}

fn jp_C_nn(cpu: &mut Cpu) {
    jp_cond_nn(cpu.get_C_flag(), cpu);
}

fn jp_HL(cpu: &mut Cpu) {
    let address = cpu.get_HL();
    cpu.set_PC(address);
}

fn jr_n(cpu: &mut Cpu) {
    let n = next_value(cpu);

    let offset = if n > 0x81 {
        n as i32 - 256
    } else {
        n as i32
    };

    let next = (cpu.get_PC() as i32 + offset + 1) as u16;

    cpu.add_cycles(4);
    cpu.set_PC(next);
}

fn jr_cond_n(cond: bool, cpu: &mut Cpu) {
    if cond {
        jr_n(cpu);
    } else {
        // Discard operand
        next_value(cpu);
    }
}

fn jr_NZ_n(cpu: &mut Cpu) {
    jr_cond_n(!cpu.get_Z_flag(), cpu);
}

fn jr_Z_n(cpu: &mut Cpu) {
    jr_cond_n(cpu.get_Z_flag(), cpu);
}

fn jr_NC_n(cpu: &mut Cpu) {
    jr_cond_n(!cpu.get_C_flag(), cpu);
}

fn jr_C_n(cpu: &mut Cpu) {
    jr_cond_n(cpu.get_C_flag(), cpu);
}

fn call_nn(cpu: &mut Cpu) {
    let nn = next_pointer(cpu);

    cpu.inc_PC();
    let next = cpu.get_PC();

    let h = (next >> 8) as u8;
    push_SP(cpu, h);

    let l = ((next << 8) >> 8) as u8;
    push_SP(cpu, l);

    // Internal delay
    cpu.add_cycles(4);

    cpu.set_PC(nn);
}

fn call_cond_nn(cond: bool, cpu: &mut Cpu) {
    if cond {
        call_nn(cpu);
    } else {
        // discard operand
        next_pointer(cpu);
    }
}

fn call_NZ_nn(cpu: &mut Cpu) {
    call_cond_nn(!cpu.get_Z_flag(), cpu);
}

fn call_Z_nn(cpu: &mut Cpu) {
    call_cond_nn(cpu.get_Z_flag(), cpu);
}

fn call_NC_nn(cpu: &mut Cpu) {
    call_cond_nn(!cpu.get_C_flag(), cpu);
}

fn call_C_nn(cpu: &mut Cpu) {
    call_cond_nn(cpu.get_C_flag(), cpu);
}

fn rst_n(n: u8, cpu: &mut Cpu) {
    cpu.inc_PC();
    let next = cpu.get_PC();

    let h = (next >> 8) as u8;
    push_SP(cpu, h);

    let l = ((next << 8) >> 8) as u8;
    push_SP(cpu, l);

    cpu.add_cycles(4);
    cpu.set_PC(n as u16);
}

fn rst_00(cpu: &mut Cpu) { rst_n(0x00, cpu); }
fn rst_08(cpu: &mut Cpu) { rst_n(0x08, cpu); }
fn rst_10(cpu: &mut Cpu) { rst_n(0x10, cpu); }
fn rst_18(cpu: &mut Cpu) { rst_n(0x18, cpu); }
fn rst_20(cpu: &mut Cpu) { rst_n(0x20, cpu); }
fn rst_28(cpu: &mut Cpu) { rst_n(0x28, cpu); }
fn rst_30(cpu: &mut Cpu) { rst_n(0x30, cpu); }
fn rst_38(cpu: &mut Cpu) { rst_n(0x38, cpu); }

fn ret(cpu: &mut Cpu) {
    let l = pop_SP(cpu);
    let h = pop_SP(cpu);

    let next = l as u16 + ((h as u16) << 8);

    cpu.add_cycles(4);
    cpu.set_PC(next);
}

fn ret_cond(cond: bool, cpu: &mut Cpu) {
    cpu.add_cycles(4);
    if cond {
        ret(cpu);
    }
}

fn ret_NZ(cpu: &mut Cpu) {
    ret_cond(!cpu.get_Z_flag(), cpu);
}

fn ret_Z(cpu: &mut Cpu) {
    ret_cond(cpu.get_Z_flag(), cpu);
}

fn ret_NC(cpu: &mut Cpu) {
    ret_cond(!cpu.get_C_flag(), cpu);
}

fn ret_C(cpu: &mut Cpu) {
    ret_cond(cpu.get_C_flag(), cpu);
}

fn reti(cpu: &mut Cpu) {
    ret(cpu);
    ei(cpu);
}

fn swap(x: u8, cpu: &mut Cpu) -> u8 {
    let result = (x >> 4) + (x << 4);

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();
    cpu.reset_C();

    result
}

fn swap_A(cpu: &mut Cpu)  {   op_A(swap, cpu); }
fn swap_B(cpu: &mut Cpu)  {   op_B(swap, cpu); }
fn swap_C(cpu: &mut Cpu)  {   op_C(swap, cpu); }
fn swap_D(cpu: &mut Cpu)  {   op_D(swap, cpu); }
fn swap_E(cpu: &mut Cpu)  {   op_E(swap, cpu); }
fn swap_H(cpu: &mut Cpu)  {   op_H(swap, cpu); }
fn swap_L(cpu: &mut Cpu)  {   op_L(swap, cpu); }
fn swap_HL(cpu: &mut Cpu) { op_HLp(swap, cpu); }

fn rlc_A(cpu: &mut Cpu)  {   op_A(rlc, cpu); }
fn rlc_B(cpu: &mut Cpu)  {   op_B(rlc, cpu); }
fn rlc_C(cpu: &mut Cpu)  {   op_C(rlc, cpu); }
fn rlc_D(cpu: &mut Cpu)  {   op_D(rlc, cpu); }
fn rlc_E(cpu: &mut Cpu)  {   op_E(rlc, cpu); }
fn rlc_H(cpu: &mut Cpu)  {   op_H(rlc, cpu); }
fn rlc_L(cpu: &mut Cpu)  {   op_L(rlc, cpu); }
fn rlc_HL(cpu: &mut Cpu) { op_HLp(rlc, cpu); }

fn rl_A(cpu: &mut Cpu)  {   op_A(rl, cpu); }
fn rl_B(cpu: &mut Cpu)  {   op_B(rl, cpu); }
fn rl_C(cpu: &mut Cpu)  {   op_C(rl, cpu); }
fn rl_D(cpu: &mut Cpu)  {   op_D(rl, cpu); }
fn rl_E(cpu: &mut Cpu)  {   op_E(rl, cpu); }
fn rl_H(cpu: &mut Cpu)  {   op_H(rl, cpu); }
fn rl_L(cpu: &mut Cpu)  {   op_L(rl, cpu); }
fn rl_HL(cpu: &mut Cpu) { op_HLp(rl, cpu); }

fn rrc_A(cpu: &mut Cpu)  {   op_A(rrc, cpu); }
fn rrc_B(cpu: &mut Cpu)  {   op_B(rrc, cpu); }
fn rrc_C(cpu: &mut Cpu)  {   op_C(rrc, cpu); }
fn rrc_D(cpu: &mut Cpu)  {   op_D(rrc, cpu); }
fn rrc_E(cpu: &mut Cpu)  {   op_E(rrc, cpu); }
fn rrc_H(cpu: &mut Cpu)  {   op_H(rrc, cpu); }
fn rrc_L(cpu: &mut Cpu)  {   op_L(rrc, cpu); }
fn rrc_HL(cpu: &mut Cpu) { op_HLp(rrc, cpu); }

fn rr_A(cpu: &mut Cpu)  {   op_A(rr, cpu); }
fn rr_B(cpu: &mut Cpu)  {   op_B(rr, cpu); }
fn rr_C(cpu: &mut Cpu)  {   op_C(rr, cpu); }
fn rr_D(cpu: &mut Cpu)  {   op_D(rr, cpu); }
fn rr_E(cpu: &mut Cpu)  {   op_E(rr, cpu); }
fn rr_H(cpu: &mut Cpu)  {   op_H(rr, cpu); }
fn rr_L(cpu: &mut Cpu)  {   op_L(rr, cpu); }
fn rr_HL(cpu: &mut Cpu) { op_HLp(rr, cpu); }

fn sla(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_7 = x >> 7;
    if bit_7 == 1 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    cpu.reset_N();
    cpu.reset_H();

    let result = x << 1;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    result
}

fn sla_A(cpu: &mut Cpu)  {   op_A(sla, cpu); }
fn sla_B(cpu: &mut Cpu)  {   op_B(sla, cpu); }
fn sla_C(cpu: &mut Cpu)  {   op_C(sla, cpu); }
fn sla_D(cpu: &mut Cpu)  {   op_D(sla, cpu); }
fn sla_E(cpu: &mut Cpu)  {   op_E(sla, cpu); }
fn sla_H(cpu: &mut Cpu)  {   op_H(sla, cpu); }
fn sla_L(cpu: &mut Cpu)  {   op_L(sla, cpu); }
fn sla_HL(cpu: &mut Cpu) { op_HLp(sla, cpu); }

fn sra(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_0 = x << 7;
    if bit_0 == 0x80 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    cpu.reset_N();
    cpu.reset_H();

    let result = (x >> 1) + ((x >> 7) << 7);

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    result
}

fn sra_A(cpu: &mut Cpu)  {   op_A(sra, cpu); }
fn sra_B(cpu: &mut Cpu)  {   op_B(sra, cpu); }
fn sra_C(cpu: &mut Cpu)  {   op_C(sra, cpu); }
fn sra_D(cpu: &mut Cpu)  {   op_D(sra, cpu); }
fn sra_E(cpu: &mut Cpu)  {   op_E(sra, cpu); }
fn sra_H(cpu: &mut Cpu)  {   op_H(sra, cpu); }
fn sra_L(cpu: &mut Cpu)  {   op_L(sra, cpu); }
fn sra_HL(cpu: &mut Cpu) { op_HLp(sra, cpu); }

fn srl(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_0 = x << 7;
    if bit_0 == 0x80 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    cpu.reset_N();
    cpu.reset_H();

    let result = x >> 1;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    result
}

fn srl_A(cpu: &mut Cpu)  {   op_A(srl, cpu); }
fn srl_B(cpu: &mut Cpu)  {   op_B(srl, cpu); }
fn srl_C(cpu: &mut Cpu)  {   op_C(srl, cpu); }
fn srl_D(cpu: &mut Cpu)  {   op_D(srl, cpu); }
fn srl_E(cpu: &mut Cpu)  {   op_E(srl, cpu); }
fn srl_H(cpu: &mut Cpu)  {   op_H(srl, cpu); }
fn srl_L(cpu: &mut Cpu)  {   op_L(srl, cpu); }
fn srl_HL(cpu: &mut Cpu) { op_HLp(srl, cpu); }

fn bit(b: u8, x: u8, cpu: &mut Cpu) {
    let mask = match b {
        0 => 0b00000001,
        1 => 0b00000010,
        2 => 0b00000100,
        3 => 0b00001000,
        4 => 0b00010000,
        5 => 0b00100000,
        6 => 0b01000000,
        7 => 0b10000000,
        _ => panic!()
    };

    if (mask & x) == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.set_H_flag();
}

fn op_bit_A(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_A_reg();
    func(b, x, cpu);
}

fn op_bit_B(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_B_reg();
    func(b, x, cpu);
}

fn op_bit_C(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_C_reg();
    func(b, x, cpu);
}

fn op_bit_D(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_D_reg();
    func(b, x, cpu);
}

fn op_bit_E(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_E_reg();
    func(b, x, cpu);
}

fn op_bit_H(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_H_reg();
    func(b, x, cpu);
}

fn op_bit_L(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = cpu.get_L_reg();
    func(b, x, cpu);
}

fn op_bit_HL(func: fn(b: u8, x: u8, cpu: &mut Cpu), b: u8, cpu: &mut Cpu) {
    let x = deref_HL(cpu);
    func(b, x, cpu);
}

fn bit_0_A(cpu: &mut Cpu)  {  op_bit_A(bit, 0, cpu); }
fn bit_0_B(cpu: &mut Cpu)  {  op_bit_B(bit, 0, cpu); }
fn bit_0_C(cpu: &mut Cpu)  {  op_bit_C(bit, 0, cpu); }
fn bit_0_D(cpu: &mut Cpu)  {  op_bit_D(bit, 0, cpu); }
fn bit_0_E(cpu: &mut Cpu)  {  op_bit_E(bit, 0, cpu); }
fn bit_0_H(cpu: &mut Cpu)  {  op_bit_H(bit, 0, cpu); }
fn bit_0_L(cpu: &mut Cpu)  {  op_bit_L(bit, 0, cpu); }
fn bit_0_HL(cpu: &mut Cpu) { op_bit_HL(bit, 0, cpu); }

fn bit_1_A(cpu: &mut Cpu)  {  op_bit_A(bit, 1, cpu); }
fn bit_1_B(cpu: &mut Cpu)  {  op_bit_B(bit, 1, cpu); }
fn bit_1_C(cpu: &mut Cpu)  {  op_bit_C(bit, 1, cpu); }
fn bit_1_D(cpu: &mut Cpu)  {  op_bit_D(bit, 1, cpu); }
fn bit_1_E(cpu: &mut Cpu)  {  op_bit_E(bit, 1, cpu); }
fn bit_1_H(cpu: &mut Cpu)  {  op_bit_H(bit, 1, cpu); }
fn bit_1_L(cpu: &mut Cpu)  {  op_bit_L(bit, 1, cpu); }
fn bit_1_HL(cpu: &mut Cpu) { op_bit_HL(bit, 1, cpu); }

fn bit_2_A(cpu: &mut Cpu)  {  op_bit_A(bit, 2, cpu); }
fn bit_2_B(cpu: &mut Cpu)  {  op_bit_B(bit, 2, cpu); }
fn bit_2_C(cpu: &mut Cpu)  {  op_bit_C(bit, 2, cpu); }
fn bit_2_D(cpu: &mut Cpu)  {  op_bit_D(bit, 2, cpu); }
fn bit_2_E(cpu: &mut Cpu)  {  op_bit_E(bit, 2, cpu); }
fn bit_2_H(cpu: &mut Cpu)  {  op_bit_H(bit, 2, cpu); }
fn bit_2_L(cpu: &mut Cpu)  {  op_bit_L(bit, 2, cpu); }
fn bit_2_HL(cpu: &mut Cpu) { op_bit_HL(bit, 2, cpu); }

fn bit_3_A(cpu: &mut Cpu)  {  op_bit_A(bit, 3, cpu); }
fn bit_3_B(cpu: &mut Cpu)  {  op_bit_B(bit, 3, cpu); }
fn bit_3_C(cpu: &mut Cpu)  {  op_bit_C(bit, 3, cpu); }
fn bit_3_D(cpu: &mut Cpu)  {  op_bit_D(bit, 3, cpu); }
fn bit_3_E(cpu: &mut Cpu)  {  op_bit_E(bit, 3, cpu); }
fn bit_3_H(cpu: &mut Cpu)  {  op_bit_H(bit, 3, cpu); }
fn bit_3_L(cpu: &mut Cpu)  {  op_bit_L(bit, 3, cpu); }
fn bit_3_HL(cpu: &mut Cpu) { op_bit_HL(bit, 3, cpu); }

fn bit_4_A(cpu: &mut Cpu)  {  op_bit_A(bit, 4, cpu); }
fn bit_4_B(cpu: &mut Cpu)  {  op_bit_B(bit, 4, cpu); }
fn bit_4_C(cpu: &mut Cpu)  {  op_bit_C(bit, 4, cpu); }
fn bit_4_D(cpu: &mut Cpu)  {  op_bit_D(bit, 4, cpu); }
fn bit_4_E(cpu: &mut Cpu)  {  op_bit_E(bit, 4, cpu); }
fn bit_4_H(cpu: &mut Cpu)  {  op_bit_H(bit, 4, cpu); }
fn bit_4_L(cpu: &mut Cpu)  {  op_bit_L(bit, 4, cpu); }
fn bit_4_HL(cpu: &mut Cpu) { op_bit_HL(bit, 4, cpu); }

fn bit_5_A(cpu: &mut Cpu)  {  op_bit_A(bit, 5, cpu); }
fn bit_5_B(cpu: &mut Cpu)  {  op_bit_B(bit, 5, cpu); }
fn bit_5_C(cpu: &mut Cpu)  {  op_bit_C(bit, 5, cpu); }
fn bit_5_D(cpu: &mut Cpu)  {  op_bit_D(bit, 5, cpu); }
fn bit_5_E(cpu: &mut Cpu)  {  op_bit_E(bit, 5, cpu); }
fn bit_5_H(cpu: &mut Cpu)  {  op_bit_H(bit, 5, cpu); }
fn bit_5_L(cpu: &mut Cpu)  {  op_bit_L(bit, 5, cpu); }
fn bit_5_HL(cpu: &mut Cpu) { op_bit_HL(bit, 5, cpu); }

fn bit_6_A(cpu: &mut Cpu)  {  op_bit_A(bit, 6, cpu); }
fn bit_6_B(cpu: &mut Cpu)  {  op_bit_B(bit, 6, cpu); }
fn bit_6_C(cpu: &mut Cpu)  {  op_bit_C(bit, 6, cpu); }
fn bit_6_D(cpu: &mut Cpu)  {  op_bit_D(bit, 6, cpu); }
fn bit_6_E(cpu: &mut Cpu)  {  op_bit_E(bit, 6, cpu); }
fn bit_6_H(cpu: &mut Cpu)  {  op_bit_H(bit, 6, cpu); }
fn bit_6_L(cpu: &mut Cpu)  {  op_bit_L(bit, 6, cpu); }
fn bit_6_HL(cpu: &mut Cpu) { op_bit_HL(bit, 6, cpu); }

fn bit_7_A(cpu: &mut Cpu)  {  op_bit_A(bit, 7, cpu); }
fn bit_7_B(cpu: &mut Cpu)  {  op_bit_B(bit, 7, cpu); }
fn bit_7_C(cpu: &mut Cpu)  {  op_bit_C(bit, 7, cpu); }
fn bit_7_D(cpu: &mut Cpu)  {  op_bit_D(bit, 7, cpu); }
fn bit_7_E(cpu: &mut Cpu)  {  op_bit_E(bit, 7, cpu); }
fn bit_7_H(cpu: &mut Cpu)  {  op_bit_H(bit, 7, cpu); }
fn bit_7_L(cpu: &mut Cpu)  {  op_bit_L(bit, 7, cpu); }
fn bit_7_HL(cpu: &mut Cpu) { op_bit_HL(bit, 7, cpu); }

fn set(b: u8, x: u8, _: &mut Cpu) -> u8 {
    let mask = match b {
        0 => 0b00000001,
        1 => 0b00000010,
        2 => 0b00000100,
        3 => 0b00001000,
        4 => 0b00010000,
        5 => 0b00100000,
        6 => 0b01000000,
        7 => 0b10000000,
        _ => panic!()
    };

    mask | x
}

fn op_set_A(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_A_reg();
    let result = func(b, x, cpu);
    cpu.set_A_reg(result);
}

fn op_set_B(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_B_reg();
    let result = func(b, x, cpu);
    cpu.set_B_reg(result);
}

fn op_set_C(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_C_reg();
    let result = func(b, x, cpu);
    cpu.set_C_reg(result);
}

fn op_set_D(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_D_reg();
    let result = func(b, x, cpu);
    cpu.set_D_reg(result);
}

fn op_set_E(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_E_reg();
    let result = func(b, x, cpu);
    cpu.set_E_reg(result);
}

fn op_set_H(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_H_reg();
    let result = func(b, x, cpu);
    cpu.set_H_reg(result);
}

fn op_set_L(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = cpu.get_L_reg();
    let result = func(b, x, cpu);
    cpu.set_L_reg(result);
}

fn op_set_HL(func: fn(b: u8, x: u8, cpu: &mut Cpu) -> u8, b: u8, cpu: &mut Cpu) {
    let x = deref_HL(cpu);
    let result = func(b, x, cpu);
    set_deref_HL(cpu, result);
}

fn set_0_A(cpu: &mut Cpu)  {  op_set_A(set, 0, cpu); }
fn set_0_B(cpu: &mut Cpu)  {  op_set_B(set, 0, cpu); }
fn set_0_C(cpu: &mut Cpu)  {  op_set_C(set, 0, cpu); }
fn set_0_D(cpu: &mut Cpu)  {  op_set_D(set, 0, cpu); }
fn set_0_E(cpu: &mut Cpu)  {  op_set_E(set, 0, cpu); }
fn set_0_H(cpu: &mut Cpu)  {  op_set_H(set, 0, cpu); }
fn set_0_L(cpu: &mut Cpu)  {  op_set_L(set, 0, cpu); }
fn set_0_HL(cpu: &mut Cpu) { op_set_HL(set, 0, cpu); }

fn set_1_A(cpu: &mut Cpu)  {  op_set_A(set, 1, cpu); }
fn set_1_B(cpu: &mut Cpu)  {  op_set_B(set, 1, cpu); }
fn set_1_C(cpu: &mut Cpu)  {  op_set_C(set, 1, cpu); }
fn set_1_D(cpu: &mut Cpu)  {  op_set_D(set, 1, cpu); }
fn set_1_E(cpu: &mut Cpu)  {  op_set_E(set, 1, cpu); }
fn set_1_H(cpu: &mut Cpu)  {  op_set_H(set, 1, cpu); }
fn set_1_L(cpu: &mut Cpu)  {  op_set_L(set, 1, cpu); }
fn set_1_HL(cpu: &mut Cpu) { op_set_HL(set, 1, cpu); }

fn set_2_A(cpu: &mut Cpu)  {  op_set_A(set, 2, cpu); }
fn set_2_B(cpu: &mut Cpu)  {  op_set_B(set, 2, cpu); }
fn set_2_C(cpu: &mut Cpu)  {  op_set_C(set, 2, cpu); }
fn set_2_D(cpu: &mut Cpu)  {  op_set_D(set, 2, cpu); }
fn set_2_E(cpu: &mut Cpu)  {  op_set_E(set, 2, cpu); }
fn set_2_H(cpu: &mut Cpu)  {  op_set_H(set, 2, cpu); }
fn set_2_L(cpu: &mut Cpu)  {  op_set_L(set, 2, cpu); }
fn set_2_HL(cpu: &mut Cpu) { op_set_HL(set, 2, cpu); }

fn set_3_A(cpu: &mut Cpu)  {  op_set_A(set, 3, cpu); }
fn set_3_B(cpu: &mut Cpu)  {  op_set_B(set, 3, cpu); }
fn set_3_C(cpu: &mut Cpu)  {  op_set_C(set, 3, cpu); }
fn set_3_D(cpu: &mut Cpu)  {  op_set_D(set, 3, cpu); }
fn set_3_E(cpu: &mut Cpu)  {  op_set_E(set, 3, cpu); }
fn set_3_H(cpu: &mut Cpu)  {  op_set_H(set, 3, cpu); }
fn set_3_L(cpu: &mut Cpu)  {  op_set_L(set, 3, cpu); }
fn set_3_HL(cpu: &mut Cpu) { op_set_HL(set, 3, cpu); }

fn set_4_A(cpu: &mut Cpu)  {  op_set_A(set, 4, cpu); }
fn set_4_B(cpu: &mut Cpu)  {  op_set_B(set, 4, cpu); }
fn set_4_C(cpu: &mut Cpu)  {  op_set_C(set, 4, cpu); }
fn set_4_D(cpu: &mut Cpu)  {  op_set_D(set, 4, cpu); }
fn set_4_E(cpu: &mut Cpu)  {  op_set_E(set, 4, cpu); }
fn set_4_H(cpu: &mut Cpu)  {  op_set_H(set, 4, cpu); }
fn set_4_L(cpu: &mut Cpu)  {  op_set_L(set, 4, cpu); }
fn set_4_HL(cpu: &mut Cpu) { op_set_HL(set, 4, cpu); }

fn set_5_A(cpu: &mut Cpu)  {  op_set_A(set, 5, cpu); }
fn set_5_B(cpu: &mut Cpu)  {  op_set_B(set, 5, cpu); }
fn set_5_C(cpu: &mut Cpu)  {  op_set_C(set, 5, cpu); }
fn set_5_D(cpu: &mut Cpu)  {  op_set_D(set, 5, cpu); }
fn set_5_E(cpu: &mut Cpu)  {  op_set_E(set, 5, cpu); }
fn set_5_H(cpu: &mut Cpu)  {  op_set_H(set, 5, cpu); }
fn set_5_L(cpu: &mut Cpu)  {  op_set_L(set, 5, cpu); }
fn set_5_HL(cpu: &mut Cpu) { op_set_HL(set, 5, cpu); }

fn set_6_A(cpu: &mut Cpu)  {  op_set_A(set, 6, cpu); }
fn set_6_B(cpu: &mut Cpu)  {  op_set_B(set, 6, cpu); }
fn set_6_C(cpu: &mut Cpu)  {  op_set_C(set, 6, cpu); }
fn set_6_D(cpu: &mut Cpu)  {  op_set_D(set, 6, cpu); }
fn set_6_E(cpu: &mut Cpu)  {  op_set_E(set, 6, cpu); }
fn set_6_H(cpu: &mut Cpu)  {  op_set_H(set, 6, cpu); }
fn set_6_L(cpu: &mut Cpu)  {  op_set_L(set, 6, cpu); }
fn set_6_HL(cpu: &mut Cpu) { op_set_HL(set, 6, cpu); }

fn set_7_A(cpu: &mut Cpu)  {  op_set_A(set, 7, cpu); }
fn set_7_B(cpu: &mut Cpu)  {  op_set_B(set, 7, cpu); }
fn set_7_C(cpu: &mut Cpu)  {  op_set_C(set, 7, cpu); }
fn set_7_D(cpu: &mut Cpu)  {  op_set_D(set, 7, cpu); }
fn set_7_E(cpu: &mut Cpu)  {  op_set_E(set, 7, cpu); }
fn set_7_H(cpu: &mut Cpu)  {  op_set_H(set, 7, cpu); }
fn set_7_L(cpu: &mut Cpu)  {  op_set_L(set, 7, cpu); }
fn set_7_HL(cpu: &mut Cpu) { op_set_HL(set, 7, cpu); }

fn reset(b: u8, x: u8, _: &mut Cpu) -> u8 {
    let mask = match b {
        0 => 0b11111110,
        1 => 0b11111101,
        2 => 0b11111011,
        3 => 0b11110111,
        4 => 0b11101111,
        5 => 0b11011111,
        6 => 0b10111111,
        7 => 0b01111111,
        _ => panic!()
    };

    mask & x
}

fn reset_0_A(cpu: &mut Cpu)  {  op_set_A(reset, 0, cpu); }
fn reset_0_B(cpu: &mut Cpu)  {  op_set_B(reset, 0, cpu); }
fn reset_0_C(cpu: &mut Cpu)  {  op_set_C(reset, 0, cpu); }
fn reset_0_D(cpu: &mut Cpu)  {  op_set_D(reset, 0, cpu); }
fn reset_0_E(cpu: &mut Cpu)  {  op_set_E(reset, 0, cpu); }
fn reset_0_H(cpu: &mut Cpu)  {  op_set_H(reset, 0, cpu); }
fn reset_0_L(cpu: &mut Cpu)  {  op_set_L(reset, 0, cpu); }
fn reset_0_HL(cpu: &mut Cpu) { op_set_HL(reset, 0, cpu); }

fn reset_1_A(cpu: &mut Cpu)  {  op_set_A(reset, 1, cpu); }
fn reset_1_B(cpu: &mut Cpu)  {  op_set_B(reset, 1, cpu); }
fn reset_1_C(cpu: &mut Cpu)  {  op_set_C(reset, 1, cpu); }
fn reset_1_D(cpu: &mut Cpu)  {  op_set_D(reset, 1, cpu); }
fn reset_1_E(cpu: &mut Cpu)  {  op_set_E(reset, 1, cpu); }
fn reset_1_H(cpu: &mut Cpu)  {  op_set_H(reset, 1, cpu); }
fn reset_1_L(cpu: &mut Cpu)  {  op_set_L(reset, 1, cpu); }
fn reset_1_HL(cpu: &mut Cpu) { op_set_HL(reset, 1, cpu); }

fn reset_2_A(cpu: &mut Cpu)  {  op_set_A(reset, 2, cpu); }
fn reset_2_B(cpu: &mut Cpu)  {  op_set_B(reset, 2, cpu); }
fn reset_2_C(cpu: &mut Cpu)  {  op_set_C(reset, 2, cpu); }
fn reset_2_D(cpu: &mut Cpu)  {  op_set_D(reset, 2, cpu); }
fn reset_2_E(cpu: &mut Cpu)  {  op_set_E(reset, 2, cpu); }
fn reset_2_H(cpu: &mut Cpu)  {  op_set_H(reset, 2, cpu); }
fn reset_2_L(cpu: &mut Cpu)  {  op_set_L(reset, 2, cpu); }
fn reset_2_HL(cpu: &mut Cpu) { op_set_HL(reset, 2, cpu); }

fn reset_3_A(cpu: &mut Cpu)  {  op_set_A(reset, 3, cpu); }
fn reset_3_B(cpu: &mut Cpu)  {  op_set_B(reset, 3, cpu); }
fn reset_3_C(cpu: &mut Cpu)  {  op_set_C(reset, 3, cpu); }
fn reset_3_D(cpu: &mut Cpu)  {  op_set_D(reset, 3, cpu); }
fn reset_3_E(cpu: &mut Cpu)  {  op_set_E(reset, 3, cpu); }
fn reset_3_H(cpu: &mut Cpu)  {  op_set_H(reset, 3, cpu); }
fn reset_3_L(cpu: &mut Cpu)  {  op_set_L(reset, 3, cpu); }
fn reset_3_HL(cpu: &mut Cpu) { op_set_HL(reset, 3, cpu); }

fn reset_4_A(cpu: &mut Cpu)  {  op_set_A(reset, 4, cpu); }
fn reset_4_B(cpu: &mut Cpu)  {  op_set_B(reset, 4, cpu); }
fn reset_4_C(cpu: &mut Cpu)  {  op_set_C(reset, 4, cpu); }
fn reset_4_D(cpu: &mut Cpu)  {  op_set_D(reset, 4, cpu); }
fn reset_4_E(cpu: &mut Cpu)  {  op_set_E(reset, 4, cpu); }
fn reset_4_H(cpu: &mut Cpu)  {  op_set_H(reset, 4, cpu); }
fn reset_4_L(cpu: &mut Cpu)  {  op_set_L(reset, 4, cpu); }
fn reset_4_HL(cpu: &mut Cpu) { op_set_HL(reset, 4, cpu); }

fn reset_5_A(cpu: &mut Cpu)  {  op_set_A(reset, 5, cpu); }
fn reset_5_B(cpu: &mut Cpu)  {  op_set_B(reset, 5, cpu); }
fn reset_5_C(cpu: &mut Cpu)  {  op_set_C(reset, 5, cpu); }
fn reset_5_D(cpu: &mut Cpu)  {  op_set_D(reset, 5, cpu); }
fn reset_5_E(cpu: &mut Cpu)  {  op_set_E(reset, 5, cpu); }
fn reset_5_H(cpu: &mut Cpu)  {  op_set_H(reset, 5, cpu); }
fn reset_5_L(cpu: &mut Cpu)  {  op_set_L(reset, 5, cpu); }
fn reset_5_HL(cpu: &mut Cpu) { op_set_HL(reset, 5, cpu); }

fn reset_6_A(cpu: &mut Cpu)  {  op_set_A(reset, 6, cpu); }
fn reset_6_B(cpu: &mut Cpu)  {  op_set_B(reset, 6, cpu); }
fn reset_6_C(cpu: &mut Cpu)  {  op_set_C(reset, 6, cpu); }
fn reset_6_D(cpu: &mut Cpu)  {  op_set_D(reset, 6, cpu); }
fn reset_6_E(cpu: &mut Cpu)  {  op_set_E(reset, 6, cpu); }
fn reset_6_H(cpu: &mut Cpu)  {  op_set_H(reset, 6, cpu); }
fn reset_6_L(cpu: &mut Cpu)  {  op_set_L(reset, 6, cpu); }
fn reset_6_HL(cpu: &mut Cpu) { op_set_HL(reset, 6, cpu); }

fn reset_7_A(cpu: &mut Cpu)  {  op_set_A(reset, 7, cpu); }
fn reset_7_B(cpu: &mut Cpu)  {  op_set_B(reset, 7, cpu); }
fn reset_7_C(cpu: &mut Cpu)  {  op_set_C(reset, 7, cpu); }
fn reset_7_D(cpu: &mut Cpu)  {  op_set_D(reset, 7, cpu); }
fn reset_7_E(cpu: &mut Cpu)  {  op_set_E(reset, 7, cpu); }
fn reset_7_H(cpu: &mut Cpu)  {  op_set_H(reset, 7, cpu); }
fn reset_7_L(cpu: &mut Cpu)  {  op_set_L(reset, 7, cpu); }
fn reset_7_HL(cpu: &mut Cpu) { op_set_HL(reset, 7, cpu); }

op_codes!(
    // ===========================
    // Unprefixed op codes
    // ===========================

    // LD nn,n
    //
    // Put value n into nn
    //
    // nn = B,C,D,E,H,K,BC,DE,HL,SP
    // n = 8 bit immediate value
    LdBn: ("LD B,n", 0x06, ld_B_n),
    LdCn: ("LD C,n", 0x0E, ld_C_n),
    LdDn: ("LD D,n", 0x16, ld_D_n),
    LdEn: ("LD E,n", 0x1E, ld_E_n),
    LdHn: ("LD H,n", 0x26, ld_H_n),
    LdLn: ("LD L,n", 0x2E, ld_L_n),

    // LD r1,r2
    //
    // Put value r2 into r1
    //
    // r1,r2 = A,B,C,D,E,H,L,(HL)
    LdAA:  ("LD A,A",    0x7F, no_op),
    LdAB:  ("LD A,B",    0x78, ld_A_B),
    LdAC:  ("LD A,C",    0x79, ld_A_C),
    LdAD:  ("LD A,D",    0x7A, ld_A_D),
    LdAE:  ("LD A,E",    0x7B, ld_A_E),
    LdAH:  ("LD A,H",    0x7C, ld_A_H),
    LdAL:  ("LD A,L",    0x7D, ld_A_L),
    LdAHL: ("LD A,(HL)", 0x7E, ld_A_HL),

    LdBA:  ("LD B,A",    0x47, ld_B_A),
    LdBB:  ("LD B,B",    0x40, no_op),
    LdBC:  ("LD B,C",    0x41, ld_B_C),
    LdBD:  ("LD B,D",    0x42, ld_B_D),
    LdBE:  ("LD B,E",    0x43, ld_B_E),
    LdBH:  ("LD B,H",    0x44, ld_B_H),
    LdBL:  ("LD B,L",    0x45, ld_B_L),
    LdBHL: ("LD B,(HL)", 0x46, ld_B_HL),

    LdCA:  ("LD C,A",    0x4F, ld_C_A),
    LdCB:  ("LD C,B",    0x48, ld_C_B),
    LdCC:  ("LD C,C",    0x49, no_op),
    LdCD:  ("LD C,D",    0x4A, ld_C_D),
    LdCE:  ("LD C,E",    0x4B, ld_C_E),
    LdCH:  ("LD C,H",    0x4C, ld_C_H),
    LdCL:  ("LD C,L",    0x4D, ld_C_L),
    LdCHL: ("LD C,(HL)", 0x4E, ld_C_HL),

    LdDA:  ("LD D,A",    0x57, ld_D_A),
    LdDB:  ("LD D,B",    0x50, ld_D_B),
    LdDC:  ("LD D,C",    0x51, ld_D_C),
    LdDD:  ("LD D,D",    0x52, no_op),
    LdDE:  ("LD D,E",    0x53, ld_D_E),
    LdDH:  ("LD D,H",    0x54, ld_D_H),
    LdDL:  ("LD D,L",    0x55, ld_D_L),
    LdDHL: ("LD D,(HL)", 0x56, ld_D_HL),

    LdEA:  ("LD E,A",    0x5F, ld_E_A),
    LdEB:  ("LD E,B",    0x58, ld_E_B),
    LdEC:  ("LD E,C",    0x59, ld_E_C),
    LdED:  ("LD E,D",    0x5A, ld_E_D),
    LdEE:  ("LD E,E",    0x5B, no_op),
    LdEH:  ("LD E,H",    0x5C, ld_E_H),
    LdEL:  ("LD E,L",    0x5D, ld_E_L),
    LdEHL: ("LD E,(HL)", 0x5E, ld_E_HL),

    LdHA:  ("LD H,A",    0x67, ld_H_A),
    LdHB:  ("LD H,B",    0x60, ld_H_B),
    LdHC:  ("LD H,C",    0x61, ld_H_C),
    LdHD:  ("LD H,D",    0x62, ld_H_D),
    LdHE:  ("LD H,E",    0x63, ld_H_E),
    LdHH:  ("LD H,H",    0x64, no_op),
    LdHL:  ("LD H,L",    0x65, ld_H_L),
    LdHHL: ("LD H,(HL)", 0x66, ld_H_HL),

    LdLA:  ("LD L,A",    0x6F, ld_L_A),
    LdLB:  ("LD L,B",    0x68, ld_L_B),
    LdLC:  ("LD L,C",    0x69, ld_L_C),
    LdLD:  ("LD L,D",    0x6A, ld_L_D),
    LdLE:  ("LD L,E",    0x6B, ld_L_E),
    LdLH:  ("LD L,H",    0x6C, ld_L_H),
    LdLL:  ("LD L,L",    0x6D, no_op),
    LdLHL: ("LD L,(HL)", 0x6E, ld_L_HL),

    LdHLB: ("LD (HL),B", 0x70, ld_HL_B),
    LdHLC: ("LD (HL),C", 0x71, ld_HL_C),
    LdHLD: ("LD (HL),D", 0x72, ld_HL_D),
    LdHLE: ("LD (HL),E", 0x73, ld_HL_E),
    LdHLH: ("LD (HL),H", 0x74, ld_HL_H),
    LdHLL: ("LD (HL),L", 0x75, ld_HL_L),
    LdHLn: ("LD (HL),n", 0x36, ld_HL_n),

    // LD A,n
    //
    // Put value n into A
    //
    // n = A,B,C,D,E,H,L,(BC),(DE),(HL),(nn),n
    // nn = two byte immediate value. (LS byte first.)
    LdABC: ("LD A,(BC)", 0x0A, ld_A_BC),
    LdADE: ("LD A,(DE)", 0x1A, ld_A_DE),
    LdAnn: ("LD A,(nn)", 0xFA, ld_A_nn),
    LdAx:  ("LD A,n",    0x3E, ld_A_x),

    // LD n,A
    //
    // Put value A into n
    //
    // n = A,B,C,D,E,H,L,(BC),(DE),(HL),(nn)
    // nn = two byte immediate value. (LS byte first)
    LdBCA: ("LD (BC),A", 0x02, ld_BC_A),
    LdDEA: ("LD (DE),A", 0x12, ld_DE_A),
    LdHLA: ("LD (HL),A", 0x77, ld_HL_A),
    LdnnA: ("LD (nn),A", 0xEA, ld_nn_A),

    // LD A,(C)
    //
    // Put value at address $FF00 + register C into A
    // Same as: LD A, ($FF00+C)
    LdAFFC: ("LD A,($FF00+C)", 0xF2, ld_A_FFC),

    // LD (C),A
    //
    // Put A into address $FF00 + register C
    LdFFCA: ("LD ($FF00+C),A", 0xE2, ld_FFC_A),

    // LDD A,(HL)
    //
    // Put value at address HL into A. Decrement HL.
    // Same as: LD A,(HL) - DEC HL
    LddAHL: ("LDD A,(HL)", 0x3A, ldd_A_HL),

    // LDD (HL),A
    //
    // Put A into memoty address HL. Decrement HL.
    // Same as: LD (HL),A - DEC HL
    LddHLA: ("LDD (HL),A", 0x32, ldd_HL_A),

    // LDI A,(HL)
    //
    // Put value at address HL into A. Increment HL.
    // Same as: LD A,(HL) - INC HL
    LdiAHL: ("LDI A,(HL)", 0x2A, ldi_A_HL),

    // LDI (HL),A
    //
    // Put A into memory address HL. Increment HL.
    // Same as: LD (HL),A - INC HL
    LdiHLA: ("LDI (HL),A", 0x22, ldi_HL_A),

    // LDH (n),A
    //
    // Put A into memory address $FF00+n
    //
    // n = one byte immediate value
    LdhnA: ("LDH ($FF00+n),A", 0xE0, ldh_n_A),

    // LDH A,(n)
    //
    // Put memory address $FF00+n into A.
    //
    // n = one byte immediate value
    LdhAn: ("LDH A,($FF00+n)", 0xF0, ldh_A_n),

    // LD n,nn
    //
    // Put value nn into n
    //
    // n = BC,DE,HL,SP
    // nn = 16 bit immediate value
    LdBCnn: ("LD BC,nn", 0x01, ld_BC_nn),
    LdDEnn: ("LD DE,nn", 0x11, ld_DE_nn),
    LdHLnn: ("LD HL,nn", 0x21, ld_HL_nn),
    LdSPnn: ("LD SP,nn", 0x31, ld_SP_nn),

    // LD SP,HL
    //
    // Put HL into Stack Pointer (SP)
    LdSPHL: ("LD SP,HL", 0xF9, ld_SP_HL),

    // LDHL SP,n
    //
    // Put SP + n effective address into HL
    //
    // n = one byte signed immediate value
    //
    // Flags:
    // Z - Reset
    // N - Reset
    // H - Set or reset according to operation
    // C - Set or reset according to operation
    LdhlSPn: ("LDHL SP,n", 0xF8, ldhl_SP_n),

    // LD (nn),SP
    //
    // Put Stack Pointer (SP) at address nn
    //
    // nn = two byte immediate address.
    LdnnSP: ("LD (nn),SP", 0x08, ld_nn_SP),

    // PUSH nn
    //
    // Push register pair nn onto stack.
    // Decrement Stack Pointer (SP) twice.
    //
    // nn = AF,BC,DE,HL
    PushAF: ("PUSH AF", 0xF5, push_AF),
    PushBC: ("PUSH BC", 0xC5, push_BC),
    PushDE: ("PUSH DE", 0xD5, push_DE),
    PushHL: ("PUSH HL", 0xE5, push_HL),

    // POP nn
    //
    // Pop two bytes off stack into register pair nn.
    // Increment Stack Pointer (SP) twice.
    //
    // nn = AF,BC,DE,HL
    PopAF: ("POP AF", 0xF1, pop_AF),
    PopBC: ("POP BC", 0xC1, pop_BC),
    PopDE: ("POP DE", 0xD1, pop_DE),
    PopHL: ("POP HL", 0xE1, pop_HL),

    // ADD A,n
    //
    // Add n to A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set if carry from bit 3
    // C - Set if carry from bit 7
    AddAA:  ("ADD A,A",    0x87, add_A_A),
    AddAB:  ("ADD A,B",    0x80, add_A_B),
    AddAC:  ("ADD A,C",    0x81, add_A_C),
    AddAD:  ("ADD A,D",    0x82, add_A_D),
    AddAE:  ("ADD A,E",    0x83, add_A_E),
    AddAH:  ("ADD A,H",    0x84, add_A_H),
    AddAL:  ("ADD A,L",    0x85, add_A_L),
    AddAHL: ("ADD A,(HL)", 0x86, add_A_HL),
    AddAx:  ("ADD A,n",    0xC6, add_A_x),

    // ADC A,n
    //
    // Add n + Carry flag to A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set if carry from bit 3
    // C - Set if carry from bit 7
    AdcAA:  ("ADC A,A",    0x8F, adc_A_A),
    AdcAB:  ("ADC A,B",    0x88, adc_A_B),
    AdcAC:  ("ADC A,C",    0x89, adc_A_C),
    AdcAD:  ("ADC A,D",    0x8A, adc_A_D),
    AdcAE:  ("ADC A,E",    0x8B, adc_A_E),
    AdcAH:  ("ADC A,H",    0x8C, adc_A_H),
    AdcAL:  ("ADC A,L",    0x8D, adc_A_L),
    AdcAHL: ("ADC A,(HL)", 0x8E, adc_A_HL),
    AdcAx:  ("ADC A,n",    0xCE, adc_A_x),

    // SUB n
    //
    // Subtract n from A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set if no borrow
    SubAA:  ("SUB A,A",    0x97, sub_A_A),
    SubAB:  ("SUB A,B",    0x90, sub_A_B),
    SubAC:  ("SUB A,C",    0x91, sub_A_C),
    SubAD:  ("SUB A,D",    0x92, sub_A_D),
    SubAE:  ("SUB A,E",    0x93, sub_A_E),
    SubAH:  ("SUB A,H",    0x94, sub_A_H),
    SubAL:  ("SUB A,L",    0x95, sub_A_L),
    SubAHL: ("SUB A,(HL)", 0x96, sub_A_HL),
    SubAx:  ("SUB A,n",    0xD6, sub_A_x),

    // SBC A,n
    //
    // Subtract n + Carry flag from A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set if no borrow
    SbcAA:  ("SBC A,A",    0x9F, sbc_A_A),
    SbcAB:  ("SBC A,B",    0x98, sbc_A_B),
    SbcAC:  ("SBC A,C",    0x99, sbc_A_C),
    SbcAD:  ("SBC A,D",    0x9A, sbc_A_D),
    SbcAE:  ("SBC A,E",    0x9B, sbc_A_E),
    SbcAH:  ("SBC A,H",    0x9C, sbc_A_H),
    SbcAL:  ("SBC A,L",    0x9D, sbc_A_L),
    SbcAHL: ("SBC A,(HL)", 0x9E, sbc_A_HL),
    SbcAx:  ("SBC A,n",    0xDE, sbc_A_x),

    // AND n
    //
    // Logically AND n with A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set
    // C - Reset
    AndAA:  ("AND A,A",    0xA7, and_A_A),
    AndAB:  ("AND A,B",    0xA0, and_A_B),
    AndAC:  ("AND A,C",    0xA1, and_A_C),
    AndAD:  ("AND A,D",    0xA2, and_A_D),
    AndAE:  ("AND A,E",    0xA3, and_A_E),
    AndAH:  ("AND A,H",    0xA4, and_A_H),
    AndAL:  ("AND A,L",    0xA5, and_A_L),
    AndAHL: ("AND A,(HL)", 0xA6, and_A_HL),
    AndAx:  ("AND A,n",    0xE6, and_A_x),

    // OR n
    //
    // Logical OR n with register A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    OrAA:  ("OR A,A",    0xB7, or_A_A),
    OrAB:  ("OR A,B",    0xB0, or_A_B),
    OrAC:  ("OR A,C",    0xB1, or_A_C),
    OrAD:  ("OR A,D",    0xB2, or_A_D),
    OrAE:  ("OR A,E",    0xB3, or_A_E),
    OrAH:  ("OR A,H",    0xB4, or_A_H),
    OrAL:  ("OR A,L",    0xB5, or_A_L),
    OrAHL: ("OR A,(HL)", 0xB6, or_A_HL),
    OrAx:  ("OR A,n",    0xF6, or_A_x),

    // XOR n
    //
    // Logical exclusive OR n with register A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    XorAA:  ("XOR A,A",    0xAF, xor_A_A),
    XorAB:  ("XOR A,B",    0xA8, xor_A_B),
    XorAC:  ("XOR A,C",    0xA9, xor_A_C),
    XorAD:  ("XOR A,D",    0xAA, xor_A_D),
    XorAE:  ("XOR A,E",    0xAB, xor_A_E),
    XorAH:  ("XOR A,H",    0xAC, xor_A_H),
    XorAL:  ("XOR A,L",    0xAD, xor_A_L),
    XorAHL: ("XOR A,(HL)", 0xAE, xor_A_HL),
    XorAx:  ("XOR A,n",    0xEE, xor_A_x),

    // CP n
    //
    // Compare A with n. This is basically an A - n subtraction
    // instruction but the results are thrown away
    //
    // n = A,B,C,D,E,H,L,(HL),n
    //
    // Flags:
    // Z - Set if result is zero. (Set if A = n)
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set for no borrow. (Set if A < n)
    CpAA:  ("CP A,A",    0xBF, cp_A_A),
    CpAB:  ("CP A,B",    0xB8, cp_A_B),
    CpAC:  ("CP A,C",    0xB9, cp_A_C),
    CpAD:  ("CP A,D",    0xBA, cp_A_D),
    CpAE:  ("CP A,E",    0xBB, cp_A_E),
    CpAH:  ("CP A,H",    0xBC, cp_A_H),
    CpAL:  ("CP A,L",    0xBD, cp_A_L),
    CpAHL: ("CP A,(HL)", 0xBE, cp_A_HL),
    CpAx:  ("CP A,n",    0xFE, cp_A_x),

    // INC n
    //
    // Increment register n
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Set if carry from bit 3
    // C - Not affected
    IncA:   ("INC A",    0x3C, inc_A),
    IncB:   ("INC B",    0x04, inc_B),
    IncC:   ("INC C",    0x0C, inc_C),
    IncD:   ("INC D",    0x14, inc_D),
    IncE:   ("INC E",    0x1C, inc_E),
    IncH:   ("INC H",    0x24, inc_H),
    IncL:   ("INC L",    0x2C, inc_L),
    IncHLp: ("INC (HL)", 0x34, inc_HLp),

    // DEC n
    //
    // Decrement register n
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Not affected
    DecA:   ("DEC A",    0x3D, dec_A),
    DecB:   ("DEC B",    0x05, dec_B),
    DecC:   ("DEC C",    0x0D, dec_C),
    DecD:   ("DEC D",    0x15, dec_D),
    DecE:   ("DEC E",    0x1D, dec_E),
    DecH:   ("DEC H",    0x25, dec_H),
    DecL:   ("DEC L",    0x2D, dec_L),
    DecHLp: ("DEC (HL)", 0x35, dec_HLp),

    // ADD HL,n
    //
    // Add n to HL
    //
    // n = BC,DE,HL,SP
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Set if carry from bit 11
    // C - Set if carry from bit 15
    AddHLBC: ("ADD HL,BC", 0x09, add_HL_BC),
    AddHLDE: ("ADD HL,DE", 0x19, add_HL_DE),
    AddHLHL: ("ADD HL,HL", 0x29, add_HL_HL),
    AddHLSP: ("ADD HL,SP", 0x39, add_HL_SP),

    // ADD SP,n
    //
    // Add n to Stack Pointer (SP)
    //
    // n = one byte signed immediate value (n)
    //
    // Flags:
    // Z - Reset
    // N - Reset
    // H - Set or reset according to operation
    // C - Set or reset according to operation
    AddSPx: ("ADD SP,n", 0xE8, add_SP_x),

    // INC nn
    //
    // Increment register nn
    //
    // nn = BC,DE,HL,SP
    IncBC: ("INC BC", 0x03, inc_BC),
    IncDE: ("INC DE", 0x13, inc_DE),
    IncHL: ("INC HL", 0x23, inc_HL),
    IncSP: ("INC SP", 0x33, inc_SP),

    // DEC nn
    //
    // Decrement register nn
    //
    // nn = BC,DE,HL,SP
    DecBC: ("DEC BC", 0x0B, dec_BC),
    DecDE: ("DEC DE", 0x1B, dec_DE),
    DecHL: ("DEC HL", 0x2B, dec_HL),
    DecSP: ("DEC SP", 0x3B, dec_SP),

    // DAA
    //
    // Decimal adjust register A
    // This instruction adjusts register A so that the
    // correct representation of Binary Coded Decimal (BCD)
    // is obtained
    //
    // Flags:
    // Z - Set if register A is zero
    // N - Not affected
    // H - Reset
    // C - Set or reset according to operation
    Daa: ("DAA", 0x27, daa_A),

    // CPL
    //
    // Complement A register. (Flip all bits).
    //
    // Flags:
    // Z - Not affected
    // N - Set
    // H - Set
    // C - Not affected
    Cpl: ("CPL", 0x2F, cpl_A),

    // CCF
    //
    // Complement carry flag.
    // If C flag is set, then reset it.
    // If C flag is reset, then set it.
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Reset
    // C - Complemented
    Ccf: ("CCF", 0x3F, ccf),

    // SCF
    //
    // Set carry flag.
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Reset
    // C - Set
    Scf: ("SCF", 0x37, scf),

    // NOP
    //
    // No operation
    Nop: ("NOP", 0x00, no_op),

    // HALT
    //
    // Power down CPU until an interrupt occurs. Use this
    // when ever possible to reduce energy consumption
    Halt: ("HALT", 0x76, halt),

    // STOP
    //
    // Halt CPU & LCD display until button pressed.
    Stop: ("STOP", 0x10, stop),

    // DI
    //
    // This instruction disables interrupts but not
    // immediately. Interrupts are disabled after
    // the instruction after DI is executed.
    Di: ("DI", 0xF3, di),

    // EI
    //
    // Enable interrupts. This instruction enables interrupts
    // but not immediately. Interrupts are enabled after
    // the instruction after EI is executed.
    Ei: ("EI", 0xFB, ei),

    // RLCA
    //
    // Rotate A left. Old bit 7 to Carry flag.
    //
    // Flags:
    // Z - Set if result is Zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data.
    Rlca: ("RLCA", 0x07, rlca),

    // RLA
    //
    // Rotate A left thorugh Carry flag.
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data
    Rla: ("RLA", 0x17, rla),

    // RRCA
    //
    // Rotate A right. Old bit 0 to Carry flag.
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    Rrca: ("RRCA", 0x0F, rrca),

    // RRA
    //
    // Rotate A right through Carry flag.
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    Rra: ("RRA", 0x1F, rra),

    // JP nn
    //
    // Jump to address nn
    //
    // nn = two bytes immediate value. (LS byte first)
    Jpnn: ("JP", 0xC3, jp_nn),

    // JP cc,nn
    //
    // Jump to address n if following condition is true:
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = Z,  Jump if C flag is set
    //
    // nn = two byte immediate value. (LS byte first)
    JpNZnn: ("JP NZ,nn", 0xC2, jp_NZ_nn),
    JpZnn:  ("JP Z,nn",  0xCA, jp_Z_nn),
    JpNCnn: ("JP NC,nn", 0xD2, jp_NC_nn),
    JpCnn:  ("JP C,nn",  0xDA, jp_C_nn),

    // JP (HL)
    //
    // Jump to address contained in HL.
    JpHL: ("JP (HL)", 0xE9, jp_HL),

    // JR n
    //
    // Add n to current address and jump to it
    //
    // n = one byte signed immediate value
    Jrn: ("JR n", 0x18, jr_n),

    // JR cc,n
    //
    // If following condition is true then add n to current
    // address and jump to it
    //
    // n = one byte signed immediate value
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    JrNZn: ("JR NZ,n", 0x20, jr_NZ_n),
    JrZn:  ("JR Z,n",  0x28, jr_Z_n),
    JrNCn: ("JR NC,n", 0x30, jr_NC_n),
    JrCn:  ("JR C,n",  0x38, jr_C_n),

    // CALL nn
    //
    // Push address of next instruction onto stack and then
    // jump to address nn
    //
    // nn = two byte immediate value. (LS byte first.)
    Callnn: ("CALL nn", 0xCD, call_nn),

    // CALL cc,nn
    //
    // Call address n if following condition is true:
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    //
    // nn = two byte immediate value. (LS byte first.)
    CallNZnn: ("CALL NZ,nn", 0xC4, call_NZ_nn),
    CallZnn:  ("CALL Z,nn",  0xCC, call_Z_nn),
    CallNCnn: ("CALL NC,nn", 0xD4, call_NC_nn),
    CallCnn:  ("CALL C,nn",  0xDC, call_C_nn),

    // RST n
    //
    // Push present address onto stack
    // Jump to address $0000 + n
    //
    // n = $00, $08, $10, $18, $20, $28, $30, $38
    Rst00: ("RST 00H", 0xC7, rst_00),
    Rst08: ("RST 08H", 0xCF, rst_08),
    Rst10: ("RST 10H", 0xD7, rst_10),
    Rst18: ("RST 18H", 0xDF, rst_18),
    Rst20: ("RST 20H", 0xE7, rst_20),
    Rst28: ("RST 28H", 0xEF, rst_28),
    Rst30: ("RST 30H", 0xF7, rst_30),
    Rst38: ("RST 38H", 0xFF, rst_38),

    // RET
    //
    // Pop two bytes from stack and jump to that address
    Ret: ("RET", 0xC9, ret),

    // RET cc
    //
    // Return if following condition is true
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    RetNZ: ("RET NZ", 0xC0, ret_NZ),
    RetZ:  ("RET Z",  0xC8, ret_Z),
    RetNC: ("RET NC", 0xD0, ret_NC),
    RetC:  ("RET C",  0xD8, ret_C),

    // RETI
    //
    // Pop two bytes from stack and jump to that address then
    // enable interrupts
    Reti: ("RETI", 0xD9, reti);

    // ===========================
    // Op codes prefixed with 0xCB
    // ===========================

    // SWAP n
    //
    // Swap upper & lower nibbles of n
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    SwapA:  ("SWAP A",    0x37, swap_A),
    SwapB:  ("SWAP B",    0x30, swap_B),
    SwapC:  ("SWAP C",    0x31, swap_C),
    SwapD:  ("SWAP D",    0x32, swap_D),
    SwapE:  ("SWAP E",    0x33, swap_E),
    SwapH:  ("SWAP H",    0x34, swap_H),
    SwapL:  ("SWAP L",    0x35, swap_L),
    SwapHL: ("SWAP (HL)", 0x36, swap_HL),

    // RLC n
    //
    // Rotate n left. Old bit 7 to Carry flag.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data.
    RlcA:  ("RLC A",    0x07, rlc_A),
    RlcB:  ("RLC B",    0x00, rlc_B),
    RlcC:  ("RLC C",    0x01, rlc_C),
    RlcD:  ("RLC D",    0x02, rlc_D),
    RlcE:  ("RLC E",    0x03, rlc_E),
    RlcH:  ("RLC H",    0x04, rlc_H),
    RlcL:  ("RLC L",    0x05, rlc_L),
    RlcHL: ("RLC (HL)", 0x06, rlc_HL),

    // RL n
    //
    // Rotate n left through Carry flag.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data
    RlA:  ("RL A",    0x17, rl_A),
    RlB:  ("RL B",    0x10, rl_B),
    RlC:  ("RL C",    0x11, rl_C),
    RlD:  ("RL D",    0x12, rl_D),
    RlE:  ("RL E",    0x13, rl_E),
    RlH:  ("RL H",    0x14, rl_H),
    RlL:  ("RL L",    0x15, rl_L),
    RlHL: ("RL (HL)", 0x16, rl_HL),

    // RRC n
    //
    // Rotate n right. Old bit 0 to Carry flag.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data.
    RrcA:  ("RRC A",    0x0F, rrc_A),
    RrcB:  ("RRC B",    0x08, rrc_B),
    RrcC:  ("RRC C",    0x09, rrc_C),
    RrcD:  ("RRC D",    0x0A, rrc_D),
    RrcE:  ("RRC E",    0x0B, rrc_E),
    RrcH:  ("RRC H",    0x0C, rrc_H),
    RrcL:  ("RRC L",    0x0D, rrc_L),
    RrcHL: ("RRC (HL)", 0x0E, rrc_HL),

    // RR n
    //
    // Rotate n right through Carry flag.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    RrA:  ("RR A",    0x1F, rr_A),
    RrB:  ("RR B",    0x18, rr_B),
    RrC:  ("RR C",    0x19, rr_C),
    RrD:  ("RR D",    0x1A, rr_D),
    RrE:  ("RR E",    0x1B, rr_E),
    RrH:  ("RR H",    0x1C, rr_H),
    RrL:  ("RR L",    0x1D, rr_L),
    RrHL: ("RR (HL)", 0x1E, rr_HL),

    // SLA n
    //
    // Shift n left into Carry. LSB of n set to 0.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data
    SlaA:  ("SLA A",    0x27, sla_A),
    SlaB:  ("SLA B",    0x20, sla_B),
    SlaC:  ("SLA C",    0x21, sla_C),
    SlaD:  ("SLA D",    0x22, sla_D),
    SlaE:  ("SLA E",    0x23, sla_E),
    SlaH:  ("SLA H",    0x24, sla_H),
    SlaL:  ("SLA L",    0x25, sla_L),
    SlaHL: ("SLA (HL)", 0x26, sla_HL),

    // SRA n
    //
    // Shift n right into Carry. MSB doesn't change.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    SraA:  ("SRA A",    0x2F, sra_A),
    SraB:  ("SRA B",    0x28, sra_B),
    SraC:  ("SRA C",    0x29, sra_C),
    SraD:  ("SRA D",    0x2A, sra_D),
    SraE:  ("SRA E",    0x2B, sra_E),
    SraH:  ("SRA H",    0x2C, sra_H),
    SraL:  ("SRA L",    0x2D, sra_L),
    SraHL: ("SRA (HL)", 0x2E, sra_HL),

    // SRL n
    //
    // Shift n right into Carry. MSB set to 0.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    SrlA:  ("SRL A",    0x3F, srl_A),
    SrlB:  ("SRL B",    0x38, srl_B),
    SrlC:  ("SRL C",    0x39, srl_C),
    SrlD:  ("SRL D",    0x3A, srl_D),
    SrlE:  ("SRL E",    0x3B, srl_E),
    SrlH:  ("SRL H",    0x3C, srl_H),
    SrlL:  ("SRL L",    0x3D, srl_L),
    SrlHL: ("SRL (HL)", 0x3E, srl_HL),

    // BIT b,r
    //
    // Test bit b in register r
    //
    // b = 0 - 7
    // r = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if bit b of register r is 0
    // N - Reset
    // H - Set
    // C - Not affected
    Bit0A:  ("BIT 0,A",    0x47, bit_0_A),
    Bit0B:  ("BIT 0,B",    0x40, bit_0_B),
    Bit0C:  ("BIT 0,C",    0x41, bit_0_C),
    Bit0D:  ("BIT 0,D",    0x42, bit_0_D),
    Bit0E:  ("BIT 0,E",    0x43, bit_0_E),
    Bit0H:  ("BIT 0,H",    0x44, bit_0_H),
    Bit0L:  ("BIT 0,L",    0x45, bit_0_L),
    Bit0HL: ("BIT 0,(HL)", 0x46, bit_0_HL),

    Bit1A:  ("BIT 1,A",    0x4F, bit_1_A),
    Bit1B:  ("BIT 1,B",    0x48, bit_1_B),
    Bit1C:  ("BIT 1,C",    0x49, bit_1_C),
    Bit1D:  ("BIT 1,D",    0x4A, bit_1_D),
    Bit1E:  ("BIT 1,E",    0x4B, bit_1_E),
    Bit1H:  ("BIT 1,H",    0x4C, bit_1_H),
    Bit1L:  ("BIT 1,L",    0x4D, bit_1_L),
    Bit1HL: ("BIT 1,(HL)", 0x4E, bit_1_HL),

    Bit2A:  ("BIT 2,A",    0x57, bit_2_A),
    Bit2B:  ("BIT 2,B",    0x50, bit_2_B),
    Bit2C:  ("BIT 2,C",    0x51, bit_2_C),
    Bit2D:  ("BIT 2,D",    0x52, bit_2_D),
    Bit2E:  ("BIT 2,E",    0x53, bit_2_E),
    Bit2H:  ("BIT 2,H",    0x54, bit_2_H),
    Bit2L:  ("BIT 2,L",    0x55, bit_2_L),
    Bit2HL: ("BIT 2,(HL)", 0x56, bit_2_HL),

    Bit3A:  ("BIT 3,A",    0x5F, bit_3_A),
    Bit3B:  ("BIT 3,B",    0x58, bit_3_B),
    Bit3C:  ("BIT 3,C",    0x59, bit_3_C),
    Bit3D:  ("BIT 3,D",    0x5A, bit_3_D),
    Bit3E:  ("BIT 3,E",    0x5B, bit_3_E),
    Bit3H:  ("BIT 3,H",    0x5C, bit_3_H),
    Bit3L:  ("BIT 3,L",    0x5D, bit_3_L),
    Bit3HL: ("BIT 3,(HL)", 0x5E, bit_3_HL),

    Bit4A:  ("BIT 4,A",    0x67, bit_4_A),
    Bit4B:  ("BIT 4,B",    0x60, bit_4_B),
    Bit4C:  ("BIT 4,C",    0x61, bit_4_C),
    Bit4D:  ("BIT 4,D",    0x62, bit_4_D),
    Bit4E:  ("BIT 4,E",    0x63, bit_4_E),
    Bit4H:  ("BIT 4,H",    0x64, bit_4_H),
    Bit4L:  ("BIT 4,L",    0x65, bit_4_L),
    Bit4HL: ("BIT 4,(HL)", 0x66, bit_4_HL),

    Bit5A:  ("BIT 5,A",    0x6F, bit_5_A),
    Bit5B:  ("BIT 5,B",    0x68, bit_5_B),
    Bit5C:  ("BIT 5,C",    0x69, bit_5_C),
    Bit5D:  ("BIT 5,D",    0x6A, bit_5_D),
    Bit5E:  ("BIT 5,E",    0x6B, bit_5_E),
    Bit5H:  ("BIT 5,H",    0x6C, bit_5_H),
    Bit5L:  ("BIT 5,L",    0x6D, bit_5_L),
    Bit5HL: ("BIT 5,(HL)", 0x6E, bit_5_HL),

    Bit6A:  ("BIT 6,A",    0x77, bit_6_A),
    Bit6B:  ("BIT 6,B",    0x70, bit_6_B),
    Bit6C:  ("BIT 6,C",    0x71, bit_6_C),
    Bit6D:  ("BIT 6,D",    0x72, bit_6_D),
    Bit6E:  ("BIT 6,E",    0x73, bit_6_E),
    Bit6H:  ("BIT 6,H",    0x74, bit_6_H),
    Bit6L:  ("BIT 6,L",    0x75, bit_6_L),
    Bit6HL: ("BIT 6,(HL)", 0x76, bit_6_HL),

    Bit7A:  ("BIT 7,A",    0x7F, bit_7_A),
    Bit7B:  ("BIT 7,B",    0x78, bit_7_B),
    Bit7C:  ("BIT 7,C",    0x79, bit_7_C),
    Bit7D:  ("BIT 7,D",    0x7A, bit_7_D),
    Bit7E:  ("BIT 7,E",    0x7B, bit_7_E),
    Bit7H:  ("BIT 7,H",    0x7C, bit_7_H),
    Bit7L:  ("BIT 7,L",    0x7D, bit_7_L),
    Bit7HL: ("BIT 7,(HL)", 0x7E, bit_7_HL),

    // SET b,r
    //
    // Set bit b in register r
    //
    // b = 0 - 7
    // r = A,B,C,D,E,H,L,(HL)
    Set0A:  ("SET 0,A",    0xC7, set_0_A),
    Set0B:  ("SET 0,B",    0xC0, set_0_B),
    Set0C:  ("SET 0,C",    0xC1, set_0_C),
    Set0D:  ("SET 0,D",    0xC2, set_0_D),
    Set0E:  ("SET 0,E",    0xC3, set_0_E),
    Set0H:  ("SET 0,H",    0xC4, set_0_H),
    Set0L:  ("SET 0,L",    0xC5, set_0_L),
    Set0HL: ("SET 0,(HL)", 0xC6, set_0_HL),

    Set1A:  ("SET 1,A",    0xCF, set_1_A),
    Set1B:  ("SET 1,B",    0xC8, set_1_B),
    Set1C:  ("SET 1,C",    0xC9, set_1_C),
    Set1D:  ("SET 1,D",    0xCA, set_1_D),
    Set1E:  ("SET 1,E",    0xCB, set_1_E),
    Set1H:  ("SET 1,H",    0xCC, set_1_H),
    Set1L:  ("SET 1,L",    0xCD, set_1_L),
    Set1HL: ("SET 1,(HL)", 0xCE, set_1_HL),

    Set2A:  ("SET 2,A",    0xD7, set_2_A),
    Set2B:  ("SET 2,B",    0xD0, set_2_B),
    Set2C:  ("SET 2,C",    0xD1, set_2_C),
    Set2D:  ("SET 2,D",    0xD2, set_2_D),
    Set2E:  ("SET 2,E",    0xD3, set_2_E),
    Set2H:  ("SET 2,H",    0xD4, set_2_H),
    Set2L:  ("SET 2,L",    0xD5, set_2_L),
    Set2HL: ("SET 2,(HL)", 0xD6, set_2_HL),

    Set3A:  ("SET 3,A",    0xDF, set_3_A),
    Set3B:  ("SET 3,B",    0xD8, set_3_B),
    Set3C:  ("SET 3,C",    0xD9, set_3_C),
    Set3D:  ("SET 3,D",    0xDA, set_3_D),
    Set3E:  ("SET 3,E",    0xDB, set_3_E),
    Set3H:  ("SET 3,H",    0xDC, set_3_H),
    Set3L:  ("SET 3,L",    0xDD, set_3_L),
    Set3HL: ("SET 3,(HL)", 0xDE, set_3_HL),

    Set4A:  ("SET 4,A",    0xE7, set_4_A),
    Set4B:  ("SET 4,B",    0xE0, set_4_B),
    Set4C:  ("SET 4,C",    0xE1, set_4_C),
    Set4D:  ("SET 4,D",    0xE2, set_4_D),
    Set4E:  ("SET 4,E",    0xE3, set_4_E),
    Set4H:  ("SET 4,H",    0xE4, set_4_H),
    Set4L:  ("SET 4,L",    0xE5, set_4_L),
    Set4HL: ("SET 4,(HL)", 0xE6, set_4_HL),

    Set5A:  ("SET 5,A",    0xEF, set_5_A),
    Set5B:  ("SET 5,B",    0xE8, set_5_B),
    Set5C:  ("SET 5,C",    0xE9, set_5_C),
    Set5D:  ("SET 5,D",    0xEA, set_5_D),
    Set5E:  ("SET 5,E",    0xEB, set_5_E),
    Set5H:  ("SET 5,H",    0xEC, set_5_H),
    Set5L:  ("SET 5,L",    0xED, set_5_L),
    Set5HL: ("SET 5,(HL)", 0xEE, set_5_HL),

    Set6A:  ("SET 6,A",    0xF7, set_6_A),
    Set6B:  ("SET 6,B",    0xF0, set_6_B),
    Set6C:  ("SET 6,C",    0xF1, set_6_C),
    Set6D:  ("SET 6,D",    0xF2, set_6_D),
    Set6E:  ("SET 6,E",    0xF3, set_6_E),
    Set6H:  ("SET 6,H",    0xF4, set_6_H),
    Set6L:  ("SET 6,L",    0xF5, set_6_L),
    Set6HL: ("SET 6,(HL)", 0xF6, set_6_HL),

    Set7A:  ("SET 7,A",    0xFF, set_7_A),
    Set7B:  ("SET 7,B",    0xF8, set_7_B),
    Set7C:  ("SET 7,C",    0xF9, set_7_C),
    Set7D:  ("SET 7,D",    0xFA, set_7_D),
    Set7E:  ("SET 7,E",    0xFB, set_7_E),
    Set7H:  ("SET 7,H",    0xFC, set_7_H),
    Set7L:  ("SET 7,L",    0xFD, set_7_L),
    Set7HL: ("SET 7,(HL)", 0xFE, set_7_HL),

    // RES b,r
    //
    // Reset bit b in register r
    //
    // b = 0 - 7
    // r = A,B,C,D,E,H,L,(HL)
    Reset0A:  ("RES 0,A",    0x87, reset_0_A),
    Reset0B:  ("RES 0,B",    0x80, reset_0_B),
    Reset0C:  ("RES 0,C",    0x81, reset_0_C),
    Reset0D:  ("RES 0,D",    0x82, reset_0_D),
    Reset0E:  ("RES 0,E",    0x83, reset_0_E),
    Reset0H:  ("RES 0,H",    0x84, reset_0_H),
    Reset0L:  ("RES 0,L",    0x85, reset_0_L),
    Reset0HL: ("RES 0,(HL)", 0x86, reset_0_HL),

    Reset1A:  ("RES 1,A",    0x8F, reset_1_A),
    Reset1B:  ("RES 1,B",    0x88, reset_1_B),
    Reset1C:  ("RES 1,C",    0x89, reset_1_C),
    Reset1D:  ("RES 1,D",    0x8A, reset_1_D),
    Reset1E:  ("RES 1,E",    0x8B, reset_1_E),
    Reset1H:  ("RES 1,H",    0x8C, reset_1_H),
    Reset1L:  ("RES 1,L",    0x8D, reset_1_L),
    Reset1HL: ("RES 1,(HL)", 0x8E, reset_1_HL),

    Reset2A:  ("RES 2,A",    0x97, reset_2_A),
    Reset2B:  ("RES 2,B",    0x90, reset_2_B),
    Reset2C:  ("RES 2,C",    0x91, reset_2_C),
    Reset2D:  ("RES 2,D",    0x92, reset_2_D),
    Reset2E:  ("RES 2,E",    0x93, reset_2_E),
    Reset2H:  ("RES 2,H",    0x94, reset_2_H),
    Reset2L:  ("RES 2,L",    0x95, reset_2_L),
    Reset2HL: ("RES 2,(HL)", 0x96, reset_2_HL),

    Reset3A:  ("RES 3,A",    0x9F, reset_3_A),
    Reset3B:  ("RES 3,B",    0x98, reset_3_B),
    Reset3C:  ("RES 3,C",    0x99, reset_3_C),
    Reset3D:  ("RES 3,D",    0x9A, reset_3_D),
    Reset3E:  ("RES 3,E",    0x9B, reset_3_E),
    Reset3H:  ("RES 3,H",    0x9C, reset_3_H),
    Reset3L:  ("RES 3,L",    0x9D, reset_3_L),
    Reset3HL: ("RES 3,(HL)", 0x9E, reset_3_HL),

    Reset4A:  ("RES 4,A",    0xA7, reset_4_A),
    Reset4B:  ("RES 4,B",    0xA0, reset_4_B),
    Reset4C:  ("RES 4,C",    0xA1, reset_4_C),
    Reset4D:  ("RES 4,D",    0xA2, reset_4_D),
    Reset4E:  ("RES 4,E",    0xA3, reset_4_E),
    Reset4H:  ("RES 4,H",    0xA4, reset_4_H),
    Reset4L:  ("RES 4,L",    0xA5, reset_4_L),
    Reset4HL: ("RES 4,(HL)", 0xA6, reset_4_HL),

    Reset5A:  ("RES 5,A",    0xAF, reset_5_A),
    Reset5B:  ("RES 5,B",    0xA8, reset_5_B),
    Reset5C:  ("RES 5,C",    0xA9, reset_5_C),
    Reset5D:  ("RES 5,D",    0xAA, reset_5_D),
    Reset5E:  ("RES 5,E",    0xAB, reset_5_E),
    Reset5H:  ("RES 5,H",    0xAC, reset_5_H),
    Reset5L:  ("RES 5,L",    0xAD, reset_5_L),
    Reset5HL: ("RES 5,(HL)", 0xAE, reset_5_HL),

    Reset6A:  ("RES 6,A",    0xB7, reset_6_A),
    Reset6B:  ("RES 6,B",    0xB0, reset_6_B),
    Reset6C:  ("RES 6,C",    0xB1, reset_6_C),
    Reset6D:  ("RES 6,D",    0xB2, reset_6_D),
    Reset6E:  ("RES 6,E",    0xB3, reset_6_E),
    Reset6H:  ("RES 6,H",    0xB4, reset_6_H),
    Reset6L:  ("RES 6,L",    0xB5, reset_6_L),
    Reset6HL: ("RES 6,(HL)", 0xB6, reset_6_HL),

    Reset7A:  ("RES 7,A",    0xBF, reset_7_A),
    Reset7B:  ("RES 7,B",    0xB8, reset_7_B),
    Reset7C:  ("RES 7,C",    0xB9, reset_7_C),
    Reset7D:  ("RES 7,D",    0xBA, reset_7_D),
    Reset7E:  ("RES 7,E",    0xBB, reset_7_E),
    Reset7H:  ("RES 7,H",    0xBC, reset_7_H),
    Reset7L:  ("RES 7,L",    0xBD, reset_7_L),
    Reset7HL: ("RES 7,(HL)", 0xBE, reset_7_HL)
);
