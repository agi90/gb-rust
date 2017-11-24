use hardware::cpu::{Cpu, Handler, HandlerHolder, Interrupt};
use hardware::opcodes::OpCode;
use hardware::ppu::{ScreenBuffer, GrayShade};
use hardware::apu::*;

use hardware::handler_holder::Key;

use std::num::Wrapping;

struct MockChannel;

impl VolumeView for MockChannel {
    fn volume(&self) -> u8 { 0 }
}

impl OutputLevelView for MockChannel {
    fn volume(&self) -> OutputLevel { OutputLevel::Mute }
}

impl WaveDutyView for MockChannel {
    fn wave_duty(&self) -> f32 { 0.0 }
}

impl WavePatternView for MockChannel {
    fn wave_pattern(&self) -> &[u8] { EMPTY_ARRAY }
}

impl PatternView for MockChannel {
    fn pattern(&self) -> NoisePattern { NoisePattern::C15 }
}

impl AudioLineView for MockChannel {
    fn playing_left(&self) -> bool { false }
    fn playing_right(&self) -> bool { false }
    fn frequency(&self) -> u64 { 0 }
}

const EMPTY_ARRAY : &'static [u8] = &[];
const MOCK_CHANNEL : &'static MockChannel = &MockChannel {};

struct MockHandlerHolder {
    memory: [u8; 512],
    screen_buffer: ScreenBuffer,
    audio_buffer: MockAudioBuffer,
    data: [u8; 1],
}

struct MockAudioBuffer;

impl AudioBuffer for MockAudioBuffer {
    fn sound_1(&self) -> &Channel1View { MOCK_CHANNEL }
    fn sound_2(&self) -> &Channel2View { MOCK_CHANNEL }
    fn sound_3(&self) -> &Channel3View { MOCK_CHANNEL }
    fn sound_4(&self) -> &Channel4View { MOCK_CHANNEL }
}

