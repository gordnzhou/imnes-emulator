use crate::SystemControl;

use super::Mapper;


pub struct TestMapper {
    prg_rom: [u8; 0x10000],
}

impl SystemControl for TestMapper {
    fn reset(&mut self) {}
}

impl Mapper for TestMapper {
    fn mapped_cpu_read(&mut self, _prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        Some(self.prg_rom[addr])
    }
    
    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        self.prg_rom[addr] = byte;
        true
    }
    
    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8 {
        chr_rom[addr]
    }
    
    fn mapped_ppu_write(&mut self, chr_rom: &mut Vec<u8>, addr: usize, byte: u8) {
        chr_rom[addr] = byte;
    }
}

#[cfg(test)]
impl TestMapper {
    pub fn new() -> Self {
        Self {
            prg_rom: [0; 0x10000],
        }
    }
}