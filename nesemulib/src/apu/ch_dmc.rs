use crate::{bus::SystemBus, SystemControl};



const PERIOD_LOOKUP: [u32; 0x10] = [214, 190, 170, 160, 143, 127, 113, 107, 95, 80, 71, 64, 53, 42, 36, 27];

pub struct Dmc {
    pub irq_enabled_flag: bool,
    pub irq_flag: bool,
    pub loop_flag: bool,

    sample_address: usize,
    address_counter: usize,
    sample_length: usize,
    pub bytes_left: usize,

    sample_buffer: Option<u8>,
    output_shift_reg: u8,
    shift_reg_index: usize,
    silence_flag: bool,

    pub output_level: u8,

    period: u32,
    cycles: u32,
}

impl SystemControl for Dmc {
    fn reset(&mut self) {
        self.irq_enabled_flag = false;
        self.irq_flag = false;
        self.loop_flag = false;
        self.sample_address = 0x0000;
        self.address_counter = 0x0000;
        self.sample_length = 0x0000;
        self.bytes_left = 0x0000;
        self.sample_buffer = None;
        self.output_shift_reg = 0;
        self.shift_reg_index = 0;
        self.silence_flag = false;
        self.output_level = 0;
        self.period = PERIOD_LOOKUP[0];
        self.cycles = 0;
    }
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            irq_enabled_flag: false,
            irq_flag: false,
            loop_flag: false,

            sample_address: 0x0000,
            address_counter: 0x0000,
            sample_length: 0x0000,
            bytes_left: 0x0000,

            sample_buffer: None,
            output_shift_reg: 0,
            shift_reg_index: 0,
            silence_flag: false,
            
            output_level: 0,

            period: PERIOD_LOOKUP[0],
            cycles: 0,
        }
    }

    pub fn set_sample_address(&mut self, byte: u8) {
        self.sample_address = 0xC000 | ((byte as usize) << 6);
        self.address_counter = self.sample_address;
    }

    pub fn set_sample_length(&mut self, byte: u8) {
        self.sample_length = ((byte as usize) << 4) | 0x0001;
        self.bytes_left = self.sample_length;
    }

    pub fn set_period(&mut self, period: usize) {
        self.period = PERIOD_LOOKUP[period];
    }

    pub fn write_status(&mut self, bit: bool) {
        if bit {
            if !self.sample_buffer.is_none() {
                self.restart();
            }
        } else {
            self.bytes_left = 0;
        }

        self.irq_flag = false;
    }

    #[inline]
    pub fn restart(&mut self) {
        self.address_counter = self.sample_address;
        self.bytes_left = self.sample_length;
    }

    pub fn clock(&mut self, bus: &mut SystemBus) -> u8 {
        
        if self.cycles == 0 {
            self.cycles = self.period;

            if self.sample_buffer.is_none() && self.bytes_left > 0 {
                // stalls 1-4 cycles depending on various factors in real hardware
                bus.dmc_read_stall = 2;

                self.sample_buffer = Some(bus.cpu_read(self.address_counter, false).unwrap_or_default());
                self.address_counter += 1;
                self.bytes_left -= 1;

                if self.address_counter > 0xFFFF {
                    self.address_counter = 0x8000;
                }

                if self.bytes_left == 0 {

                    if self.loop_flag {
                        self.restart();
                    } else if self.irq_enabled_flag {
                        self.irq_flag = true;
                    }         
                }
            }

            if self.shift_reg_index == 0 {
                self.shift_reg_index = 8;

                match self.sample_buffer {
                    None => self.silence_flag = true,
                    Some(sample_buffer) => {
                        self.silence_flag = false;
                        self.output_shift_reg = sample_buffer;
                        self.sample_buffer = None;
                    },
                };
            }

            if !self.silence_flag {
                if self.output_shift_reg & 0x01 != 0 {
                    if self.output_level <= 125 {
                        self.output_level += 2;
                    }
                } else {
                    if self.output_level >= 2 {
                        self.output_level -= 2;
                    }
                }

                self.output_shift_reg >>= 1;
            }

            if self.shift_reg_index > 0 {
                self.shift_reg_index -= 1;
            }
        }

        self.cycles -= 1;

        self.output_level
    }
}