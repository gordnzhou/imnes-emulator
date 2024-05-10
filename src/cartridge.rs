use std::{fs::read, io};

use crate::mapper::*;

// TODO: implement SRAM, EXP ROM mapping
const PRG_ROM_START: usize = 0x8000;
// const PRG_ROM_END: usize = 0xFFFF;

// The size of each PRG_ROM bank
const PRG_ROM_LENGTH: usize = 0x4000;

// The size of each CHR_ROM bank
const CHR_ROM_LENGTH: usize = 0x2000;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN,
}

#[allow(dead_code)]
pub struct CartridgeNes { 
    prg_rom: Vec<[u8; PRG_ROM_LENGTH]>,
    chr_rom: Vec<[u8; CHR_ROM_LENGTH]>,
    prg_rom_banks: u8,
    chr_rom_banks: u8,
    mirroring: Mirroring,
    mapper: Box<dyn Mapper>,
}

impl CartridgeNes {
    pub fn from_ines_file(file_path: &str) -> Result<Self, io::Error> {
        let data = read(file_path)?;

        CartridgeNes::from_ines_bytes(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn from_ines_bytes(data: &[u8]) -> Result<Self, &str> {
        // First three bytes must be "NES" in ASCII, followed by 0x1A
        if &data[0..=3] != &[0x4E, 0x45, 0x53, 0x1A] {
            return Err("Not a iNES file");
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

        let mapper_num = (data[7] & 0b11110000) | (data[6] >> 4);

        let mapper = match mapper_num {
            0 => Box::new(Mapper0::new()),
            _ => return Err("Unsupported iNES mapper")
        };


        // TODO: implement memory parsing for other mappers
        let prg_rom_offset = 0x10;
        let chr_rom_offset = PRG_ROM_LENGTH + prg_rom_offset;

        let prg_rom_bank = data[prg_rom_offset..prg_rom_offset+PRG_ROM_LENGTH]
            .try_into()
            .map_err(|_| "Could not parse PRG-ROM")?;

        let chr_rom_bank = data[chr_rom_offset..chr_rom_offset+CHR_ROM_LENGTH]
            .try_into()
            .map_err(|_| "Could not parse CHR-ROM")?;

        let prg_rom = vec![prg_rom_bank, prg_rom_bank];
        let chr_rom = vec![chr_rom_bank];

        Ok(CartridgeNes { 
            prg_rom,
            chr_rom,
            prg_rom_banks,
            chr_rom_banks,
            mirroring,
            mapper,
        })
    }

    // TODO: implement mapped writes for CPU and PPU
    pub fn cpu_read(&mut self, addr: usize) -> u8 {
        let mapped_addr = self.mapper.mapped_cpu_read(addr);
        self.prg_rom[0][(mapped_addr - PRG_ROM_START) % PRG_ROM_LENGTH]
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) {
        let _mapped_addr = self.mapper.mapped_cpu_write(addr, byte);
        // self.prg_rom[0][(mapped_addr - PRG_ROM_START) % PRG_ROM_LENGTH] = byte;
    }

    pub fn ppu_read(&mut self, addr: usize) -> u8 {
        let mapped_addr = self.mapper.mapped_ppu_read(addr);

        self.chr_rom[0][mapped_addr % CHR_ROM_LENGTH]
    }

    pub fn ppu_write(&mut self, addr: usize, byte: u8) {
        let _mapped_addr = self.mapper.mapped_ppu_write(addr, byte);
        // self.chr_rom[0][mapped_addr % CHR_ROM_LENGTH] = byte;
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
impl CartridgeNes {
    pub fn new() -> Self {
        CartridgeNes {
            prg_rom: vec![[0; PRG_ROM_LENGTH]],
            chr_rom: vec![[0; CHR_ROM_LENGTH]],
            prg_rom_banks: 1,
            chr_rom_banks: 1,
            mirroring: Mirroring::HORIZONTAL,
            mapper: Box::new(Mapper0::new())
        }
    }
}