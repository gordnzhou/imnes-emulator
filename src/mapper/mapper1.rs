use crate::cartridge::{Mirroring, CHR_ROM_SIZE, PRG_ROM_SIZE};

use super::{Mapper, CHR_ROM_HI_END, CHR_ROM_HI_START, CHR_ROM_LO_END, CHR_ROM_LO_START, PRG_ROM_HI_END, PRG_ROM_HI_START, PRG_ROM_LO_END, PRG_ROM_LO_START};

const SAVE_RAM_START: usize = 0x6000;
const SAVE_RAM_END: usize = 0x7FFF;

const SAVE_RAM_SIZE: usize = 0x2000;

pub struct Mapper1 {
    save_ram: [u8; SAVE_RAM_SIZE],
    mirroring: Mirroring,
    prg_rom_banks: usize,
    chr_rom_banks: usize,
    
    chr_bank_lo4: usize,
    chr_bank_hi4: usize,
    chr_bank_full8: usize,

    prg_bank_lo16: usize,
    prg_bank_hi16: usize,
    prg_bank_full32: usize,

    load_reg: u8,
    control_reg: u8,
    load_count: u8,
}

impl Mapper for Mapper1 {
    fn mapped_cpu_read(&mut self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            SAVE_RAM_START..=SAVE_RAM_END => {
                Some(self.save_ram[addr & 0x1FFF])
            },
            PRG_ROM_LO_START..=PRG_ROM_LO_END => {
                if self.control_reg & 0b01000 != 0 {
                    Some(prg_rom[self.prg_bank_lo16 * PRG_ROM_SIZE + (addr & 0x3FFF)])
                } else {
                    Some(prg_rom[self.prg_bank_full32 * (PRG_ROM_SIZE << 1) + (addr & 0x7FFF)])
                }
            },
            PRG_ROM_HI_START..=PRG_ROM_HI_END => {
                if self.control_reg & 0b01000 != 0 {
                    Some(prg_rom[self.prg_bank_hi16 * PRG_ROM_SIZE + (addr & 0x3FFF)])
                } else {
                    Some(prg_rom[self.prg_bank_full32 * (PRG_ROM_SIZE << 1) + (addr & 0x7FFF)])
                }
            },
            _ => None
        }
    }

    // TODO: 
    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        match addr {
            SAVE_RAM_START..=SAVE_RAM_END => {
                self.save_ram[addr & 0x1FFF] = byte;
                true
            }
            PRG_ROM_LO_START..=PRG_ROM_HI_END => {
                if byte & 0b10000000 != 0 {
                    self.load_reg = 0x00;
                    self.load_count = 0;
                    self.control_reg |= 0b00001100;
                } else {
                    self.load_reg >>= 1;
                    self.load_reg |= (byte & 0x01) << 4;
                    self.load_count += 1;

                    if self.load_count < 5 {
                        return true;
                    }

                    match (addr >> 13) & 0b00000011 {
                        0 => {
                            self.control_reg = self.load_reg & 0b00011111;

                            self.mirroring = match self.control_reg & 0b00000011 {
                                0 => Mirroring::ONESCREEN_LO,
                                1 => Mirroring::ONESCREEN_HI,
                                2 => Mirroring::VERTICAL,
                                3 => Mirroring::HORIZONTAL,
                                _ => unreachable!()
                            }
                        },
                        1 => {
                            if self.control_reg & 0b10000 != 0 {
                                self.chr_bank_lo4 = (self.load_reg & 0b00011111) as usize;
                            } else {
                                self.chr_bank_full8 = (self.load_reg & 0b00011110) as usize;
                            }
                        },
                        2 => {
                            if self.control_reg & 0b10000 != 0 {
                                self.chr_bank_hi4 = (self.load_reg & 0b00011111) as usize;
                            }
                        },
                        3 => {
                            match (self.control_reg >> 2) & 0b00000011 {
                                0 | 1 => {
                                    self.prg_bank_full32 = ((self.load_reg & 0b00001110) >> 1) as usize
                                },
                                2 => {
                                    self.prg_bank_lo16 = 0;
                                    self.prg_bank_hi16 = (self.load_reg & 0b00001111) as usize;
                                },
                                3 => {
                                    self.prg_bank_lo16 = (self.load_reg & 0b00001111) as usize;
                                    self.prg_bank_hi16 = self.prg_rom_banks - 1;
                                },
                                _ => unreachable!()
                            }
                        },
                        _ => unreachable!()
                    }
                }

                self.load_reg = 0x00;
                self.load_count = 0;

                true
            }
            _ => false
        }
    }

    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8 {
        if self.chr_rom_banks == 0 {
            return chr_rom[addr];
        }

        match addr {
            CHR_ROM_LO_START..=CHR_ROM_LO_END => {
                if self.control_reg & 0b10000 != 0 {
                    chr_rom[self.chr_bank_lo4 * (CHR_ROM_SIZE >> 1) + (addr & 0x0FFF)]
                } else {
                    chr_rom[self.chr_bank_full8 * CHR_ROM_SIZE + (addr & 0x1FFF)]
                }
            },
            CHR_ROM_HI_START..=CHR_ROM_HI_END => {
                if self.control_reg & 0b10000 != 0 {
                    chr_rom[self.chr_bank_hi4 * (CHR_ROM_SIZE >> 1) + (addr & 0x0FFF)]
                } else {
                    chr_rom[self.chr_bank_full8 * CHR_ROM_SIZE + (addr & 0x1FFF)]
                }
            },
            _ => unreachable!("Tried to address: {:04X} in CHR-ROM, should be in 0x0000-0x1FFF", addr)
        }
    }

    fn mapped_ppu_write(&mut self, chr_rom: &mut Vec<u8>, addr: usize, byte: u8) {
        if self.chr_rom_banks == 0 {
            chr_rom[addr] = byte;
        }
    }

    fn get_updated_mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }

    fn reset(&mut self) {
        self.chr_bank_lo4 = 0;
        self.chr_bank_hi4 = 0;
        self.chr_bank_full8 = 0;

        self.prg_bank_lo16 = 0;
        self.prg_bank_hi16 = self.prg_rom_banks - 1;
        self.prg_bank_full32 = 0;

        self.load_reg = 0x00;
        self.control_reg = 0x1C;
        self.load_count = 0;
    }
}

impl Mapper1 {
    pub fn new(prg_rom_banks: usize, chr_rom_banks: usize) -> Self {
        Self {
            save_ram: [0; SAVE_RAM_SIZE],
            mirroring: Mirroring::HORIZONTAL,
            prg_rom_banks,
            chr_rom_banks,

            chr_bank_lo4: 0,
            chr_bank_hi4: 0,
            chr_bank_full8: 0,

            prg_bank_lo16: 0,
            prg_bank_hi16: prg_rom_banks - 1,
            prg_bank_full32: 0,

            load_reg: 0x00,
            control_reg: 0x1C,
            load_count: 0,
        }
    }
}