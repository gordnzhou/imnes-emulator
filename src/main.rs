extern crate sdl2;

#[macro_use]
extern crate lazy_static;

mod cpu;
mod bus;
mod cartridge;
mod ppu;

use bus::Bus;
use cartridge::CartridgeNes;
use cpu::Cpu6502;
use ppu::Ppu2C03;
use std::fs::read;

use std::io::Write;
use std::fs::OpenOptions;

const ROM_PATH: &str = "roms/nestest.nes";

fn clear_log_file() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("logs/log.txt")?;

    write!(file, "")
}

fn main() {
    let data = match read(ROM_PATH) {
        Ok(data) => data,
        Err(e) => panic!("Unable to read file: {}", e),
    };

    clear_log_file().unwrap();
    let mut total_cycles: u32 = 0;

    let cartridge = CartridgeNes::from_ines_bytes(&data);

    let mut bus = Bus::new(cartridge);

    let mut cpu = Cpu6502::new();
    let mut ppu = Ppu2C03::new();

    cpu.reset(&mut bus);
    
    loop {
        ppu.clock(&mut bus);
    
        if total_cycles % 3 == 0 {
            cpu.clock(&mut bus);
        }

        if ppu.nmi_requested() {
            cpu.nmi(&mut bus);
        }

        total_cycles = total_cycles.wrapping_add(1);
    }
}
