
// TODO: implement SRAM, EXP ROM mapping
const PRG_ROM_START: usize = 0x8000;
const PRG_ROM_END: usize = 0xFFFF;

const PRG_ROM_LENGTH: usize = 0x8000;

const CHR_ROM_LENGTH: usize = 0x2000;

pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    NONE,
}

pub struct CartridgeNes { 
    prg_rom: [u8; PRG_ROM_LENGTH],
    chr_rom: [u8; CHR_ROM_LENGTH],
    mirroring: Mirroring,
}

impl CartridgeNes {
    #[allow(dead_code)]
    pub fn new() -> Self {
        CartridgeNes {
            prg_rom: [0; PRG_ROM_LENGTH],
            chr_rom: [0; CHR_ROM_LENGTH],
            mirroring: Mirroring::NONE,
        }
    }

    // TODO: read bytes in iNes format
    pub fn from_ines_bytes(data: &[u8]) -> Self {
        let length = 0x4000;
        let offset = 0x10;

        assert!(data.len() >= length);

        let mut prg_rom = [0; PRG_ROM_LENGTH];

        prg_rom[0..length].copy_from_slice(&data[offset..length + offset]);
        prg_rom[length..PRG_ROM_LENGTH].copy_from_slice(&data[offset..length + offset]);

        CartridgeNes { 
            prg_rom,
            chr_rom: [0; CHR_ROM_LENGTH],
            mirroring: Mirroring::HORIZONTAL,
        }
    }

    pub fn cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => self.prg_rom[addr - PRG_ROM_START],
            _ => unimplemented!()
        }
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) {
        match addr {
            PRG_ROM_START..=PRG_ROM_END => self.prg_rom[addr - PRG_ROM_START] = byte,
            _ => unimplemented!()
        }
    }

    // pub fn ppu_read(&mut self, addr: usize) -> u8 {
    //     0
    // }

    // pub fn ppu_write(&mut self, addr: usize, byte: u8) {

    // }
}