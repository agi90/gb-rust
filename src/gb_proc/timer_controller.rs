use std::num::Wrapping;
use gb_proc::cpu::Interrupt;

// Every X clocks
enum ClockSelect {
    C16,
    C64,
    C256,
    C1024,
}

pub struct TimerController {
    clock: usize,
    modulo: u8,
    divider: u8,
    timer: u8,

    timer_enabled: bool,
    clock_select: ClockSelect,

    last_clock: usize,
    last_divider: usize,
}

impl TimerController {
    pub fn new() -> TimerController {
        TimerController {
            clock: 0,
            modulo: 0,
            divider: 0,
            timer: 0,

            timer_enabled: false,
            clock_select: ClockSelect::C1024,

            last_clock: 0,
            last_divider: 0,
        }
    }

    pub fn add_cycles(&mut self, cycles: usize) -> Vec<Interrupt> {
        self.last_clock += cycles;
        self.last_divider += cycles;

        let mut interrupts = vec![];
        if self.last_clock > 16 {
            self.last_clock -= 16;
            let clock = self.inc_clock();
            let divider = self.inc_divider();

            if clock || divider {
                interrupts.push(Interrupt::Timer);
            }
        }

        if !self.timer_enabled {
            return vec![];
        }

        interrupts
    }

    fn inc_clock(&mut self) -> bool {
        self.clock = (Wrapping(self.clock) + Wrapping(1)).0;

        let should_increment = match self.clock_select {
            ClockSelect::C16   => true,
            ClockSelect::C64   => (self.clock %  4) == 0,
            ClockSelect::C256  => (self.clock % 16) == 0,
            ClockSelect::C1024 => (self.clock % 64) == 0,
        };

        if should_increment {
            self.timer += (Wrapping(self.timer) + Wrapping(1)).0;
            self.timer == 0
        } else {
            false
        }
    }

    fn inc_divider(&mut self) -> bool {
        self.divider = (Wrapping(self.divider) + Wrapping(1)).0;
        self.divider == 0x00
    }

    fn write_counter(&mut self, v: u8) {
        self.timer = v;
    }

    fn write_modulo(&mut self, v: u8) {
        // not totally sure about this
        self.modulo = v;
    }

    fn write_control(&mut self, v: u8) {
        self.timer_enabled  = (v & 0b00000100) > 0;
        let clock_select    = v & 0b00000011;

        self.clock_select = match clock_select {
            0 => ClockSelect::C1024,
            1 => ClockSelect::C16,
            2 => ClockSelect::C64,
            3 => ClockSelect::C256,
            _ => unreachable!(),
        }
    }

    fn read_divider(&self) -> u8 {
        self.divider
    }

    fn read_counter(&self) -> u8 {
        self.timer
    }

    fn read_modulo(&self) -> u8 {
        unimplemented!();
    }

    fn read_control(&self) -> u8 {
        unimplemented!();
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.read_divider(),
            0xFF05 => self.read_counter(),
            0xFF06 => self.read_modulo(),
            0xFF07 => self.read_control(),
            _      => panic!(),
        }
    }

    pub fn write(&mut self, address: u16, v: u8) {
        match address {
            // Divider is always set to zero regardless of v
            0xFF04 => { self.divider = 0 },
            0xFF05 => self.write_counter(v),
            0xFF06 => self.write_modulo(v),
            0xFF07 => self.write_control(v),
            _      => panic!(),
        }
    }
}
