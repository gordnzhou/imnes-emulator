mod audio;
mod screen;
mod joypad;

use std::{fs, io, path::Path, time::Duration};

use glium::Display;
use glutin::surface::WindowSurface;
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use winit::{event::ElementState, keyboard::PhysicalKey};

use nesemulib::{Apu2A03, CartridgeNes, Cpu6502, Ppu2C03, SystemBus, SystemControl, BASE_PPU_FREQUENCY};
use self::{audio::AudioPlayer, joypad::Joypad};
pub use screen::Screen;

const SAMPLING_RATE_HZ: u32 = 48000;

const SAVE_FOLDER: &str = "saves/";

pub struct Emulator {
    cpu: Cpu6502,
    ppu: Ppu2C03,
    audio_player: AudioPlayer,
    screen: Screen,
    joypad: Joypad,

    cartridge_name: Option<String>,
    bus: Option<SystemBus>,

    pub paused: bool,
    game_speed: f32,
    pub total_cycles: u64,
}

impl Emulator {
    pub fn new(screen: Screen) -> Self {
        let apu = Apu2A03::new(SAMPLING_RATE_HZ);

        Self {
            cpu: Cpu6502::new(apu),
            ppu: Ppu2C03::new(),
            audio_player: AudioPlayer::new(SAMPLING_RATE_HZ),
            screen,
            joypad: Joypad::new(),

            cartridge_name: None,
            bus: None,

            game_speed: 1.0,
            paused: true,
            total_cycles: 0,
        }
    }

    pub fn load_ines_cartridge(&mut self, file_name: &str) -> Result<(), io::Error> {
        self.unload_cartridge();
        
        let cartridge = CartridgeNes::from_ines_file(file_name)?;
        let mut bus = SystemBus::new(cartridge);

        let file_stem = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
        let save_path = &format!("{}{}.sav", SAVE_FOLDER, file_stem);

        if let Ok(save_ram) = fs::read(&save_path) {
            
            if let Err(e) = bus.cartridge.load_save_ram(save_ram) {
                eprintln!("Unable to load save RAM for {}: {}", file_name, e);
            } else {
                println!("loaded SAVE RAM from: {}", save_path);
            }
        }

        self.cartridge_name = Some(String::from(file_name));
        self.bus = Some(bus);
        self.reset();
        self.paused = false;

        Ok(())
    }

    pub fn unload_cartridge(&mut self) {
        if let Some(bus) = &mut self.bus {

            if let Some(ram) = bus.cartridge.get_save_ram() {
                let file_name = match &self.cartridge_name {
                    Some(name) => name,
                    _ => panic!("Cartridge but no cartridge name")
                };

                let file_stem = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
                let save_path = &format!("{}{}.sav", SAVE_FOLDER, file_stem);

                println!("SAVE RAM TO: {}", save_path);
                fs::write(save_path, ram)
                    .expect(&format!("Unable save game for: {}", save_path));
            }
        }

        self.reset();
        self.paused = true;
        self.bus = None;
        self.cartridge_name = None;
    }

    pub fn reset(&mut self) {
        if let Some(bus) = &mut self.bus {
            bus.reset();
            self.cpu.reset(bus);
        } 
        
        self.screen.reset();
        self.ppu.reset();
        self.total_cycles = 0;
    }

