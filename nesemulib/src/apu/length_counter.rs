use crate::SystemControl;


const LENGTH_LOOKUP: [u8; 0x20] = [
    10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
];

pub struct LengthCounter {
    pub halted: bool,
    enabled_flag: bool,
    pub counter: u8,
}

impl SystemControl for LengthCounter {
    fn reset(&mut self) {
        self.halted = false;
        self.enabled_flag = false;
        self.counter = 0;
    }
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            halted: false,
            enabled_flag: false,
            counter: 0,
        }
    }

    pub fn clock(&mut self) {
        if !self.enabled_flag {
            self.counter = 0;
        }

        if !self.halted && self.counter > 0 {
            self.counter -= 1;
        }
    }

    pub fn load_counter(&mut self, lookup: u8) {
        if self.enabled_flag {
            self.counter = LENGTH_LOOKUP[lookup as usize];
        }
    }

    pub fn set_enabled_flag(&mut self, enabled: bool) {
        self.enabled_flag = enabled;
        if !self.enabled_flag {
            self.counter = 0;
        }
    }

    pub fn silenced(&self) -> bool {
        self.counter == 0 || !self.enabled_flag
    }
}