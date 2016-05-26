use gb_proc::cpu::{Cpu, Handler, HandlerHolder, print_cpu_status};
use gb_proc::opcodes::OpCode;

use std::num::Wrapping;

struct MockHandlerHolder {
    memory: [u8; 512],
}

impl MockHandlerHolder {
    fn new() -> MockHandlerHolder {
        MockHandlerHolder {
            memory: [0; 512],
        }
    }
}

impl Handler for MockHandlerHolder {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, v: u8) {
        self.memory[address as usize] = v;
    }
}

impl HandlerHolder for MockHandlerHolder {
    fn get_handler_read(&self, address: u16) -> &Handler {
        self as &Handler
    }

    fn get_handler_write(&mut self, address: u16) -> &mut Handler {
        self as &mut Handler
    }
}

fn reset_all_registers(cpu: &mut Cpu) {
    cpu.set_A_reg(0x00);
    cpu.set_B_reg(0x00);
    cpu.set_C_reg(0x00);
    cpu.set_D_reg(0x00);
    cpu.set_E_reg(0x00);
    cpu.set_F_reg(0x00);
    cpu.set_L_reg(0x00);
    cpu.set_H_reg(0x00);
    cpu.set_PC(0x0000);
    cpu.reset_call_set_PC();

    cpu.set_SP(0x0100);
}

fn assert_all_reg_0(cpu: &Cpu) {
    assert_eq!(cpu.get_A_reg(), 0x00);
    assert_eq!(cpu.get_B_reg(), 0x00);
    assert_eq!(cpu.get_C_reg(), 0x00);
    assert_eq!(cpu.get_D_reg(), 0x00);
    assert_eq!(cpu.get_E_reg(), 0x00);
    assert_eq!(cpu.get_H_reg(), 0x00);
    assert_eq!(cpu.get_L_reg(), 0x00);
}

#[test]
/** Tests that a jump forward correctly jumps to the
 * right location. */
fn test_jr_n_forward() {
    let mut handler = MockHandlerHolder::new();
    // JR
    handler.memory[0] = 0x18;
    // $+05
    handler.memory[1] = 0x03;
    // LD A, (nn)
    handler.memory[2] = 0x3A;
    // nn = 0x0001
    handler.memory[3] = 0x00;
    handler.memory[4] = 0x01;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    print_cpu_status(&cpu);
    cpu.next_instruction();

    print_cpu_status(&cpu);
    assert_all_reg_0(&cpu);
    assert_eq!(cpu.get_PC(), 0x0005);
}

#[test]
fn test_jr_n_backwards() {
    let mut handler = MockHandlerHolder::new();
    // 0..3 = 0x00
    // JR
    handler.memory[4] = 0x18;
    // $-05
    handler.memory[5] = 0xFA;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_PC(0x0004);
    cpu.reset_call_set_PC();

    print_cpu_status(&cpu);
    cpu.next_instruction();

    print_cpu_status(&cpu);
    assert_all_reg_0(&cpu);
    assert_eq!(cpu.get_PC(), 0x0000);
}

fn test_half_carry(opcode: u8, a: u8, b: u8, expected: bool) {
    println!("Half carry for {:02X}, {:02X}", a, b);
    let mut handler = MockHandlerHolder::new();

    handler.memory[0x0000] = opcode;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_A_reg(a);
    cpu.set_B_reg(b);

    cpu.next_instruction();

    assert_eq!(cpu.get_H_flag(), expected);
    assert_eq!(cpu.get_A_reg(), (Wrapping(a) + Wrapping(b)).0);

    cpu.reset_C();
}

fn add_n(a: u8, n: u8, H: bool, C: bool, Z: bool) {
    let mut handler = MockHandlerHolder::new();

    handler.memory[0x0000] = 0xC6;
    handler.memory[0x0001] = n;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_A_reg(a);

    cpu.next_instruction();

    assert_eq!(cpu.get_A_reg(), (Wrapping(a) + Wrapping(n)).0);
    assert_eq!(cpu.get_H_flag(), H);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_C_flag(), C);
    assert_eq!(cpu.get_Z_flag(), Z);
}

