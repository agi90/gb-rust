use std::num::Wrapping;
use bitfield::Bitfield;
use hardware::cpu;

u8_enum!{
    ClockSelect {
        // Every X clocks
        C1024 = 0b00,
        C16   = 0b01,
        C64   = 0b10,
        C256  = 0b11,
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

    pub fn cpu_step(&mut self) -> Option<cpu::Interrupt> {
        self.last_clock += cpu::CYCLES_PER_STEP;
        self.last_divider += cpu::CYCLES_PER_STEP;

        let mut interrupt = None;
        while self.last_clock >= 16 {
            self.last_clock -= 16;

            self.clock = (Wrapping(self.clock) + Wrapping(1)).0;

            let clock = self.inc_clock();
            let divider = self.inc_divider();

            if clock || divider {
                interrupt = Some(cpu::Interrupt::Timer);
            }
        }

        if self.mapper.timer_enabled() == 0 {
            return None;
        }

        interrupt
    }

    fn inc_clock(&mut self) -> bool {
        let should_increment = match self.mapper.clock_select() {
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
        0xFF04, 0b00000000, divider, 0;
        0xFF05, 0b00000000, timer,   0;
        0xFF06, 0b00000000, modulo,  0;
    ],
    bitfields: {
        getters: [
            0xFF07, 0b00000111, control, 0, [
                get_01, clock_select,  ClockSelect;
                get_2,  timer_enabled, u8
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
