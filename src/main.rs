extern crate sdl2;

#[macro_use]
extern crate lazy_static;

mod cpu;
mod bus;

use cpu::Cpu6502;
use std::fs::read;

use crate::bus::Bus;

const ROM_PATH: &str = "roms/6502_functional_test.bin";

fn main() {
    let data = match read(ROM_PATH) {
        Ok(data) => data,
        Err(e) => panic!("Unable to read file: {}", e),
    };
    
    let mut bus = Bus::new();
    bus.load_memory(&data);

    let mut cpu = Cpu6502::new(bus);
    cpu.execute();

    println!("Hello, world!");
}
