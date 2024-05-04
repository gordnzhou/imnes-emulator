extern crate sdl2;

#[macro_use]
extern crate lazy_static;

mod cpu;

use cpu::Cpu6502;

fn main() {

    let mut cpu = Cpu6502::new();

    cpu.execute();

    println!("Hello, world!");
}
