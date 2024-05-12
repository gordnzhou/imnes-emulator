use super::Mapper;
use crate::cartridge::{PRG_ROM_SIZE, CHR_ROM_SIZE};

pub struct Mapper0 {
    ram: [u8; 0x3FE0],
    prg_rom: Vec<[u8; PRG_ROM_SIZE]>,
    chr_rom: [u8; CHR_ROM_SIZE],
    prg_rom_banks: u8, // 1 or 2 bank(s)
    chr_rom_banks: u8, // 0 or 1 bank (used as RAM if no banks)
}

impl Mapper for Mapper0 {
    fn mapped_cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            0x4020..=0x7FFF => {
                self.ram[addr - 0x4020]
            }
            0x8000..=0xFFFF => {
                let addr = addr - 0x8000;

                if self.prg_rom_banks == 1 {
                    self.prg_rom[0][addr % PRG_ROM_SIZE]
                } else {
                    self.prg_rom[addr / PRG_ROM_SIZE][addr % PRG_ROM_SIZE]
                }
            },
            _ => unreachable!()
        }
    }
    
    fn mapped_cpu_write(&mut self, addr: usize, byte: u8) {
        match addr {
            0x4020..=0x7FFF => {
                self.ram[addr - 0x4020] = byte; 
            }
            0x8000..=0xFFFF => {
                let addr = addr - 0x8000;

                if self.prg_rom_banks == 1 {
                    self.prg_rom[0][addr % PRG_ROM_SIZE] = byte;
                } else {
                    self.prg_rom[addr / PRG_ROM_SIZE][addr % PRG_ROM_SIZE] = byte;
                }
            },
            _ => unreachable!()
        }
    }
    
    fn mapped_ppu_read(&mut self, addr: usize) -> u8 { 
        match addr {
            0x0000..=0x1FFF => {
                self.chr_rom[addr]
            },
            _ => 0
        }
    }
    
    fn mapped_ppu_write(&mut self, addr: usize, byte: u8) {
        match addr {
            0x0000..=0x1FFF =>  {
                if self.chr_rom_banks == 0 {
                    self.chr_rom[addr] = byte;
                }
            }
            _ => {}
        }
    }
}

impl Mapper0 {
    pub fn new(data: &[u8], prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
        let mut header_size = 0x10;

        if data[6] & 0x04 != 0 {
            header_size += 0x200;
        }

        let prg_rom_size = prg_rom_banks as usize * PRG_ROM_SIZE;
        let chr_rom_size = chr_rom_banks as usize * CHR_ROM_SIZE;

        let mut prg_rom: Vec<[u8; PRG_ROM_SIZE]> = data[header_size..header_size + prg_rom_size]
            .chunks(PRG_ROM_SIZE)
            .map(|bank| {
                let mut res = [0; PRG_ROM_SIZE];
                res.copy_from_slice(&bank[..PRG_ROM_SIZE]);
                res
            })
            .collect();
        
        if prg_rom.len() == 1 {
            prg_rom.push(*prg_rom.get(0).clone().unwrap());
        }

        let mut chr_rom = [0; CHR_ROM_SIZE];
        if chr_rom_banks > 0 {
            chr_rom.copy_from_slice(&data[header_size + prg_rom_size..header_size + prg_rom_size + chr_rom_size]);
        }

        Self {
            ram: [0; 0x3FE0],
            prg_rom,
            chr_rom,
            prg_rom_banks,
            chr_rom_banks,
        }
    }
}