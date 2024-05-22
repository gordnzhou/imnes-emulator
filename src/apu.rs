mod frame_sequencer;
mod length_counter;
mod envelope;
mod sweep;
mod ch_pulse;
mod ch_triangle;
mod ch_noise;
mod ch_dmc;
mod lookup;

use self::ch_dmc::Dmc;
use self::ch_noise::Noise;
use self::frame_sequencer::FrameSequencer;
use self::ch_triangle::Triangle;
use self::ch_pulse::Pulse;
use self::lookup::{PULSE_TABLE, TND_TABLE};

use crate::bus::SystemBus;
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
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    pulse1_sample: u8,
    pulse2_sample: u8,
    triangle_sample: u8,
    noise_sample: u8,
    dmc_sample: u8,

    total_cycles: u32,
    interrupt_flag: bool,
}

impl SystemControl for Apu2A03 {
    fn reset(&mut self) {
        self.pulse1.length_counter.enabled_flag = false;
        self.pulse2.length_counter.enabled_flag = false;
        self.triangle.length_counter.enabled_flag = false;
        self.noise.length_counter.enabled_flag = false;
    }
}

impl Apu2A03 {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            time_per_sample: 1e9 / (sample_rate as f32),
            time_since_last_sample: 0.0,

            frame_sequencer: FrameSequencer::new(),
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),

            pulse2_sample: 0,
            pulse1_sample: 0,
            triangle_sample: 0,
            noise_sample: 0,
            dmc_sample: 0,

            total_cycles: 0,
            interrupt_flag: false,
        }
    } 

    pub fn irq_active(&mut self) -> bool {
        let ret = self.interrupt_flag || self.dmc.irq_flag;
        self.dmc.irq_flag = false;
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

        let pulse_out = PULSE_TABLE[(self.pulse1_sample + self.pulse2_sample) as usize];
        let tnd_out = TND_TABLE[(3 * self.triangle_sample + (self.noise_sample << 1) + self.dmc_sample) as usize];

        Some(pulse_out + tnd_out)
    }


    pub fn cpu_clock(&mut self, bus: &mut SystemBus) {
        match self.frame_sequencer.clock(&mut self.interrupt_flag) {
            Some((_mode, 1)) | Some((_mode, 3)) => {
                // Quarter frame clock
                self.pulse1.envelope.clock();
                self.pulse2.envelope.clock();
                self.noise.envelope.clock();
                self.triangle.linear_counter.clock();
            },
            Some((_, 2)) | Some((0, 4)) | Some((1, 5)) => {
                // Half frame clock
                self.pulse1.envelope.clock();
                self.pulse2.envelope.clock();
                self.noise.envelope.clock();
                self.triangle.linear_counter.clock();

                self.pulse1.length_counter.clock();
                self.pulse2.length_counter.clock();
                self.triangle.length_counter.clock();
                self.noise.length_counter.clock();

                self.pulse1.sweep.clock();
                self.pulse2.sweep.clock();
            },
            _ => {}
        }

        self.total_cycles += 1;

        self.triangle_sample = self.triangle.clock();

        if self.total_cycles % 2 == 0 {
            self.pulse1_sample = self.pulse1.clock();
            self.pulse2_sample = self.pulse2.clock();
            self.noise_sample = self.noise.clock();
            self.dmc_sample = self.dmc.clock(bus);
        }
    } 

    pub fn read_register(&mut self, addr: usize) -> u8 {
        match addr {
            0x4015 => {
                let mut byte = 0;

                if self.pulse1.length_counter.counter   > 0 { byte |= 1 << 0; }
                if self.pulse2.length_counter.counter   > 0 { byte |= 1 << 1; }
                if self.triangle.length_counter.counter > 0 { byte |= 1 << 2; }
                if self.noise.length_counter.counter    > 0 { byte |= 1 << 3; }
                if self.dmc.bytes_left > 0                  { byte |= 1 << 4; }

                if self.interrupt_flag                      { byte |= 1 << 6; }
                if self.dmc.irq_flag                        { byte |= 1 << 7; }

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
            0x4001 => {
                self.pulse1.sweep.write_byte(byte)
            },
            0x4002 => {
                self.pulse1.sweep.period &= 0b11100000000;
                self.pulse1.sweep.period |= byte as u32;
            },
            0x4003 => {
                self.pulse1.length_counter.load_counter((byte & 0b11111000) >> 3);
                self.pulse1.envelope.start_flag = true;

                self.pulse1.sweep.period &= 0b00011111111;
                self.pulse1.sweep.period |= ((byte as u32) & 0b00000111) << 8;
                self.pulse1.cycles = self.pulse1.sweep.period;

                self.pulse1.duty_step = 0;
            },
            0x4004 => {
                self.pulse2.duty_sequence = DUTY_SEQUENCES[((byte & 0b11000000) >> 6) as usize];

                self.pulse2.envelope.loop_flag = (byte & 0b00100000) != 0;
                self.pulse2.length_counter.halted = (byte & 0b00100000) != 0;

                self.pulse2.envelope.constant_flag = (byte & 0b00010000) != 0;

                self.pulse2.envelope.set_volume(byte & 0b00001111);
            },
            0x4005 => {
                self.pulse2.sweep.write_byte(byte)
            },
            0x4006 => {
                self.pulse2.sweep.period &= 0b11100000000;
                self.pulse2.sweep.period |= byte as u32;
            },
            0x4007 => {
                self.pulse2.length_counter.load_counter((byte & 0b11111000) >> 3);
                self.pulse2.envelope.start_flag = true;

                self.pulse2.sweep.period &= 0b00011111111;
                self.pulse2.sweep.period |= ((byte as u32) & 0b00000111) << 8;
                self.pulse1.cycles = self.pulse1.sweep.period;

                self.pulse2.duty_step = 0;
            },
            0x4008 => {
                self.triangle.length_counter.halted = (byte & 0b10000000) != 0;
                self.triangle.linear_counter.control_flag = (byte & 0b10000000) != 0;
                
                self.triangle.linear_counter.reload = byte & 0b01111111;
            },
            0x4009 => {
                // Unused
            },
            0x400A => {
                self.triangle.period &= 0b11100000000;
                self.triangle.period |= byte as u32;
            },
            0x400B => {
                self.triangle.period &= 0b00011111111;
                self.triangle.period |= ((byte as u32) & 0b00000111) << 8;

                self.triangle.length_counter.load_counter((byte & 0b11111000) >> 3);

                self.triangle.linear_counter.reload_flag = true;
            },
            0x400C => {
                self.noise.envelope.loop_flag = (byte & 0b00100000) != 0;
                self.noise.length_counter.halted = (byte & 0b00100000) != 0;

                self.noise.envelope.constant_flag = (byte & 0b00010000) != 0;

                self.noise.envelope.set_volume(byte & 0b00001111);
            },
            0x400D => {
                // Unused
            },
            0x400E => {
                self.noise.shift_mode = (byte & 0b10000000) != 0;

                self.noise.set_period((byte & 0b00001111) as usize);
            },
            0x400F => {
                self.noise.length_counter.load_counter((byte & 0b11111000) >> 3);

                self.noise.envelope.start_flag = true;
            },
            0x4010 => {
                self.dmc.irq_enabled_flag = (byte & 0b10000000) != 0;
                self.dmc.loop_flag = (byte & 0b01000000) != 0;
                self.dmc.set_period((byte & 0b00001111) as usize);

                self.dmc.irq_flag &= self.dmc.irq_enabled_flag;
            },
            0x4011 => {
                self.dmc.output_level = byte & 0b01111111;
            },
            0x4012 => {
                self.dmc.set_sample_address(byte);
            },
            0x4013 => {
                self.dmc.set_sample_length(byte);
            },
            0x4015 => { // Status
                self.pulse1.length_counter.enabled_flag   = (byte & 0b00000001) != 0;
                self.pulse2.length_counter.enabled_flag   = (byte & 0b00000010) != 0;
                self.triangle.length_counter.enabled_flag = (byte & 0b00000100) != 0;
                self.noise.length_counter.enabled_flag    = (byte & 0b00001000) != 0;
                self.dmc.write_status((byte & 0b00010000) != 0);
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
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),

            pulse2_sample: 0,
            pulse1_sample: 0,
            triangle_sample: 0,
            noise_sample: 0,
            dmc_sample: 0,

            total_cycles: 0,
            interrupt_flag: false,
        }
    }
}