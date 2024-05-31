use crate::{cartridge::PRG_ROM_SIZE, SystemControl};

use super::{Mapper, PRG_ROM_END, PRG_ROM_HI_END, PRG_ROM_HI_START, PRG_ROM_LO_END, PRG_ROM_LO_START, PRG_ROM_START};

pub struct Mapper2 {
    prg_rom_banks: usize,
    prg_bank_lo: usize,
}

impl SystemControl for Mapper2 {
    fn reset(&mut self) {
        self.prg_bank_lo = 0;
    }
}

impl Mapper for Mapper2 {
    fn mapped_cpu_read(&self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            PRG_ROM_LO_START..=PRG_ROM_LO_END => {
                Some(prg_rom[self.prg_bank_lo * PRG_ROM_SIZE + (addr & 0x3FFF)])
            },
            PRG_ROM_HI_START..=PRG_ROM_HI_END => {
                Some(prg_rom[(self.prg_rom_banks - 1) * PRG_ROM_SIZE + (addr & 0x3FFF)])
            },
            _ => None
        }
    }

    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => {
                self.prg_bank_lo = (byte & 0b00001111) as usize;
                true
            }
            _ => false
        }
    }

    fn mapped_ppu_read(&self, chr_rom: &Vec<u8>, addr: usize) -> u8 {
        chr_rom[addr]
    }
}

impl Mapper2 {
    pub fn new(prg_rom_banks: usize) -> Self {
        Self {
            prg_rom_banks,
            prg_bank_lo: 0,
        }
    }
}