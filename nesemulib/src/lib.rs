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

pub use apu::Apu2A03;
pub use bus::SystemBus;
pub use cartridge::CartridgeNes;
pub use cpu::Cpu6502;
pub use ppu::Ppu2C03;

pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 240;

pub trait SystemControl {
    fn reset(&mut self);
}