#[test]
fn test_pop_XY() {
    for a in [0xC1, 0xD1, 0xE1, 0xF1].into_iter() {
        let mut handler = MockHandlerHolder::new();

        handler.memory[0x0000] = *a;

        // Stack data
        handler.memory[0x00FF] = 0x01;
        handler.memory[0x00FE] = 0xF0;

        let mut cpu = Cpu::new(Box::new(handler));
        reset_all_registers(&mut cpu);

        cpu.set_SP(0x00FE);

        cpu.next_instruction();

        match *a {
            // POP BC
            0xC1 => {
                assert_eq!(cpu.get_B_reg(), 0x01);
                assert_eq!(cpu.get_C_reg(), 0xF0);
            },
            // POP DE
            0xD1 => {
                assert_eq!(cpu.get_D_reg(), 0x01);
                assert_eq!(cpu.get_E_reg(), 0xF0);
            },
            // POP HL
            0xE1 => {
                assert_eq!(cpu.get_H_reg(), 0x01);
                assert_eq!(cpu.get_L_reg(), 0xF0);
            },
            // POP AF
            0xF1 => {
                assert_eq!(cpu.get_A_reg(), 0x01);
                assert_eq!(cpu.get_F_reg(), 0xF0);
            }
            _ => panic!(),
        }

        println!("SP = {:04X}", cpu.get_SP());
        assert_eq!(cpu.get_SP(), 0x0100);
    }
}
#[test]
fn test_push_XY() {
    for a in [0xC5, 0xD5, 0xE5, 0xF5].into_iter() {
        let mut handler = MockHandlerHolder::new();

        handler.memory[0x0000] = *a;

        let mut cpu = Cpu::new(Box::new(handler));
        reset_all_registers(&mut cpu);

        match *a {
            // PUSH BC
            0xC5 => {
                cpu.set_B_reg(0x01);
                cpu.set_C_reg(0xF0);
            },
            // PUSH DE
            0xD5 => {
                cpu.set_D_reg(0x01);
                cpu.set_E_reg(0xF0);
            },
            // PUSH HL
            0xE5 => {
                cpu.set_H_reg(0x01);
                cpu.set_L_reg(0xF0);
            },
            // PUSH AF
            0xF5 => {
                cpu.set_A_reg(0x01);
                cpu.set_F_reg(0xF0);
            }
            _ => panic!(),
        }

        cpu.next_instruction();

        println!("SP = {:04X}", cpu.get_SP());
        assert_eq!(cpu.get_SP(), 0x00FE);
        assert_eq!(cpu.deref(0x00FF), 0x01);
        assert_eq!(cpu.deref(0x00FE), 0xF0);
    }
}

#[test]
fn test_add_n() {
    add_n(0xF8, 0xF8,  true,  true, false);
    add_n(0x08, 0x08,  true, false, false);
    add_n(0x0F, 0x01,  true, false, false);
    add_n(0xF0, 0x01, false, false, false);
    add_n(0x00, 0x00, false, false, true);
    add_n(0xFF, 0x01,  true,  true, true);
    add_n(0x01, 0xFF,  true,  true, true);
    add_n(0x00, 0xFF, false, false, false);
}

#[test]
fn test_add_half_carry() {
    // $80 is ADD A,B
    test_half_carry(0x80, 0x0F, 0x0F, true);
    test_half_carry(0x80, 0x08, 0x08, true);
    test_half_carry(0x80, 0xE2, 0x0E, true);
    test_half_carry(0x80, 0x01, 0xFF, true);
    test_half_carry(0x80, 0xFF, 0x01, true);
    test_half_carry(0x80, 0x07, 0x08, false);
    test_half_carry(0x80, 0x10, 0x1F, false);
    test_half_carry(0x80, 0xFF, 0x00, false);
    test_half_carry(0x80, 0xF0, 0x0F, false);
    test_half_carry(0x80, 0x01, 0xFF, true);
}

