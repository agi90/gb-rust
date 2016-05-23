// Every X clocks
enum ClockSelect {
    C16,
    C64,
    C256,
    C1024,
}

pub struct TimerController {
    clock_frequency: u8,
    modulo: u8,
    divider: u8,
    timer_enabled: bool,
    clock_select: ClockSelect,
}

impl TimerController {
    pub fn new() -> TimerController {
        TimerController {
            clock_frequency: 0,
            modulo: 0,
            divider: 0,
            timer_enabled: false,
            clock_select: ClockSelect::C1024,
        }
    }

    fn write_counter(&mut self, v: u8) {
        unimplemented!();
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
        unimplemented!();
    }

    fn read_counter(&self) -> u8 {
        unimplemented!();
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
