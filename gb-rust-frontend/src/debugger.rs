use std::process;
use std::collections::HashSet;
use std::io::Write;
use std::io;

use gb::{
    Emulator,
    OpCode,
};

pub struct Debugger {
    stepping: bool,
    address_breakpoints: HashSet<u16>,
    op_breakpoints: HashSet<u8>,
    last_command: String,
    watches: HashSet<u16>,
}

fn print_cpu_status(emulator: &Emulator, watches: &HashSet<u16>) {
    let cpu = &emulator.cpu;

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
    println!("IF = ${:02X}",  cpu.deref_debug(0xFFFF));
    println!("IE = ${:02X}",  cpu.deref_debug(0xFF0F));
    println!("state = {:?}",  cpu.get_state());
    println!("cycles = {:?}", cpu.get_cycles());
    println!("$FF05 = {:02X}", cpu.deref_debug(0xFF05));
    if watches.len() > 0 {
        println!("=== WATCH ===");
    }
    for w in watches {
        println!("${:04X} = {:02X}", w, cpu.deref_debug(*w));
    }

    println!("=== STACK ===");
    println!("${:04X} = {:02X}", cpu.get_SP(),     cpu.deref_debug(cpu.get_SP()));

    if cpu.get_SP() != 0xFFFF && cpu.get_SP() != 0xDFFF {
        println!("${:04X} = {:02X}", cpu.get_SP() + 1, cpu.deref_debug(cpu.get_SP() + 1));
    }
    println!("");
}

fn print_help() {
    println!("Help: ");
    println!("clear            -- clear breakpoints");
    println!("[l]ist           -- list breakpoints");
    println!("bo [u8]          -- breakpoint for opcode [u8]");
    println!("ba [u16]         -- breakpoint for address [u16]");
    println!("bm [u16]         -- breaks whenever the memory at [u16] is accessed");
    println!("[w]atch [u16]    -- prints address [u16] everytime the debugger breaks");
    println!("p cpu            -- display cpu information and registers");
    println!("[p]rint [u16]    -- print memory at [u16]");
    println!("po [u16]         -- print opcode at [u16]");
    println!("[s]et [u16] [u8] -- put value [u8] at memory address [u16]");
    println!("[c]ontinue       -- continue execution");
    println!("[s]tep           -- go to next instruction");
    println!("d                -- continue execution but print cpu information");
    println!("[q]uit           -- quit application");
    println!("");
}

fn to_value(arg: &str) -> Result<u8, ()> {
    u8::from_str_radix(arg, 16).map_err(|_| ())
}

