use super::{Mapper, PRG_ROM_END, PRG_ROM_START};
use crate::{cartridge::PRG_ROM_SIZE, SystemControl};


const SAVE_RAM_START: usize = 0x4020;
const SAVE_RAM_END: usize = 0x7FFF;

const SAVE_RAM_SIZE: usize = 0x3FE0;

pub struct Mapper0 {
    save_ram: [u8; SAVE_RAM_SIZE],
    prg_rom_banks: usize, // 1 or 2 bank(s)
}

impl SystemControl for Mapper0 {
    fn reset(&mut self) { }
}

impl Mapper for Mapper0 {
    fn mapped_cpu_read(&self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            SAVE_RAM_START..=SAVE_RAM_END => {
                Some(self.save_ram[addr - SAVE_RAM_START])
            }
            PRG_ROM_START..=PRG_ROM_END => {
                let addr = addr - PRG_ROM_START;

                Some(if self.prg_rom_banks == 1 {
                    // address wraps back for ROMs with only a single 16KB bank
                    prg_rom[addr % PRG_ROM_SIZE]
                } else {
                    prg_rom[addr]
                })
            },
            _ => None
        }
    }
    
    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        match addr {
            SAVE_RAM_START..=SAVE_RAM_END => {
                self.save_ram[addr - SAVE_RAM_START] = byte; 
                true
            }
            _ => false
        }
    }
    
    fn mapped_ppu_read(&self, chr_rom: &Vec<u8>, addr: usize) -> u8 { 
        match addr {
            0x0000..=0x1FFF => chr_rom[addr],
            _ => unreachable!(),
        }
    }
}

impl Mapper0 {
    pub fn new(prg_rom_banks: usize) -> Self {
        Self {
            save_ram: [0; SAVE_RAM_SIZE],
            prg_rom_banks,
        }
    }
}