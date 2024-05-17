mod mapper0;
mod mapper1;
mod mapper2;
mod mapper3;
mod mapper4;
mod mapper66;
mod testmapper;

use crate::cartridge::{Mirroring, PRG_ROM_SIZE};
use crate::SystemControl;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper3::Mapper3;
pub use self::mapper4::Mapper4;
pub use self::mapper66::Mapper66;

#[cfg(test)]
pub use self::testmapper::TestMapper;


const PRG_ROM_START: usize = 0x8000;
const PRG_ROM_END: usize = 0xFFFF;

const PRG_ROM_LO_START: usize = PRG_ROM_START;
const PRG_ROM_LO_END: usize = PRG_ROM_START + PRG_ROM_SIZE - 1;
const PRG_ROM_HI_START: usize = PRG_ROM_START + PRG_ROM_SIZE;
const PRG_ROM_HI_END: usize = PRG_ROM_END;

const CHR_ROM_LO_START: usize = 0x0000;
const CHR_ROM_LO_END: usize = 0x0FFF;
const CHR_ROM_HI_START: usize = 0x1000;
const CHR_ROM_HI_END: usize = 0x1FFF;

pub trait Mapper: SystemControl {

    /// Some contains the successfully read byte; None means read is meant to be done from elsewhere...
    fn mapped_cpu_read(&mut self, prg_rom: &mut Vec<u8>, addr: usize) -> Option<u8>;


    /// Returns true if write was successful; false if write did nothing to the mapper...
    fn mapped_cpu_write(&mut self, prg_rom: &mut Vec<u8>, addr: usize, byte: u8) -> bool;


    /// Returns the addressed pattern table byte (from PPU 0x0000 to 0x1FFF)
    fn mapped_ppu_read(&mut self, chr_rom: &mut Vec<u8>, addr: usize) -> u8;
    

    /// not used by any mappers so far...
    fn mapped_ppu_write(&mut self, _chr_rom: &mut Vec<u8>, _addr: usize, _byte: u8) {}

    /// Some mappers can dynamically change mirroring mode during execution
    fn get_updated_mirroring(&self) -> Option<Mirroring> {
        None
    }

    /// Some mappers require knowledge of when the PPU's scanline has been updated
    fn notify_scanline(&mut self) {}

    /// Returns true if the mapper is sending an IRQ interrupt to the 6502
    fn irq_active(&mut self) -> bool { false }
}