fn to_address(arg: &str) -> Result<u16, ()> {
    u16::from_str_radix(arg, 16).map_err(|_| ())
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            stepping: false,
            address_breakpoints: HashSet::new(),
            op_breakpoints: HashSet::new(),
            last_command: "s".to_string(),
            watches: HashSet::new(),
        }
    }

    pub fn handle_unary(&mut self, command: &str, emulator: &mut Emulator) -> Result<bool, ()> {
        match command {
            "l" | "list" => {
                for op in self.op_breakpoints.clone() {
                    println!("Breakpoint for opcode {}",
                             OpCode::from_byte(op, false).to_string());
                }
                for address in self.address_breakpoints.clone() {
                    println!("Breakpoint for address {:04X}", address);
                }
            },
            "clear" => {
                self.op_breakpoints.clear();
                self.address_breakpoints.clear();
                emulator.cpu.clear_watch();
            },
            "c" | "continue" => {
                self.stepping = false;
                emulator.cpu.set_debug(false);
                println!("Continuing.");

                return Ok(true);
            },
            "d" => {
                self.stepping = false;

                return Ok(true);
            },
            "s" | "step" => {
                return Ok(true);
            },
            "q" | "quit" => {
                println!("Quitting.");
                process::exit(0);
            },
            "h" | "help" => {
                print_help();
            },
            _ => {
                return Err(());
            },
        }

        Ok(false)
    }

    pub fn handle_binary(&mut self, command: &str, arg: &str, emulator: &mut Emulator) -> Result<bool, ()> {
        match command {
            "bo" => {
                let op = try!(to_value(arg));
                println!("Breakpoint for opcode {}",
                         OpCode::from_byte(op, false).to_string());
                self.op_breakpoints.insert(op);
            },
            "ba" => {
                let address = try!(to_address(arg));
                println!("Breakpoint for address {:04X}", address);
                self.address_breakpoints.insert(address);
            },
            "bm" => {
                let address = try!(to_address(arg));
                println!("Breakpoint for memory access at address {:04X}", address);
                emulator.cpu.watch(address);
            },
            "p" | "print" => {
                if arg == "cpu" {
                    print_cpu_status(emulator, &self.watches);
                } else {
                    let address = try!(to_address(arg));
                    println!("${:04X}={:02X}", address, emulator.cpu.deref_debug(address));
                }
            },
            "po" => {
                let address = try!(to_address(arg));
                println!("${:04X} = {}", address,
                         OpCode::from_byte(emulator.cpu.deref_debug(address), false).to_string());
            },
            "w" | "watch" => {
                let address = try!(to_address(arg));
                println!("Adding {:04X} to the watch list.", address);
                self.watches.insert(address);
            },
            _ => {
                return Err(());
            },
        }

        Ok(false)
    }

    pub fn handle_trinary(&mut self, command: &str, arg1: &str, arg2: &str,
                          emulator: &mut Emulator) -> Result<bool, ()> {
        match command {
            "s" | "set" => {
                let address = try!(to_address(arg1));
                let v = try!(to_value(arg2));

                println!("Setting ${:04X}={:02X}h", address, v);
                emulator.cpu.set_deref_debug(address, v);
            },
            _ => {
                return Err(());
            },
        }

        Ok(false)
    }

    fn handle(&mut self, args: Vec<&str>, emulator: &mut Emulator) -> Result<bool, ()> {
        match args.len() {
            0 => Ok(true),
            1 => self.handle_unary(args[0], emulator),
            2 => self.handle_binary(args[0], args[1], emulator),
            3 => self.handle_trinary(args[0], args[1], args[2], emulator),
            _ => Err(()),
        }
    }

    pub fn check_breakpoints(&mut self, emulator: &mut Emulator) {
        let address = emulator.cpu.get_PC();
        let op = emulator.cpu.deref_debug(address);

        if !self.address_breakpoints.contains(&address) &&
                !self.op_breakpoints.contains(&op) &&
                !emulator.cpu.address_breakpoint() {
            return;
        }

        emulator.cpu.set_debug(true);
        self.stepping = true;
        println!("Brakepoint hit! at {:04X}", emulator.cpu.get_PC());
    }

    pub fn next_instruction(&mut self, emulator: &mut Emulator) {
        if !self.stepping {
            return;
        }

        print_cpu_status(emulator, &self.watches);
        loop {
            let mut input = String::new();
            print!(">");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();
            input = input.replace("\n", "")
                .replace("\r", "");

            if input != "" {
                self.last_command = input.clone();
            } else {
                input = self.last_command.clone();
                println!("{}", input);
            }

            match self.exec_private(&input, emulator) {
                Err(error_string) => {
                    println!("Error: {}", error_string);
                    print_help();
                },
                Ok(true) => {
                    break;
                },
                Ok(false) => {
                    println!("");
                },
            }
        }
    }

    // We don't want to expose the result value to the external world since it's part of the
    // debugger internals.
    pub fn exec(&mut self, command: &str, emulator: &mut Emulator) -> Result<(), String> {
        self.exec_private(command, emulator)
            .map(|_| ())
    }

    fn exec_private(&mut self, command: &str, emulator: &mut Emulator) -> Result<bool, String> {
        let pieces: Vec<&str> = command.split(' ').collect();
        self.handle(pieces, emulator)
            .map_err(|_| format!("Command not understood {:?}", command))
    }

    pub fn breakpoint(&mut self, emulator: &mut Emulator) {
        self.stepping = true;
        emulator.cpu.set_debug(true);
        print_cpu_status(emulator, &self.watches);
    }
}
