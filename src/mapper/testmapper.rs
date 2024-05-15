use super::Mapper;


pub struct TestMapper {
    prg_rom: [u8; 0x10000],
    chr_rom: [u8; 0x2000],
}

impl Mapper for TestMapper {
    fn mapped_cpu_read(&mut self, _prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8> {
        Some(self.prg_rom[addr])
    }
    
    fn mapped_cpu_write(&mut self, _prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool {
        self.prg_rom[addr] = byte;
        true
    }
    
    fn mapped_ppu_read(&mut self, _chr_rom: &mut Vec<u8>, addr: usize) -> u8 {
        self.chr_rom[addr]
    }
    
    fn mapped_ppu_write(&mut self, _chr_rom: &mut Vec<u8>, addr: usize, byte: u8) {
        self.chr_rom[addr] = byte;
    }
}

#[cfg(test)]
impl TestMapper {
    pub fn new() -> Self {
        Self {
            prg_rom: [0; 0x10000],
            chr_rom: [0; 0x2000],
        }
    }
}