impl MockHandlerHolder {
    fn new() -> MockHandlerHolder {
        MockHandlerHolder {
            memory: [0; 512],
            screen_buffer: [[GrayShade::C00; 160]; 144],
            audio_buffer: MockAudioBuffer{},
            data: [0],
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
    fn get_handler_read(&self, _: u16) -> &Handler {
        self as &Handler
    }

    fn get_handler_write(&mut self, _: u16) -> &mut Handler {
        self as &mut Handler
    }
    fn cpu_step(&mut self) {}
    fn check_interrupts(&mut self) -> Option<Interrupt> { None }
    fn key_down(&mut self, _: Key) {}
    fn key_up(&mut self, _: Key) {}
    fn get_screen_buffer(&self) -> &ScreenBuffer {
        &self.screen_buffer
    }
    fn should_refresh(&mut self) -> bool { false }
    fn get_audio_buffer(&self) -> &AudioBuffer {
        &self.audio_buffer
    }
    fn reset(&mut self) {}
    fn ram(&mut self) -> &mut [u8] { &mut self.data }
    fn rtc(&mut self) -> Option<&mut i64> { None }
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

    cpu.next_instruction();

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

    cpu.next_instruction();

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
    println!("Testing Add n A={:02X} n={:02X}", a, n);
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

        cpu.next_instruction();

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

#[test]
fn test_inc() {
    inc(0x01, 0x02, 0x01, 0x03);
    inc(0x01, 0xFF, 0x02, 0x00);
    inc(0xFF, 0xFF, 0x00, 0x00);
    inc(0x00, 0x00, 0x00, 0x01);
    inc(0xFF, 0x00, 0xFF, 0x01);
}

fn inc(x: u8, y: u8, exp_x: u8, exp_y: u8) {
    for a in [0x03, 0x13, 0x23, 0x33].into_iter() {
        let mut handler = MockHandlerHolder::new();
        handler.memory[0x0000] = *a;

        let mut cpu = Cpu::new(Box::new(handler));
        reset_all_registers(&mut cpu);

        match *a {
            0x03 => {
                cpu.set_B_reg(x);
                cpu.set_C_reg(y);
            },
            0x13 => {
                cpu.set_D_reg(x);
                cpu.set_E_reg(y);
            },
            0x23 => {
                cpu.set_H_reg(x);
                cpu.set_L_reg(y);
            },
            0x33 => {
                cpu.set_SP((y as u16) + ((x as u16) << 8));
            },
            _ => panic!(),
        }

        cpu.next_instruction();

        match *a {
            0x03 => {
                assert_eq!(cpu.get_B_reg(), exp_x);
                assert_eq!(cpu.get_C_reg(), exp_y);
            },
            0x13 => {
                assert_eq!(cpu.get_D_reg(), exp_x);
                assert_eq!(cpu.get_E_reg(), exp_y);
            },
            0x23 => {
                assert_eq!(cpu.get_H_reg(), exp_x);
                assert_eq!(cpu.get_L_reg(), exp_y);
            },
            0x33 => {
                assert_eq!(cpu.get_SP(), (exp_y as u16) + ((exp_x as u16) << 8));
            },
            _ => panic!(),
        }

        assert_eq!(cpu.get_Z_flag(), false);
        assert_eq!(cpu.get_H_flag(), false);
        assert_eq!(cpu.get_N_flag(), false);
        assert_eq!(cpu.get_C_flag(), false);
    }
}

#[test]
fn test_call_nn() {
    let mut handler = MockHandlerHolder::new();
    // CALL nn
    handler.memory[0x0000] = 0xCD;
    handler.memory[0x0001] = 0x10;
    handler.memory[0x0002] = 0x00;

    // RET
    handler.memory[0x0010] = 0xC9;

    // This will be overwritten by CALL $0010
    handler.memory[0x00FF] = 0xFF;
    handler.memory[0x00FE] = 0xFF;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    // Execute CALL
    cpu.next_instruction();
    assert_eq!(cpu.get_PC(), 0x0010);
    assert_eq!(cpu.get_SP(), 0x00FE);
    assert_eq!(cpu.deref(0x00FF), 0x00);
    assert_eq!(cpu.deref(0x00FE), 0x03);
    assert_eq!(cpu.get_Z_flag(), false);
    assert_eq!(cpu.get_H_flag(), false);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_C_flag(), false);

    // Execute RET
    cpu.next_instruction();
    assert_eq!(cpu.get_PC(), 0x0003);
    assert_eq!(cpu.get_SP(), 0x0100);
    assert_eq!(cpu.get_Z_flag(), false);
    assert_eq!(cpu.get_H_flag(), false);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_C_flag(), false);
}

fn and(a: u8, x: u8, expected: u8) {
    for op in 0xA0 .. 0xA7 {
        println!("Testing {} a={:02X} x={:02X} expected={:02X}",
                 OpCode::from_byte(op, false).to_string(), a, x, expected);

        let mut handler = MockHandlerHolder::new();
        handler.memory[0x0000] = op;
        handler.memory[0x0010] = x;

        let mut cpu = Cpu::new(Box::new(handler));
        reset_all_registers(&mut cpu);

        cpu.set_A_reg(a);

        match op {
            0xA0 => { cpu.set_B_reg(x); },
            0xA1 => { cpu.set_C_reg(x); },
            0xA2 => { cpu.set_D_reg(x); },
            0xA3 => { cpu.set_E_reg(x); },
            0xA4 => { cpu.set_H_reg(x); },
            0xA5 => { cpu.set_L_reg(x); },
            // AND (HL)
            0xA6 => { cpu.set_H_reg(0x00); cpu.set_L_reg(0x10);
            },
            _ => panic!(),
        }

        cpu.next_instruction();

        assert_eq!(cpu.get_A_reg(), expected);

        assert_eq!(cpu.get_Z_flag(), expected == 0);
        assert_eq!(cpu.get_H_flag(), true);
        assert_eq!(cpu.get_C_flag(), false);
        assert_eq!(cpu.get_N_flag(), false);
    }
}

#[test]
fn test_and() {
    and(0x01, 0xFF, 0x01);
    and(0xFF, 0xFF, 0xFF);
    and(0xF0, 0x0F, 0x00);
}

fn add_HL_BC(hl: u16, bc: u16, expected: u16, h: bool, c: bool) {
    println!("Testing ADD HL={:04X}, BC={:04X}", hl, bc);
    let mut handler = MockHandlerHolder::new();
    handler.memory[0x0000] = 0x09;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_HL(hl);
    cpu.set_BC(bc);

    cpu.next_instruction();

    assert_eq!(cpu.get_HL(), expected);
    assert_eq!(cpu.get_BC(), bc);
    assert_eq!(cpu.get_Z_flag(), false);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_H_flag(), h);
    assert_eq!(cpu.get_C_flag(), c);
}

#[test]
fn test_add_HL_BC() {
    add_HL_BC(0x0FFF, 0x0001, 0x1000, true, false);
    add_HL_BC(0x0800, 0x0800, 0x1000, true, false);
    add_HL_BC(0x0001, 0x0FFF, 0x1000, true, false);
    add_HL_BC(0x8000, 0x8000, 0x0000, false, true);
    add_HL_BC(0xFFFF, 0x0001, 0x0000, true, true);
    add_HL_BC(0x0001, 0xFFFF, 0x0000, true, true);
    add_HL_BC(0x0080, 0x0080, 0x0100, false, false);
}

