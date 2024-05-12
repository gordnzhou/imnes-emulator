mod ppubus;
mod registers;

pub use ppubus::PpuBus;

use crate::{bus::Bus, palette::{Colour, DISPLAY_PALETTE}};

use self::{ppubus::{ATTR_TABLE_START, NAME_TABLE_START, OAM_SIZE, PALETTE_TABLE_START}, registers::PpuStatus};

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

const SPRITE_CACHE_SIZE: usize = 8;

const OAM_ENTRY_BYTES: usize = 4;

// number of bytes occupied by a single tile in pattern memory
const TILE_BYTES: usize = 16;

#[derive(Clone, Copy)]
struct OAMEntry {
    pub y: usize,
    pub id: usize,
    attributes: usize,
    pub x: usize,
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

pub struct Ppu2C03 {
    pub frame_buffer: [Colour; SCREEN_WIDTH * SCREEN_HEIGHT],
    cycles: u32,
    scanline: i32,

    sprite_cache: [OAMEntry; SPRITE_CACHE_SIZE],
    sprite_cache_count: usize,
    spr_patt_lo_shifter: [u8; SPRITE_CACHE_SIZE],
    spr_patt_hi_shifter: [u8; SPRITE_CACHE_SIZE],
    contains_spr_0: bool,
    spr_0_rendered: bool,

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
        Self { 
            frame_buffer: [DISPLAY_PALETTE[0x3F]; SCREEN_WIDTH * SCREEN_HEIGHT],
            cycles: 0,
            scanline: 0,

            sprite_cache: [OAMEntry::default(); SPRITE_CACHE_SIZE],
            sprite_cache_count: 0,
            spr_patt_lo_shifter: [0; SPRITE_CACHE_SIZE],
            spr_patt_hi_shifter: [0; SPRITE_CACHE_SIZE],
            contains_spr_0: false,
            spr_0_rendered: false,

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

                if self.scanline == 0 && self.cycles == 0 
                    && (ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr()) {

                    self.cycles = 1;
                }

                if self.scanline == -1 && self.cycles == 1 {
                    ppu_bus.status.set(PpuStatus::IN_VBLANK, false);
                    ppu_bus.status.set(PpuStatus::SPR_OVERFLOW, false);
                    ppu_bus.status.set(PpuStatus::SPR_0_HIT, false);

                    self.spr_patt_lo_shifter = [0; SPRITE_CACHE_SIZE];
                    self.spr_patt_hi_shifter = [0; SPRITE_CACHE_SIZE];
                }

                // Visible Cycle
                if matches!(self.cycles, 2..=257 | 321..=337) {
                    self.update_shifters(ppu_bus);

                    match (self.cycles - 1) % 8 {
                        0 => { // fetch tile id
                            self.load_bg_shifters();
                            let bg_next_tile_id_addr = NAME_TABLE_START | ((ppu_bus.vram_addr.0 as usize) & 0x0FFF);
                            self.bg_next_tile_id = bus.ppu_read(bg_next_tile_id_addr);
                        }
                        2 => { // fetch tile palette attribute
                            let coarse_x = ppu_bus.vram_addr.coarse_x() as usize;
                            let coarse_y = ppu_bus.vram_addr.coarse_y() as usize;

                            let bg_next_tile_attr_addr = ATTR_TABLE_START 
                                | ((ppu_bus.vram_addr.name_table_y() as usize) << 11)
                                | ((ppu_bus.vram_addr.name_table_x() as usize) << 10)
                                | ((coarse_y >> 2) << 3)
                                | (coarse_x >> 2);

                            self.bg_next_tile_attr = bus.ppu_read(bg_next_tile_attr_addr);

                            if coarse_y & 0x02 != 0 { self.bg_next_tile_attr >>= 4; }
                            if coarse_x & 0x02 != 0 { self.bg_next_tile_attr >>= 2; }
                            self.bg_next_tile_attr &= 0x03;
                        },
                        4 => { // fetch LOW plane of tile pattern
                            let bg_next_tile_lo_addr = ppu_bus.ctrl.bg_pattern_addr()
                                + ((self.bg_next_tile_id as usize) * TILE_BYTES)
                                + ppu_bus.vram_addr.fine_y() as usize;

                            self.bg_next_tile_lo = bus.ppu_read(bg_next_tile_lo_addr);
                        },
                        6 => { // fetch HIGH plane of tile pattern
                            let bg_next_tile_hi_addr = ppu_bus.ctrl.bg_pattern_addr()
                                + ((self.bg_next_tile_id as usize) * TILE_BYTES)
                                + ppu_bus.vram_addr.fine_y() as usize
                                + 8;

                            self.bg_next_tile_hi = bus.ppu_read(bg_next_tile_hi_addr);
                        },
                        7 => { // update vram address X 
                            Ppu2C03::increment_scroll_x(ppu_bus);
                        },
                        _ => {},
                    }
                }
                
                match self.cycles {
                    256 => { // update vram address Y on scanline end
                        Ppu2C03::increment_scroll_y(&mut bus.ppu_bus)
                    },
                    257 => { // reset vram address X on scanline end
                        self.load_bg_shifters();
                        Ppu2C03::reset_vram_x(&mut bus.ppu_bus);
                    },
                    280..=304 if self.scanline == -1 => { // reset vram address Y after each VBlank
                        Ppu2C03::reset_vram_y(&mut bus.ppu_bus)
                    },
                    338 | 340 => {
                        let bg_next_tile_id_addr = NAME_TABLE_START | ((bus.ppu_bus.vram_addr.0 as usize) & 0x0FFF);
                        self.bg_next_tile_id = bus.ppu_read(bg_next_tile_id_addr);
                    }
                    _ => {}
                }

                // Sprite / Foreground Rendering 
                match self.cycles {
                    257 if self.scanline >= 0 => { // fetch ALL sprites for next scanline and update SPR_OVERFLOW
                        self.sprite_cache = [OAMEntry::default(); SPRITE_CACHE_SIZE];
                        self.spr_patt_lo_shifter = [0; SPRITE_CACHE_SIZE];
                        self.spr_patt_hi_shifter = [0; SPRITE_CACHE_SIZE];
                        self.sprite_cache_count = 0;
                        
                        self.contains_spr_0 = false;
    
                        let mut oam_pos = 0;
                        while oam_pos < OAM_SIZE && self.sprite_cache_count <= SPRITE_CACHE_SIZE {
                            let sprite_dist = self.scanline as i32 - bus.ppu_bus.oam[oam_pos + 0] as i32;
    
                            if sprite_dist >= 0 && sprite_dist < bus.ppu_bus.ctrl.spr_height() as i32 {
    
                                if self.sprite_cache_count < SPRITE_CACHE_SIZE {

                                    if oam_pos == 0 {
                                        self.contains_spr_0 = true;
                                    }

                                    self.sprite_cache[self.sprite_cache_count] = OAMEntry {
                                        y: bus.ppu_bus.oam[oam_pos + 0] as usize,
                                        id: bus.ppu_bus.oam[oam_pos + 1] as usize,
                                        attributes: bus.ppu_bus.oam[oam_pos + 2] as usize,
                                        x: bus.ppu_bus.oam[oam_pos + 3] as usize,
                                    };
                                    self.sprite_cache_count += 1;
                                } else {
                                    bus.ppu_bus.status.set(PpuStatus::SPR_OVERFLOW, true);
                                }
                            }
    
                            oam_pos += OAM_ENTRY_BYTES;
                        }
                    }
                    340 => { // load sprite shifters
                        for i in 0..self.sprite_cache_count {
                            let sprite = self.sprite_cache[i];
                            let y_dist = (self.scanline as usize) - sprite.y;

                            let mut y_offset = y_dist & 0x07;
                            if sprite.y_flipped() { y_offset = 7 - y_offset; }

                            let pattern_addr_lo = if bus.ppu_bus.ctrl.spr_height() == 8 {
                                bus.ppu_bus.ctrl.spr_pattern_addr()
                                    | (sprite.id * TILE_BYTES)
                                    | y_offset
                            } else {
                                let tile_offset = if y_dist < 8 {
                                    (sprite.id & 0b11111110) * TILE_BYTES
                                } else {
                                    ((sprite.id & 0b11111110) + 1) * TILE_BYTES
                                };

                                ((sprite.id & 0x01) << 12) 
                                    | tile_offset 
                                    | y_offset
                            };

                            let mut sprite_pattern_lo = bus.ppu_read(pattern_addr_lo + 0);
                            let mut sprite_pattern_hi = bus.ppu_read(pattern_addr_lo + 8);

                            if sprite.x_flipped() {
                                sprite_pattern_lo = REVERSED_BYTE[sprite_pattern_lo as usize];
                                sprite_pattern_hi = REVERSED_BYTE[sprite_pattern_hi as usize];
                            }

                            self.spr_patt_lo_shifter[i] = sprite_pattern_lo;
                            self.spr_patt_hi_shifter[i] = sprite_pattern_hi;
                        }
                    }
                    _ => {}
                }
            }
            240 => {}
            241..=260 => { // In VBlank
                if self.scanline == 241 && self.cycles == 1 {
                    bus.ppu_bus.status.set(PpuStatus::IN_VBLANK, true);

                    if bus.ppu_bus.ctrl.nmi_enabled() {
                        self.nmi = true;
                    }
                }
            }
            _ => {}
        }

