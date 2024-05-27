use crate::SystemControl;

use super::{envelope::Envelope, length_counter::LengthCounter};

const PERIOD_LOOKUP: [u32; 0x10] = [2, 4, 8, 16, 32, 48, 64, 80, 101, 127, 190, 254, 381, 508, 1017, 2034];
pub struct Noise {
    pub length_counter: LengthCounter,
    pub envelope: Envelope,
    pub shift_mode: bool,

    period: u32,
    cycles: u32,
    shift_reg: u16,
}

impl SystemControl for Noise {
    fn reset(&mut self) {
        self.length_counter.reset();
        self.envelope.reset();
        self.shift_mode = false;
        self.period = PERIOD_LOOKUP[0];
        self.cycles = 0;
        self.shift_reg = 1;
    }
}

impl Noise {
    pub fn new() -> Self {
        Self {
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),
            shift_mode: false,

            period: PERIOD_LOOKUP[0],
            cycles: 0,
            shift_reg: 1,
        }
    }

    pub fn set_period(&mut self, period: usize) {
        self.period = PERIOD_LOOKUP[period];
    }

    pub fn clock(&mut self) -> u8 {
        if self.cycles == 0 {
            self.cycles = self.period;

            let feedback = (self.shift_reg & 0x01) ^ if self.shift_mode {
                (self.shift_reg & 0x40) >> 6
            } else {
                (self.shift_reg & 0x02) >> 1
            } != 0;

            self.shift_reg >>= 1;
            self.shift_reg |= (feedback as u16) << 14

        }

        self.cycles -= 1;

        if self.length_counter.counter > 0 && (self.shift_reg & 0x01) == 0 {
            self.envelope.output_volume()
        } else {
            0
        }
    }
}