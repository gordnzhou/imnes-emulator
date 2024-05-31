use crate::cartridge::CartridgeNes;
use crate::ppu::PpuBus;
use crate::SystemControl;

const CPU_RAM_START: usize = 0x0000;
const CPU_RAM_END: usize = 0x1FFF;
const PPU_REG_START: usize = 0x2000;
const PPU_REG_END: usize = 0x3FFF;

pub const DMA_REG_ADDR: usize = 0x4014;
const JOYPAD1_REG: usize = 0x4016;
const JOYPAD2_REG: usize = 0x4017;

const CPU_RAM_LENGTH: usize = 0x800;

pub struct SystemBus {
    pub cartridge: CartridgeNes,
    pub ppu_bus: PpuBus,

    cpu_ram: [u8; CPU_RAM_LENGTH],
    joypad_registers: [u8; 2],
    joypad_state: [u8; 2],

    dma_page: u8,
    dma_addr: u8,
    dma_data: u8,
    pub dma_transferring: bool,
    false_dma: bool,

    // APU's DMC stalls the CPU whenever it reads a sample byte 
    pub dmc_read_stall: u8,
}

impl SystemControl for SystemBus {
    fn reset(&mut self) {
        self.cartridge.reset();
        self.ppu_bus.reset();
        self.joypad_registers = [0; 2];
        self.joypad_state = [0; 2];
        self.dma_page = 0x00;
        self.dma_addr = 0x00;
        self.dma_data = 0x00;
        self.dma_transferring = false;
        self.false_dma = true;
    }
}

impl SystemBus {
    pub fn new(cartridge: CartridgeNes) -> Self {
        Self {
            cartridge,
            cpu_ram: [0; CPU_RAM_LENGTH],

            ppu_bus: PpuBus::new(),

            joypad_registers: [0; 2],
            joypad_state: [0; 2],

            dma_page: 0,
            dma_addr: 0,
            dma_data: 0,
            dma_transferring: false,
            false_dma: true,

            dmc_read_stall: 0
        }
    }

    pub fn cpu_read(&mut self, addr: usize, read_only: bool) -> Option<u8> {
        match self.cartridge.cpu_read(addr) {
            Some(byte) => return Some(byte),
            None => {}
        }

        match addr {
            CPU_RAM_START..=CPU_RAM_END => {
                Some(self.cpu_ram[addr % CPU_RAM_LENGTH])
            },
            PPU_REG_START..=PPU_REG_END => {
                Some(self.ppu_bus.cpu_read_reg(addr, &mut self.cartridge, read_only))
            },
            DMA_REG_ADDR => {
                Some(0)
            },
            JOYPAD1_REG | JOYPAD2_REG => {
                let ret = (self.joypad_registers[addr & 0x01] & 0b10000000) != 0;
                self.joypad_registers[addr & 0x01] <<= 1;

                Some(ret as u8)
            }
            _ => None
        }
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) -> bool {
        if self.cartridge.cpu_write(addr, byte) {
            return true;
        }

        let mut success = true;

        match addr {
            CPU_RAM_START..=CPU_RAM_END => self.cpu_ram[addr % CPU_RAM_LENGTH] = byte,
            PPU_REG_START..=PPU_REG_END => {
                self.ppu_bus.cpu_write_reg(addr, byte, &mut self.cartridge)
            },
            DMA_REG_ADDR => {
                self.dma_page = byte;
                self.dma_addr = 0x00;
                self.dma_transferring = true;
            }
            JOYPAD1_REG => {
                self.joypad_registers[0] = self.joypad_state[0];
                self.joypad_registers[1] = self.joypad_state[1];
            },
            _ => success = false
        };

        success
    }

    pub fn dma_clock(&mut self, system_cycles: u32) {
        if self.false_dma {

            if system_cycles & 0x01 != 0  {
                self.false_dma = false;
            }
        } else {
            // read on even clock cycles, write on odd cycles
            if system_cycles & 0x01 == 0 {
                let data_addr = (self.dma_page as usize) << 8 | (self.dma_addr as usize);
                self.dma_data = self.cpu_read(data_addr, false).unwrap_or_default();
            } else {
                self.ppu_bus.transfer_to_oam(self.dma_addr as usize, self.dma_data);
                self.dma_addr = self.dma_addr.wrapping_add(1);

                if self.dma_addr == 0x00 {
                    self.dma_transferring = false;
                    self.false_dma = true;
                }
            }
        }
    }

    pub fn ppu_read(&mut self, addr: usize) -> u8 {
        self.ppu_bus.ppu_read(addr, &mut self.cartridge)
    }

    pub fn update_joypad_state(&mut self, joypad_state1: u8, joypad_state2: u8) {
        self.joypad_state[0] = joypad_state1;
        self.joypad_state[1] = joypad_state2;
    }

    pub fn irq_active(&mut self) -> bool {
        self.cartridge.irq_active()
    }
}

#[cfg(test)]
impl SystemBus {
    pub fn load_ram(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            self.cartridge.cpu_write(i, data[i]);
        }
    }

    pub fn test_new() -> Self {
        Self {
            cartridge: CartridgeNes::test_new(),
            cpu_ram: [0; CPU_RAM_LENGTH],

            ppu_bus: PpuBus::new(),

            joypad_registers: [0; 2],
            joypad_state: [0; 2],

            dma_page: 0,
            dma_addr: 0,
            dma_data: 0,
            dma_transferring: false,
            false_dma: true,

            dmc_read_stall: 0,
        }
    }
}