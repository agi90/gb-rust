use std::num::Wrapping;
use bitfield::Bitfield;
use gb_proc::cpu::{Handler, Interrupt};

// Every X clocks
#[derive(Clone, Copy, Debug)]
enum ClockSelect {
    C1024 = 0b00,
    C16   = 0b01,
    C64   = 0b10,
    C256  = 0b11,
}

impl Into<u8> for ClockSelect {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for ClockSelect {
    fn from(v: u8) -> ClockSelect {
        match v {
            0b00 => ClockSelect::C1024,
            0b01 => ClockSelect::C16,
            0b10 => ClockSelect::C64,
            0b11 => ClockSelect::C256,
            _ => panic!(),
        }
    }
}

pub struct TimerController {
    clock: usize,
    last_clock: usize,
    last_divider: usize,

    mapper: TimerMemoryMapper,
}

impl TimerController {
    pub fn new() -> TimerController {
        TimerController {
            clock: 0,
            last_clock: 0,
            last_divider: 0,

            mapper: TimerMemoryMapper::new(),
        }
    }

    pub fn add_cycles(&mut self, cycles: usize) -> Vec<Interrupt> {
        // println!("add_cycles cycles={} last_clock={}", cycles, self.last_clock);
        self.last_clock += cycles;
        self.last_divider += cycles;

        let mut interrupts = vec![];
        while self.last_clock >= 16 {
            self.last_clock -= 16;

            self.clock = (Wrapping(self.clock) + Wrapping(1)).0;

            let clock = self.inc_clock();
            let divider = self.inc_divider();

            if clock || divider {
                interrupts.push(Interrupt::Timer);
            }
        }

        if self.mapper.get_timer_enabled() == 0 {
            return vec![];
        }

        interrupts
    }

    fn inc_clock(&mut self) -> bool {
        let should_increment = match self.mapper.get_clock_select() {
            ClockSelect::C16   => true,
            ClockSelect::C64   => (self.clock %  4) == 0,
            ClockSelect::C256  => (self.clock % 16) == 0,
            ClockSelect::C1024 => (self.clock % 64) == 0,
        };

        if should_increment {
            if self.mapper.timer == 0xFF {
                self.mapper.timer = self.mapper.modulo;
                true
            } else {
                self.mapper.timer = self.mapper.timer + 1;
                false
            }
        } else {
            false
        }
    }

    fn inc_divider(&mut self) -> bool {
        if self.clock % 16 == 0 {
            self.mapper.divider = (Wrapping(self.mapper.divider) + Wrapping(1)).0;
            self.mapper.divider == 0x00
        } else {
            false
        }
    }

    pub fn write_callback(&mut self, address: u16) {
        match address {
            0xFF04 => { self.mapper.divider = 0 },
            _ => {},
        }
    }
}

memory_mapper!{
    name: TimerMemoryMapper,
    fields: [
        0xFF04, divider, 0;
        0xFF05, timer,   0;
        0xFF06, modulo,  0;
    ],
    bitfields: {
        getters: [
            0xFF07, control, 0, [
                get_01, get_clock_select,  ClockSelect;
                get_2,  get_timer_enabled, u8
            ]
        ],
        getter_setters: [],
    },
}

memory_handler!{
    parent: TimerController,
    mapper: mapper,
    callback: write_callback,
}
