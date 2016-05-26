use std::fs::File;
use std::io::Read;

pub mod gb_proc;

use self::gb_proc::handler_holder::GBHandlerHolder;
use self::gb_proc::cartridge::Cartridge;
use self::gb_proc::cpu::{Cpu, CpuState, print_cpu_status};
use self::gb_proc::opcodes::OpCode;

use std::io;

#[cfg(test)]
mod tests;

pub fn main() {
    let mut f = File::open("rom.gb").unwrap();
    let cartridge = Cartridge::from_file(&mut f);

    let handler = GBHandlerHolder::new(cartridge);
    let mut cpu = Cpu::new(Box::new(handler));
    cpu.set_debug(false);

    let mut stepping = false;
    loop {
        cpu.next_instruction();

        if cpu.get_debug() {
            print_cpu_status(&cpu);
        }

        if cpu.get_PC() == 0x20B6 || stepping {
            cpu.set_debug(true);
            stepping = true;
            io::stdin().read_line(&mut String::new()).unwrap();
        }
    }
}
