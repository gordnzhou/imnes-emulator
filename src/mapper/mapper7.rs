use crate::{cartridge::{CHR_ROM_SIZE, PRG_ROM_SIZE}, SystemControl};

use super::{Mapper, PRG_ROM_END, PRG_ROM_START};

pub struct Mapper7 {
    prg_rom_select: usize,
    chr_rom_1kb: usize,
}

impl SystemControl for Mapper7 {
    fn reset(&mut self) {
        self.prg_rom_select = 0;
        self.chr_rom_1kb = 0;
    }
}

impl Mapper for Mapper7 {
    fn mapped_cpu_read(&mut self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => {
                Some(prg_rom[self.prg_rom_select * (PRG_ROM_SIZE << 1) + (addr & 0x7FFF)])
            }
            _ => None
        }
    }

    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => {
                self.chr_rom_1kb = ((byte & 0b00010000) != 0) as usize;
                self.prg_rom_select = (byte & 0b00000111) as usize;
                true
            }
            _ => false
        }
    }

    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8 {
        chr_rom[self.chr_rom_1kb * (CHR_ROM_SIZE >> 1) + (addr & 0x0FFF)]
    }
}

impl Mapper7 {
    pub fn new() -> Self {
        Self {
            prg_rom_select: 0,
            chr_rom_1kb: 0,
        }
    }
}