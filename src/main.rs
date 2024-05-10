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
mod palette;

use bus::Bus;
use cartridge::CartridgeNes;
use cpu::Cpu6502;
use ppu::Ppu2C03;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::EventPump;

use std::io::Write;
use std::fs::OpenOptions;

const ROM_PATH: &str = "roms/nestest.nes";
const SCREEN_SCALE: u32 = 3;
const DISPLAY_WIDTH: u32 = 256;
const DISPLAY_HEIGHT: u32 = 240;

fn clear_log_file() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("logs/log.txt")?;

    write!(file, "")
}

fn main() -> Result<(), String> {
    clear_log_file().unwrap();
    let mut total_cycles: u32 = 0;

    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;

    let window_width = DISPLAY_WIDTH * SCREEN_SCALE;
    let window_height = DISPLAY_HEIGHT * SCREEN_SCALE;
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

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .map_err(|e| e.to_string())
        .unwrap();
    
    let rect = Rect::new(0, 0, window_width, window_height);

    let cartridge = match CartridgeNes::from_ines_file(ROM_PATH) {
        Ok(cartridge) => cartridge,
        Err(e) => panic!("{}", e)
    };


    let mut cpu = Cpu6502::new();
    let mut ppu = Ppu2C03::new();
    let mut bus = Bus::new(cartridge);
    let mut pattern_index = 0;
    let mut palette = 0;

    cpu.reset(&mut bus);
    
    loop {
        ppu.clock(&mut bus);
    
        if total_cycles % 3 == 0 {
            cpu.clock(&mut bus);
        }

        if total_cycles % 3000 == 0 {
            match get_events(&mut event_pump, &mut pattern_index, &mut palette) {
                Ok(_) => {},
                Err(e) => panic!("{}", e)
            }

            // let pattern_table = ppu.get_pattern_table(pattern_index, &mut bus, palette)
            //     .iter()
            //     .flat_map(|color| vec![color.2, color.1, color.0, 0xFF])
            //     .collect::<Vec<u8>>();

            let name_table = ppu.get_name_table(&mut bus)
                .iter()
                .flat_map(|color| vec![color.2, color.1, color.0, 0xFF])
                .collect::<Vec<u8>>();

            texture
                .update(None, &name_table, 4 * DISPLAY_WIDTH as usize)
                .expect("texture update failed");

            canvas.copy(&texture, None, rect).unwrap();
            canvas.present();
        }

        if ppu.nmi_requested() {
            cpu.nmi(&mut bus);
        }

        total_cycles = total_cycles.wrapping_add(1);
    }
}

fn get_events(event_pump: &mut EventPump, pattern_index: &mut usize, palette: &mut u16) -> Result<(), String> { 
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return Err(String::from("User Exited"));
            },
            Event::KeyDown { keycode: Some(key), ..} => {  
                match key {
                    Keycode::Q => *pattern_index = 1 - *pattern_index,
                    Keycode::W => *palette = (*palette + 1) % 8,
                    _ => {}
                };
            }
            Event::KeyUp { keycode: Some(_key), .. } => {
                // Key Released
            }
            _ => {}
        }
    }

    Ok(())
}
