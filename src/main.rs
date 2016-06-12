use std::fs::File;
use std::io::Write;
use std::process;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[macro_use]
extern crate glium;

pub mod gb_proc;
pub mod gpu;
pub mod controller;

use self::gb_proc::handler_holder::GBHandlerHolder;
use self::gb_proc::cartridge::Cartridge;
use self::gb_proc::cpu::{Cpu, print_cpu_status};
use self::gpu::renderer::Renderer;
use self::gb_proc::opcodes::OpCode;
use self::controller::{Event, Controller};

use std::io;
use std::thread;

#[cfg(test)]
mod tests;

pub fn print_help() {
    println!("Help: ");
    println!("clear         -- clear breakpoints");
    println!("list          -- list breakpoints");
    println!("bo [u8]       -- breakpoint for opcode [u8]");
    println!("ba [u16]      -- breakpoint for address [u16]");
    println!("p cpu         -- display cpu information and registers");
    println!("p [u16]       -- print memory at [u16]");
    println!("s [u16] [u8]  -- put value [u8] at memory address [u16]");
    println!("c             -- continue execution");
    println!("d             -- continue execution but print cpu information");
    println!("q             -- quit application");
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

    let mut last_cycles = 0;
    let mut last_update = Instant::now();

    // Refresh 59.7 times a sec
    let target = Duration::new(0, (1e9/59.7) as u32);
    loop {
        if (cpu.get_PC() == 0x100 && cpu.get_debug()) ||
            address_breakpoints.contains(&cpu.get_PC()) ||
                op_breakpoints.contains(&cpu.deref_PC()) {
            cpu.set_debug(true);
            stepping = true;
            println!("Brakepoint hit! at {:04X}", cpu.get_PC());
        }

        cpu.next_instruction();

        if cpu.handler_holder.should_refresh() {
            match controller.check_events(&mut cpu) {
                Event::Quit => break,
                Event::Break => {
                    stepping = true;
                    cpu.set_debug(true);
                },
                Event::Continue => {},
            }

            let screen = cpu.handler_holder.get_screen_buffer();
            controller.refresh(screen);

            let diff = Instant::now() - last_update;

            if target > diff {
                thread::sleep(target - diff);
            }

            last_cycles = cpu.get_cycles();
            last_update = Instant::now();
        }

        if stepping {
            print_cpu_status(&cpu);
            loop {
                let mut input = String::new();
                print!(">");
                io::stdout().flush().unwrap();
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
                } else if input.starts_with("s ") {
                    if let Ok(address) = u16::from_str_radix(&input[2..6], 16) {
                        if let Ok(v) = u8::from_str_radix(&input[7..9], 16) {
                            println!("Setting ${:04X}={:02X}h", address, v);
                            cpu.set_deref(address, v);
                        } else {
                            println!("Value not understood: {:?}", &input[3..5]);
                        }
                    } else {
                        println!("Address not understood: {:?}", &input[2..6]);
                    }
                } else if input == "list" {
                    for op in op_breakpoints.clone() {
                        println!("Breakpoint for opcode {}",
                                 OpCode::from_byte(op, false).to_string());
                    }
                    for address in address_breakpoints.clone() {
                        println!("Breakpoint for address {:04X}", address);
                    }
                } else if input == "clear" {
                    op_breakpoints.clear();
                    address_breakpoints.clear();
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
                    print_help();
                }

                println!("");
            }
        }
    }
}