    pub fn run_for_duration(&mut self, duration: Duration) {
        if self.paused {
            return;
        }

        if let Some(bus) = &mut self.bus {
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
    }
 
    pub fn update_screen(&mut self, display: &mut Display<WindowSurface>, renderer: &mut Renderer, ui: &mut Ui)  {    
        self.screen.update(self.ppu.try_get_frame(), display, renderer, ui)
    }

    pub fn update_joypad(&mut self, physical_key: PhysicalKey, state: ElementState) {
        if let Some(bus) = &mut self.bus {

            if self.joypad.update_joypad(physical_key, state) {
                bus.update_joypad_state(self.joypad.key_state, 0);
            }
        }
    }

    pub fn show_options(&mut self, ui: &mut Ui) {
        ui.window("Emulation Options")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([20.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if ui.button(if self.paused {"Unpause"} else {"Pause"}) {
                    self.paused = !self.paused
                }
                if ui.button("Restart") {
                    self.reset();
                }
                if ui.button("Stop") {
                    self.unload_cartridge();
                }

                if ui.slider("Game Speed", 0.5, 2.0, &mut self.game_speed) {
                    self.cpu.apu.adjust_cpu_clock_rate(self.game_speed);
                }
            });
    }

    pub fn show_cpu_state(&mut self, ui: &mut Ui) {
        ui.window("CPU state")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([900.0, 20.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if self.bus.is_some() {
                    ui.label_text("A", format!("{:02X}", self.cpu.accumulator));
                    ui.label_text("X", format!("{:02X}", self.cpu.x_index_reg));
                    ui.label_text("Y", format!("{:02X}", self.cpu.y_index_reg));
                    ui.label_text("P", format!("{:02X}", self.cpu.processor_status));
                    ui.label_text("PC", format!("{:04X}", self.cpu.program_counter));
                    ui.label_text("SP", format!("{:04X}", self.cpu.stack_pointer));
                    ui.label_text("CPU cycles", format!("{}", self.cpu.total_cycles));
                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    pub fn show_ppu_state(&mut self, ui: &mut Ui) {
        ui.window("PPU state")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([900.0, 400.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if let Some(bus) = &self.bus {
                    ui.label_text("PPU scanline", format!("{}", self.ppu.scanline));
                    ui.label_text("PPU dot", format!("{}", self.ppu.cycles));

                    ui.label_text("NMI enabled", format!("{}", bus.ppu_bus.ctrl.nmi_enabled()));
                    ui.label_text("Sprite Height", format!("{}", bus.ppu_bus.ctrl.spr_height()));

                    ui.label_text("Show BG", format!("{}", bus.ppu_bus.mask.show_bg()));
                    ui.label_text("Show SPR", format!("{}", bus.ppu_bus.mask.show_spr()));
                    ui.label_text("Show BG left", format!("{}", bus.ppu_bus.mask.show_bg_left()));
                    ui.label_text("Show SPR left", format!("{}", bus.ppu_bus.mask.show_spr_left()));

                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    pub fn show_apu_state(&mut self, ui: &mut Ui) {
        ui.window("APU state")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([20.0, 450.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if self.bus.is_some() {
                    let _ = ui.checkbox("Pulse 1", &mut self.cpu.apu.pulse1_enabled);
                    ui.plot_lines("Pulse 1", &self.cpu.apu.pulse1_samples)
                        .scale_min(0.0)
                        .scale_max(15.0)
                        .build();

                    let _ = ui.checkbox("Pulse 2", &mut self.cpu.apu.pulse2_enabled);
                    ui.plot_lines("Pulse 2", &self.cpu.apu.pulse2_samples)
                        .scale_min(0.0)
                        .scale_max(15.0)
                        .build();

                    let _ = ui.checkbox("Triangle", &mut self.cpu.apu.triangle_enabled);
                    ui.plot_lines("Triangle", &self.cpu.apu.triangle_samples)
                        .scale_min(0.0)
                        .scale_max(15.0)
                        .build();

                    let _ = ui.checkbox("Noise", &mut self.cpu.apu.noise_enabled);
                    ui.plot_lines("Noise", &self.cpu.apu.noise_samples)
                        .scale_min(0.0)
                        .scale_max(15.0)
                        .build();

                    let _ = ui.checkbox("DMC", &mut self.cpu.apu.dmc_enabled);
                    ui.plot_lines("DMC", &self.cpu.apu.dmc_samples)
                        .scale_min(0.0)
                        .scale_max(127.0)
                        .build();

                    self.cpu.apu.pulse1_samples.clear();
                    self.cpu.apu.pulse2_samples.clear();
                    self.cpu.apu.triangle_samples.clear();
                    self.cpu.apu.noise_samples.clear();
                    self.cpu.apu.dmc_samples.clear();
                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }
}