#[test]
fn test_adc_half_carry() {
    // $88 is ADC A,B
    test_half_carry(0x88, 0x0F, 0x0F, true);
    test_half_carry(0x88, 0x08, 0x08, true);
    test_half_carry(0x88, 0xE2, 0x0E, true);
    test_half_carry(0x88, 0xFF, 0x01, true);
    test_half_carry(0x88, 0x01, 0xFF, true);
    test_half_carry(0x88, 0x07, 0x08, false);
    test_half_carry(0x88, 0x10, 0x1F, false);
    test_half_carry(0x88, 0xFF, 0x00, false);
    test_half_carry(0x88, 0xF0, 0x0F, false);
}

#[test]
fn test_add_A_X() {
    for a in 0x80 .. 0x88 {
        let mut handler = MockHandlerHolder::new();
        handler.memory[0x0000] = a;
        handler.memory[0x0001] = 0x08;

        println!("Testing {:02X} {:?}", a, OpCode::from_byte(a, false));
        let mut cpu = Cpu::new(Box::new(handler));
        cpu.set_A_reg(0x01);
        cpu.set_B_reg(0x02);
        cpu.set_C_reg(0x03);
        cpu.set_D_reg(0x04);
        cpu.set_E_reg(0x05);
        cpu.set_H_reg(0x06);
        cpu.set_L_reg(0x07);
        cpu.set_SP(0x0100);
        cpu.set_PC(0x0000);
        cpu.reset_call_set_PC();

        if a == 0x86 {
            // This is for ADC A,(HL)
            cpu.set_H_reg(0x00);
            cpu.set_L_reg(0x01);
        }

        let expected = match a {
            // ADC A, B
            0x80 => 0x03,
            // ADC A, C
            0x81 => 0x04,
            // ADC A, D
            0x82 => 0x05,
            // ADC A, E
            0x83 => 0x06,
            // ADC A, H
            0x84 => 0x07,
            // ADC A, L
            0x85 => 0x08,
            // ADC A, (HL)
            0x86 => 0x09,
            // ADC A, A
            0x87 => 0x02,
            _    => panic!(),
        };

        cpu.next_instruction();
        assert_eq!(cpu.get_A_reg(), expected);
        assert_eq!(cpu.get_Z_flag(), false);
        assert_eq!(cpu.get_H_flag(), false);
        assert_eq!(cpu.get_N_flag(), false);
        assert_eq!(cpu.get_C_flag(), false);
    }
}

#[test]
fn test_adc_A_X() {
    for a in 0x88 .. 0x90 {
        for carry_flag in [false, true].into_iter() {
            let mut handler = MockHandlerHolder::new();
            handler.memory[0x0000] = a;
            handler.memory[0x0001] = 0x08;

            println!("Testing {:02X} {:?}", a, OpCode::from_byte(a, false));
            let mut cpu = Cpu::new(Box::new(handler));
            cpu.set_A_reg(0x01);
            cpu.set_B_reg(0x02);
            cpu.set_C_reg(0x03);
            cpu.set_D_reg(0x04);
            cpu.set_E_reg(0x05);
            cpu.set_H_reg(0x06);
            cpu.set_L_reg(0x07);
            cpu.set_SP(0x0100);
            cpu.set_PC(0x0000);
            cpu.reset_call_set_PC();

            if a == 0x8E {
                // This is for ADC A,(HL)
                cpu.set_H_reg(0x00);
                cpu.set_L_reg(0x01);
            }

            if *carry_flag {
                cpu.set_C_flag();
            } else {
                cpu.reset_C();
            }

            let expected = match a {
                // ADC A, B
                0x88 => 0x03,
                // ADC A, C
                0x89 => 0x04,
                // ADC A, D
                0x8A => 0x05,
                // ADC A, E
                0x8B => 0x06,
                // ADC A, H
                0x8C => 0x07,
                // ADC A, L
                0x8D => 0x08,
                // ADC A, (HL)
                0x8E => 0x09,
                // ADC A, A
                0x8F => 0x02,
                _    => panic!(),
            } + (if *carry_flag { 1 } else { 0 });

            cpu.next_instruction();
            assert_eq!(cpu.get_A_reg(), expected);
            assert_eq!(cpu.get_Z_flag(), false);
            assert_eq!(cpu.get_N_flag(), false);
            assert_eq!(cpu.get_C_flag(), false);
        }
    }
}

