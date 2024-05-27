use crate::SystemControl;

use super::length_counter::LengthCounter;

pub struct Triangle {
    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,

    pub period: u32,
    cycles: u32,
    duty_step: u8,
}

impl SystemControl for Triangle {
    fn reset(&mut self) {
        self.length_counter.reset();
        self.linear_counter.reset();
        self.cycles = 0;
        self.period = 0;
        self.duty_step = 0;
    }

}

impl Triangle {
    pub fn new() -> Self {
        Self {
            length_counter: LengthCounter::new(),
            linear_counter: LinearCounter::new(),

            cycles: 0,
            period: 0,
            duty_step: 0,
        }
    }

    pub fn clock(&mut self) -> u8 {
        
        if self.length_counter.counter > 0 && self.linear_counter.counter > 0 && self.period > 0 {
    
            if self.cycles == 0 {
                self.cycles = self.period;

                self.duty_step = (self.duty_step + 1) & 0x1F;
            }

            if self.cycles > 0 {
                self.cycles -= 1;
            }
        }

        
        if self.duty_step <= 15 {
            15 - self.duty_step
        } else {
            self.duty_step - 16
        }
    }
}

pub struct LinearCounter {
    pub counter: u8,
    pub control_flag: bool,
    pub reload: u8,
    pub reload_flag: bool,
}

impl SystemControl for LinearCounter {
    fn reset(&mut self) {
        self.counter = 0;
        self.control_flag = false;
        self.reload = 0;
        self.reload_flag = false;
    }
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            control_flag: false,
            reload: 0,
            reload_flag: false,
        }
    }

    pub fn clock(&mut self) {
        if self.reload_flag {
            self.counter = self.reload;
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if !self.control_flag {
            self.reload_flag = false;
        }
    }
}