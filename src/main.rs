extern crate sdl2;

#[macro_use]
extern crate lazy_static;

mod cpu;
mod bus;

use cpu::Cpu6502;
use std::fs::read;

use crate::bus::Bus;
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

    let mut bus = Bus::new();
    bus.load_rom(&data);

    let mut cpu = Cpu6502::new(bus);
    cpu.execute();
}
