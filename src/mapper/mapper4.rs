use crate::{cartridge::{Mirroring, CHR_ROM_SIZE, PRG_ROM_SIZE}, SystemControl};

use super::{Mapper, PRG_ROM_END, PRG_ROM_HI_END, PRG_ROM_HI_START, PRG_ROM_LO_END, PRG_ROM_LO_START, PRG_ROM_START};


const SAVE_RAM_START: usize = 0x6000;
const SAVE_RAM_END: usize = 0x7FFF;

const SAVE_RAM_SIZE: usize = 0x2000;

pub struct Mapper4 {
    save_ram: [u8; SAVE_RAM_SIZE],
    mirroring: Mirroring,
    prg_rom_banks: usize,

    prg_bank_offset: [usize; 4],
    chr_bank_offset: [usize; 8],

    prg_mode: bool,
    chr_inversion: bool,
    registers: [usize; 8],
    target_register: usize,

    irq_counter: u16,
    irq_reload: u16,
    irq_active: bool,
    irq_enable: bool,
    irq_update: bool,
}

impl SystemControl for Mapper4 {
    fn reset(&mut self) {
        self.mirroring = Mirroring::HORIZONTAL;

        self.prg_bank_offset[0] = 0;
        self.prg_bank_offset[1] = PRG_ROM_SIZE >> 1;
        self.prg_bank_offset[2] = ((self.prg_rom_banks << 1) - 2) * (PRG_ROM_SIZE >> 1);
        self.prg_bank_offset[3] = ((self.prg_rom_banks << 1) - 1) * (PRG_ROM_SIZE >> 1);
        self.chr_bank_offset = [0; 8];

        self.prg_mode = false;
        self.chr_inversion = false;
        self.registers = [0; 8];
        self.target_register = 0;

        self.irq_counter = 0;
        self.irq_reload = 0;
        self.irq_active = false;
        self.irq_enable = false;
        self.irq_update = false;
    }
}

impl Mapper for Mapper4 {
    fn mapped_cpu_read(&mut self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        match addr {
            SAVE_RAM_START..=SAVE_RAM_END => {
                Some(self.save_ram[addr & 0x1FFF])
            },
            PRG_ROM_START..=PRG_ROM_END => {
                let bank_index = (addr & 0x6000) >> 13;
                Some(prg_rom[self.prg_bank_offset[bank_index] + (addr & 0x1FFF)])
            },
            _ => None
        }
    }

    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        match addr {
            SAVE_RAM_START..=SAVE_RAM_END => {
                self.save_ram[addr & 0x1FFF] = byte;
                true
            },
            PRG_ROM_LO_START..=PRG_ROM_LO_END => {    
                if addr < (PRG_ROM_LO_START + (PRG_ROM_SIZE >> 1)) {

                    if addr & 0x01 == 0 {
                        self.target_register = (byte & 0b00000111) as usize;
                        self.prg_mode = byte & 0b01000000 != 0;
                        self.chr_inversion = byte & 0b10000000 != 0;
                        
                    } else {
                        let mut byte = byte;
                        if self.target_register == 6 || self.target_register == 7 {
                            byte &= 0b00111111;
                        }
                        self.registers[self.target_register] = byte as usize;


                        let prg_half_bank_size = PRG_ROM_SIZE >> 1;

                        if self.prg_mode {
                            self.prg_bank_offset[2] = self.registers[6] * prg_half_bank_size;
                            self.prg_bank_offset[0] = ((self.prg_rom_banks << 1) - 2) * prg_half_bank_size;
                        } else {
                            self.prg_bank_offset[0] = self.registers[6] * (prg_half_bank_size);
                            self.prg_bank_offset[2] = ((self.prg_rom_banks << 1) - 2) * prg_half_bank_size;
                        }
                        self.prg_bank_offset[1] = self.registers[7] * (prg_half_bank_size);
                        self.prg_bank_offset[3] = ((self.prg_rom_banks << 1) - 1) * prg_half_bank_size;


                        let chr_half_bank_size = CHR_ROM_SIZE >> 3;
                        
                        let r0lo = (self.registers[0] & 0xFE) * chr_half_bank_size;
                        let r0hi = (self.registers[0] + 1) * chr_half_bank_size;
                        let r1lo = (self.registers[1] & 0xFE) * chr_half_bank_size;
                        let r1hi = (self.registers[1] + 1) * chr_half_bank_size;
                        let r2 = self.registers[2] * chr_half_bank_size;
                        let r3 = self.registers[3] * chr_half_bank_size;
                        let r4 = self.registers[4] * chr_half_bank_size;
                        let r5 = self.registers[5] * chr_half_bank_size;

                        self.chr_bank_offset = [r0lo, r0hi, r1lo, r1hi, r2, r3, r4, r5];
                        if self.chr_inversion {
                            self.chr_bank_offset.swap(0, 4);
                            self.chr_bank_offset.swap(1, 5);
                            self.chr_bank_offset.swap(2, 6);
                            self.chr_bank_offset.swap(3, 7);
                        }
                    }
            
                } else {
                    if addr & 0x01 == 0 {
                        self.mirroring = if byte & 0x01 != 0 {
                            Mirroring::HORIZONTAL
                        } else {
                            Mirroring::VERTICAL
                        };
                    } else {
                        // TODO: add implementation for PRG-RAM protect register
                    }
                }

                true
            },
            PRG_ROM_HI_START..=PRG_ROM_HI_END => {
                if addr < (PRG_ROM_HI_START + (PRG_ROM_SIZE >> 1)) {
                    if addr & 0x01 == 0 {
                        self.irq_reload = byte as u16;
                    } else {
                        self.irq_counter = 0x0000;
                    }
                } else {
                    if addr & 0x01 == 0 {
                        self.irq_enable = false;
                        self.irq_active = false;
                    } else {
                        self.irq_enable = true;
                    }
                }

                true
            }
            _ => false
        }
    }

    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8 {
        let bank_index = (addr & 0x1C00) >> 10;
        chr_rom[self.chr_bank_offset[bank_index] + (addr & 0x03FF)]
    }

    fn notify_scanline(&mut self) {
        if self.irq_counter == 0 {
            self.irq_counter = self.irq_reload;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enable {
            self.irq_active = true;
        }
    }

    fn irq_active(&mut self) -> bool {
        let ret = self.irq_active;
        self.irq_active = false;
        ret
    }

    fn get_updated_mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }
}

impl Mapper4 {
    pub fn new(prg_rom_banks: usize) -> Self {
        Self {
            save_ram: [0; SAVE_RAM_SIZE],
            mirroring: Mirroring::HORIZONTAL,
            prg_rom_banks,

            target_register: 0,
            prg_mode: false,
            chr_inversion: false,

            prg_bank_offset: [0; 4],
            chr_bank_offset: [0; 8],
            registers: [0; 8],

            irq_counter: 0,
            irq_reload: 0,
            irq_active: false,
            irq_enable: false,
            irq_update: false,
        }
    }
}