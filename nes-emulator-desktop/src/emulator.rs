mod audio;
mod screen;
mod joypad;

use std::{borrow::Cow, fs, io, path::Path, time::Duration};

use glium::Display;
use glutin::surface::WindowSurface;
use imgui::{TabItem, Ui};
use imgui_glium_renderer::Renderer;
use winit::{event::ElementState, keyboard::PhysicalKey};

use nesemulib::{Apu2A03, CartridgeNes, Cpu6502, Ppu2C03, SystemBus, SystemControl, BASE_PPU_FREQUENCY};

use crate::logger::Logger;

use self::{audio::AudioPlayer, joypad::Joypad};
pub use screen::Screen;

const SAMPLING_RATE_HZ: u32 = 48000;

const SAVE_FOLDER: &str = "saves/";

const ROMS_FOLDER: &str = "roms/";

pub struct Emulator {
    cpu: Cpu6502,
    ppu: Ppu2C03,
    audio_player: AudioPlayer,
    screen: Screen,
    joypad: Joypad,
    
    pub auto_save: bool,
    selected_file: usize,
    file_names: Vec<String>,
    cartridge_name: Option<String>,
    bus: Option<SystemBus>,

    pub paused: bool,
    game_speed: f32,
    pub total_cycles: u64,
}

impl Emulator {
    pub fn new(screen: Screen) -> Self {
        let apu = Apu2A03::new(SAMPLING_RATE_HZ);

        let selected_file = 0;
        let mut file_names: Vec<String> = fs::read_dir(ROMS_FOLDER).unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().unwrap().is_file())
            .map(|e| e.file_name().into_string().unwrap())
            .collect();
        file_names.sort(); 

