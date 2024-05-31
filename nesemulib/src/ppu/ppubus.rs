use crate::{cartridge::{CartridgeNes, Mirroring}, SystemControl};

use super::registers::{LoopyPpuReg, PpuCtrl, PpuMask, PpuStatus};

const PATTERN_TABLE_START: usize = 0x0000;
const PATTERN_TABLE_END: usize = 0x1FFF;

pub const NAME_TABLE_START: usize = 0x2000;
const NAME_TABLE_END: usize = 0x3EFF;
pub const ATTR_TABLE_START: usize = 0x23C0;

pub const PALETTE_TABLE_START: usize = 0x3F00;
const PALETTE_TABLE_END: usize = 0x3FFF;

const PALETTE_TABLE_SIZE: usize = 0x20;
const NAME_TABLE_SIZE: usize = 0x400;
pub const OAM_SIZE: usize = 0x100;

#[derive(Clone, Copy)]
pub struct OAMEntry {
    pub y: usize,
    pub id: usize,
    attributes: usize,
    pub x: usize,
}

impl Default for OAMEntry {
    fn default() -> Self {
        Self { 
            y: 0xFF, 
            id: 0xFF,
            attributes: 0xFF,
            x: 0xFF
        }
    }
}

impl OAMEntry {
    pub fn y_flipped(&self) -> bool {
        self.attributes & 0x80 != 0
    }

    pub fn x_flipped(&self) -> bool {
        self.attributes & 0x40 != 0
    }

    pub fn priority(&self) -> bool {
        self.attributes & 0x20 == 0
    }

    pub fn palette(&self) -> usize {
        self.attributes & 0x03
    }
}

pub struct PpuBus {
    name_table: [[u8; NAME_TABLE_SIZE]; 2],
    palette_table: [u8; PALETTE_TABLE_SIZE],
    oam: [u8; OAM_SIZE],

    pub ctrl: PpuCtrl,
    pub mask: PpuMask,
    pub status: PpuStatus,
    pub oam_addr_reg: u8,

    // Loopy Registers
    pub vram_addr: LoopyPpuReg,
    pub tram_addr: LoopyPpuReg,
    pub fine_x: u8,

    ppu_addr_latch: bool,
    ppu_data_buffer: u8,
}

impl SystemControl for PpuBus {
    fn reset(&mut self) {
        self.ctrl = PpuCtrl::from_bits_truncate(0);
        self.mask = PpuMask::from_bits_truncate(0);
        self.status = PpuStatus::from_bits_truncate(0);
        self.oam_addr_reg = 0;

        self.vram_addr = LoopyPpuReg::default();
        self.tram_addr = LoopyPpuReg::default();
        self.fine_x = 0;

        self.ppu_addr_latch = false;
        self.ppu_data_buffer = 0;
    }
}

impl PpuBus {
    pub fn new() -> Self {
        Self {
            name_table: [[0; NAME_TABLE_SIZE]; 2],
            palette_table: [0; PALETTE_TABLE_SIZE],
            oam: [0; OAM_SIZE],

            ctrl: PpuCtrl::from_bits_truncate(0),
            mask: PpuMask::from_bits_truncate(0),
            status: PpuStatus::from_bits_truncate(0),
            oam_addr_reg: 0,

            vram_addr: LoopyPpuReg::default(),
            tram_addr: LoopyPpuReg::default(),
            fine_x: 0,

            ppu_addr_latch: false,
            ppu_data_buffer: 0
        }
    }

    pub fn read_oam(&self, addr: usize) -> u8 {
        self.oam[addr]
    }

    pub fn read_oam_entry(&self, oam_pos: usize) -> OAMEntry {
        OAMEntry {
            y:          self.oam[oam_pos + 0] as usize,
            id:         self.oam[oam_pos + 1] as usize,
            attributes: self.oam[oam_pos + 2] as usize,
            x:          self.oam[oam_pos + 3] as usize,
        }
    }

    pub fn transfer_to_oam(&mut self, addr: usize, byte: u8) {
        self.oam[((self.oam_addr_reg as usize) + addr) & 0xFF] = byte;
    }

    // CPU can only access the PPU memory map through the PPU registers
    pub fn cpu_read_reg(&mut self, addr: usize, cartridge: &mut CartridgeNes, read_only: bool) -> u8 {
        match addr & 0x0007 {
            0x0000 => 0,
            0x0001 => 0,
            0x0002 => {
                let ret = (self.status.bits() & 0b11100000) | (self.ppu_data_buffer & 0b00011111);

                if !read_only {
                    self.status.remove(PpuStatus::IN_VBLANK);
                    self.ppu_addr_latch = false;
                }

                ret
            },
            0x0003 => 0,
            0x0004 => {
                self.oam[self.oam_addr_reg as usize]
            },
            0x0005 => 0,
            0x0006 => 0,
            0x0007 => {
                let mut ret = self.ppu_data_buffer;

                if !read_only {

                    if (self.vram_addr.0 as usize & 0x3FFF) >= PALETTE_TABLE_START {
                        self.ppu_data_buffer = self.ppu_read((self.vram_addr.0 - 0x1000) as usize, cartridge);
                        ret = self.ppu_data_buffer;
                    } else {
                        self.ppu_data_buffer = self.ppu_read(self.vram_addr.0 as usize, cartridge);
                    }
    
                    self.vram_addr.0 += self.ctrl.vram_addr_inc();
                } else {

                    if (self.vram_addr.0 as usize & 0x3FFF) >= PALETTE_TABLE_START {
                        ret = self.ppu_data_buffer;
                    }
                }

                ret
            },
            _ => unreachable!()
        }
    }

