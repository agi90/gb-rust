use std::fs::File;
use std::time::{Duration, Instant};
use std::thread;

#[macro_use]
extern crate glium;

pub mod gb_proc;
pub mod gpu;
pub mod controller;
mod bitfield;
mod debugger;

use self::debugger::Debugger;

use self::gb_proc::handler_holder::GBHandlerHolder;
use self::gb_proc::cartridge::Cartridge;
use self::gb_proc::cpu::Cpu;
use self::gpu::renderer::Renderer;
use self::controller::{Event, Controller};

#[cfg(test)]
mod tests;

pub fn main() {
    let mut f = File::open("rom.gb").unwrap();
    let cartridge = Cartridge::from_file(&mut f);

    let mut controller = Controller::new();;

    let handler = GBHandlerHolder::new(Box::new(cartridge));
    let mut cpu = Cpu::new(Box::new(handler));

    cpu.set_debug(false);

    let mut last_update = Instant::now();

    let mut debugger = Debugger::new();
    let mut natural_speed = true;
    // This target seems to match the actual Game Boy in terms of speed
    let target = Duration::new(0, (1e9/63.0) as u32);
    loop {
        debugger.check_breakpoints(&mut cpu);

        cpu.next_instruction();

        if cpu.handler_holder.should_refresh() {
            match controller.check_events(&mut cpu) {
                Event::Quit => break,
                Event::Break => {
                    debugger.breakpoint(&mut cpu);
                },
                Event::ToggleSpeed => { natural_speed = !natural_speed },
                Event::Continue => {},
            }

            let screen = cpu.handler_holder.get_screen_buffer();
            controller.refresh(screen);

            let diff = Instant::now() - last_update;

            if target > diff && natural_speed {
                thread::sleep(target - diff);
            }

            last_update = Instant::now();
        }

        debugger.next_instruction(&mut cpu);
    }
}