        // Render pixel if in visible range
        if matches!(self.scanline, 0..=239) && matches!(self.cycles, 0..=255) {
            let ppu_bus = &mut bus.ppu_bus;

            let mut bg_pixel = 0;
            let mut bg_palette = 0;
            if ppu_bus.mask.show_bg() && (ppu_bus.mask.show_bg_left() || self.cycles >= 9) {
                let mask = 0x8000 >> ppu_bus.fine_x;

                let bg_pixel_bot = ((self.bg_patt_lo_shifter & mask) != 0) as usize;
                let bg_pixel_top = ((self.bg_patt_hi_shifter & mask) != 0) as usize;
                bg_pixel = (bg_pixel_top << 1) | bg_pixel_bot;

                let bg_palette_bot = ((self.bg_attr_lo_shifter & mask) != 0) as usize;
                let bg_palette_top = ((self.bg_attr_hi_shifter & mask) != 0) as usize;
                bg_palette = (bg_palette_top << 1) | bg_palette_bot
            }

            let mut spr_pixel = 0;
            let mut spr_palette = 0;
            let mut spr_priority = false;
            if ppu_bus.mask.show_spr() && (ppu_bus.mask.show_spr_left() || self.cycles >= 9) {
                self.spr_0_rendered = false;

                for i in 0..self.sprite_cache_count {
                    let sprite = &mut self.sprite_cache[i];

                    if sprite.x > 0 {
                        continue;
                    }
                    
                    let spr_pixel_bot = ((self.spr_patt_lo_shifter[i] & 0b10000000) != 0) as usize;
                    let spr_pixel_top = ((self.spr_patt_hi_shifter[i] & 0b10000000) != 0) as usize;
                    spr_pixel = (spr_pixel_top << 1) | spr_pixel_bot;

                    spr_palette = sprite.palette() + 0x04;
                    spr_priority = sprite.priority();

                    if spr_pixel != 0 {

                        if i == 0 {
                            self.spr_0_rendered = true;
                        }

                        break;
                    }
                }
            }

            // resolve background and sprite priority
            let (pixel, palette) = match (bg_pixel, spr_pixel) {
                (0, 0) => (0, 0),
                (0, spr_pixel) => (spr_pixel, spr_palette),
                (bg_pixel, 0) => (bg_pixel, bg_palette),
                (bg_pixel, spr_pixel) => {
                    // check for SPR_0_HIT flag, which occurs only if BG and SPR pixels are both non-zero
                    if self.contains_spr_0 && self.spr_0_rendered
                        && ppu_bus.mask.show_bg() && ppu_bus.mask.show_spr() {
                        
                        let spr_0_hit = if !(ppu_bus.mask.show_bg_left() || ppu_bus.mask.show_spr_left()) {
                            matches!(self.cycles, 9..=257)
                        } else {
                            matches!(self.cycles, 1..=257)
                        };

                        ppu_bus.status.set(PpuStatus::SPR_0_HIT, spr_0_hit);
                    }

                    if spr_priority {
                        (spr_pixel, spr_palette)
                    } else {
                        (bg_pixel, bg_palette)
                    }
                }
            };

            let frame_index = (self.scanline as usize) * SCREEN_WIDTH + (self.cycles as usize);
            self.frame_buffer[frame_index] = Ppu2C03::get_colour_from_palette(bus, palette, pixel);
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

    #[inline]
    pub fn get_colour_from_palette(bus: &mut Bus, palette: usize, pixel: usize) -> Colour {
        let palette_index = bus.ppu_read(PALETTE_TABLE_START + (palette << 2) + pixel) as usize;
        DISPLAY_PALETTE[palette_index]
    }

    #[inline]
    pub fn increment_scroll_x(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {

            if ppu_bus.vram_addr.coarse_x() >= 31 {
                ppu_bus.vram_addr.set_coarse_x(0);
                ppu_bus.vram_addr.set_name_table_x(!ppu_bus.vram_addr.name_table_x());
            } else {
                ppu_bus.vram_addr.set_coarse_x(ppu_bus.vram_addr.coarse_x() + 1);
            }
        }
    }

    #[inline]
    pub fn increment_scroll_y(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {

            if ppu_bus.vram_addr.fine_y() >= 7 {
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

    #[inline]
    fn load_bg_shifters(&mut self) {
        self.bg_patt_lo_shifter = (self.bg_patt_lo_shifter & 0xFF00) | self.bg_next_tile_lo as u16;
        self.bg_patt_hi_shifter = (self.bg_patt_hi_shifter & 0xFF00) | self.bg_next_tile_hi as u16;

        self.bg_attr_lo_shifter = (self.bg_attr_lo_shifter & 0xFF00) 
            | if self.bg_next_tile_attr & 0b01 != 0 { 0xFF } else { 0x00 };
        self.bg_attr_hi_shifter = (self.bg_attr_hi_shifter & 0xFF00) 
            | if self.bg_next_tile_attr & 0b10 != 0 { 0xFF } else { 0x00 };
    }

    #[inline]
    fn update_shifters(&mut self, ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() {
            self.bg_attr_hi_shifter <<= 1;
            self.bg_attr_lo_shifter <<= 1;
            self.bg_patt_hi_shifter <<= 1;
            self.bg_patt_lo_shifter <<= 1;
        }

        if ppu_bus.mask.show_spr() && matches!(self.cycles, 1..=257) {
            for i in 0..self.sprite_cache_count {
                let sprite = &mut self.sprite_cache[i];

                if sprite.x > 0 {
                    sprite.x -= 1;
                    continue;
                }

                self.spr_patt_lo_shifter[i] <<= 1;
                self.spr_patt_hi_shifter[i] <<= 1;
            }
        }
    }

    #[inline]
    fn reset_vram_x(ppu_bus: &mut PpuBus) {
        if ppu_bus.mask.show_bg() || ppu_bus.mask.show_spr() {
            ppu_bus.vram_addr.set_name_table_x(ppu_bus.tram_addr.name_table_x());
            ppu_bus.vram_addr.set_coarse_x(ppu_bus.tram_addr.coarse_x())
        }
    }

    #[inline]
    fn reset_vram_y(ppu_bus: &mut PpuBus) {
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

    pub fn frame_complete(&mut self) -> bool {
        let ret = self.frame_complete;
        self.frame_complete = false;
        ret
    }
}

lazy_static! {
    static ref REVERSED_BYTE: Vec<u8> = (0..=255).map(|x| {
        let x = ((x >> 1) & 0x55) | ((x & 0x55) << 1);
        let x = ((x >> 2) & 0x33) | ((x & 0x33) << 2);
        let x = ((x >> 4) & 0x0F) | ((x & 0x0F) << 4);
        x
    }).collect();
}