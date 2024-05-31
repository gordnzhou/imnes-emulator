use crate::{cartridge::{CHR_ROM_SIZE, PRG_ROM_SIZE}, SystemControl};

use super::{Mapper, PRG_ROM_END, PRG_ROM_START};

pub struct Mapper3 {
    prg_rom_banks: usize, // 1 or 2 16Kb banks
    chr_bank_select: usize,
}

impl SystemControl for Mapper3 {
    fn reset(&mut self) { 
        self.chr_bank_select = 0;
    }
}

impl Mapper for Mapper3 {
    fn mapped_cpu_read(&self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => {
                let addr = addr - PRG_ROM_START;

                Some(if self.prg_rom_banks == 1 {
                    // address wraps back for ROMs with only a single 16KB bank
                    prg_rom[addr % PRG_ROM_SIZE]
                } else {
                    prg_rom[addr]
                })
            }
            _ => None
        }
    }

    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => {
                self.chr_bank_select = (byte & 0b00000111) as usize;
                true
            }
            _ => false
        }
    }

    fn mapped_ppu_read(&self, chr_rom: &Vec<u8>, addr: usize) -> u8 {
        chr_rom[self.chr_bank_select * CHR_ROM_SIZE + addr]
    }
}

impl Mapper3 {
    pub fn new(prg_rom_banks: usize) -> Self {
        Self {
            prg_rom_banks,
            chr_bank_select: 0,
        }
    }
}