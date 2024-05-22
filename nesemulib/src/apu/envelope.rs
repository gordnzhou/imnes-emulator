pub struct Envelope {
    pub start_flag: bool,
    pub loop_flag: bool,
    pub constant_flag: bool,
    constant_volume: u8,

    counter: u8,
    counter_period: u8,

    decay_counter: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            constant_volume: 0,

            start_flag: false,
            loop_flag: false,
            constant_flag: false,

            counter: 0,
            counter_period: 0,

            decay_counter: 0,
        }
    }

    pub fn clock(&mut self) {

        if self.start_flag {
            self.start_flag = false;
            self.decay_counter = 15;
            self.counter = self.counter_period;
        } else {
            if self.counter == 0 {
                self.counter = self.counter_period;

                if self.decay_counter > 0 {
                    self.decay_counter -= 1;
                } else if self.loop_flag {
                    self.decay_counter = 15;
                }
            } else {
                self.counter -= 1;
            }
        }
    }

    pub fn set_volume(&mut self, value: u8) {
        self.constant_volume = value;
        self.counter_period = value;
    }

    pub fn output_volume(&self) -> u8 {
        if self.constant_flag {
            self.constant_volume
        } else {
            self.decay_counter
        }
    }
}