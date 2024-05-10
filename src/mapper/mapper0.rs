use super::Mapper;

pub struct Mapper0 {

}

impl Mapper for Mapper0 {
    fn mapped_cpu_read(&mut self, addr: usize) -> usize {
        addr
    }
    
    fn mapped_cpu_write(&mut self, addr: usize, _byte: u8) -> usize {
        addr
    }
    
    fn mapped_ppu_read(&mut self, addr: usize) -> usize {
        addr
    }
    
    fn mapped_ppu_write(&mut self, addr: usize, _byte: u8) -> usize {
        addr
    }
}

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 {

        }
    }
}