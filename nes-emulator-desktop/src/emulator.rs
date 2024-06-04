mod audio;
mod screen;
mod joypad;

use std::{io, time::Duration};

use imgui::Ui;
use imgui_glium_renderer::Renderer;
use winit::{event::ElementState, keyboard::PhysicalKey};

use nesemulib::{Apu2A03, Cpu6502, Ppu2C03, SystemControl, BASE_PPU_FREQUENCY};

use crate::{logger::Logger, rom::RomManager};

use self::{audio::AudioPlayer, joypad::Joypad};
pub use screen::Screen;

pub struct Emulator {
    pub cpu: Cpu6502,
    pub ppu: Ppu2C03,
    pub audio_player: AudioPlayer,
    pub rom_manager: RomManager,
    pub joypad: Joypad,
    screen: Screen,

    pub paused: bool,
    pub game_speed: f32,
    pub total_cycles: u64,
}

impl Emulator {
    pub fn new(screen: Screen) -> Self {
        let audio_player = AudioPlayer::new();
        let apu = Apu2A03::new(audio_player.get_sample_rate());

        Self {
            cpu: Cpu6502::new(apu),
            ppu: Ppu2C03::new(),
            audio_player,
            screen,
            joypad: Joypad::new(),
            rom_manager: RomManager::new(),

            game_speed: 1.0,
            paused: true,
            total_cycles: 0,
        }
    }

    pub fn load_ines_cartridge(&mut self, file_name: &str, logger: &mut Logger) -> Result<(), io::Error> {
        self.rom_manager.load_ines_cartridge(file_name, logger)?;
        self.reset();
        self.paused = false;

        Ok(())
    }

    pub fn unload_cartridge(&mut self, logger: &mut Logger) {
        self.rom_manager.unload_cartridge(logger);
        self.reset();
        self.paused = true;
    }

    pub fn reset(&mut self) {
        if let Some(bus) = &mut self.rom_manager.bus {
            bus.reset();
            self.cpu.reset(bus);
        } 
        
        self.screen.reset();
        self.ppu.reset();
        self.total_cycles = 0;
    }

    pub fn run_for_duration(&mut self, duration: Duration, logger: &mut Logger) {
        if self.paused {
            return;
        }

        if let Some(bus) = &mut self.rom_manager.bus {
            let mut duration_cycles = duration.as_nanos() as u64 / (1e9 / (self.game_speed * BASE_PPU_FREQUENCY)) as u64;
            while duration_cycles > 0 {
                self.ppu.clock(bus);
        
                if self.total_cycles % 3 == 0 {
                    // CPU clock
                    if bus.dma_transferring {
                        bus.dma_clock(self.total_cycles as u32);
                    } else if bus.dmc_read_stall > 0 {
                        bus.dmc_read_stall -= 1;
                    } else {
                        self.cpu.clock(bus);
                    }
        
                    self.cpu.apu.cpu_clock(bus);
        
                    if let Some(sample) = self.cpu.apu.cpu_try_clock_sample() {
                        self.audio_player.send_sample(sample)
                    }
                }
        
                if self.ppu.nmi_requested() {
                    self.cpu.nmi(bus);
                }
        
                if bus.irq_active() || self.cpu.apu.irq_active() {
                    self.cpu.irq(bus);
                }
        
                self.total_cycles += 1;
                duration_cycles -= 1;
            }
        }

        if self.cpu.jammed {
            logger.log_error("Unable to continue executing ROM as JAM was called");
            self.reset();
        }
    }
 
    pub fn draw_screen(&mut self, renderer: &mut Renderer, ui: &mut Ui)  {    
        self.screen.draw(self.ppu.try_get_frame(), renderer, ui, &self.rom_manager.cartridge_name)
    }

    pub fn update_joypad(&mut self, physical_key: PhysicalKey, state: ElementState) {
        if let Some(bus) = &mut self.rom_manager.bus {

            if self.joypad.update_joypad(physical_key, state) {

                // future TODO: add support for second joypad
                bus.update_joypad_state(self.joypad.get_key_state(), 0);
            }
        } else {
            let _ = self.joypad.update_joypad(physical_key, state);
        }
    }

    pub fn stop_emulation(&mut self, logger: &mut Logger, renderer: &mut Renderer) {
        self.unload_cartridge(logger);
        self.screen.clear_screen(renderer);
    }
}