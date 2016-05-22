use gb_proc::cpu::{Cpu, CpuState};
use std::num::Wrapping;

macro_rules! op_codes {
    // First the unprefixed op codes
    ($($element: ident: ($tostring: expr,
                         $hex: expr,
                         $cycles: expr,
                         $func: path)),+;
    // Then the 0XCB prefixed op codes
     $($cb_element: ident: ($cb_tostring: expr,
                            $cb_hex: expr,
                            $cb_cycles: expr,
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
                        _ => panic!("Op code not implemented")
                    }
                } else {
                    match hex {
                        $($hex => OpCode::$element),*,
                        _ => panic!("Op code not implemented")
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

            pub fn get_cycles(&self) -> usize {
                match self {
                    $(&OpCode::$element => $cycles),*,
                    $(&OpCode::$cb_element => $cb_cycles),*,
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

fn unimplemented(_: &mut Cpu) {
    unimplemented!();
}

fn no_op(_: &mut Cpu) {}

/* Get the value (8-bit) pointed by the Program Counter (PC) */
fn next_value(cpu: &mut Cpu) -> u8 {
    cpu.inc_PC();
    cpu.deref_PC()
}

/* Get the pointer (16-bit value) pointed  by the Program Counter (PC) */
fn next_pointer(cpu: &mut Cpu) -> u16 {
    cpu.inc_PC();
    let l = cpu.deref_PC();

    cpu.inc_PC();
    let h = cpu.deref_PC();

    ((h as u16) << 8) + (l as u16)
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
    let v = cpu.deref_HL();
    cpu.set_A_reg(v);
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
    let v = cpu.deref_HL();
    cpu.set_B_reg(v);
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
    let v = cpu.deref_HL();
    cpu.set_C_reg(v);
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
    let v = cpu.deref_HL();
    cpu.set_D_reg(v);
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
    let v = cpu.deref_HL();
    cpu.set_E_reg(v);
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
    let v = cpu.deref_HL();
    cpu.set_H_reg(v);
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
    let v = cpu.deref_HL();
    cpu.set_L_reg(v);
}

fn ld_HL_B(cpu: &mut Cpu) {
    let v = cpu.get_B_reg();
    cpu.set_deref_HL(v);
}

fn ld_HL_C(cpu: &mut Cpu) {
    let v = cpu.get_C_reg();
    cpu.set_deref_HL(v);
}

fn ld_HL_D(cpu: &mut Cpu) {
    let v = cpu.get_D_reg();
    cpu.set_deref_HL(v);
}

fn ld_HL_E(cpu: &mut Cpu) {
    let v = cpu.get_E_reg();
    cpu.set_deref_HL(v);
}

fn ld_HL_L(cpu: &mut Cpu) {
    let v = cpu.get_L_reg();
    cpu.set_deref_HL(v);
}

fn ld_HL_H(cpu: &mut Cpu) {
    let v = cpu.get_H_reg();
    cpu.set_deref_HL(v);
}

fn ld_HL_n(cpu: &mut Cpu) {
    let v = next_value(cpu);
    cpu.set_deref_HL(v);
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

fn op_A_BC(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.deref_BC();
    op_A_X(func, y, cpu);
}

fn op_A_DE(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.deref_DE();
    op_A_X(func, y, cpu);
}

fn op_A_HL(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = cpu.deref_HL();
    op_A_X(func, y, cpu);
}

fn op_A_nn(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    let y = cpu.deref(address);
    op_A_X(func, y, cpu);
}

fn op_A_x(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let y = next_value(cpu);
    op_A_X(func, y, cpu);
}

fn op_X_A(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, x: u8, cpu: &mut Cpu) -> u8 {
    let y = cpu.get_A_reg();
    func(x, y, cpu)
}

fn op_BC_A(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let x = cpu.deref_BC();
    let result = op_X_A(func, x, cpu);
    cpu.set_deref_BC(result);
}

fn op_DE_A(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let x = cpu.deref_DE();
    let result = op_X_A(func, x, cpu);
    cpu.set_deref_DE(result);
}

fn op_HL_A(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let x = cpu.deref_HL();
    let result = op_X_A(func, x, cpu);
    cpu.set_deref_HL(result);
}

fn op_nn_A(func: fn(x: u8, y: u8, cpu: &mut Cpu) -> u8, cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    let x = cpu.deref(address);
    let result = op_X_A(func, x, cpu);
    cpu.set_deref(address, result);
}

fn ld(_: u8, y: u8, _: &mut Cpu) -> u8 { y }

fn ld_A_BC(cpu: &mut Cpu) { op_A_BC(ld, cpu) }
fn ld_A_DE(cpu: &mut Cpu) { op_A_DE(ld, cpu) }
fn ld_A_nn(cpu: &mut Cpu) { op_A_nn(ld, cpu) }
fn ld_A_x(cpu: &mut Cpu)  {  op_A_x(ld, cpu) }

fn ld_BC_A(cpu: &mut Cpu) { op_BC_A(ld, cpu) }
fn ld_DE_A(cpu: &mut Cpu) { op_DE_A(ld, cpu) }
fn ld_HL_A(cpu: &mut Cpu) { op_HL_A(ld, cpu) }
fn ld_nn_A(cpu: &mut Cpu) { op_nn_A(ld, cpu) }

fn ldd(_: u8, y: u8, cpu: &mut Cpu) -> u8 {
    cpu.deref(0xFF00 + y as u16)
}

fn ld_A_FFC(cpu: &mut Cpu) { op_A_C(ldd, cpu); }

fn ld_FFC_A(cpu: &mut Cpu) {
    let address = 0xFF00 + cpu.get_C_reg() as u16;
    let v = cpu.get_A_reg();
    cpu.set_deref(address, v);
}

fn ldd_A_HL(cpu: &mut Cpu) {
    let v = cpu.deref_HL();
    let hl = cpu.get_HL();
    cpu.set_HL(hl - 1);
    cpu.set_A_reg(v);
}

fn ldd_HL_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    let hl = cpu.get_HL();
    cpu.set_deref_HL(v);
    cpu.set_HL(hl - 1);
}

fn ldi_A_HL(cpu: &mut Cpu) {
    let v = cpu.deref_HL();
    let hl = cpu.get_HL();
    cpu.set_HL(hl + 1);
    cpu.set_A_reg(v);
}

fn ldi_HL_A(cpu: &mut Cpu) {
    let v = cpu.get_A_reg();
    let hl = cpu.get_HL();
    cpu.set_deref_HL(v);
    cpu.set_HL(hl + 1);
}

fn ldh_x(_: u8, cpu: &mut Cpu) -> u16 {
    0xFF00 + next_value(cpu) as u16
}

fn op_x_A(func: fn(x: u8, cpu: &mut Cpu) -> u16, cpu: &mut Cpu) {
    let x = cpu.get_A_reg();
    let address = func(x, cpu);
    cpu.set_deref(address, x);
}

fn ldh_n_A(cpu: &mut Cpu) { op_x_A(ldh_x, cpu); }

fn ldh(_: u8, y: u8, cpu: &mut Cpu) -> u8 {
    cpu.deref(0xFF00 + y as u16)
}

fn ldh_A_n(cpu: &mut Cpu) { op_A_x(ldh, cpu); }

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
    let address = cpu.get_HL();
    cpu.set_SP(address);
}

fn ldhl_SP_n(cpu: &mut Cpu) {
    let address = Wrapping(cpu.get_SP());
    let n = Wrapping(next_value(cpu) as u16);

    let result = address + n;

    cpu.set_HL(result.0);
    cpu.reset_Z();
    cpu.reset_N();
    if (address.0 as u8) as u16 + n.0 as u16 > 0xFF {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    if result.0 < address.0 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }
}

fn ld_nn_SP(cpu: &mut Cpu) {
    let address = next_pointer(cpu);
    cpu.set_SP(address);
}

fn push_AF(cpu: &mut Cpu) {
    let a = cpu.get_A_reg();
    cpu.push_SP(a);

    let f = cpu.get_F_reg();
    cpu.push_SP(f);
}

fn push_BC(cpu: &mut Cpu) {
    let b = cpu.get_B_reg();
    cpu.push_SP(b);

    let c = cpu.get_C_reg();
    cpu.push_SP(c);
}

fn push_DE(cpu: &mut Cpu) {
    let d = cpu.get_D_reg();
    cpu.push_SP(d);

    let e = cpu.get_E_reg();
    cpu.push_SP(e);
}

fn push_HL(cpu: &mut Cpu) {
    let h = cpu.get_H_reg();
    cpu.push_SP(h);

    let l = cpu.get_L_reg();
    cpu.push_SP(l);
}

fn pop_AF(cpu: &mut Cpu) {
    let f = cpu.pop_SP();
    cpu.set_F_reg(f);

    let a = cpu.pop_SP();
    cpu.set_A_reg(a);
}

fn pop_BC(cpu: &mut Cpu) {
    let c = cpu.pop_SP();
    cpu.set_C_reg(c);

    let b = cpu.pop_SP();
    cpu.set_B_reg(b);
}

fn pop_DE(cpu: &mut Cpu) {
    let e = cpu.pop_SP();
    cpu.set_E_reg(e);

    let d = cpu.pop_SP();
    cpu.set_D_reg(d);
}

fn pop_HL(cpu: &mut Cpu) {
    let l = cpu.pop_SP();
    cpu.set_L_reg(l);

    let h = cpu.pop_SP();
    cpu.set_H_reg(h);
}

/* Add two values and set relevant flags */
fn add(x: u8, y: u8, cpu: &mut Cpu) -> u8 {
    // TODO: revisit if this is too slow
    let x_u16 = x as u16;
    let y_u16 = x as u16;

    let x_half = x >> 4;
    let y_half = y >> 4;

    let result = x_u16 + y_u16;
    let result_u8 = result as u8;

    cpu.reset_N();

    if result_u8 == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    if result > 0xFF {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    if x_half + y_half > 0xF {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    result_u8
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
        add(x, y + 1, cpu)
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
    // TODO: revisit if this is too slow
    let x_i16 = x as i16;
    let y_i16 = x as i16;

    let x_half = (x >> 4) as i8;
    let y_half = (y >> 4) as i8;

    let result = x_i16 - y_i16;
    let result_u8 = result as u8;

    cpu.set_N_flag();

    if result_u8 == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    if result < 0x00 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    if x_half - y_half < 0x00 {
        cpu.set_H_flag();
    } else {
        cpu.reset_H();
    }

    result_u8
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
        sub(x + 1, y, cpu)
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
        result = 0;
    } else {
        cpu.reset_Z();
        result = v + 1;
    }

    if (v >> 4) == 0xF {
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
    let v = cpu.deref_HL();
    let result = func(v, cpu);
    cpu.set_deref_HL(result);
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

    if (v >> 4) == 0x0 {
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
    let result = x as u32 + y as u32;

    if result > 0xFFFF {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let x_half = x << 4;
    let y_half = y << 4;

    if x_half as u32 + y_half as u32 > 0xFFFF {
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
    add_16(x, y as u16, cpu)
}

fn add_SP_x(cpu: &mut Cpu) { op_SP_x(add_16_8, cpu); }

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

fn inc_16(x: u16, _: &mut Cpu) -> u16 {
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

fn dec_16(x: u16, _: &mut Cpu) -> u16 {
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

fn daa(x: u8, cpu: &mut Cpu) -> u8 {
    if x == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_H();

    if x > 99 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let first = x % 10;
    let second = (x % 100) / 10;

    first + (second << 4)
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

fn rlca(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_7 = x >> 7;

    if bit_7 == 1 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = x << 1 + bit_7;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rlca_A(cpu: &mut Cpu) { op_A(rlca, cpu); }

fn rla(x: u8, cpu: &mut Cpu) -> u8 {
    let c_flag = if cpu.get_C_flag() { 1 } else { 0 };

    if (x >> 7) == 1 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = x << 1 + c_flag;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rla_A(cpu: &mut Cpu) { op_A(rla, cpu); }

fn rrca(x: u8, cpu: &mut Cpu) -> u8 {
    let bit_7 = x << 7;

    if bit_7 == 0x80 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = x >> 1 + bit_7;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rrca_A(cpu: &mut Cpu) { op_A(rrca, cpu); }

fn rca(x: u8, cpu: &mut Cpu) -> u8 {
    let c_flag = if cpu.get_C_flag() { 0x80 } else { 0 };

    if (x << 7) == 0x80 {
        cpu.set_C_flag();
    } else {
        cpu.reset_C();
    }

    let result = x >> 1 + c_flag;

    if result == 0 {
        cpu.set_Z_flag();
    } else {
        cpu.reset_Z();
    }

    cpu.reset_N();
    cpu.reset_H();

    result
}

fn rca_A(cpu: &mut Cpu) { op_A(rca, cpu); }

fn jp_nn(cpu: &mut Cpu) {
    let nn = next_pointer(cpu);
    cpu.set_PC(nn);
}

fn jp_cond_nn(cond: bool, cpu: &mut Cpu) {
    if cond {
        jp_nn(cpu);
    } else {
        cpu.inc_PC();
        cpu.inc_PC();
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
    let next = cpu.get_PC() + n as u16;
    cpu.set_PC(next);
}

fn jr_cond_n(cond: bool, cpu: &mut Cpu) {
    if cond {
        jr_n(cpu);
    } else {
        cpu.inc_PC();
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
    cpu.push_SP(h);

    let l = ((next << 8) >> 8) as u8;
    cpu.push_SP(l);

    cpu.set_PC(nn);
}

fn call_cond_nn(cond: bool, cpu: &mut Cpu) {
    if cond {
        call_nn(cpu);
    } else {
        cpu.inc_PC();
        cpu.inc_PC();
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
    let next = cpu.get_PC();

    let h = (next >> 8) as u8;
    cpu.push_SP(h);

    let l = ((next << 8) >> 8) as u8;
    cpu.push_SP(l);

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
    let l = cpu.pop_SP();
    let h = cpu.pop_SP();

    let next = l as u16 + ((h as u16) << 8);

    cpu.set_PC(next);
}

fn ret_cond(cond: bool, cpu: &mut Cpu) {
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
    LdBn: ("LD B,n", 0x06, 8, ld_B_n),
    LdCn: ("LD C,n", 0x0E, 8, ld_C_n),
    LdDn: ("LD D,n", 0x16, 8, ld_D_n),
    LdEn: ("LD E,n", 0x1E, 8, ld_E_n),
    LdHn: ("LD H,n", 0x26, 8, ld_H_n),
    LdLn: ("LD L,n", 0x2E, 8, ld_L_n),

    // LD r1,r2
    //
    // Put value r2 into r1
    //
    // r1,r2 = A,B,C,D,E,H,L,(HL)
    LdAA:  ("LD A,A",    0x7F, 4, no_op),
    LdAB:  ("LD A,B",    0x78, 4, ld_A_B),
    LdAC:  ("LD A,C",    0x79, 4, ld_A_C),
    LdAD:  ("LD A,D",    0x7A, 4, ld_A_D),
    LdAE:  ("LD A,E",    0x7B, 4, ld_A_E),
    LdAH:  ("LD A,H",    0x7C, 4, ld_A_H),
    LdAL:  ("LD A,L",    0x7D, 4, ld_A_L),
    LdAHL: ("LD A,(HL)", 0x7E, 8, ld_A_HL),

    LdBB:  ("LD B,B",    0x40, 4, no_op),
    LdBC:  ("LD B,C",    0x41, 4, ld_B_C),
    LdBD:  ("LD B,D",    0x42, 4, ld_B_D),
    LdBE:  ("LD B,E",    0x43, 4, ld_B_E),
    LdBH:  ("LD B,H",    0x44, 4, ld_B_H),
    LdBL:  ("LD B,L",    0x45, 4, ld_B_L),
    LdBHL: ("LD B,(HL)", 0x46, 8, ld_B_HL),

    LdCB:  ("LD C,B",    0x48, 4, ld_C_B),
    LdCC:  ("LD C,C",    0x49, 4, no_op),
    LdCD:  ("LD C,D",    0x4A, 4, ld_C_D),
    LdCE:  ("LD C,E",    0x4B, 4, ld_C_E),
    LdCH:  ("LD C,H",    0x4C, 4, ld_C_H),
    LdCL:  ("LD C,L",    0x4D, 4, ld_C_L),
    LdCHL: ("LD C,(HL)", 0x4E, 8, ld_C_HL),

    LdDB:  ("LD D,B",    0x50, 4, ld_D_B),
    LdDC:  ("LD D,C",    0x51, 4, ld_D_C),
    LdDD:  ("LD D,D",    0x52, 4, no_op),
    LdDE:  ("LD D,E",    0x53, 4, ld_D_E),
    LdDH:  ("LD D,H",    0x54, 4, ld_D_H),
    LdDL:  ("LD D,L",    0x55, 4, ld_D_L),
    LdDHL: ("LD D,(HL)", 0x56, 8, ld_D_HL),

    LdEB:  ("LD E,B",    0x58, 4, ld_E_B),
    LdEC:  ("LD E,C",    0x59, 4, ld_E_C),
    LdED:  ("LD E,D",    0x5A, 4, ld_E_D),
    LdEE:  ("LD E,E",    0x5B, 4, no_op),
    LdEH:  ("LD E,H",    0x5C, 4, ld_E_H),
    LdEL:  ("LD E,L",    0x5D, 4, ld_E_L),
    LdEHL: ("LD E,(HL)", 0x5E, 8, ld_E_HL),

    LdHB:  ("LD H,B",    0x60, 4, ld_H_B),
    LdHC:  ("LD H,C",    0x61, 4, ld_H_C),
    LdHD:  ("LD H,D",    0x62, 4, ld_H_D),
    LdHE:  ("LD H,E",    0x63, 4, ld_H_E),
    LdHH:  ("LD H,H",    0x64, 4, no_op),
    LdHL:  ("LD H,L",    0x65, 4, ld_H_L),
    LdHHL: ("LD H,(HL)", 0x66, 8, ld_H_HL),

    LdLB:  ("LD L,B",    0x68, 4, ld_L_B),
    LdLC:  ("LD L,C",    0x69, 4, ld_L_C),
    LdLD:  ("LD L,D",    0x6A, 4, ld_L_D),
    LdLE:  ("LD L,E",    0x6B, 4, ld_L_E),
    LdLH:  ("LD L,H",    0x6C, 4, ld_L_H),
    LdLL:  ("LD L,L",    0x6D, 4, no_op),
    LdLHL: ("LD L,(HL)", 0x6E, 8, ld_L_HL),

    LdHLB:  ("LD (HL),B", 0x70, 8,  ld_HL_B),
    LdHLC:  ("LD (HL),C", 0x71, 8,  ld_HL_C),
    LdHLD:  ("LD (HL),D", 0x72, 8,  ld_HL_D),
    LdHLE:  ("LD (HL),E", 0x73, 8,  ld_HL_E),
    LdHLH:  ("LD (HL),H", 0x74, 8,  ld_HL_H),
    LdHLL:  ("LD (HL),L", 0x75, 8,  ld_HL_L),
    LdHLHL: ("LD (HL),n", 0x36, 12, ld_HL_n),

    // LD A,n
    //
    // Put value n into A
    //
    // n = A,B,C,D,E,H,L,(BC),(DE),(HL),(nn),#
    // nn = two byte immediate value. (LS byte first.)
    LdABC: ("LD A,(BC)", 0x0A, 8,  ld_A_BC),
    LdADE: ("LD A,(DE)", 0x1A, 8,  ld_A_DE),
    LdAnn: ("LD A,(nn)", 0xFA, 16, ld_A_nn),
    LdAx:  ("LD A,#",    0x3E, 8,  ld_A_x),

    // LD n,A
    //
    // Put value A into n
    //
    // n = A,B,C,D,E,H,L,(BC),(DE),(HL),(nn)
    // nn = two byte immediate value. (LS byte first)
    LdBCA: ("LD (BC),A", 0x02, 8,  ld_BC_A),
    LdDEA: ("LD (DE),A", 0x12, 8,  ld_DE_A),
    LdHLA: ("LD (HL),A", 0x77, 8,  ld_HL_A),
    LdnnA: ("LD (nn),A", 0xEA, 16, ld_nn_A),

    // LD A,(C)
    //
    // Put value at address $FF00 + register C into A
    // Same as: LD A, ($FF00+C)
    LdAFFC: ("LD A,($FF00+C)", 0xF2, 8, ld_A_FFC),

    // LD (C),A
    //
    // Put A into address $FF00 + register C
    LdFFCA: ("LD ($FF00+C),A", 0xE2, 8, ld_FFC_A),

    // LDD A,(HL)
    //
    // Put value at address HL into A. Decrement HL.
    // Same as: LD A,(HL) - DEC HL
    LddAHL: ("LDD A,(HL)", 0x3A, 8, ldd_A_HL),

    // LDD (HL),A
    //
    // Put A into memoty address HL. Decrement HL.
    // Same as: LD (HL),A - DEC HL
    LddHLA: ("LDD (HL),A", 0x32, 8, ldd_HL_A),

    // LDI A,(HL)
    //
    // Put value at address HL into A. Increment HL.
    // Same as: LD A,(HL) - INC HL
    LdiAHL: ("LDI A,(HL)", 0x2A, 8, ldi_A_HL),

    // LDI (HL),A
    //
    // Put A into memory address HL. Increment HL.
    // Same as: LD (HL),A - INC HL
    LdiHLA: ("LDI (HL),A", 0x22, 8, ldi_HL_A),

    // LDH (n),A
    //
    // Put A into memory address $FF00+n
    //
    // n = one byte immediate value
    LdhNA: ("LD ($FF00+n),A", 0xE0, 12, ldh_n_A),

    // LDH A,(n)
    //
    // Put memory address $FF00+n into A.
    //
    // n = one byte immediate value
    LdhAHL: ("LD A,($FF00+n)", 0xF0, 12, ldh_A_n),

    // LD n,nn
    //
    // Put value nn into n
    //
    // n = BC,DE,HL,SP
    // nn = 16 bit immediate value
    LdBCnn: ("LD BC,nn", 0x01, 12, ld_BC_nn),
    LdDEnn: ("LD DE,nn", 0x11, 12, ld_DE_nn),
    LdHLnn: ("LD HL,nn", 0x21, 12, ld_HL_nn),
    LdSPnn: ("LD SP,nn", 0x31, 12, ld_SP_nn),

    // LD SP,HL
    //
    // Put HL into Stack Pointer (SP)
    LdSPHL: ("LD SP,HL", 0xF9, 8, ld_SP_HL),

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
    LDHLSPn: ("LDHL SP,n", 0xF8, 12, ldhl_SP_n),

    // LD (nn),SP
    //
    // Put Stack Pointer (SP) at address nn
    //
    // nn = two byte immediate address.
    LDnnSP: ("LD (nn),SP", 0x08, 20, ld_nn_SP),

    // PUSH nn
    //
    // Push register pair nn onto stack.
    // Decrement Stack Pointer (SP) twice.
    //
    // nn = AF,BC,DE,HL
    PushAF: ("PUSH AF", 0xF5, 16, push_AF),
    PushBC: ("PUSH BC", 0xC5, 16, push_BC),
    PushDE: ("PUSH DE", 0xD5, 16, push_DE),
    PushHL: ("PUSH HL", 0xE5, 16, push_HL),

    // POP nn
    //
    // Pop two bytes off stack into register pair nn.
    // Increment Stack Pointer (SP) twice.
    //
    // nn = AF,BC,DE,HL
    PopAF: ("POP AF", 0xF1, 12, pop_AF),
    PopBC: ("POP BC", 0xC1, 12, pop_BC),
    PopDE: ("POP DE", 0xD1, 12, pop_DE),
    PopHL: ("POP HL", 0xE1, 12, pop_HL),

    // ADD A,n
    //
    // Add n to A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set if carry from bit 3
    // C - Set if carry from bit 7
    AddAA:  ("ADD A,A",    0x87, 4, add_A_A),
    AddAB:  ("ADD A,B",    0x80, 4, add_A_B),
    AddAC:  ("ADD A,C",    0x81, 4, add_A_C),
    AddAD:  ("ADD A,D",    0x82, 4, add_A_D),
    AddAE:  ("ADD A,E",    0x83, 4, add_A_E),
    AddAH:  ("ADD A,H",    0x84, 4, add_A_H),
    AddAL:  ("ADD A,L",    0x85, 4, add_A_L),
    AddAHL: ("ADD A,(HL)", 0x86, 8, add_A_HL),
    AddAx:  ("ADD A,#",    0xC6, 8, add_A_x),

    // ADC A,n
    //
    // Add n + Carry flag to A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set if carry from bit 3
    // C - Set if carry from bit 7
    AdcAA:  ("ADC A,A",    0x8F, 4, adc_A_A),
    AdcAB:  ("ADC A,B",    0x88, 4, adc_A_B),
    AdcAC:  ("ADC A,C",    0x89, 4, adc_A_C),
    AdcAD:  ("ADC A,D",    0x8A, 4, adc_A_D),
    AdcAE:  ("ADC A,E",    0x8B, 4, adc_A_E),
    AdcAH:  ("ADC A,H",    0x8C, 4, adc_A_H),
    AdcAL:  ("ADC A,L",    0x8D, 4, adc_A_L),
    AdcAHL: ("ADC A,(HL)", 0x8E, 8, adc_A_HL),
    AdcAx:  ("ADC A,#",    0xCE, 8, adc_A_x),

    // SUB n
    //
    // Subtract n from A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set if no borrow
    SubAA:  ("SUB A,A",    0x97, 4, sub_A_A),
    SubAB:  ("SUB A,B",    0x90, 4, sub_A_B),
    SubAC:  ("SUB A,C",    0x91, 4, sub_A_C),
    SubAD:  ("SUB A,D",    0x92, 4, sub_A_D),
    SubAE:  ("SUB A,E",    0x93, 4, sub_A_E),
    SubAH:  ("SUB A,H",    0x94, 4, sub_A_H),
    SubAL:  ("SUB A,L",    0x95, 4, sub_A_L),
    SubAHL: ("SUB A,(HL)", 0x96, 8, sub_A_HL),
    SubAx:  ("SUB A,#",    0xD6, 8, sub_A_x),

    // SBC A,n
    //
    // Subtract n + Carry flag from A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set if no borrow
    SbcAA:  ("SBC A,A",    0x9F, 4, sbc_A_A),
    SbcAB:  ("SBC A,B",    0x98, 4, sbc_A_B),
    SbcAC:  ("SBC A,C",    0x99, 4, sbc_A_C),
    SbcAD:  ("SBC A,D",    0x9A, 4, sbc_A_D),
    SbcAE:  ("SBC A,E",    0x9B, 4, sbc_A_E),
    SbcAH:  ("SBC A,H",    0x9C, 4, sbc_A_H),
    SbcAL:  ("SBC A,L",    0x9D, 4, sbc_A_L),
    SbcAHL: ("SBC A,(HL)", 0x9E, 8, sbc_A_HL),

    // AND n
    //
    // Logically AND n with A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set
    // C - Reset
    AndAA:  ("AND A,A",    0xA7, 4, and_A_A),
    AndAB:  ("AND A,B",    0xA0, 4, and_A_B),
    AndAC:  ("AND A,C",    0xA1, 4, and_A_C),
    AndAD:  ("AND A,D",    0xA2, 4, and_A_D),
    AndAE:  ("AND A,E",    0xA3, 4, and_A_E),
    AndAH:  ("AND A,H",    0xA4, 4, and_A_H),
    AndAL:  ("AND A,L",    0xA5, 4, and_A_L),
    AndAHL: ("AND A,(HL)", 0xA6, 8, and_A_HL),
    AndAx:  ("AND A,#",    0xE6, 8, and_A_x),

    // OR n
    //
    // Logical OR n with register A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    OrAA:  ("OR A,A",    0xB7, 4, or_A_A),
    OrAB:  ("OR A,B",    0xB0, 4, or_A_B),
    OrAC:  ("OR A,C",    0xB1, 4, or_A_C),
    OrAD:  ("OR A,D",    0xB2, 4, or_A_D),
    OrAE:  ("OR A,E",    0xB3, 4, or_A_E),
    OrAH:  ("OR A,H",    0xB4, 4, or_A_H),
    OrAL:  ("OR A,L",    0xB5, 4, or_A_L),
    OrAHL: ("OR A,(HL)", 0xB6, 8, or_A_HL),
    OrAx:  ("OR A,#",    0xF6, 8, or_A_x),

    // XOR n
    //
    // Logical exclusive OR n with register A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    XorAA:  ("XOR A,A",    0xAF, 4, xor_A_A),
    XorAB:  ("XOR A,B",    0xA8, 4, xor_A_B),
    XorAC:  ("XOR A,C",    0xA9, 4, xor_A_C),
    XorAD:  ("XOR A,D",    0xAA, 4, xor_A_D),
    XorAE:  ("XOR A,E",    0xAB, 4, xor_A_E),
    XorAH:  ("XOR A,H",    0xAC, 4, xor_A_H),
    XorAL:  ("XOR A,L",    0xAD, 4, xor_A_L),
    XorAHL: ("XOR A,(HL)", 0xAE, 8, xor_A_HL),
    XorAx:  ("XOR A,#",    0xEE, 8, xor_A_x),

    // CP n
    //
    // Compare A with n. This is basically an A - n subtraction
    // instruction but the results are thrown away
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero. (Set if A = n)
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set for no borrow. (Set if A < n)
    CpAA:  ("CP A,A",    0xBF, 4, cp_A_A),
    CpAB:  ("CP A,B",    0xB8, 4, cp_A_B),
    CpAC:  ("CP A,C",    0xB9, 4, cp_A_C),
    CpAD:  ("CP A,D",    0xBA, 4, cp_A_D),
    CpAE:  ("CP A,E",    0xBB, 4, cp_A_E),
    CpAH:  ("CP A,H",    0xBC, 4, cp_A_H),
    CpAL:  ("CP A,L",    0xBD, 4, cp_A_L),
    CpAHL: ("CP A,(HL)", 0xBE, 8, cp_A_HL),
    CpAx:  ("CP A,#",    0xFE, 8, cp_A_x),

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
    IncA:   ("INC A",    0x3C, 4,  inc_A),
    IncB:   ("INC B",    0x04, 4,  inc_B),
    IncC:   ("INC C",    0x0C, 4,  inc_C),
    IncD:   ("INC D",    0x14, 4,  inc_D),
    IncE:   ("INC E",    0x1C, 4,  inc_E),
    IncH:   ("INC H",    0x24, 4,  inc_H),
    IncL:   ("INC L",    0x2C, 4,  inc_L),
    IncHLp: ("INC (HL)", 0x34, 12, inc_HLp),

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
    DecA:   ("DEC A",    0x3D, 4,  dec_A),
    DecB:   ("DEC B",    0x05, 4,  dec_B),
    DecC:   ("DEC C",    0x0D, 4,  dec_C),
    DecD:   ("DEC D",    0x15, 4,  dec_D),
    DecE:   ("DEC E",    0x1D, 4,  dec_E),
    DecH:   ("DEC H",    0x25, 4,  dec_H),
    DecL:   ("DEC L",    0x2D, 4,  dec_L),
    DecHLp: ("DEC (HL)", 0x35, 12, dec_HLp),

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
    AddHLBC: ("ADD HL,BC", 0x09, 8, add_HL_BC),
    AddHLDE: ("ADD HL,DE", 0x19, 8, add_HL_DE),
    AddHLHL: ("ADD HL,HL", 0x29, 8, add_HL_HL),
    AddHLSP: ("ADD HL,SP", 0x39, 8, add_HL_SP),

    // ADD SP,n
    //
    // Add n to Stack Pointer (SP)
    //
    // n = one byte signed immediate value (#)
    //
    // Flags:
    // Z - Reset
    // N - Reset
    // H - Set or reset according to operation
    // C - Set or reset according to operation
    AddSPx: ("ADD SP,#", 0xE8, 16, add_SP_x),

    // INC nn
    //
    // Increment register nn
    //
    // nn = BC,DE,HL,SP
    IncBC: ("INC BC", 0x03, 8, inc_BC),
    IncDE: ("INC DE", 0x13, 8, inc_DE),
    IncHL: ("INC HL", 0x23, 8, inc_HL),
    IncSP: ("INC SP", 0x33, 8, inc_SP),

    // DEC nn
    //
    // Decrement register nn
    //
    // nn = BC,DE,HL,SP
    DecBC: ("DEC BC", 0x0B, 8, dec_BC),
    DecDE: ("DEC DE", 0x1B, 8, dec_DE),
    DecHL: ("DEC HL", 0x2B, 8, dec_HL),
    DecSP: ("DEC SP", 0x3B, 8, dec_SP),

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
    Daa: ("DAA", 0x27, 4, daa_A),

    // CPL
    //
    // Complement A register. (Flip all bits).
    //
    // Flags:
    // Z - Not affected
    // N - Set
    // H - Set
    // C - Not affected
    Cpl: ("CPL", 0x2F, 4, cpl_A),

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
    Ccf: ("CCF", 0x3F, 4, ccf),

    // SCF
    //
    // Set carry flag.
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Reset
    // C - Set
    Scf: ("SCF", 0x37, 4, scf),

    // NOP
    //
    // No operation
    Nop: ("NOP", 0x00, 4, no_op),

    // HALT
    //
    // Power down CPU until an interrupt occurs. Use this
    // when ever possible to reduce energy consumption
    Halt: ("HALT", 0x76, 4, halt),

    // STOP
    //
    // Halt CPU & LCD display until button pressed.
    Stop: ("STOP", 0x10, 4, stop),

    // DI
    //
    // This instruction disables interrupts but not
    // immediately. Interrupts are disabled after
    // the instruction after DI is executed.
    Di: ("DI", 0xF3, 4, di),

    // EI
    //
    // Enable interrupts. This instruction enables interrupts
    // but not immediately. Interrupts are enabled after
    // the instruction after EI is executed.
    Ei: ("EI", 0xFB, 4, ei),

    // RLCA
    //
    // Rotate A left. Old bit 7 to Carry flag.
    //
    // Flags:
    // Z - Set if result is Zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data.
    Rlca: ("RLCA", 0x07, 4, rlca_A),

    // RLA
    //
    // Rotate A left thorugh Carry flag.
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data
    Rla: ("RLA", 0x17, 4, rla_A),

    // RRCA
    //
    // Rotate A right. Old bit 0 to Carry flag.
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    Rrca: ("RRCA", 0x0F, 4, rrca_A),

    // RRA
    //
    // Rotate A right through Carry flag.
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    Rra: ("Rra", 0x1F, 4, rca_A),

    // JP nn
    //
    // Jump to address nn
    //
    // nn = two bytes immediate value. (LS byte first)
    Jp: ("JP", 0xC3, 12, jp_nn),

    // JP cc,nn
    //
    // Jump to address n if following condition is true:
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = Z,  Jump if C flag is set
    //
    // nn = two byte immediate value. (LS byte first)
    JpNZnn: ("JP NZ,nn", 0xC2, 12, jp_NZ_nn),
    JpZnn:  ("JP Z,nn",  0xCA, 12, jp_Z_nn),
    JpNCnn: ("JP NC,nn", 0xD2, 12, jp_NC_nn),
    JpCnn:  ("JP C,nn",  0xDA, 12, jp_C_nn),

    // JP (HL)
    //
    // Jump to address contained in HL.
    JpHL: ("JP (HL)", 0xE9, 4, jp_HL),

    // JR n
    //
    // Add n to current address and jump to it
    //
    // n = one byte signed immediate value
    Jrn: ("JR n", 0x18, 8, jr_n),

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
    JrNZn: ("JR NZ,n", 0x20, 8, jr_NZ_n),
    JrZn:  ("JR Z,n",  0x28, 8, jr_Z_n),
    JrNCn: ("JR NC,n", 0x30, 8, jr_NC_n),
    JrCn:  ("JR C,n",  0x38, 8, jr_C_n),

    // CALL nn
    //
    // Push address of next instruction onto stack and then
    // jump to address nn
    //
    // nn = two byte immediate value. (LS byte first.)
    Call: ("CALL nn", 0xCD, 12, call_nn),

    // CALL cc,nn
    //
    // Call address n if following condition is true:
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    //
    // nn = two byte immediate value. (LS byte first.)
    CallNZnn: ("CALL NZ,nn", 0xC4, 12, call_NZ_nn),
    CallZnn:  ("CALL Z,nn",  0xCC, 12, call_Z_nn),
    CallNCnn: ("CALL NC,nn", 0xD4, 12, call_NC_nn),
    CallCnn:  ("CALL C,nn",  0xDC, 12, call_C_nn),

    // RST n
    //
    // Push present address onto stack
    // Jump to address $0000 + n
    //
    // n = $00, $08, $10, $18, $20, $28, $30, $38
    Rst00: ("RST 00H", 0xC7, 32, rst_00),
    Rst08: ("RST 08H", 0xCF, 32, rst_08),
    Rst10: ("RST 10H", 0xD7, 32, rst_10),
    Rst18: ("RST 18H", 0xDF, 32, rst_18),
    Rst20: ("RST 20H", 0xE7, 32, rst_20),
    Rst28: ("RST 28H", 0xEF, 32, rst_28),
    Rst30: ("RST 30H", 0xF7, 32, rst_30),
    Rst38: ("RST 38H", 0xFF, 32, rst_38),

    // RET
    //
    // Pop two bytes from stack and jump to that address
    Ret: ("RET", 0xC9, 8, ret),

    // RET cc
    //
    // Return if following condition is true
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    RetNZ: ("RET NZ", 0xC0, 8, ret_NZ),
    RetZ:  ("RET Z",  0xC8, 8, ret_Z),
    RetNC: ("RET NC", 0xD0, 8, ret_NC),
    RetC:  ("RET C",  0xD8, 8, ret_C),

    // RETI
    //
    // Pop two bytes from stack and jump to that address then
    // enable interrupts
    Reti: ("RETI", 0xD9, 8, reti);

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
    SwapA:  ("SWAP A",    0x37, 8,  unimplemented),
    SwapB:  ("SWAP B",    0x30, 8,  unimplemented),
    SwapC:  ("SWAP C",    0x31, 8,  unimplemented),
    SwapD:  ("SWAP D",    0x32, 8,  unimplemented),
    SwapE:  ("SWAP E",    0x33, 8,  unimplemented),
    SwapH:  ("SWAP H",    0x34, 8,  unimplemented),
    SwapL:  ("SWAP L",    0x35, 8,  unimplemented),
    SwapHL: ("SWAP (HL)", 0x36, 16, unimplemented),

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
    RlcA:  ("RLC A",    0x07, 8,  unimplemented),
    RlcB:  ("RLC B",    0x00, 8,  unimplemented),
    RlcC:  ("RLC C",    0x01, 8,  unimplemented),
    RlcD:  ("RLC D",    0x02, 8,  unimplemented),
    RlcE:  ("RLC E",    0x03, 8,  unimplemented),
    RlcH:  ("RLC H",    0x04, 8,  unimplemented),
    RlcL:  ("RLC L",    0x05, 8,  unimplemented),
    RlcHL: ("RLC (HL)", 0x06, 16, unimplemented),

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
    RlA:  ("RL A",    0x17, 8,  unimplemented),
    RlB:  ("RL B",    0x10, 8,  unimplemented),
    RlC:  ("RL C",    0x11, 8,  unimplemented),
    RlD:  ("RL D",    0x12, 8,  unimplemented),
    RlE:  ("RL E",    0x13, 8,  unimplemented),
    RlH:  ("RL H",    0x14, 8,  unimplemented),
    RlL:  ("RL L",    0x15, 8,  unimplemented),
    RlHL: ("RL (HL)", 0x16, 16, unimplemented),

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
    RrcA:  ("RRC A",    0x0F, 8,  unimplemented),
    RrcB:  ("RRC B",    0x08, 8,  unimplemented),
    RrcC:  ("RRC C",    0x09, 8,  unimplemented),
    RrcD:  ("RRC D",    0x0A, 8,  unimplemented),
    RrcE:  ("RRC E",    0x0B, 8,  unimplemented),
    RrcH:  ("RRC H",    0x0C, 8,  unimplemented),
    RrcL:  ("RRC L",    0x0D, 8,  unimplemented),
    RrcHL: ("RRC (HL)", 0x0E, 16, unimplemented),

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
    RrA:  ("RR A",    0x1F, 8,  unimplemented),
    RrB:  ("RR B",    0x18, 8,  unimplemented),
    RrC:  ("RR C",    0x19, 8,  unimplemented),
    RrD:  ("RR D",    0x1A, 8,  unimplemented),
    RrE:  ("RR E",    0x1B, 8,  unimplemented),
    RrH:  ("RR H",    0x1C, 8,  unimplemented),
    RrL:  ("RR L",    0x1D, 8,  unimplemented),
    RrHL: ("RR (HL)", 0x1E, 16, unimplemented),

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
    SlaA:  ("SLA A",    0x27, 8,  unimplemented),
    SlaB:  ("SLA B",    0x20, 8,  unimplemented),
    SlaC:  ("SLA C",    0x21, 8,  unimplemented),
    SlaD:  ("SLA D",    0x22, 8,  unimplemented),
    SlaE:  ("SLA E",    0x23, 8,  unimplemented),
    SlaH:  ("SLA H",    0x24, 8,  unimplemented),
    SlaL:  ("SLA L",    0x25, 8,  unimplemented),
    SlaHL: ("SLA (HL)", 0x26, 16, unimplemented),

    // SRA n
    //
    // Shift n left into Carry. MSB doesn't change.
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    SraA:  ("SRA A",    0x2F, 8,  unimplemented),
    SraB:  ("SRA B",    0x28, 8,  unimplemented),
    SraC:  ("SRA C",    0x29, 8,  unimplemented),
    SraD:  ("SRA D",    0x2A, 8,  unimplemented),
    SraE:  ("SRA E",    0x2B, 8,  unimplemented),
    SraH:  ("SRA H",    0x2C, 8,  unimplemented),
    SraL:  ("SRA L",    0x2D, 8,  unimplemented),
    SraHL: ("SRA (HL)", 0x2E, 16, unimplemented),

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
    SrlA:  ("SRL A",    0x3F, 8,  unimplemented),
    SrlB:  ("SRL B",    0x38, 8,  unimplemented),
    SrlC:  ("SRL C",    0x39, 8,  unimplemented),
    SrlD:  ("SRL D",    0x3A, 8,  unimplemented),
    SrlE:  ("SRL E",    0x3B, 8,  unimplemented),
    SrlH:  ("SRL H",    0x3C, 8,  unimplemented),
    SrlL:  ("SRL L",    0x3D, 8,  unimplemented),
    SrlHL: ("SRL (HL)", 0x3E, 16, unimplemented),

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
    BitbA:  ("BIT b,A",    0x47, 8,  unimplemented),
    BitbB:  ("BIT b,B",    0x40, 8,  unimplemented),
    BitbC:  ("BIT b,C",    0x41, 8,  unimplemented),
    BitbD:  ("BIT b,D",    0x42, 8,  unimplemented),
    BitbE:  ("BIT b,E",    0x43, 8,  unimplemented),
    BitbH:  ("BIT b,H",    0x44, 8,  unimplemented),
    BitbL:  ("BIT b,L",    0x45, 8,  unimplemented),
    BitbHL: ("BIT b,(HL)", 0x46, 16, unimplemented),

    // SET b,r
    //
    // Set bit b in register r
    //
    // b = 0 - 7
    // r = A,B,C,D,E,H,L,(HL)
    SetbA:  ("SET b,A",    0xC7, 8,  unimplemented),
    SetbB:  ("SET b,B",    0xC0, 8,  unimplemented),
    SetbC:  ("SET b,C",    0xC1, 8,  unimplemented),
    SetbD:  ("SET b,D",    0xC2, 8,  unimplemented),
    SetbE:  ("SET b,E",    0xC3, 8,  unimplemented),
    SetbH:  ("SET b,H",    0xC4, 8,  unimplemented),
    SetbL:  ("SET b,L",    0xC5, 8,  unimplemented),
    SetbHL: ("SET b,(HL)", 0xC6, 16, unimplemented),

    // RES b,r
    //
    // Reset bit b in register r
    //
    // b = 0 - 7
    // r = A,B,C,D,E,H,L,(HL)
    ResbA:  ("RES b,A",    0x87, 8,  unimplemented),
    ResbB:  ("RES b,B",    0x80, 8,  unimplemented),
    ResbC:  ("RES b,C",    0x81, 8,  unimplemented),
    ResbD:  ("RES b,D",    0x82, 8,  unimplemented),
    ResbE:  ("RES b,E",    0x83, 8,  unimplemented),
    ResbH:  ("RES b,H",    0x84, 8,  unimplemented),
    ResbL:  ("RES b,L",    0x85, 8,  unimplemented),
    ResbHL: ("RES b,(HL)", 0x86, 16, unimplemented)
);