    pub fn cpu_write_reg(&mut self, addr: usize, byte: u8, cartridge: &mut CartridgeNes) {
        match addr & 0x0007 {
            0x0000 => { 
                self.ctrl = PpuCtrl::from_bits_truncate(byte);

                self.tram_addr.set_mask(LoopyPpuReg::NAME_TABLE_X, 
                    self.ctrl.name_table_x() as u16);
                self.tram_addr.set_mask(LoopyPpuReg::NAME_TABLE_Y,
                    self.ctrl.name_table_y() as u16);
            },
            0x0001 => {
                self.mask = PpuMask::from_bits_truncate(byte);
            },
            0x0002 => {},
            0x0003 => {
                self.oam_addr_reg = byte;
            },
            0x0004 => {
                self.oam[self.oam_addr_reg as usize] = byte;
                self.oam_addr_reg = self.oam_addr_reg.wrapping_add(1);
            },
            0x0005 => {
                if !self.ppu_addr_latch {
                    self.fine_x = byte & 0x07;
                    self.tram_addr.set_mask(LoopyPpuReg::COARSE_X, (byte as u16) >> 3);
                } else {
                    self.tram_addr.set_mask(LoopyPpuReg::FINE_Y,(byte as u16) & 0x07);
                    self.tram_addr.set_mask(LoopyPpuReg::COARSE_Y, (byte as u16) >> 3);
                }

                self.ppu_addr_latch = !self.ppu_addr_latch;
            }
            0x0006 => {
                if !self.ppu_addr_latch {
                    self.tram_addr.0 = (((byte & 0x003F) as u16) << 8) | (self.tram_addr.0 & 0x00FF);
                } else {
                    self.tram_addr.0 = (self.tram_addr.0 & 0x7F00) | (byte as u16);
                    self.vram_addr.0 = self.tram_addr.0;
                }

                self.ppu_addr_latch = !self.ppu_addr_latch;
            }
            0x0007 => {
                self.ppu_write(self.vram_addr.0 as usize, byte, cartridge);
                self.vram_addr.0 += self.ctrl.vram_addr_inc();
            }
            _ => unreachable!()
        }
    }

    pub fn ppu_read(&self, addr: usize, cartridge: &CartridgeNes) -> u8 {
        let mut addr = addr & 0x3FFF;
        
        match addr {
            PATTERN_TABLE_START..=PATTERN_TABLE_END => cartridge.ppu_read(addr),
            NAME_TABLE_START..=NAME_TABLE_END => {
                match cartridge.mirroring() {
                    Mirroring::HORIZONTAL => {
                        // [        A        ]|[        a        ]
                        // [ 0x2000 - 0x3FFF ]|[ 0x4000 - 0x7FFF ]
                        // --------------------------------------
                        // [        B        ]|[        b        ]
                        // [ 0x8000 - 0xBFFF ]|[ 0xC000 - 0xFFFF ]

                        self.name_table[(addr >> 11) & 0x01][addr & 0x3FF] 
                    },
                    Mirroring::VERTICAL => {
                        // [        A        ]|[        B        ]
                        // [ 0x2000 - 0x3FFF ]|[ 0x4000 - 0x7FFF ]
                        // --------------------------------------
                        // [        a        ]|[        b        ]
                        // [ 0x8000 - 0xBFFF ]|[ 0xC000 - 0xFFFF ]

                        self.name_table[(addr >> 10) & 0x01][addr & 0x3FF] 
                    },
                    Mirroring::ONESCREEN_LO => {
                        self.name_table[0][addr & 0x3FF] 
                    }
                    Mirroring::ONESCREEN_HI => {
                        self.name_table[1][addr & 0x3FF] 
                    }
                    _ => unimplemented!()
                }
            },
            PALETTE_TABLE_START..=PALETTE_TABLE_END => {
                addr &= 0x001F;

                if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
                    addr -= 0x10;
                }

                self.palette_table[addr] & if self.mask.greyscale_on() { 0x30 } else { 0x3F }
            }
            _ => unreachable!()
        }
    }

    pub fn ppu_write(&mut self, addr: usize, byte: u8, cartridge: &mut CartridgeNes) {
        let mut addr = addr as usize & 0x3FFF;

        match addr {
            PATTERN_TABLE_START..=PATTERN_TABLE_END => cartridge.ppu_write(addr, byte),
            NAME_TABLE_START..=NAME_TABLE_END => {
                match cartridge.mirroring() {
                    Mirroring::HORIZONTAL => {
                        // [        A        ]|[        a        ]
                        // [ 0x2000 - 0x3FFF ]|[ 0x4000 - 0x7FFF ]
                        // --------------------------------------
                        // [        B        ]|[        b        ]
                        // [ 0x8000 - 0xBFFF ]|[ 0xC000 - 0xFFFF ]

                        self.name_table[(addr >> 11) & 0x01][addr & 0x3FF] = byte;
                    },
                    Mirroring::VERTICAL => {
                        // [        A        ]|[        B        ]
                        // [ 0x2000 - 0x3FFF ]|[ 0x4000 - 0x7FFF ]
                        // --------------------------------------
                        // [        a        ]|[        b        ]
                        // [ 0x8000 - 0xBFFF ]|[ 0xC000 - 0xFFFF ]
                        
                        self.name_table[(addr >> 10) & 0x01][addr & 0x3FF] = byte;
                    },
                    Mirroring::ONESCREEN_LO => {
                        self.name_table[0][addr & 0x3FF] = byte;
                    }
                    Mirroring::ONESCREEN_HI => {
                        self.name_table[1][addr & 0x3FF] = byte;
                    }
                    _ => unimplemented!()
                }
            },
            PALETTE_TABLE_START..=PALETTE_TABLE_END => {
                addr &= 0x001F;

                if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
                    addr -= 0x10;
                }
                
                self.palette_table[addr] = byte;
            }
            _ => {}
        }
    }
}