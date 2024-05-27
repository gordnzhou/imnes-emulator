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
pub use ppu::*;

pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 240;

// Based on a NTSC system
pub const BASE_CPU_FREQUENCY: f32 = 1_789_773.0;
pub const BASE_PPU_FREQUENCY: f32 = 3.0 * BASE_CPU_FREQUENCY;
pub const DEFAULT_TIME_PER_6502_CLOCK: f32 = 1e9 / BASE_CPU_FREQUENCY;

pub trait SystemControl {
    fn reset(&mut self);
}