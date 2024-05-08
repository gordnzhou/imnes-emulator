use crate::bus::Bus;




pub struct Ppu2C03 {
}

impl Ppu2C03 {
    pub fn new() -> Self {
        Ppu2C03 { 
        }
    }

    pub fn clock(&mut self, _bus: &mut Bus) {
        
    }

    pub fn nmi_requested(&self) -> bool {
        false
    }
}