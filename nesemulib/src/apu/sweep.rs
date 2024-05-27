use crate::SystemControl;

pub struct Sweep {
    pub period: u32,
    pub muted: bool,
    target_period: u32,

    shift: u8,
    negate_flag: bool,
    enabled_flag: bool,
    reload_flag: bool,

    divider: u8,
    counter: u8,
}

impl SystemControl for Sweep {
    fn reset(&mut self) {
        self.period = 0;
        self.muted = false;
        self.target_period = 0;
        self.shift = 0;
        self.negate_flag = false;
        self.enabled_flag = false;
        self.reload_flag = false;
        self.divider = 0;
        self.counter = 0;
    }
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            period: 0,
            muted: false,
            target_period: 0,

            shift: 0,
            negate_flag: false,
            enabled_flag: false,
            reload_flag: false,

            divider: 0,
            counter: 0,
        }
    }

    pub fn update(&mut self, channel: bool) {
        let change = self.period >> self.shift;

        self.target_period = if self.negate_flag {
            let diff = (self.period as i32) - (change as i32) - (channel as i32);

            if diff < 0 { 0 } else { diff as u32 }
        } else {
            self.period.wrapping_add(change)
        };

        self.muted = (self.period < 8) || (self.target_period > 0x7FF);
    }

    pub fn clock(&mut self) {
        if self.counter == 0 && self.enabled_flag && self.shift > 0 && !self.muted {

            if self.period >= 8 && self.target_period <= 0x07FF {
                self.period = self.target_period;
            }
        }

        if self.counter == 0 || self.reload_flag {
            self.counter = self.divider;
            self.reload_flag = false;
        } else {
            self.counter -= 1;
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.enabled_flag = (byte & 0b10000000) != 0;
        self.divider = (byte & 0b01110000) >> 4;
        self.negate_flag = (byte & 0b00001000) != 0;
        self.shift = byte & 0b00000111;
        self.reload_flag = true;
    }
}