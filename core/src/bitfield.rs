use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Bitfield {
    data: u8,
}

impl Deref for Bitfield {
    type Target = u8;

    fn deref(&self) -> &u8 {
        &self.data
    }
}

impl DerefMut for Bitfield {
    fn deref_mut(&mut self) -> &mut u8 {
        &mut self.data
    }
}

impl Bitfield {
    pub fn new(v: u8) -> Bitfield {
        Bitfield { data: v }
    }

    pub fn get(&self) -> u8 {
        self.data
    }

    pub fn set(&mut self, v: u8) {
        self.data = v;
    }

    pub fn get_0(&self) -> u8 {
        if self.data & 0b00000001 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_1(&self) -> u8 {
        if self.data & 0b00000010 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_2(&self) -> u8 {
        if self.data & 0b00000100 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_3(&self) -> u8 {
        if self.data & 0b00001000 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_4(&self) -> u8 {
        if self.data & 0b00010000 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_5(&self) -> u8 {
        if self.data & 0b00100000 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_6(&self) -> u8 {
        if self.data & 0b01000000 > 0 {
            1
        } else {
            0
        }
    }
    pub fn get_7(&self) -> u8 {
        if self.data & 0b10000000 > 0 {
            1
        } else {
            0
        }
    }

    pub fn set_0(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b00000001
        } else {
            self.data & 0b11111110
        };
    }
    pub fn set_1(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b00000010
        } else {
            self.data & 0b11111101
        };
    }
    pub fn set_2(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b00000100
        } else {
            self.data & 0b11111011
        };
    }
    pub fn set_3(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b00001000
        } else {
            self.data & 0b11110111
        };
    }
    pub fn set_4(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b00010000
        } else {
            self.data & 0b11101111
        };
    }
    pub fn set_5(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b00100000
        } else {
            self.data & 0b11011111
        };
    }
    pub fn set_6(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b01000000
        } else {
            self.data & 0b10111111
        };
    }
    pub fn set_7(&mut self, v: u8) {
        self.data = if v == 0x1 {
            self.data | 0b10000000
        } else {
            self.data & 0b01111111
        };
    }

    pub fn get_01(&self) -> u8 {
        self.data & 0b00000011
    }
    pub fn get_23(&self) -> u8 {
        (self.data & 0b00001100) >> 2
    }
    pub fn get_45(&self) -> u8 {
        (self.data & 0b00110000) >> 4
    }
    pub fn get_56(&self) -> u8 {
        (self.data & 0b01100000) >> 5
    }
    pub fn get_67(&self) -> u8 {
        (self.data & 0b11000000) >> 6
    }

    pub fn set_01(&mut self, v: u8) {
        self.data = (self.data & 0b11111100) + v
    }
    pub fn set_23(&mut self, v: u8) {
        self.data = (self.data & 0b11110011) + (v << 2)
    }
    pub fn set_45(&mut self, v: u8) {
        self.data = (self.data & 0b11001111) + (v << 4)
    }
    pub fn set_56(&mut self, v: u8) {
        self.data = (self.data & 0b10011111) + (v << 5)
    }
    pub fn set_67(&mut self, v: u8) {
        self.data = (self.data & 0b00111111) + (v << 6)
    }

    pub fn get_012(&self) -> u8 {
        self.data & 0b00000111
    }
    pub fn set_012(&mut self, v: u8) {
        assert!(v <= 0b111);
        self.data = (self.data & 0b11111000) + v;
    }

    pub fn get_456(&self) -> u8 {
        (self.data & 0b01110000) >> 4
    }
    pub fn set_456(&mut self, v: u8) {
        assert!(v <= 0b111);
        self.data = (self.data & 0b10001111) + (v << 4);
    }

    pub fn set_3456(&mut self, v: u8) {
        assert!(v <= 0b1111);
        self.data = (self.data & 0b10000111) + (v << 3);
    }

    pub fn get_4567(&self) -> u8 {
        (self.data & 0b11110000) >> 4
    }
    pub fn set_4567(&mut self, v: u8) {
        assert!(v <= 0b1111);
        self.data = (self.data & 0b00001111) + (v << 4);
    }

    pub fn get_012345(&self) -> u8 {
        self.data & 0b00111111
    }
    pub fn set_012345(&mut self, v: u8) {
        assert!(v <= 0b00111111);
        self.data = (self.data & 0b11000000) + v;
    }
}
