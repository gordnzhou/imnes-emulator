mod ppubus;

pub use ppubus::PpuBus;

use crate::{bus::Bus, palette::{Colour, DISPLAY_PALETTE}};

use self::ppubus::{NAME_TABLE_START, PALETTE_TABLE_START};

const TILE_SIZE: usize = 0x0010;

bitflags! {
    pub struct PpuCtrl: u8 {
        const NAME_TABLE_ADDR = 0b00000011;
        const VRAM_ADDR_INC   = 0b00000100;
        const SPR_TABLE_ADDR  = 0b00001000;
        const BG_TABLE_ADDR   = 0b00010000;
        const SPR_SIZE        = 0b00100000;
        const MASTER_SELECT   = 0b01000000;
        const GEN_NMI         = 0b10000000;
    }

    pub struct PpuMask: u8 {
        const GREY_SCALE_ON = 0b00000001;
        const SHOW_BG_LEFT  = 0b00000010;
        const SHOW_SPR_LEFT = 0b00000100;
        const SHOW_BG       = 0b00001000;
        const SHOW_SPR      = 0b00010000;
        const EMP_RED       = 0b00100000;
        const EMP_GREEN     = 0b01000000;
        const EMP_BLUE      = 0b10000000;
    }

    pub struct PpuStatus: u8 {
        const SPR_OVERFLOW = 0b00100000;
        const SPR_0_HIT    = 0b01000000;
        const IN_VBLANK    = 0b10000000;
    }
}

pub struct Ppu2C03 {

    cycles: u32,
    scanline: i32,
}

impl Ppu2C03 {
    pub fn new() -> Self {
        Ppu2C03 { 
            cycles: 0,
            scanline: 0,
        }
    }

    pub fn clock(&mut self, _bus: &mut Bus) {

        self.cycles += 1;
        if self.cycles >= 341 {
            self.cycles = 0;

            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;

                // draw a pixel here
            }
        }
    }

    pub fn nmi_requested(&self) -> bool {
        false
    }

    pub fn get_pattern_table(&self, index: usize, bus: &mut Bus, palette: u16) -> [Colour; 128 * 128] {
        let mut result = [DISPLAY_PALETTE[0x01]; 128 * 128];

        for tile_y in 0..16 {

            for tile_x in 0..16 {
                let tile_offset = (tile_y * 256) + tile_x * TILE_SIZE;

                for row in 0..8 {
                    let tile_lo = bus.ppu_read(index * 0x1000 + tile_offset + row + 0x0000);
                    let tile_hi = bus.ppu_read(index * 0x1000 + tile_offset + row + 0x0008);

                    for col in 0..8 {
                        let pixel = (((tile_hi & (1 << col)) >> col) << 1)
                            | ((tile_lo & (1 << col)) >> col);

                        let colour = bus.ppu_read(
                            PALETTE_TABLE_START + ((palette as usize) << 2) + pixel as usize
                        ) as usize;

                        result[((tile_y * 8 + row) << 7) | tile_x * 8 + (7 - col)] = DISPLAY_PALETTE[colour];
                    }
                }
            }
        }

        result
    }

    pub fn get_name_table(&self, bus: &mut Bus) -> [Colour; 256 * 240] {
        let mut result = [DISPLAY_PALETTE[0x01]; 256 * 240];

        for y in 0..30 {
            for x in 0..32 {
                let id = bus.ppu_read(NAME_TABLE_START + y * 32 + x);
                for row in 0..8 {
                    for col in 0..8 {
                        result[((y * 8 + row) << 8) | (x * 8 + col)] = DISPLAY_PALETTE[(id as usize) & 0x3F];
                    }
                }
            }
        }

        result
    }
}