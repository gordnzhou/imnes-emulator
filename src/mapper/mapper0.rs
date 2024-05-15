use super::Mapper;
use crate::cartridge::PRG_ROM_SIZE;

pub struct Mapper0 {
    cpu_ram: [u8; 0x3FE0],
    prg_rom_banks: usize, // 1 or 2 bank(s)
    chr_rom_banks: usize, // 0 or 1 bank (used as RAM if no banks)
}

impl Mapper for Mapper0 {
    fn mapped_cpu_read(&mut self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            0x4020..=0x7FFF => {
                Some(self.cpu_ram[addr - 0x4020])
            }
            0x8000..=0xFFFF => {
                let addr = addr - 0x8000;

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
            0x4020..=0x7FFF => {
                self.cpu_ram[addr - 0x4020] = byte; 
                true
            }
            _ => false
        }
    }
    
    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8 { 
        match addr {
            0x0000..=0x1FFF => chr_rom[addr],
            _ => unreachable!(),
        }
    }
    
    fn mapped_ppu_write(&mut self, chr_rom: &mut Vec<u8>, addr: usize, byte: u8) {
        if self.chr_rom_banks == 0 {
            chr_rom[addr] = byte;
        }
    }
}

impl Mapper0 {
    pub fn new(prg_rom_banks: usize, chr_rom_banks: usize) -> Self {
        Self {
            cpu_ram: [0; 0x3FE0],
            prg_rom_banks,
            chr_rom_banks,
        }
    }
}