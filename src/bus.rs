use crate::cartridge::CartridgeNes;

const CPU_RAM_START: usize = 0x0000;
const CPU_RAM_END: usize = 0x1FFF;
const PPU_REG_START: usize = 0x2000;
const PPU_REG_END: usize = 0x3FFF;
const IO_REG_START: usize = 0x4000;
const IO_REG_END: usize = 0x401F;
const CARTRIDGE_START: usize = 0x4020;
const CARTRIDGE_END: usize = 0xFFFF;

const CPU_RAM_LENGTH: usize = 0x800;
const PPU_REG_LENGTH: usize = 8;

pub struct Bus {
    cartridge: CartridgeNes,
    cpu_ram: [u8; CPU_RAM_LENGTH],
    ppu_registers: [u8; PPU_REG_LENGTH],

    // TODO: TEMPORARY
    io_registers: [u8; IO_REG_END - IO_REG_START + 1],
}

impl Bus {
    pub fn new(cartridge: CartridgeNes) -> Self {
        Bus {
            cartridge,
            cpu_ram: [0; CPU_RAM_LENGTH],
            ppu_registers: [0; PPU_REG_LENGTH],
            io_registers: [0; IO_REG_END - IO_REG_START + 1],
        }
    }

    pub fn cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            CPU_RAM_START..=CPU_RAM_END => self.cpu_ram[addr % CPU_RAM_LENGTH],
            PPU_REG_START..=PPU_REG_END => self.ppu_registers[(addr - PPU_REG_START) % PPU_REG_LENGTH],
            IO_REG_START..=IO_REG_END => self.io_registers[addr - IO_REG_START],
            CARTRIDGE_START..=CARTRIDGE_END => self.cartridge.cpu_read(addr),
            _ => unimplemented!()
        }
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) {
        match addr {
            CPU_RAM_START..=CPU_RAM_END => self.cpu_ram[addr % CPU_RAM_LENGTH] = byte,
            PPU_REG_START..=PPU_REG_END => self.ppu_registers[(addr - PPU_REG_START) % PPU_REG_LENGTH] = byte,
            IO_REG_START..=IO_REG_END => self.io_registers[addr - IO_REG_START] = byte,
            CARTRIDGE_START..=CARTRIDGE_END => self.cartridge.cpu_write(addr, byte),
            _ => unimplemented!()
        }
    }

    // pub fn ppu_read(&mut self, addr: usize) -> u8 {
    //     0
    // }

    // pub fn ppu_write(&mut self, addr: usize, byte: u8) {

    // }
}

#[cfg(test)]
impl Bus {
    pub fn load_ram(&mut self, data: &[u8]) {
        self.cpu_ram[..data.len()].copy_from_slice(&data[..data.len()]);
    }
}