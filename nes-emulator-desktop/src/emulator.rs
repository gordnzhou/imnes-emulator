use std::{io, sync::mpsc::SyncSender, time::Duration};

use nesemulib::{Apu2A03, CartridgeNes, Cpu6502, Ppu2C03, SystemBus, SystemControl, BASE_PPU_FREQUENCY, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};
use cpal::{SampleRate, StreamConfig};

const SAMPLING_RATE_HZ: u32 = 48000;
const AUDIO_SAMPLES: usize = 512;

pub struct Emulator {
    cpu: Cpu6502,
    ppu: Ppu2C03,
    bus: SystemBus,
    pub total_cycles: u64,
    joypad_state: u8,

    _stream: Stream,

    audio_tx: SyncSender<[f32; AUDIO_SAMPLES]>,
    audio_buffer: [f32; AUDIO_SAMPLES],
    buffer_index: usize,
}

impl Emulator {
    pub fn new() -> Result<Self, io::Error> {
        let cartridge = CartridgeNes::from_ines_file("roms/smb.nes")?;

        let apu = Apu2A03::new(SAMPLING_RATE_HZ);
        let mut cpu = Cpu6502::new(apu);
        let mut ppu = Ppu2C03::new();
        let mut bus = SystemBus::new(cartridge);

        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel::<[f32; AUDIO_SAMPLES]>(4);

        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(SAMPLING_RATE_HZ),
            buffer_size: cpal::BufferSize::Fixed(AUDIO_SAMPLES as u32),
        };
        
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                match audio_rx.try_recv() {
                    Ok(buffer) => {
                        let len = data.len().min(buffer.len());
                        data[0..len].copy_from_slice(&buffer[0..len]);
                    }
                    Err(_) => {
                        // for i in 0..data.len() {
                        //     data[i] = 0.0;
                        // }
                    }
                } 
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None
        ).expect("Failed to build output stream");

        stream.play().expect("Failed to start stream");


        bus.reset(); 
        cpu.reset(&mut bus);
        ppu.reset();

        Ok(Self {
            cpu,
            ppu,
            bus,
            total_cycles: 0,
            joypad_state: 0,

            _stream: stream,

            audio_tx,
            audio_buffer: [0.0; AUDIO_SAMPLES],
            buffer_index: 0,
        })
    }

    pub fn run_for_duration(&mut self, duration: Duration) {
        let mut duration_cycles = duration.as_nanos() as u64 / (1e9 / BASE_PPU_FREQUENCY) as u64;
        while duration_cycles > 0 {
            self.ppu.clock(&mut self.bus);
    
            if self.total_cycles % 3 == 0 {
                // CPU clock
                if self.bus.dma_transferring {
                    self.bus.dma_clock(self.total_cycles as u32);
                } else if self.bus.dmc_read_stall > 0 {
                    self.bus.dmc_read_stall -= 1;
                } else {
                    self.cpu.clock(&mut self.bus);
                }
    
                self.cpu.apu.cpu_clock(&mut self.bus);
    
                match self.cpu.apu.cpu_try_clock_sample() {
                    Some(sample) => {
                        self.audio_buffer[self.buffer_index] = sample;
                        self.buffer_index += 1;
                
                        if self.buffer_index == AUDIO_SAMPLES {
                            let _ = self.audio_tx.try_send(self.audio_buffer);
                            self.buffer_index = 0;
                        }
                    }
                    None => {}
                }
            }
    
            if self.ppu.nmi_requested() {
                self.cpu.nmi(&mut self.bus);
            }
    
            if self.bus.irq_active() || self.cpu.apu.irq_active() {
                self.cpu.irq(&mut self.bus);
            }
    
            self.total_cycles += 1;
            duration_cycles -= 1;
        }
    }
 
    pub fn get_updated_frame(&mut self) -> Option<[u8; 4 * DISPLAY_WIDTH * DISPLAY_HEIGHT]> {    
        match self.ppu.try_get_frame() {
            Some(colour_frame) => {
                let mut frame = [0xFF; 4 * DISPLAY_WIDTH * DISPLAY_HEIGHT];

                for i in 0..DISPLAY_WIDTH * DISPLAY_HEIGHT {
                    frame[4 * i + 0] = colour_frame[i].0;
                    frame[4 * i + 1] = colour_frame[i].1;
                    frame[4 * i + 2] = colour_frame[i].2;
                }
                
                Some(frame)
            }
            None => None
        }
    }

    pub fn update_joypad(&mut self, physical_key: PhysicalKey, state: ElementState) {
        let pressed = matches!(state, ElementState::Pressed);
        let mask = match physical_key {
            PhysicalKey::Code(KeyCode::KeyD) => 0x01,
            PhysicalKey::Code(KeyCode::KeyA) => 0x02,
            PhysicalKey::Code(KeyCode::KeyS) => 0x04,
            PhysicalKey::Code(KeyCode::KeyW) => 0x08,
            PhysicalKey::Code(KeyCode::KeyI) => 0x10,
            PhysicalKey::Code(KeyCode::KeyJ) => 0x20,
            PhysicalKey::Code(KeyCode::KeyK) => 0x40,
            PhysicalKey::Code(KeyCode::KeyL) => 0x80,
            _ => return
        };

        if pressed {
            self.joypad_state |= mask;
        } else {
            self.joypad_state &= !mask;
        }

        self.bus.update_joypad_state(self.joypad_state, 0);
    }
}