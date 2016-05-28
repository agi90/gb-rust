pub struct SoundController {
    sound_enabled: bool,
}

impl SoundController {
    pub fn new() -> SoundController {
        SoundController { sound_enabled: true }
    }

    pub fn sound_status_read(&self) -> u8 {
        // Not really implemented for now
        (if self.sound_enabled { 0b10000000 } else { 0 }) +
                                 0b00000001
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF14 => 0b00000000,
            0xFF19 => 0b01000000,
            0xFF26 => self.sound_status_read(),
            0xFF25 => 0b00000000,
            0xFF24 => 0b11111111,
            _      => unimplemented!(),
        }
    }

    pub fn write(&mut self, address: u16, v: u8) {
        // TODO:
    }
}


