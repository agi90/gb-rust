use std::fs::File;
use std::io::{Read, Write};
use std::process;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::thread;

#[macro_use]
extern crate glium;

pub mod gb_proc;
pub mod gpu;
pub mod controller;

use self::gb_proc::handler_holder::GBHandlerHolder;
use self::gb_proc::cartridge::{BootRom, Cartridge};
use self::gb_proc::video_controller::GrayShade;
use self::gb_proc::cpu::{Cpu, CpuState, Interrupt, print_cpu_status};
use self::gpu::renderer::{Renderer, GLRenderer};
use self::gb_proc::opcodes::OpCode;
use self::controller::{Controller, Hardware, Key};

use std::io;

#[cfg(test)]
mod tests;

// Used for debugging
struct NullRenderer;

impl Renderer for NullRenderer {
    // fn build_buffer(&mut self, ) {}
    fn refresh(&mut self, pixels: &[[GrayShade; 160]; 144]) {}
}

struct HardwareGlue {
    cpu: Cpu,
    handler_holder: GBHandlerHolder,
}

impl HardwareGlue {
    pub fn new(cpu: Cpu, handler_holder: GBHandlerHolder) -> HardwareGlue {
        HardwareGlue {
            cpu: cpu,
            handler_holder: handler_holder,
        }
    }
}

impl Hardware for HardwareGlue {
    fn interrupt(&mut self, interrupt: Interrupt) {
        self.cpu.request_interrupt(interrupt);
    }

    fn key_up(&mut self, key: Key) {
    }

    fn key_down(&mut self, key: Key) {
    }

    fn next(&mut self) {
        self.cpu.next_instruction();
    }
}

pub fn main() {
    let mut f = File::open("rom.gb").unwrap();
    let cartridge = Cartridge::from_file(&mut f);

    let mut controller = Controller::new();;

    let handler = GBHandlerHolder::new(Box::new(cartridge));
    let mut cpu = Cpu::new(Box::new(handler));

    cpu.set_debug(false);

    let mut stepping = false;
    let mut address_breakpoints = HashSet::new();
    let mut op_breakpoints: HashSet<u8> = HashSet::new();

    let mut last_tick = 0;

    loop {
        if (cpu.get_PC() == 0x100 && cpu.get_debug()) ||
            address_breakpoints.contains(&cpu.get_PC()) ||
                op_breakpoints.contains(&cpu.deref_PC()) {
            cpu.set_debug(true);
            stepping = true;
            println!("Brakepoint hit! at {:04X}", cpu.get_PC());
        }

        cpu.next_instruction();

        if cpu.get_debug() {
            // print_cpu_status(&cpu);
        }

        let cycles = cpu.get_cycles();
        if cycles - last_tick > 71590 {
            let screen = cpu.handler_holder.video_controller().get_screen();
            controller.refresh(screen);

            last_tick = cycles;
        }

        if stepping {
            loop {
                let mut input = String::new();
                print!(">");
                io::stdout().flush();
                io::stdin().read_line(&mut input).unwrap();
                input = input[..input.len()-1].to_string();

                if input.len() == 0 {
                    break;
                } else if input.starts_with("bo ") {
                    if let Ok(op) = u8::from_str_radix(&input[3..5], 16) {
                        println!("Breakpoint for opcode {}",
                                 OpCode::from_byte(op, false).to_string());
                        op_breakpoints.insert(op);
                    } else {
                        println!("OP not understood: {:?}", &input[3..5]);
                    }
                } else if input.starts_with("ba ") {
                    if let Ok(address) = u16::from_str_radix(&input[3..7], 16) {
                        println!("Breakpoint for address {:04X}", address);
                        address_breakpoints.insert(address);
                    } else {
                        println!("Address not understood: {:?}", &input[3..7]);
                    }
                } else if input == "p cpu" {
                    print_cpu_status(&cpu);
                } else if input.starts_with("p ") {
                    if let Ok(address) = u16::from_str_radix(&input[2..6], 16) {
                        println!("${:04X}={:02X}", address, cpu.deref(address));
                        address_breakpoints.insert(address);
                    } else {
                        println!("Address not understood: {:?}", &input[2..6]);
                    }
                } else if input == "c" {
                    stepping = false;
                    cpu.set_debug(false);
                    break;
                } else if input == "d" {
                    stepping = false;
                    break;
                } else if input == "q" {
                    println!("Quitting.");
                    process::exit(0);
                } else {
                    println!("Command not understood: {:?}", input);
                }

                println!("");
            }
        }
    }
}
