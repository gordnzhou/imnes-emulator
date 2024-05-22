const PATTERN_TABLE_1_ADDR: usize = 0x0000;
const PATTERN_TABLE_2_ADDR: usize = 0x1000;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PpuCtrl: u8 {
        const NAME_TABLE_X     = 0b00000001;
        const NAME_TABLE_Y     = 0b00000010;
        const VRAM_ADDR_INC    = 0b00000100;
        const SPR_PATTERN_ADDR = 0b00001000;
        const BG_PATTERN_ADDR  = 0b00010000;
        const SPR_SIZE         = 0b00100000;
        const MASTER_SELECT    = 0b01000000;
        const NMI_ENABLED      = 0b10000000;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct PpuMask: u8 {
        const GREYSCALE_ON  = 0b00000001;
        const SHOW_BG_LEFT  = 0b00000010;
        const SHOW_SPR_LEFT = 0b00000100;
        const SHOW_BG       = 0b00001000;
        const SHOW_SPR      = 0b00010000;
        const EMP_RED       = 0b00100000;
        const EMP_GREEN     = 0b01000000;
        const EMP_BLUE      = 0b10000000;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct PpuStatus: u8 {
        const SPR_OVERFLOW = 0b00100000;
        const SPR_0_HIT    = 0b01000000;
        const IN_VBLANK    = 0b10000000;
    }
}

impl PpuCtrl { 
    #[inline]
    pub fn name_table_x(&self) -> bool {
        self.contains(PpuCtrl::NAME_TABLE_X)
    }

    #[inline]
    pub fn name_table_y(&self) -> bool {
        self.contains(PpuCtrl::NAME_TABLE_Y)
    }

    #[inline]
    pub fn vram_addr_inc(&self) -> u16 {
        if self.contains(PpuCtrl::VRAM_ADDR_INC) {
            32
        } else {
            1
        }
    }

    #[inline]
    pub fn spr_pattern_addr(&self) -> usize {
        if self.contains(PpuCtrl::SPR_PATTERN_ADDR) {
            PATTERN_TABLE_2_ADDR
        } else {
            PATTERN_TABLE_1_ADDR
        }
    }

    #[inline]
    pub fn bg_pattern_addr(&self) -> usize {
        if self.contains(PpuCtrl::BG_PATTERN_ADDR) {
            PATTERN_TABLE_2_ADDR
        } else {
            PATTERN_TABLE_1_ADDR
        }
    }

    #[inline]
    pub fn spr_height(&self) -> usize {
        if self.contains(PpuCtrl::SPR_SIZE) {
            16
        } else {
            8
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn master_select(&self) -> bool {
        self.contains(PpuCtrl::MASTER_SELECT)
    }

    #[inline]
    pub fn nmi_enabled(&self) -> bool {
        self.contains(PpuCtrl::NMI_ENABLED)
    }
}

impl PpuMask { 
    #[inline]
    pub fn greyscale_on(&self) -> bool {
        self.contains(PpuMask::GREYSCALE_ON)
    }

    #[inline]
    pub fn show_bg_left(&self) -> bool {
        self.contains(PpuMask::SHOW_BG_LEFT)
    }

    #[inline]
    pub fn show_spr_left(&self) -> bool {
        self.contains(PpuMask::SHOW_SPR_LEFT)
    }

    #[inline]
    pub fn show_bg(&self) -> bool {
        self.contains(PpuMask::SHOW_BG)
    }

    #[inline]
    pub fn show_spr(&self) -> bool {
        self.contains(PpuMask::SHOW_SPR)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn emp_red(&self) -> bool {
        self.contains(PpuMask::EMP_RED)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn emp_green(&self) -> bool {
        self.contains(PpuMask::EMP_GREEN)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn emp_blue(&self) -> bool {
        self.contains(PpuMask::EMP_BLUE)
    }
}


#[derive(Default, Clone, Copy, Debug)]
pub struct LoopyPpuReg(pub u16);

// Credits to Loopy: https://www.nesdev.org/wiki/PPU_scrolling
impl LoopyPpuReg {
    pub const COARSE_X: u16     = 0b0000000000011111;
    pub const COARSE_Y: u16     = 0b0000001111100000;
    pub const NAME_TABLE_X: u16 = 0b0000010000000000;
    pub const NAME_TABLE_Y: u16 = 0b0000100000000000;
    pub const FINE_Y: u16       = 0b0111000000000000;

    #[inline]
    pub fn increment_horizontal(&mut self) {
        if self.coarse_x() >= 31 {
            self.set_coarse_x(0);
            self.set_name_table_x(!self.name_table_x());
        } else {
            self.set_coarse_x(self.coarse_x() + 1);
        }
    }

    #[inline]
    pub fn set_horizontal_to_tram(&mut self, tram_addr: &LoopyPpuReg) {
        self.set_name_table_x(tram_addr.name_table_x());
        self.set_coarse_x(tram_addr.coarse_x());
    }

    #[inline]
    pub fn increment_vertical(&mut self) {
        if self.fine_y() >= 7 {
            self.set_fine_y(0);

            if self.coarse_y() == 29 {
                self.set_coarse_y(0);
                self.set_name_table_y(!self.name_table_y());

            } else if self.coarse_y() == 31 {
                self.set_coarse_y(0);
                
            } else {
                self.set_coarse_y(self.coarse_y() + 1)
            }

        } else {
            self.set_fine_y(self.fine_y() + 1)
        }
    }

    #[inline]
    pub fn set_vertical_to_tram(&mut self, tram_addr: &LoopyPpuReg) {
        self.set_name_table_y(tram_addr.name_table_y());
        self.set_coarse_y(tram_addr.coarse_y());
        self.set_fine_y(tram_addr.fine_y());
    }

    #[inline]
    pub fn coarse_x(&self) -> u16 {
        self.get_mask(LoopyPpuReg::COARSE_X)
    }

    #[inline]
    pub fn coarse_y(&self) -> u16 {
        self.get_mask(LoopyPpuReg::COARSE_Y)
    }

    #[inline]
    pub fn name_table_x(&self) -> bool {
        self.get_mask(LoopyPpuReg::NAME_TABLE_X) != 0
    }

    #[inline]
    pub fn name_table_y(&self) -> bool {
        self.get_mask(LoopyPpuReg::NAME_TABLE_Y) != 0
    }

    #[inline]
    pub fn fine_y(&self) -> u16 {
        self.get_mask(LoopyPpuReg::FINE_Y)
    }

    #[inline]
    pub fn set_coarse_x(&mut self, val: u16) {
        self.set_mask(LoopyPpuReg::COARSE_X, val);
    }

    #[inline]
    pub fn set_coarse_y(&mut self, val: u16) {
        self.set_mask(LoopyPpuReg::COARSE_Y, val);
    }

    #[inline]
    pub fn set_name_table_x(&mut self, val: bool) {
        self.set_mask(LoopyPpuReg::NAME_TABLE_X, val as u16);
    }

    #[inline]
    pub fn set_name_table_y(&mut self, val: bool) {
        self.set_mask(LoopyPpuReg::NAME_TABLE_Y, val as u16);
    }

    #[inline]
    pub fn set_fine_y(&mut self, val: u16) {
        self.set_mask(LoopyPpuReg::FINE_Y, val)
    }

    #[inline]
    pub fn set_mask(&mut self, mask: u16, val: u16) {
        self.0 &= !mask;
        self.0 |= (val << mask.trailing_zeros()) & mask
    }

    #[inline]
    pub fn get_mask(&self, mask: u16) -> u16 {
        (self.0 & mask) >> mask.trailing_zeros()
    }
}