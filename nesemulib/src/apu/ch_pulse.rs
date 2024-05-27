use crate::SystemControl;

use super::{envelope::Envelope, length_counter::LengthCounter, sweep::Sweep, DUTY_SEQUENCES};

pub struct Pulse {
    pub duty_sequence: u8,
    pub duty_step: u8,

    pub length_counter: LengthCounter,
    pub envelope: Envelope,
    pub sweep: Sweep,

    pub cycles: u32,
    channel: bool,
}

impl SystemControl for Pulse {
    fn reset(&mut self) {
        self.duty_sequence = DUTY_SEQUENCES[0];
        self.duty_step = 0;
        self.length_counter.reset();
        self.envelope.reset();
        self.sweep.reset();
        self.cycles = 0;
    }
}

impl Pulse {
    pub fn new(channel: bool,) -> Self {
        Self {
            duty_sequence: DUTY_SEQUENCES[0],
            duty_step: 0,

            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),
            sweep: Sweep::new(),

            cycles: 0,
            channel,
        }
    }

    pub fn clock(&mut self) -> u8 {
        self.sweep.update(self.channel);

        if self.sweep.period >= 8 {
            
            if self.cycles == 0 {
                self.cycles = self.sweep.period;
    
                self.duty_step = (self.duty_step + 1) & 0x07;
            }

            self.cycles -= 1;
        }

        let mut sample = ((self.duty_sequence & (1 << self.duty_step)) != 0) as u8;

        sample *= if !self.sweep.muted { self.envelope.output_volume() } else { 0 };

        if self.length_counter.silenced() {
            sample = 0;
        }

        sample
    }
}