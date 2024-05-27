use crate::SystemControl;

pub struct FrameSequencer {
    pub mode: bool,
    pub irq_inhibit_flag: bool,

    cycles: u32,
    skipped_cycle: bool,
}

impl SystemControl for FrameSequencer {
    fn reset(&mut self) {
        self.cycles = 0;
        self.mode = false;
        self.skipped_cycle = false;
        self.irq_inhibit_flag = false;
    }
}

impl FrameSequencer {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            mode: false,
            skipped_cycle: false,
            irq_inhibit_flag: false,
        }
    }

    pub fn clock(&mut self, irq_flag: &mut bool) -> Option<(u8, u8)> {
        if !self.mode && self.cycles >= 14914 {
            if !self.irq_inhibit_flag {
                *irq_flag = true;
            }
        } 
        
        if !self.skipped_cycle {
            self.skipped_cycle = true;

            if self.mode {
                match self.cycles {
                    3728  => Some((0, 1)),
                    7456  => Some((0, 2)),
                    11185 => Some((0, 3)),
                    14914 => Some((0, 4)),
                    _ => None
                }
            } else {
                match self.cycles {
                    3728  => Some((1, 1)),
                    7456  => Some((1, 2)),
                    11185 => Some((1, 3)),
                    14914 => Some((1, 4)),
                    18640 => Some((1, 5)),
                    _ => None
                }
            }
        } else {
            self.cycles += 1;
            self.skipped_cycle = false;

            if (!self.mode && self.cycles == 14915) || (self.mode && self.cycles == 18641) {
                self.cycles = 0;
            }

            None
        }
    }
}

