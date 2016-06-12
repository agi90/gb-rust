use gb_proc::cpu::Handler;

#[derive(Clone, Copy, Debug)]
enum WavePattern {
    C12 = 0b00,
    C25 = 0b01,
    C50 = 0b10,
    C75 = 0b11,
}

impl WavePattern {
    pub fn from_byte(v: u8) -> WavePattern {
        match v {
            0b00 => WavePattern::C12,
            0b01 => WavePattern::C25,
            0b10 => WavePattern::C50,
            0b11 => WavePattern::C75,
            _ => panic!(),
        }
    }
}

pub struct SoundController {
    sound_enabled: bool,
    selection_sound: u8,

    channel_1_duration: u8,
    channel_1_pattern: WavePattern,
    channel_1_counter: u8,
    channel_1_consecutive: bool,
    channel_1_on: bool,
    channel_1_level: u8,

    channel_2_duration: u8,
    channel_2_pattern: WavePattern,
    channel_2_counter: u8,
    channel_2_consecutive: bool,
    channel_2_on: bool,
    channel_2_level: u8,

    last_cycle: usize,
}

impl SoundController {
    pub fn new() -> SoundController {
        SoundController {
            sound_enabled: true,
            selection_sound: 0xF3,

            channel_1_duration: 0x01,
            channel_1_pattern: WavePattern::C50,
            channel_1_counter: 0x00,
            channel_1_consecutive: false,
            channel_1_on: false,
            channel_1_level: 0b111,

            channel_2_duration: 0x01,
            channel_2_pattern: WavePattern::C50,
            channel_2_counter: 0x00,
            channel_2_consecutive: false,
            channel_2_on: false,
            channel_2_level: 0b111,

            last_cycle: 0,
        }
    }

    pub fn add_cycles(&mut self, cycles: usize) {
        self.last_cycle += cycles;

        // 1/256th of second
        if self.last_cycle > 16384 {
            self.last_cycle -= 16384;

            self.channel_1_counter += 1;
            if self.channel_1_counter > self.channel_1_duration {
                self.channel_1_counter = 0;
            }

            self.channel_2_counter += 1;
            if self.channel_2_counter > self.channel_2_duration {
                self.channel_2_counter = 0;
            }
        }
    }

    fn get_channel_2_sound(&self) -> u8 {
        (self.channel_2_pattern as u8) << 6
    }

    fn get_channel_1_sound(&self) -> u8 {
        (self.channel_1_pattern as u8) << 6
    }

    fn set_channel_2_sound(&mut self, v: u8) {
        self.channel_2_pattern = WavePattern::from_byte((v & 0b11000000) >> 6);
        self.channel_2_duration = v & 0b00111111;
        self.channel_2_counter = 0;
    }

    fn set_channel_1_sound(&mut self, v: u8) {
        self.channel_1_pattern = WavePattern::from_byte((v & 0b11000000) >> 6);
        self.channel_1_duration = v & 0b00111111;
        self.channel_1_counter = 0;
    }

    fn set_channel_control(&mut self, v: u8) {
        self.channel_2_on    = (v & 0b10000000) > 0;
        self.channel_2_level = (v & 0b01110000) >> 4;
        self.channel_1_on    = (v & 0b00001000) > 0;
        self.channel_1_level = v & 0b00000111;
    }

    fn get_channel_control(&self) -> u8 {
        (if self.channel_2_on { 0b10000000 } else { 0 }) +
           (self.channel_2_level << 4) +
        (if self.channel_1_on { 0b00001000 } else { 0 }) +
            self.channel_1_level
    }

    fn get_channel_2_frequency(&self) -> u8 {
        if self.channel_2_consecutive { 0b01000000 } else { 0 }
    }

    fn get_channel_1_frequency(&self) -> u8 {
        if self.channel_1_consecutive { 0b01000000 } else { 0 }
    }

    fn set_channel_2_frequency(&mut self, v: u8) {
        self.channel_2_consecutive = v & 0b01000000 > 0;
        // The other bits are devoted to outputting sound
    }

    fn set_channel_1_frequency(&mut self, v: u8) {
        self.channel_1_consecutive = v & 0b01000000 > 0;
        // The other bits are devoted to outputting sound
    }

    pub fn sound_status_read(&self) -> u8 {
        // Not really implemented for now
        (if self.sound_enabled { 0b10000000 } else { 0 }) +
                                 0b00000001
    }
}

impl Handler for SoundController {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF11 => self.get_channel_1_sound(),
            0xFF14 => self.get_channel_1_frequency(),
            0xFF16 => self.get_channel_2_sound(),
            0xFF19 => self.get_channel_2_frequency(),
            0xFF26 => self.sound_status_read(),
            0xFF24 => self.get_channel_control(),
            0xFF25 => self.selection_sound,
            _      => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF11 => { self.set_channel_1_sound(v) },
            0xFF14 => { self.set_channel_1_frequency(v) },
            0xFF16 => { self.set_channel_2_sound(v) },
            0xFF19 => { self.set_channel_2_frequency(v) },
            0xFF24 => { self.set_channel_control(v) },
            0xFF25 => { self.selection_sound = v },
            _      => {
                // TODO
            }
        }
    }
}


