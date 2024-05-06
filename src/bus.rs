
const MEM_SIZE: usize = 0x10000;
const RAM_START: usize = 0x0000;
const RAM_END: usize = 0x07FF;
const RAM_MIRROR_START: usize = 0x0800;
const RAM_MIRROR_END: usize = 0x1FFF;
const IO_START: usize = 0x2000;
const IO_END: usize = 0x401F;
const EXP_ROM_START: usize = 0x4020;
const EXP_ROM_END: usize = 0x5FFF;
const SRAM_START: usize = 0x6000;
const SRAM_END: usize = 0x7FFF;
const PRG_ROM_START: usize = 0x8000;
const PRG_ROM_END: usize = 0xFFFF;

pub struct Bus {
    memory: [u8; MEM_SIZE]
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            memory: [0; MEM_SIZE],
        }
    }

    pub fn load_memory(&mut self, data: &[u8]) {
        assert!(data.len() <= MEM_SIZE);

        self.memory[..data.len()].copy_from_slice(&data[..data.len()]);
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        assert!(data.len() <= MEM_SIZE - PRG_ROM_START);

        self.memory[PRG_ROM_START..PRG_ROM_START+0x4000].copy_from_slice(&data[0x0010..0x4010]);
        self.memory[PRG_ROM_START+0x4000..=PRG_ROM_END].copy_from_slice(&data[0x0010..0x4010]);
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    pub fn read_io_register(&self, addr: u16) -> u8 {
        0
    }

    pub fn write_io_register(&mut self, addr: u16, byte: u8) {
        
    }
}