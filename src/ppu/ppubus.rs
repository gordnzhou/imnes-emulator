use crate::bus::DMA_REG_ADDR;
use crate::cartridge::{CartridgeNes, Mirroring};

use super::{PpuCtrl, PpuMask, PpuStatus};

const PATTERN_TABLE_START: usize = 0x0000;
const PATTERN_TABLE_END: usize = 0x1FFF;
pub const NAME_TABLE_START: usize = 0x2000;
const NAME_TABLE_END: usize = 0x3EFF;

pub const PALETTE_TABLE_START: usize = 0x3F00;

const PALETTE_TABLE_END: usize = 0x3FFF;

const PALETTE_TABLE_SIZE: usize = 0x20;
const NAME_TABLE_SIZE: usize = 0x400;

pub struct PpuBus {
    name_table: [[u8; NAME_TABLE_SIZE]; 2],
    palette_table: [u8; PALETTE_TABLE_SIZE],

    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    ppu_status: PpuStatus,
    oam_addr_reg: u8,
    oam_data_reg: u8,
    scroll_reg: u8,
    dma_addr_reg: u8,

    ppu_addr_latch: bool,
    ppu_data_buffer: u8,
    ppu_addr_buffer: u16,
}

impl PpuBus {
    pub fn new() -> Self {
        Self {
            name_table: [[0; NAME_TABLE_SIZE]; 2],
            palette_table: [0; PALETTE_TABLE_SIZE],

            ppu_ctrl: PpuCtrl::from_bits_truncate(0),
            ppu_mask: PpuMask::from_bits_truncate(0),
            ppu_status: PpuStatus::from_bits_truncate(0),
            oam_addr_reg: 0,
            oam_data_reg: 0,
            scroll_reg: 0,
            dma_addr_reg: 0,

            ppu_addr_latch: false,
            ppu_data_buffer: 0,
            ppu_addr_buffer: 0,
        }
    }

    // CPU can only access the PPU memory map through the PPU registers
    pub fn cpu_read(&mut self, addr: usize, cartridge: &mut CartridgeNes) -> u8 {
        if addr == DMA_REG_ADDR {
            // TODO: implement DMA transfer
            return self.dma_addr_reg;
        }

        match addr & 0x0007 {
            0x0000 => self.ppu_ctrl.bits(),
            0x0001 => self.ppu_mask.bits(),
            0x0002 => {
                self.ppu_status.insert(PpuStatus::IN_VBLANK);
                
                let ret = (self.ppu_status.bits() & 0b11100000) | (self.ppu_data_buffer & 0b00011111);

                self.ppu_status.remove(PpuStatus::IN_VBLANK);
                self.ppu_addr_latch = false;

                ret
            },
            0x0003 => self.oam_addr_reg,
            0x0004 => self.oam_data_reg,
            0x0005 => self.scroll_reg,
            0x0006 => 0,
            0x0007 => {
                let mut ret = self.ppu_data_buffer;

                self.ppu_data_buffer = self.ppu_read(self.ppu_addr_buffer as usize, cartridge);
                if self.ppu_addr_buffer as usize >= PALETTE_TABLE_START {
                    ret = self.ppu_data_buffer;
                }

                self.ppu_addr_buffer += 1;

                ret
            },
            _ => unreachable!()
        }
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8, cartridge: &mut CartridgeNes) {
        if addr == DMA_REG_ADDR {
            self.dma_addr_reg = byte;
            return;
        }

        match addr & 0x0007 {
            0x0000 => self.ppu_ctrl = PpuCtrl::from_bits_truncate(byte),
            0x0001 => self.ppu_mask = PpuMask::from_bits_truncate(byte),
            0x0002 => {},
            0x0003 => self.oam_addr_reg = byte,
            0x0004 => self.oam_data_reg = byte,
            0x0005 => self.scroll_reg = byte,
            0x0006 => {
                if !self.ppu_addr_latch {
                    self.ppu_addr_buffer = (self.ppu_addr_buffer & 0x00FF) | ((byte as u16) << 8);
                    self.ppu_addr_latch = true;
                } else {
                    self.ppu_addr_buffer = (self.ppu_addr_buffer & 0xFF00) | (byte as u16);
                    self.ppu_addr_latch = false;
                }
            }
            0x0007 => {
                self.ppu_write(self.ppu_addr_buffer as usize, byte, cartridge);
                self.ppu_addr_buffer += 1;
            }
            _ => unreachable!()
        }
    }

    pub fn ppu_read(&mut self, addr: usize, cartridge: &mut CartridgeNes) -> u8 {
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

                        self.name_table[(addr >> 11) & 0x01][addr % NAME_TABLE_SIZE] 
                    },
                    Mirroring::VERTICAL => {
                        // [        A        ]|[        B        ]
                        // [ 0x2000 - 0x3FFF ]|[ 0x4000 - 0x7FFF ]
                        // --------------------------------------
                        // [        a        ]|[        b        ]
                        // [ 0x8000 - 0xBFFF ]|[ 0xC000 - 0xFFFF ]

                        self.name_table[(addr >> 10) & 0x01][addr % NAME_TABLE_SIZE] 
                    },
                    _ => unimplemented!()
                }
            },
            PALETTE_TABLE_START..=PALETTE_TABLE_END => {
                addr &= 0x1F;

                if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
                    addr -= 0x10;
                }

                self.palette_table[addr]
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

                        self.name_table[(addr >> 11) & 0x01][addr % NAME_TABLE_SIZE] = byte;
                    },
                    Mirroring::VERTICAL => {
                        // [        A        ]|[        B        ]
                        // [ 0x2000 - 0x3FFF ]|[ 0x4000 - 0x7FFF ]
                        // --------------------------------------
                        // [        a        ]|[        b        ]
                        // [ 0x8000 - 0xBFFF ]|[ 0xC000 - 0xFFFF ]
                        
                        self.name_table[(addr >> 10) & 0x01][addr % NAME_TABLE_SIZE] = byte;
                    },
                    _ => unimplemented!()
                }
            },
            PALETTE_TABLE_START..=PALETTE_TABLE_END => {
                addr -= PALETTE_TABLE_START;
                addr %= PALETTE_TABLE_SIZE;

                if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
                    addr -= 0x10;
                }
                
                self.palette_table[addr] = byte;
            }
            _ => unreachable!()
        }
    }
}