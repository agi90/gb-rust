pub struct Bitfield {
    data: u8,
}

impl Bitfield {
    pub fn new(v: u8) -> Bitfield {
        Bitfield {
            data: v,
        }
    }

    pub fn get(&self) -> u8 {
        self.data
    }

    pub fn set(&mut self, v: u8) {
        self.data = v;
    }

    pub fn get_0(&self) -> bool { self.data & 0b00000001 > 0 }
    pub fn get_1(&self) -> bool { self.data & 0b00000010 > 0 }
    pub fn get_2(&self) -> bool { self.data & 0b00000100 > 0 }
    pub fn get_3(&self) -> bool { self.data & 0b00001000 > 0 }
    pub fn get_4(&self) -> bool { self.data & 0b00010000 > 0 }
    pub fn get_5(&self) -> bool { self.data & 0b00100000 > 0 }
    pub fn get_6(&self) -> bool { self.data & 0b01000000 > 0 }
    pub fn get_7(&self) -> bool { self.data & 0b10000000 > 0 }

    pub fn set_0(&mut self, v: bool) { self.data = if v { self.data | 0b00000001 } else { self.data & 0b11111110 }; }
    pub fn set_1(&mut self, v: bool) { self.data = if v { self.data | 0b00000010 } else { self.data & 0b11111101 }; }
    pub fn set_2(&mut self, v: bool) { self.data = if v { self.data | 0b00000100 } else { self.data & 0b11111011 }; }
    pub fn set_3(&mut self, v: bool) { self.data = if v { self.data | 0b00001000 } else { self.data & 0b11110111 }; }
    pub fn set_4(&mut self, v: bool) { self.data = if v { self.data | 0b00010000 } else { self.data & 0b11101111 }; }
    pub fn set_5(&mut self, v: bool) { self.data = if v { self.data | 0b00100000 } else { self.data & 0b11011111 }; }
    pub fn set_6(&mut self, v: bool) { self.data = if v { self.data | 0b01000000 } else { self.data & 0b10111111 }; }
    pub fn set_7(&mut self, v: bool) { self.data = if v { self.data | 0b10000000 } else { self.data & 0b01111111 }; }

    pub fn get_01(&self) -> u8 {  self.data & 0b00000011 }
    pub fn get_23(&self) -> u8 { (self.data & 0b00001100) >> 2 }
    pub fn get_45(&self) -> u8 { (self.data & 0b00110000) >> 4 }
    pub fn get_67(&self) -> u8 { (self.data & 0b11000000) >> 6 }

    pub fn set_01(&mut self, v: u8) { self.data = (self.data & 0b11111100) + v }
    pub fn set_23(&mut self, v: u8) { self.data = (self.data & 0b11110011) + (v << 2) }
    pub fn set_45(&mut self, v: u8) { self.data = (self.data & 0b11001111) + (v << 4) }
    pub fn set_67(&mut self, v: u8) { self.data = (self.data & 0b00111111) + (v << 6) }
}
