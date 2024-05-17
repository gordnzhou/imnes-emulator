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

use bus::SystemBus;
use cartridge::CartridgeNes;
use cpu::Cpu6502;
use ppu::{Ppu2C03, SdlScreen, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use std::io::Write;
use std::fs::OpenOptions;

const ROM_PATH: &str = "roms/smb3.nes";

const SCREEN_SCALE: u32 = 3;

pub trait SystemControl {
    fn reset(&mut self);
}

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

fn main() -> Result<(), String> {
    clear_log_file().unwrap();

    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;

    let window_width = (DISPLAY_WIDTH as u32) * SCREEN_SCALE;
    let window_height = (DISPLAY_HEIGHT as u32) * SCREEN_SCALE;
    let window = video_subsystem
        .window("Gameboy Emulator", window_width, window_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas= window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.window_mut().set_title("NES Emulator").unwrap();

    let mut event_pump = sdl_context.event_pump()?;

    let cartridge = match CartridgeNes::from_ines_file(ROM_PATH) {
        Ok(cartridge) => cartridge,
        Err(e) => panic!("Unable to load cartridge: {}", e)
    };

    let sdl_screen = SdlScreen::new(canvas);

    let mut total_cycles: u64 = 0;
    let mut joypad_state = 0;
    let mut cpu = Cpu6502::new();
    let mut ppu = Ppu2C03::new(Box::new(sdl_screen));
    let mut bus = SystemBus::new(cartridge);

    bus.reset(); 
    cpu.reset(&mut bus);
    ppu.reset();

    loop {
        if total_cycles % 3 == 0 {
            if bus.dma_transferring {
                bus.dma_clock(total_cycles as u32);
            } else {
                cpu.clock(&mut bus);
            }
        }

        ppu.clock(&mut bus);

        if total_cycles % 1000 == 0 {
            match get_events(&mut event_pump, &mut joypad_state) {
                Ok(_) => bus.update_joypad_state(joypad_state),
                Err(e) => panic!("Emulator exited: {}", e)
            }
        }

        if ppu.nmi_requested() {
            cpu.nmi(&mut bus);
        }

        if bus.cartridge.mapper.irq_active() {
            cpu.irq(&mut bus);
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
