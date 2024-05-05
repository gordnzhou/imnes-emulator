
pub const MEM_SIZE: usize = 0x10000;

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

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }
}