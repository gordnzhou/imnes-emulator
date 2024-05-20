mod frame_sequencer;
mod length_counter;
mod envelope;
mod sweep;
mod ch_pulse;

use self::frame_sequencer::FrameSequencer;
use self::ch_pulse::Pulse;

use crate::SystemControl;

// Based on a NTSC system
const TIME_PER_6502_CLOCK: f32 = 1e9 / 1_789_773.0;

const DUTY_SEQUENCES: [u8; 4] = [
    0b01000000,
    0b01100000,
    0b01111000,
    0b10011111,
];


pub struct Apu2A03 {
    time_per_sample: f32,
    time_since_last_sample: f32,

    frame_sequencer: FrameSequencer,
    pulse1: Pulse,
    pulse1_sample: u8,
    pulse2: Pulse,
    pulse2_sample: u8,

    total_cycles: u32,
    interrupt_flag: bool,
}

impl SystemControl for Apu2A03 {
    fn reset(&mut self) {
        self.pulse1.length_counter.enabled_flag = false;
    }
}

impl Apu2A03 {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            time_per_sample: 1e9 / (sample_rate as f32),
            time_since_last_sample: 0.0,

            frame_sequencer: FrameSequencer::new(),
            pulse1: Pulse::new(true),
            pulse1_sample: 0,
            pulse2: Pulse::new(false),
            pulse2_sample: 0,

            total_cycles: 0,
            interrupt_flag: false,
        }
    } 

    pub fn irq_active(&mut self) -> bool {
        let ret = self.interrupt_flag;
        self.interrupt_flag = false;
        ret
    }

    /// ASSUMING this function is called ONCE PER CPU CYCLE, outputs a sample
    /// matching the APU's sample_rate (based on the NES's CPU frequency)
    pub fn cpu_try_clock_sample(&mut self) -> Option<f32> {
        self.time_since_last_sample += TIME_PER_6502_CLOCK;

        if self.time_since_last_sample < self.time_per_sample {
            return None;
        }

        self.time_since_last_sample -= self.time_per_sample;


        // TODO: Mix digital channels to generate final analog sample
        let mut sample =  -1.0 + (self.pulse1_sample as f32 / 7.5);
        sample +=  -1.0 + (self.pulse2_sample as f32 / 7.5);

        Some(sample / 2.0)
    }


    pub fn cpu_clock(&mut self) {
        match self.frame_sequencer.clock(&mut self.interrupt_flag) {
            Some((_mode, 1)) | Some((_mode, 3)) => {
                // Quarter frame clock
                self.pulse1.envelope.clock();
                self.pulse2.envelope.clock();
            },
            Some((_, 2)) | Some((0, 4)) | Some((1, 5)) => {
                // Half frame clock
                self.pulse1.envelope.clock();
                self.pulse2.envelope.clock();

                self.pulse1.length_counter.clock();
                self.pulse2.length_counter.clock();

                self.pulse1.sweep.clock();
                self.pulse2.sweep.clock();
            },
            _ => {}
        }

        self.total_cycles += 1;
        if self.total_cycles % 2 == 0 {

            // Processes digital samples ranging from 0-15
            self.pulse1_sample = self.pulse1.clock();
            self.pulse2_sample = self.pulse2.clock();
        }
    } 

    pub fn read_register(&mut self, addr: usize) -> u8 {
        match addr {
            0x4015 => {
                let mut byte = 0;

                if self.pulse1.length_counter.counter > 0 { byte |= 1 << 0; }
                if self.pulse2.length_counter.counter > 0 { byte |= 1 << 1; }
                if self.interrupt_flag { byte |= 1 << 6}
                self.interrupt_flag = false;

                byte
            },
            _ => 0 // open bus
        }
    }

    pub fn write_register(&mut self, addr: usize, byte: u8) {
        match addr {
            0x4000 => {
                self.pulse1.duty_sequence = DUTY_SEQUENCES[((byte & 0b11000000) >> 6) as usize];

                self.pulse1.envelope.loop_flag = (byte & 0b00100000) != 0;
                self.pulse1.length_counter.halted = (byte & 0b00100000) != 0;

                self.pulse1.envelope.constant_flag = (byte & 0b00010000) != 0;

                self.pulse1.envelope.set_volume(byte & 0b00001111);
            },
            0x4001 => self.pulse1.sweep.write_byte(byte),
            0x4002 => {
                self.pulse1.sweep.period &= 0b11100000000;
                self.pulse1.sweep.period |= byte as u32;
            },
            0x4003 => {
                self.pulse1.length_counter.load_counter((byte & 0b11111000) >> 3);
                self.pulse1.envelope.start_flag = true;

                self.pulse1.sweep.period &= 0b00011111111;
                self.pulse1.sweep.period |= ((byte as u32) & 0b00000111) << 8;

                self.pulse1.duty_step = 0;
            },
            0x4004 => {
                self.pulse2.duty_sequence = DUTY_SEQUENCES[((byte & 0b11000000) >> 6) as usize];

                self.pulse2.envelope.loop_flag = (byte & 0b00100000) != 0;
                self.pulse2.length_counter.halted = (byte & 0b00100000) != 0;

                self.pulse2.envelope.constant_flag = (byte & 0b00010000) != 0;

                self.pulse2.envelope.set_volume(byte & 0b00001111);
            },
            0x4005 => self.pulse2.sweep.write_byte(byte),
            0x4006 => {
                self.pulse2.sweep.period &= 0b11100000000;
                self.pulse2.sweep.period |= byte as u32;
            },
            0x4007 => {
                self.pulse2.length_counter.load_counter((byte & 0b11111000) >> 3);
                self.pulse2.envelope.start_flag = true;

                self.pulse2.sweep.period &= 0b00011111111;
                self.pulse2.sweep.period |= ((byte as u32) & 0b00000111) << 8;

                self.pulse2.duty_step = 0;
            },
            0x4008 => {},
            0x4009 => {},
            0x400A => {},
            0x400B => {},
            0x400C => {},
            0x400D => {},
            0x400E => {},
            0x400F => {},
            0x4010 => {},
            0x4011 => {},
            0x4012 => {},
            0x4013 => {},
            0x4015 => { // Status
                self.pulse1.length_counter.enabled_flag = (byte & 0b00000001) != 0;
                self.pulse2.length_counter.enabled_flag = (byte & 0b00000010) != 0;
            },
            0x4017 => { // Frame Counter
                self.frame_sequencer.mode = (byte & 0b10000000) != 0;
                self.frame_sequencer.irq_inhibit_flag = (byte & 0b01000000) != 0;

                if self.frame_sequencer.irq_inhibit_flag {
                    self.interrupt_flag = false;
                }
            },
            _ => {}
        }
    }
}

#[cfg(test)]
impl Apu2A03 {
    pub fn test_new() -> Self {
        Apu2A03 {
            time_per_sample: 10000.0, 
            time_since_last_sample: 0.0,

            frame_sequencer: FrameSequencer::new(),
            pulse1: Pulse::new(true),
            pulse1_sample: 0,
            pulse2: Pulse::new(false),
            pulse2_sample: 0,

            total_cycles: 0,
            interrupt_flag: false,
        }
    }
}