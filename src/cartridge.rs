use std::{fs::read, io};

use crate::mapper::*;

// The size of each PRG_ROM bank
pub const PRG_ROM_SIZE: usize = 0x4000;

// The size of each CHR_ROM bank
pub const CHR_ROM_SIZE: usize = 0x2000;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN,
}

#[allow(dead_code)]
pub struct CartridgeNes {
    mirroring: Mirroring,
    mapper: Box<dyn Mapper>,
}

impl CartridgeNes {
    pub fn from_ines_file(file_path: &str) -> Result<Self, io::Error> {
        let data = read(file_path)?;

        CartridgeNes::from_ines_bytes(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn from_ines_bytes(data: &[u8]) -> Result<Self, String> {
        // First three bytes must be "NES" in ASCII, followed by 0x1A
        if &data[0..=3] != &[0x4E, 0x45, 0x53, 0x1A] {
            return Err(String::from("Not a iNES file"));
        }

        let prg_rom_banks = data[4];

        let chr_rom_banks = data[5];

        let mut mirroring = if data[6] & 0x01 == 0 {
            Mirroring::HORIZONTAL
        } else {
            Mirroring::VERTICAL
        };

        if data[6] & 0b00001000 != 0 {
            mirroring = Mirroring::FOUR_SCREEN;
        }

        let battery_backed = data[6] & 0x02 != 0;

        println!("PRG-ROM banks:{} CHR-ROM banks:{} {:?} Trainer?:{} Battery?:{}", 
            prg_rom_banks, chr_rom_banks, mirroring, data[6] & 0x04, battery_backed);

        let mapper_num = (data[7] & 0b11110000) | (data[6] >> 4);

        let mapper = match mapper_num {
            0 => Box::new(Mapper0::new(data, prg_rom_banks, chr_rom_banks)),
            _ => return Err(format!("Unsupported iNES mapper {}", mapper_num))
        };

        Ok(Self { 
            mirroring,
            mapper,
        })
    }

    pub fn cpu_read(&mut self, addr: usize) -> u8 {
        self.mapper.mapped_cpu_read(addr)
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) {
        self.mapper.mapped_cpu_write(addr, byte);
    }

    // TODO: can mappers access PPU reads/writes addressed outside of 0x0000 and 0x2000???
    pub fn ppu_read(&mut self, addr: usize) -> u8 {
        self.mapper.mapped_ppu_read(addr)
    }

    pub fn ppu_write(&mut self, addr: usize, byte: u8) {
        self.mapper.mapped_ppu_write(addr, byte);
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
impl CartridgeNes {
    pub fn new() -> Self {
        CartridgeNes {
            mirroring: Mirroring::HORIZONTAL,
            mapper: Box::new(Mapper0::new(&[0; 0x010 + PRG_ROM_SIZE + CHR_ROM_SIZE], 1, 1))
        }
    }
}