#[test]
fn test_ld_X_Y() {
    // From LD B,B to LD A,A
    for a in 0x40 .. 0x80 {
        if a == 0x76 {
            // 0x76 is HALT
            continue;
        }

        let mut handler = MockHandlerHolder::new();
        handler.memory[0] = a;
        handler.memory[0x08] = 0x08;

        let mut cpu = Cpu::new(Box::new(handler));
        cpu.set_A_reg(0x01);
        cpu.set_B_reg(0x02);
        cpu.set_C_reg(0x03);
        cpu.set_D_reg(0x04);
        cpu.set_E_reg(0x05);
        cpu.set_H_reg(0x06);
        cpu.set_L_reg(0x07);
        cpu.set_SP(0x0100);
        cpu.set_PC(0x0000);
        cpu.reset_call_set_PC();


        // This is testing (HL) so we need to set up HL to point
        // inside the memory
        match a {
            0x46 | 0x56 | 0x66 | 0x4E | 0x5E | 0x6E |
                0x70 ... 0x75 | 0x77 | 0x7E => {
                    cpu.set_H_reg(0x00);
                    cpu.set_L_reg(0x08);
            }
            _ => {},
        }

        println!("Testing {:02X} {:?}", a, OpCode::from_byte(a, false));

        print_cpu_status(&cpu);
        cpu.next_instruction();
        print_cpu_status(&cpu);

        assert_eq!(cpu.get_PC(), 0x0001);
        assert_eq!(cpu.get_SP(), 0x0100);
        assert_eq!(cpu.get_Z_flag(), false);
        assert_eq!(cpu.get_H_flag(), false);
        assert_eq!(cpu.get_N_flag(), false);
        assert_eq!(cpu.get_C_flag(), false);

        let expected = match a {
            0x40 | 0x50 | 0x60 | 0x70 | 0x48 | 0x58 | 0x68 | 0x78 => 0x02,
            0x41 | 0x51 | 0x61 | 0x71 | 0x49 | 0x59 | 0x69 | 0x79 => 0x03,
            0x42 | 0x52 | 0x62 | 0x72 | 0x4A | 0x5A | 0x6A | 0x7A => 0x04,
            0x43 | 0x53 | 0x63 | 0x73 | 0x4B | 0x5B | 0x6B | 0x7B => 0x05,
            0x44 | 0x54 | 0x64 |        0x4C | 0x5C | 0x6C | 0x7C => 0x06,
            0x74                                                  => 0x00,
            0x75                                                  => 0x08,
            0x45 | 0x55 | 0x65 |        0x4D | 0x5D | 0x6D | 0x7D => 0x07,
            0x46 | 0x56 | 0x66 |        0x4E | 0x5E | 0x6E | 0x7E => 0x08,
            0x47 | 0x57 | 0x67 | 0x77 | 0x4F | 0x5F | 0x6F | 0x7F => 0x01,
            _ => panic!(),
        };

        match a {
            0x40 ... 0x47 => assert_eq!(cpu.get_B_reg(), expected),
            0x48 ... 0x4F => assert_eq!(cpu.get_C_reg(), expected),
            0x50 ... 0x57 => assert_eq!(cpu.get_D_reg(), expected),
            0x58 ... 0x5F => assert_eq!(cpu.get_E_reg(), expected),
            0x60 ... 0x67 => assert_eq!(cpu.get_H_reg(), expected),
            0x68 ... 0x6F => assert_eq!(cpu.get_L_reg(), expected),
            0x70 ... 0x77 => assert_eq!(cpu.deref(0x0008), expected),
            0x78 ... 0x7F => assert_eq!(cpu.get_A_reg(), expected),
            _ => panic!(),
        }
    }
}