        Self {
            cpu: Cpu6502::new(apu),
            ppu: Ppu2C03::new(),
            audio_player: AudioPlayer::new(SAMPLING_RATE_HZ),
            screen,
            joypad: Joypad::new(),

            auto_save: true,
            selected_file,
            file_names,
            cartridge_name: None,
            bus: None,

            game_speed: 1.0,
            paused: true,
            total_cycles: 0,
        }
    }

    pub fn load_ines_cartridge(&mut self, file_name: &str, logger: &mut Logger) -> Result<(), io::Error> {
        self.unload_cartridge(logger);
        
        let cartridge = CartridgeNes::from_ines_file(file_name)?;
        let bus = SystemBus::new(cartridge);

        logger.log_event(&format!("Loaded ROM cartridge: {}", file_name));

        self.cartridge_name = Some(String::from(file_name));
        self.bus = Some(bus);
        self.reset();
        self.load_save_from_file(file_name, logger);
        self.paused = false;

        Ok(())
    }

    pub fn unload_cartridge(&mut self, logger: &mut Logger) {
        if let Some(name) = &self.cartridge_name {
            logger.log_event(&format!("Unloaded ROM cartridge: {}", name))
        }

        self.write_save_to_file(logger);
        self.reset();
        self.paused = true;
        self.bus = None;
        self.cartridge_name = None;
    }

    fn load_save_from_file(&mut self, file_name: &str, logger: &mut Logger) {
        if let Some(bus) = &mut self.bus {

            let file_stem = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
            let save_path = &format!("{}{}.sav", SAVE_FOLDER, file_stem);

            match fs::read(&save_path) {
                Ok(save_ram) => if let Err(e) = bus.cartridge.load_save_ram(save_ram) {
                    logger.log_event(&format!("Unable to load save RAM for {}:\n{}", file_name, e));
                } else {
                    logger.log_event(&format!("Successfully loaded save RAM from: {}", save_path))
                }
                Err(e) if bus.cartridge.mapper.get_save_ram().is_some() => {
                    logger.log_error(&format!("Failed to load save RAM for {}:\n{}", file_name, e));
                }
                _ => {}
            };
        }
    }

    pub fn write_save_to_file(&mut self, logger: &mut Logger) {
        if let Some(bus) = &mut self.bus {

            if let Some(ram) = bus.cartridge.get_save_ram() {
                let file_name = match &self.cartridge_name {
                    Some(name) => name,
                    _ => panic!("Cartridge but no cartridge name")
                };

                let file_stem = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
                let save_path = &format!("{}{}.sav", SAVE_FOLDER, file_stem);

                if let Err(e) = fs::write(save_path, ram) {
                    logger.log_error(&format!("Failed to save RAM to {}:\n{}", save_path, e));
                } else {
                    logger.log_event(&format!("Successfully saved RAM to: {}", save_path));
                }
            }
        }
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
 
    pub fn draw_screen(&mut self, display: &mut Display<WindowSurface>, renderer: &mut Renderer, ui: &mut Ui)  {    
        self.screen.draw(self.ppu.try_get_frame(), display, renderer, ui, &self.cartridge_name)
    }

    pub fn update_joypad(&mut self, physical_key: PhysicalKey, state: ElementState) {
        if let Some(bus) = &mut self.bus {

            if self.joypad.update_joypad(physical_key, state) {
                bus.update_joypad_state(self.joypad.key_state, 0);
            }
        } else {
            let _ = self.joypad.update_joypad(physical_key, state);
        }
    }

    pub fn show_options(&mut self, ui: &Ui, display: &mut Display<WindowSurface>, renderer: &mut Renderer, logger: &mut Logger) {
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
                    self.unload_cartridge(logger);
                    self.screen.clear_screen(display, renderer);
                }

                if ui.slider("Game Speed", 0.5, 2.0, &mut self.game_speed) {
                    self.cpu.apu.adjust_cpu_clock_rate(self.game_speed);
                }

                if ui.button("Default Game Speed") {
                    self.game_speed = 1.0;
                    self.cpu.apu.adjust_cpu_clock_rate(1.0);
                }
            });
    }

    pub fn show_cpu_state(&mut self, ui: &Ui) {
        ui.window("CPU state")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([900.0, 20.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if self.bus.is_some() {
                    ui.label_text("A", format!("0x{:02X}", self.cpu.accumulator));
                    ui.label_text("X", format!("0x{:02X}", self.cpu.x_index_reg));
                    ui.label_text("Y", format!("0x{:02X}", self.cpu.y_index_reg));
                    ui.label_text("P", format!("0x{:02X}", self.cpu.processor_status));
                    ui.label_text("PC", format!("0x{:04X}", self.cpu.program_counter));
                    ui.label_text("SP", format!("0x{:04X}", self.cpu.stack_pointer));
                    ui.label_text("CPU cycles", format!("{}", self.cpu.total_cycles));
                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    pub fn show_ppu_state(&mut self, ui: &Ui) {
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

    pub fn show_apu_state(&mut self, ui: &Ui) {
        ui.window("APU state")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([20.0, 450.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if self.bus.is_some() {
                    let _ = ui.slider("Master Volume", 0.0, 1.0, &mut self.audio_player.master_volume);

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

    pub fn show_roms(&mut self, ui: &Ui, logger: &mut Logger) {
        ui.window("ROMs")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([20.0, 20.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.combo("Select a ROM file", &mut self.selected_file, &self.file_names, |i| {
                    Cow::Borrowed(i)
                });

                if ui.button("Load Selected File") {
                    let file_name = format!("{}{}", ROMS_FOLDER, &self.file_names[self.selected_file]);
                    match self.load_ines_cartridge(&file_name, logger) {
                        Err(e) => println!("Error loading ROM: {}", e),
                        _ => {}
                    }
                }

                if let Some(bus) = &self.bus {
                    ui.text(format!("Mapper: {}", bus.cartridge.mapper_num));
                    ui.same_line_with_spacing(10.0, 80.0);

                    ui.text(format!("Mirroring: {:?}", 
                        if let Some(mirroring) = bus.cartridge.mapper.get_updated_mirroring() {
                            mirroring
                        } else { 
                            bus.cartridge.mirroring 
                    }));

                    ui.text(format!("PRG-ROM banks: {}", bus.cartridge.prg_rom_banks));
                    ui.text(format!("CHR-ROM banks: {}", bus.cartridge.chr_rom_banks));
                    ui.text(format!("Battery Backed: {}", bus.cartridge.battery_backed));
                }
            });
    }

    pub fn show_settings(&mut self, ui: &Ui) {
        ui.modal_popup_config("Settings")
            .build(|| {
                ui.child_window("Settings Child")
                    .size([600.0, 400.0])
                    .build(|| {
                        if let Some(_) = ui.tab_bar("Settings Tab") {  
                            TabItem::new("General").build(ui, || {
                                ui.checkbox("Enable Autosave", &mut self.auto_save);
                            });

                            TabItem::new("Controls").build(ui, || {
                                self.joypad.show_key_settings(ui);
                            });
                        };
                    });

                if ui.button("Close") {
                    ui.close_current_popup();
                }
                ui.same_line_with_spacing(10.0, 80.0);
                if ui.button("Reset to Default") {
                    self.joypad.reset_keys();
                }
            });
    }
}