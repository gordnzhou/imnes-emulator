extern crate sdl2;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

mod cpu;
mod bus;
mod cartridge;
mod ppu;
mod mapper;
mod apu;

use apu::Apu2A03;
use bus::SystemBus;
use cartridge::CartridgeNes;
use cpu::Cpu6502;
use ppu::Ppu2C03;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

use std::io::Write;
use std::fs::OpenOptions;
use std::sync::mpsc::Receiver;
use std::time::Duration;

const ROM_PATH: &str = "roms/megaman3.nes";
const SCREEN_SCALE: u32 = 3;
const SAMPLING_RATE_HZ: u32 = 44100;
const AUDIO_SAMPLES: usize = 1024;

// (LSB) Right, Left, Down, Up, Start, Select, A, B (MSB)
const KEYMAPPINGS: [Keycode; 8] = [
    Keycode::D,
    Keycode::A,
    Keycode::S,
    Keycode::W,
    Keycode::I,
    Keycode::J,
    Keycode::K,
    Keycode::L,
];

fn clear_log_file() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("logs/log.txt")?;

    write!(file, "")
}

// LIB.RS INTERFACE START
pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 240;

pub trait SystemControl {
    fn reset(&mut self);
}

// LIB.RS INTERFACE END

fn main() -> Result<(), String> {
    clear_log_file().unwrap();

    let sdl_context = sdl2::init()?;

    // SDL video
    let video_subsystem = sdl_context.video()?;
    let window_width = (DISPLAY_WIDTH as u32) * SCREEN_SCALE;
    let window_height = (DISPLAY_HEIGHT as u32) * SCREEN_SCALE;
    let window = video_subsystem
        .window("NES Emulator", window_width, window_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas= window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    // SDL Audio
    let mut audio_buffer = [0.0; AUDIO_SAMPLES];
    let mut audio_buffer_size = 0;
    let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel(2);
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLING_RATE_HZ as i32),
            channels: Some(1),
            samples: Some(AUDIO_SAMPLES as u16),
        };
        let _audio_subsystem = sdl_context.audio()?;
        let _audio_device = _audio_subsystem.open_playback(None, &desired_spec, |_spec| {
            Callback { audio_rx, prev_sample: 0.0 }
        }).unwrap();
        _audio_device.resume();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
        .map_err(|e| e.to_string())
        .unwrap();

    let mut frame = [0; DISPLAY_HEIGHT * DISPLAY_WIDTH * 4];
    for i in 0..(DISPLAY_WIDTH * DISPLAY_HEIGHT) {
        frame[4 * i + 3] = 0xFF;
    }

    let mut event_pump = sdl_context.event_pump()?;

    let cartridge = match CartridgeNes::from_ines_file(ROM_PATH) {
        Ok(cartridge) => cartridge,
        Err(e) => panic!("Unable to load cartridge: {}", e)
    };

    let mut total_cycles: u64 = 0;
    let mut joypad_state = 0;
    let apu = Apu2A03::new(SAMPLING_RATE_HZ);
    let mut cpu = Cpu6502::new(apu);
    let mut ppu = Ppu2C03::new();
    let mut bus = SystemBus::new(cartridge);

    bus.reset(); 
    cpu.reset(&mut bus);
    ppu.reset();

    let mut frame_count = 0;
    let mut last_fps_update = std::time::Instant::now();
    loop {
        ppu.clock(&mut bus);

        if total_cycles % 3 == 0 {
            // CPU clock
            if bus.dma_transferring {
                bus.dma_clock(total_cycles as u32);
            } else if bus.dmc_read_stall > 0 {
                bus.dmc_read_stall -= 1;
            } else {
                cpu.clock(&mut bus);
            }

            cpu.apu.cpu_clock(&mut bus);

            match cpu.apu.cpu_try_clock_sample() {
                Some(sample) => {
                    audio_buffer[audio_buffer_size] = sample;
                    audio_buffer_size += 1;
    
                    if audio_buffer_size == AUDIO_SAMPLES {
                        audio_tx.send(audio_buffer).unwrap();
                        audio_buffer_size = 0;
                    }
                }
                None => {}
            }
        }

        if total_cycles % 8000 == 0 {
            match get_events(&mut event_pump, &mut joypad_state) {
                Ok(_) => bus.update_joypad_state(joypad_state, 0),
                Err(e) => panic!("Emulator exited: {}", e)
            }
        }

        if ppu.nmi_requested() {
            cpu.nmi(&mut bus);
        }

        match ppu.try_get_frame() {
            Some(colour_frame) => {
                for i in 0..DISPLAY_WIDTH*DISPLAY_HEIGHT {
                    frame[4 * i + 0] = colour_frame[i].2;
                    frame[4 * i + 1] = colour_frame[i].1;
                    frame[4 * i + 2] = colour_frame[i].0;
                }
                frame_count += 1;

                texture
                    .update(None, &frame, DISPLAY_WIDTH * 4)
                    .expect("texture update failed");

                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
            None => {}
        }

        if bus.irq_active() || cpu.apu.irq_active() {
            cpu.irq(&mut bus);
        }

        if last_fps_update.elapsed() >= std::time::Duration::from_secs(1) {
            println!("FPS: {}", frame_count);
            frame_count = 0;
            last_fps_update = std::time::Instant::now();
        }

        total_cycles += 1;
    }
}

fn get_events(event_pump: &mut EventPump, joypad_state: &mut u8) -> Result<(), String> { 
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return Err(String::from("User Exited"));
            },
            Event::KeyDown { keycode: Some(key), ..} => {   
                for i in 0..8 {
                    if KEYMAPPINGS[i] == key {
                        *joypad_state |= 1 << i;
                    }
                }
            }
            Event::KeyUp { keycode: Some(key), .. } => {
                for i in 0..8 {
                    if KEYMAPPINGS[i] == key {
                        *joypad_state &= !(1 << i);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

struct Callback {
    audio_rx: Receiver<[f32; AUDIO_SAMPLES]>,
    prev_sample: f32,
}

impl AudioCallback for Callback {
    type Channel = f32;

    fn callback(&mut self, stream: &mut [f32]) {
        match self.audio_rx.recv_timeout(Duration::from_millis(30)) {
            Ok(buffer) => {
                stream[0..].copy_from_slice(&buffer);
                self.prev_sample = buffer[buffer.len() - 1];
            }
            Err(_) => {
                for i in 0..stream.len() {
                    stream[i] = self.prev_sample;
                }
            }
        }
    }
}
