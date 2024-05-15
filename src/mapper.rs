mod mapper0;
mod mapper1;
mod testmapper;

use crate::cartridge::Mirroring;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;

#[cfg(test)]
pub use self::testmapper::TestMapper;


const PRG_ROM_LO_START: usize = 0x8000;
const PRG_ROM_LO_END: usize = 0xBFFF;
const PRG_ROM_HI_START: usize = 0xC000;
const PRG_ROM_HI_END: usize = 0xFFFF;

const CHR_ROM_LO_START: usize = 0x0000;
const CHR_ROM_LO_END: usize = 0x0FFF;
const CHR_ROM_HI_START: usize = 0x1000;
const CHR_ROM_HI_END: usize = 0x1FFF;

pub trait Mapper {
    /// Some contains the successfully read byte; None means read is meant to be done from elsewhere...
    fn mapped_cpu_read(&mut self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8>;


    /// Returns true if write was successful; false if write was meant to be done elsewhere...
    fn mapped_cpu_write(&mut self, prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool;


    /// Returns the addressed pattern table byte (from PPU 0x0000 to 0x1FFF)
    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8;


    /// TODO: not used by any mappers so far...
    fn mapped_ppu_write(&mut self, chr_rom: &mut Vec<u8>, addr: usize, byte: u8);


    /// Some mappers can dynamically change mirroring mode during execution
    fn get_updated_mirroring(&self) -> Option<Mirroring> {
        None
    }

    /// Resets mapper's internal state, NOT including memory
    fn reset(&mut self) {}
}