fn add_SP_n(sp: u16, n: u8, expected: u16, h: bool, c: bool) {
    println!("Testing SP={:04X} n={:02X}", sp, n);
    let mut handler = MockHandlerHolder::new();
    handler.memory[0x0000] = 0xE8;
    handler.memory[0x0001] = n;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_SP(sp);

    cpu.next_instruction();

    assert_eq!(cpu.get_SP(), expected);
    assert_eq!(cpu.get_Z_flag(), false);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_H_flag(), h);
    assert_eq!(cpu.get_C_flag(), c);
}

#[test]
fn test_add_SP_n() {
    add_SP_n(0x000F, 0x01, 0x0010, true, false);
    add_SP_n(0x0008, 0x08, 0x0010, true, false);
    add_SP_n(0x000F, 0x01, 0x0010, true, false);
add_SP_n(0x0001, 0xFF, 0x0000, true, true);
    add_SP_n(0x00FF, 0x01, 0x0100, true, true);
    add_SP_n(0x0080, 0x80, 0x0000, false, true);

    add_SP_n(0x0000, 0xFF, 0xFFFF, false, false);
    add_SP_n(0x0000, 0x7F, 0x007F, false, false);
    add_SP_n(0x0001, 0x7F, 0x0080, true, false);
}

fn add_HL_SP_n(sp: u16, n: u8, expected: u16, h: bool, c: bool) {
    println!("Testing SP={:04X} n={:02X}", sp, n);
    let mut handler = MockHandlerHolder::new();
    handler.memory[0x0000] = 0xF8;
    handler.memory[0x0001] = n;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_SP(sp);

    cpu.next_instruction();

    assert_eq!(cpu.get_SP(), sp);
    assert_eq!(cpu.get_HL(), expected);
    assert_eq!(cpu.get_Z_flag(), false);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_H_flag(), h);
    assert_eq!(cpu.get_C_flag(), c);
}

#[test]
fn test_add_HL_SP_n() {
    add_HL_SP_n(0x000F, 0x01, 0x0010, true, false);
    add_HL_SP_n(0x0008, 0x08, 0x0010, true, false);
    add_HL_SP_n(0x000F, 0x01, 0x0010, true, false);

    add_HL_SP_n(0x0001, 0xFF, 0x0000, true, true);
    add_HL_SP_n(0x00FF, 0x01, 0x0100, true, true);
    add_HL_SP_n(0x0080, 0x80, 0x0000, false, true);

    add_HL_SP_n(0x0000, 0xFF, 0xFFFF, false, false);
    add_HL_SP_n(0x0000, 0x7F, 0x007F, false, false);
    add_HL_SP_n(0x0001, 0x7F, 0x0080, true, false);
}

fn sbc_A(a: u8, b: u8, expected: u8, init_C: bool, Z: bool, H: bool, C: bool) {
    println!("Testing SBC A={:02X}, B={:02X}", a, b);
    let mut handler = MockHandlerHolder::new();
    handler.memory[0x0000] = 0x98;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_A_reg(a);
    cpu.set_B_reg(b);
    if init_C {
        cpu.set_C_flag();
    }

    cpu.next_instruction();

    assert_eq!(cpu.get_A_reg(), expected);
    assert_eq!(cpu.get_N_flag(), true);
    assert_eq!(cpu.get_Z_flag(), Z);
    assert_eq!(cpu.get_C_flag(), C);
    assert_eq!(cpu.get_H_flag(), H);
}

#[test]
fn test_sbc_A_B() {
    //    A     B     exp   init_C Z      H      C
    sbc_A(0x00, 0x00, 0x00, false, true,  false, false);
    sbc_A(0x01, 0x00, 0x00, true,  true,  false, false);
    sbc_A(0x00, 0x00, 0xFF, true,  false, true,  true);
    sbc_A(0x10, 0x00, 0x0F, true,  false, true,  false);
    sbc_A(0x10, 0x1F, 0xF0, true,  false, true,  true);
    sbc_A(0x11, 0x1F, 0xF2, false, false, true,  true);
    sbc_A(0xFF, 0x00, 0xFE, true,  false, false, false);
    sbc_A(0x80, 0x7F, 0x00, true,  true,  true,  false);
    sbc_A(0x81, 0x7F, 0x01, true,  false, true,  false);
    sbc_A(0x81, 0x70, 0x10, true,  false, false, false);
}

fn rlc_A(a: u8, expected: u8, C: bool) {
    println!("Testing RLC A={:02X}", a);
    let mut handler = MockHandlerHolder::new();
    handler.memory[0x0000] = 0x07;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_A_reg(a);

    cpu.next_instruction();

    assert_eq!(cpu.get_A_reg(), expected);
    assert_eq!(cpu.get_N_flag(), false);
    assert_eq!(cpu.get_H_flag(), false);
    assert_eq!(cpu.get_Z_flag(), false);
    assert_eq!(cpu.get_C_flag(), C);
}

