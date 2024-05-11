mod ppubus;
mod registers;

pub use ppubus::PpuBus;

use crate::{bus::Bus, palette::{Colour, DISPLAY_PALETTE}};

use self::{ppubus::{ATTR_TABLE_START, NAME_TABLE_START, PALETTE_TABLE_START}, registers::PpuStatus};

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

pub struct Ppu2C03 {
    pub frame_buffer: [Colour; SCREEN_WIDTH * SCREEN_HEIGHT],
    cycles: u32,
    scanline: i32,

    bg_next_tile_id: u8,
    bg_next_tile_attr: u8,
    bg_next_tile_lo: u8,
    bg_next_tile_hi: u8,
    bg_patt_lo_shifter: u16,
    bg_patt_hi_shifter: u16,
    bg_attr_lo_shifter: u16,
    bg_attr_hi_shifter: u16,

    nmi: bool,
    frame_complete: bool,
}

impl Ppu2C03 {
    pub fn new() -> Self {
        Ppu2C03 { 
            frame_buffer: [DISPLAY_PALETTE[0]; SCREEN_WIDTH * SCREEN_HEIGHT],
            cycles: 0,
            scanline: 240,

            bg_next_tile_id: 0,
            bg_next_tile_attr: 0,
            bg_next_tile_lo: 0,
            bg_next_tile_hi: 0,
            bg_patt_lo_shifter: 0,
            bg_patt_hi_shifter: 0,
            bg_attr_lo_shifter: 0,
            bg_attr_hi_shifter: 0,

            nmi: false,
            frame_complete: false,
        }
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        match self.scanline {
            -1..=239 => {
                let ppu_bus = &mut bus.ppu_bus;

                if self.scanline == 0 && self.cycles == 0 {
                    self.cycles = 1;
                }

                if self.scanline == -1 && self.cycles == 1 {
                    ppu_bus.status.set(PpuStatus::IN_VBLANK, false);
                }

                // Visible Cycle
                if matches!(self.cycles, 2..=257 | 321..=337) {
                    self.update_shifters(ppu_bus);

                    match (self.cycles - 1) % 8 {
                        0 => {
                            self.load_bg_shifters();
                            let bg_next_tile_id_addr = NAME_TABLE_START | ((ppu_bus.vram_addr.0 as usize) & 0x0FFF);
                            self.bg_next_tile_id = bus.ppu_read(bg_next_tile_id_addr);
                        }
                        2 => {
                            let coarse_x = ppu_bus.vram_addr.coarse_x() as usize;
                            let coarse_y = ppu_bus.vram_addr.coarse_y() as usize;

                            let bg_next_tile_attr_addr = ATTR_TABLE_START 
                                | ((ppu_bus.vram_addr.name_table_y() as usize) << 11)
                                | ((ppu_bus.vram_addr.name_table_x() as usize) << 10)
                                | ((coarse_y << 2) << 3)
                                | (coarse_x >> 2);

                            self.bg_next_tile_attr = bus.ppu_read(bg_next_tile_attr_addr);

                            if coarse_y & 0x02 != 0 { self.bg_next_tile_attr >>= 4; }
                            if coarse_x & 0x02 != 0 { self.bg_next_tile_attr >>= 2; }
                            self.bg_next_tile_attr &= 0x03;
                        },
                        4 => {
                            let bg_next_tile_lo_addr = ppu_bus.ctrl.bg_table_addr()
                                + ((self.bg_next_tile_id as usize) << 4) 
                                + ppu_bus.vram_addr.fine_y() as usize;

                            self.bg_next_tile_lo = bus.ppu_read(bg_next_tile_lo_addr);
                        },
                        6 => {
                            let bg_next_tile_hi_addr = ppu_bus.ctrl.bg_table_addr()
                                + ((self.bg_next_tile_id as usize) << 4) 
                                + ppu_bus.vram_addr.fine_y() as usize
                                + 8;

                            self.bg_next_tile_hi = bus.ppu_read(bg_next_tile_hi_addr);
                        },
                        7 => {
                            Ppu2C03::increment_scroll_x(ppu_bus);
                        },
                        _ => {},
                    }
                }
                 
                if self.cycles == 256 {
                    Ppu2C03::increment_scroll_y(&mut bus.ppu_bus)
                }

                if self.cycles == 257 {
                    Ppu2C03::reset_vram_x(&mut bus.ppu_bus);
                }

                if self.scanline == -1 && matches!(self.cycles, 280..=304) {
                    Ppu2C03::reset_vram_y(&mut bus.ppu_bus);
                }
            }
            241..=260 => {
                if self.scanline == 241 && self.cycles == 1 {
                    bus.ppu_bus.status.set(PpuStatus::IN_VBLANK, true);

                    if bus.ppu_bus.ctrl.nmi_enabled() {
                        self.nmi = true;
                    }
                }
            }
            _ => {
            }
        }

        // Render pixel
        if matches!(self.scanline, 0..=239) && matches!(self.cycles, 0..=255) {
            let ppu_bus = &mut bus.ppu_bus;

            let mut bg_pixel = 0;
            let mut bg_palette = 0;

            if ppu_bus.mask.show_bg() {
                let mask = 0x8000 >> ppu_bus.fine_x;

                let bg_pixel_bot = ((self.bg_patt_lo_shifter & mask) != 0) as usize;
                let bg_pixel_top = ((self.bg_patt_hi_shifter & mask) != 0) as usize;
                bg_pixel = (bg_pixel_top << 1) | bg_pixel_bot;

                let bg_palette_bot = ((self.bg_attr_lo_shifter & mask) != 0) as usize;
                let bg_palette_top = ((self.bg_attr_hi_shifter & mask) != 0) as usize;
                bg_palette = (bg_palette_top << 1) | bg_palette_bot
            }

            let bg_colour = bus.ppu_read(
                PALETTE_TABLE_START + ((bg_palette as usize) << 2) + bg_pixel as usize
            ) as usize;

            self.frame_buffer[(self.scanline as usize) * SCREEN_WIDTH + (self.cycles as usize) - 1] = DISPLAY_PALETTE[bg_colour];
        }

        self.cycles += 1;
        if self.cycles >= 341 {
            self.cycles = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }

    pub fn frame_complete(&mut self) -> bool {
        let ret = self.frame_complete;
        self.frame_complete = false;
        ret
    }

    pub fn increment_scroll_x(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {

            if ppu_bus.vram_addr.coarse_x() == 31 {
                ppu_bus.vram_addr.set_coarse_x(0);
                ppu_bus.vram_addr.set_name_table_x(!ppu_bus.vram_addr.name_table_x());
            } else {
                ppu_bus.vram_addr.set_coarse_x(ppu_bus.vram_addr.coarse_x() + 1);
            }
        }
    }

    pub fn increment_scroll_y(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {

            if ppu_bus.vram_addr.fine_y() == 7 {
                ppu_bus.vram_addr.set_fine_y(0);

                if ppu_bus.vram_addr.coarse_y() == 29 {
                    ppu_bus.vram_addr.set_coarse_y(0);
                    ppu_bus.vram_addr.set_name_table_y(!ppu_bus.vram_addr.name_table_y());

                } else if ppu_bus.vram_addr.coarse_y() == 31 {
                    ppu_bus.vram_addr.set_coarse_y(0);

                } else {
                    ppu_bus.vram_addr.set_coarse_y(ppu_bus.vram_addr.coarse_y() + 1)

                }
            } else {
                ppu_bus.vram_addr.set_fine_y(ppu_bus.vram_addr.fine_y() + 1)
            }
        }
    }

    pub fn load_bg_shifters(&mut self) {
        self.bg_patt_lo_shifter = (self.bg_patt_lo_shifter & 0xFF00) | self.bg_next_tile_lo as u16;
        self.bg_patt_hi_shifter = (self.bg_patt_hi_shifter & 0xFF00) | self.bg_next_tile_hi as u16;

        self.bg_attr_lo_shifter = (self.bg_attr_lo_shifter & 0xFF00) 
            | if self.bg_next_tile_attr & 0b01 != 0 { 0xFF } else { 0 };
        self.bg_attr_hi_shifter = (self.bg_attr_hi_shifter & 0xFF00) 
            | if self.bg_next_tile_attr & 0b10 != 0 { 0xFF } else { 0 };
    }

    pub fn update_shifters(&mut self, ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() {
            self.bg_attr_hi_shifter <<= 1;
            self.bg_attr_lo_shifter <<= 1;
            self.bg_patt_hi_shifter <<= 1;
            self.bg_patt_lo_shifter <<=  1;
        }
    }

    pub fn reset_vram_x(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {
            ppu_bus.vram_addr.set_name_table_x(ppu_bus.tram_addr.name_table_x());
            ppu_bus.vram_addr.set_coarse_x(ppu_bus.tram_addr.coarse_x())
        }
    }

    pub fn reset_vram_y(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {
            ppu_bus.vram_addr.set_name_table_y(ppu_bus.tram_addr.name_table_y());
            ppu_bus.vram_addr.set_coarse_y(ppu_bus.tram_addr.coarse_y());
            ppu_bus.vram_addr.set_fine_y(ppu_bus.tram_addr.fine_y());
        }
    }

    pub fn nmi_requested(&mut self) -> bool {
        let ret = self.nmi;
        self.nmi = false;
        ret
    }

    // pub fn get_pattern_table(&self, index: usize, bus: &mut Bus, palette: u16) -> [Colour; 128 * 128] {
    //     let mut result = [DISPLAY_PALETTE[0x01]; 128 * 128];

    //     for tile_y in 0..16 {

    //         for tile_x in 0..16 {
    //             let tile = self.get_tile_pattern(bus, tile_x, tile_y, index, palette);

    //             for row in 0..8 {
    //                 for col in 0..8 {
    //                     result[((tile_y * 8 + row) << 7) | tile_x * 8 + (7 - col)] = tile[row * 8 | (7 - col)]
    //                 }  
    //             }
    //         }
    //     }

    //     result
    // }

    // fn get_tile_pattern(&self, bus: &mut Bus, tile_x: usize, tile_y: usize, index: usize, palette: u16) -> [Colour; 64] {
    //     let mut result = [DISPLAY_PALETTE[0]; 64];
    //     let tile_offset = (tile_y * 256) + tile_x * TILE_SIZE;

    //     for row in 0..8 {
    //         let tile_lo = bus.ppu_read(index * 0x1000 + tile_offset + row + 0x0000);
    //         let tile_hi = bus.ppu_read(index * 0x1000 + tile_offset + row + 0x0008);

    //         for col in 0..8 {
    //             let pixel = (((tile_hi & (1 << col)) >> col) << 1)
    //                 | ((tile_lo & (1 << col)) >> col);

    //             let colour = bus.ppu_read(
    //                 PALETTE_TABLE_START + ((palette as usize) << 2) + pixel as usize
    //             ) as usize;

    //             result[row * 8 | (7 - col)] = DISPLAY_PALETTE[colour];
    //         }
    //     }

    //     result
    // }

    // pub fn get_name_table(&self, bus: &mut Bus) -> [Colour; 256 * 240] {
    //     let mut result = [DISPLAY_PALETTE[0]; 256 * 240];

    //     for y in 0..30 {
    //         for x in 0..32 {
    //             let id = bus.ppu_read(NAME_TABLE_START + y * 32 + x) as usize;
    //             let tile = self.get_tile_pattern(bus, id & 0xF, (id >> 4) & 0xF, 0, 0);

    //             for row in 0..8 {
    //                 for col in 0..8 {
    //                     result[((y * 8 + row) << 8) | (x * 8 + col)] = tile[row * 8 | col];
    //                 }
    //             }
    //         }
    //     }

    //     result
    // }
}