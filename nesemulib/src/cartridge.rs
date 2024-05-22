use std::{fs::read, io};

use crate::{mapper::*, SystemControl};

// The size of each PRG_ROM bank
pub const PRG_ROM_SIZE: usize = 0x4000;

// The size of each CHR_ROM bank
pub const CHR_ROM_SIZE: usize = 0x2000;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    ONESCREEN_LO,
    ONESCREEN_HI,
    FOUR_SCREEN,
}

pub struct CartridgeNes {
    mirroring: Mirroring,
    mapper: Box<dyn Mapper>,
    no_chr_rom: bool,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl SystemControl for CartridgeNes {
    fn reset(&mut self) {
        self.mapper.reset()
    }
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

        let prg_rom_banks = data[4] as usize;

        let chr_rom_banks = data[5] as usize;

        if prg_rom_banks < 1 {
            return Err(String::from("File must contain at least one PRG-ROM bank")); 
        }

        let mut mirroring = if data[6] & 0x01 == 0 {
            Mirroring::HORIZONTAL
        } else {
            Mirroring::VERTICAL
        };

        if data[6] & 0b00001000 != 0 {
            mirroring = Mirroring::FOUR_SCREEN;
        }

        let battery_backed = data[6] & 0x02 != 0;

        let mapper_num = (data[7] & 0b11110000) | (data[6] >> 4);

        println!("Mapper:{} PRG-ROM banks:{} CHR-ROM banks:{} {:?} Trainer?:{} Battery?:{}", 
            mapper_num, prg_rom_banks, chr_rom_banks, mirroring, data[6] & 0x04, battery_backed);

        let mapper: Box<dyn Mapper> =  match mapper_num {
            0  => Box::new(Mapper0::new(prg_rom_banks)),
            1  => Box::new(Mapper1::new(prg_rom_banks)),
            2  => Box::new(Mapper2::new(prg_rom_banks)),
            3  => Box::new(Mapper3::new(prg_rom_banks)),
            4  => Box::new(Mapper4::new(prg_rom_banks)),
            7  => Box::new(Mapper7::new()),
            66 => Box::new(Mapper66::new()),
            _ => return Err(format!("Unsupported iNES mapper {}", mapper_num))
        };

        let mut offset = 0x10;

        if data[6] & 0x04 != 0 {
            offset += 0x200;
        }

        let prg_rom_size = prg_rom_banks * PRG_ROM_SIZE;
        let mut prg_rom = vec![0; prg_rom_size];
        prg_rom.copy_from_slice(&data[offset..offset + prg_rom_size]);
        offset += prg_rom_size;


        let chr_rom = if chr_rom_banks > 0 {
            let mut chr_rom = vec![0; chr_rom_banks * CHR_ROM_SIZE];
            chr_rom.copy_from_slice(&data[offset..offset + chr_rom_banks * CHR_ROM_SIZE]);
            chr_rom
        } else {
            vec![0; CHR_ROM_SIZE]
        };

        Ok(Self { 
            mirroring,
            prg_rom,
            chr_rom,
            no_chr_rom: chr_rom_banks == 0,
            mapper,
        })
    }

    pub fn cpu_read(&mut self, addr: usize) -> Option<u8> {
        self.mapper.mapped_cpu_read(&mut self.prg_rom, addr)
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) -> bool {
        self.mapper.mapped_cpu_write(&mut self.prg_rom, addr, byte)
    }

    pub fn ppu_read(&mut self, addr: usize) -> u8 {
        if self.no_chr_rom {
            return self.chr_rom[addr];
        }

        self.mapper.mapped_ppu_read(&mut self.chr_rom, addr)
    }

    pub fn ppu_write(&mut self, addr: usize, byte: u8) {
        if self.no_chr_rom {
            self.chr_rom[addr] = byte;
            return;
        }

        self.mapper.mapped_ppu_write(&mut self.chr_rom, addr, byte)
    }

    pub fn mirroring(&self) -> Mirroring {
        match self.mapper.get_updated_mirroring() {
            Some(mirroring) => mirroring,
            None => self.mirroring,
        }
    }

    pub fn notify_scanline(&mut self) {
        self.mapper.notify_scanline()
    }

    pub fn irq_active(&mut self) -> bool {
        self.mapper.irq_active()
    }
}

#[cfg(test)]
mod tests {
    use super::{CartridgeNes, Mirroring};
    use crate::mapper::TestMapper;

    impl CartridgeNes {
        pub fn test_new() -> Self {
            CartridgeNes {
                prg_rom: vec![0; 0x10000],
                chr_rom: vec![0; 0x2000],
                no_chr_rom: true,
                mirroring: Mirroring::HORIZONTAL,
                mapper: Box::new(TestMapper::new())
            }
        }
    }
}