#[test]
fn test_rlc_A() {
    rlc_A(0b11111111, 0b11111111, true);
    rlc_A(0b00000001, 0b00000010, false);
    rlc_A(0b00000010, 0b00000100, false);
    rlc_A(0b00000100, 0b00001000, false);
    rlc_A(0b00001000, 0b00010000, false);
    rlc_A(0b00010000, 0b00100000, false);
    rlc_A(0b00100000, 0b01000000, false);
    rlc_A(0b01000000, 0b10000000, false);
    rlc_A(0b10000000, 0b00000001, true);

    rlc_A(0b10000001, 0b00000011, true);
    rlc_A(0b10000010, 0b00000101, true);
    rlc_A(0b10000100, 0b00001001, true);
    rlc_A(0b10001000, 0b00010001, true);
    rlc_A(0b10010000, 0b00100001, true);
    rlc_A(0b10100000, 0b01000001, true);
    rlc_A(0b11000000, 0b10000001, true);

    rlc_A(0b00000000, 0b00000000, false);
}

fn daa(a: u8, expected: u8, C: bool, H: bool, N: bool, exp_C: bool, exp_Z: bool) {
    println!("Testing DAA A={:02X}", a);
    let mut handler = MockHandlerHolder::new();
    handler.memory[0x0000] = 0x27;

    let mut cpu = Cpu::new(Box::new(handler));
    reset_all_registers(&mut cpu);

    cpu.set_A_reg(a);
    if C { cpu.set_C_flag(); }
    if H { cpu.set_H_flag(); }
    if N { cpu.set_N_flag(); }

    cpu.next_instruction();

    assert_eq!(cpu.get_A_reg(), expected);
    assert_eq!(cpu.get_N_flag(), N);
    assert_eq!(cpu.get_H_flag(), false);
    assert_eq!(cpu.get_Z_flag(), exp_Z);
    assert_eq!(cpu.get_C_flag(), exp_C);
}

#[test]
fn test_daa() {
    //              C      H      N      exp_C  exp_Z
    for j in 0x0..0xA {
        for i in 0x0..0xA {
            let x = i + (j << 4);
            daa(x, x, false, false, false, false, x == 0);
        }
    }

    for j in 0x0..0x9 {
        for i in 0xA..0x10 {
            let x = i + (j << 4);
            daa(x, (x as u16 + 0x06) as u8, false, false, false, false, false);
        }
    }

    for j in 0x0..0xA {
        for i in 0x0..0x4 {
            let x = i + (j << 4);
            daa(x, (x as u16 + 0x06) as u8, false, true,  false, false, false);
        }
    }

    for j in 0xA..0x10 {
        for i in 0x0..0xA {
            let x = i + (j << 4);
            let result = (x as u16 + 0x60) as u8;
            daa(x, result, false, false,  false, true, result == 0);
        }
    }

    for j in 0x9..0x10 {
        for i in 0xA..0x10 {
            let x = i + (j << 4);
            let result = (x as u16 + 0x66) as u8;
            daa(x, result, false, false, false, true, result == 0);
        }
    }

    for j in 0xA..0x10 {
        for i in 0x0..0x4 {
            let x = i + (j << 4);
            let result = (x as u16 + 0x66) as u8;
            daa(x, result, false, true, false, true, result == 0);
        }
    }

    for j in 0x0..0x03 {
        for i in 0x0..0xA {
            let x = i + (j << 4);
            let result = (x as u16 + 0x60) as u8;
            daa(x, result, true, false, false, true, result == 0);
        }
    }

    for j in 0x0..0x03 {
        for i in 0xA..0x10 {
            let x = i + (j << 4);
            let result = (x as u16 + 0x66) as u8;
            daa(x, result, true, false, false, true, result == 0);
        }
    }

    for j in 0x0..0x04 {
        for i in 0xA..0x04 {
            let x = i + (j << 4);
            let result = (x as u16 + 0x66) as u8;
            daa(x, result, true, true, false, true, result == 0);
        }
    }

    for j in 0x0..0x0A {
        for i in 0x0..0x0A {
            let x = i + (j << 4);
            let result = (x as u16 + 0x00) as u8;
            daa(x, result, false, false, true, false, result == 0);
        }
    }

    for j in 0x0..0x09 {
        for i in 0x6..0x10 {
            let x = i + (j << 4);
            let result = (x as u16 + 0xFA) as u8;
            daa(x, result, false, true, true, false, result == 0);
        }
    }

    for j in 0x7..0x10 {
        for i in 0x0..0x0A {
            let x = i + (j << 4);
            let result = (x as u16 + 0xA0) as u8;
            daa(x, result, true, false, true, true, result == 0);
        }
    }

    for j in 0x6..0x08 {
        for i in 0x6..0x10 {
            let x = i + (j << 4);
            let result = (x as u16 + 0x9A) as u8;
            daa(x, result, true, true, true, true, result == 0);
        }
    }
}
