use std::fs::File;
use std::io::Read;

pub mod gb_proc;

use self::gb_proc::cartridge::Cartridge;
use self::gb_proc::cpu::{Cpu, CpuState};
use self::gb_proc::opcodes::OpCode;

use std::io;

pub fn main() {
    let mut f = File::open("rom.gb").unwrap();
    let cartridge = Cartridge::from_file(&mut f);

    let mut cpu = Cpu::new(cartridge);

    loop {
        let hex = cpu.deref_PC();

        let mut pc = cpu.get_PC();
        let op = if hex == 0xCB {
            cpu.inc_PC();
            pc = cpu.get_PC();
            OpCode::from_byte(cpu.deref_PC(), true)
        } else {
            OpCode::from_byte(hex, false)
        };

        println!("{:04X} [{:02X}] {}", pc, hex, op.to_string());
        op.execute(&mut cpu);

        if !cpu.did_call_set_PC() {
            // No jump happened so we need to increase PC
            cpu.inc_PC();
        } else {
            cpu.reset_call_set_PC();
        }

        print_cpu_status(&cpu);

        if cpu.get_PC() == 0x38 || cpu.get_state() != &CpuState::Running {
            io::stdin().read_line(&mut String::new()).unwrap();
        }
    }
}

fn print_cpu_status(cpu: &Cpu) {
    println!("[Z,N,H,C] = [{},{},{},{}]",
             cpu.get_Z_flag(),
             cpu.get_N_flag(),
             cpu.get_H_flag(),
             cpu.get_C_flag());

    println!("A = ${:02X}",  cpu.get_A_reg());
    println!("B = ${:02X}",  cpu.get_B_reg());
    println!("C = ${:02X}",  cpu.get_C_reg());
    println!("D = ${:02X}",  cpu.get_D_reg());
    println!("E = ${:02X}",  cpu.get_E_reg());
    println!("F = ${:02X}",  cpu.get_F_reg());
    println!("H = ${:02X}",  cpu.get_H_reg());
    println!("L = ${:02X}",  cpu.get_L_reg());
    println!("PC = ${:02X}", cpu.get_PC());
    println!("SP = ${:02X}", cpu.get_SP());
    println!("state = {:?}", cpu.get_state());
    println!("");
}
