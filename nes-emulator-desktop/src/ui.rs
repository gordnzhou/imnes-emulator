use std::borrow::Cow;

use imgui::{TabItem, Ui};
use glium::Display;
use glutin::surface::WindowSurface;
use imgui_glium_renderer::Renderer;

use crate::{emulator::{Emulator, DEFAULT_SAMPLE_RATE}, logger::Logger};


pub struct EmulatorUi {
    cpu_window: bool,
    apu_window: bool,
    ppu_window: bool,

    new_sample_rate: u32,
}

impl EmulatorUi {
    pub fn new() -> Self {
        Self {
            cpu_window: false,
            apu_window: false,
            ppu_window: false,

            new_sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }

    pub fn render_emulation(&mut self, emulator: &mut Emulator, ui: &Ui, logger: &mut Logger, display: &mut Display<WindowSurface>, renderer: &mut Renderer) {
        
        self.show_options(emulator, ui, display, renderer, logger);

        self.emulation_state_windows(ui, emulator);

        self.rom_window(ui, logger, emulator);

        self.main_menu(emulator, ui);
    }

    fn emulation_state_windows(&self, ui: &Ui, emulator: &mut Emulator) {
        if self.apu_window {
            self.apu_state_window(ui, emulator);
        }
        
        if self.ppu_window {
            self.ppu_state_window(ui, emulator);
        }
        
        if self.cpu_window {
            self.cpu_state_window(ui, emulator);
        }
    }

    pub fn rom_window(&self, ui: &Ui, logger: &mut Logger, emulator: &mut Emulator) {
        ui.window("ROMs")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([20.0, 20.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let mut file_name = None;

                {
                    let rm = &mut emulator.rom_manager;

                    ui.combo("Select a ROM file", &mut rm.selected_file, &rm.file_names, |i| {
                        Cow::Borrowed(i)
                    });

                    if ui.button("Load Selected File") {
                        file_name = Some(format!("{}{}", rm.roms_folder, &rm.file_names[rm.selected_file]));  
                    }
                }

                // Try to load a ROM if a file was chosen and confirmed by button
                if let Some(file_name) = file_name {
                    match emulator.load_ines_cartridge(&file_name, logger) {
                        Err(e) => logger.log_error(&format!("Unable to load ROM: {}", e)),
                        _ => {}
                    }
                }

                if let Some(bus) = &emulator.rom_manager.bus {
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

    // TODO: add pattern table viewer
    fn ppu_state_window(&self, ui: &Ui, emulator: &mut Emulator) {
        ui.window("PPU state")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([900.0, 400.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if let Some(bus) = &emulator.rom_manager.bus {
                    let ppu = &emulator.ppu;
                    let ppu_bus = &bus.ppu_bus;

                    ui.label_text("PPU scanline", format!("{}", ppu.scanline));
                    ui.label_text("PPU dot", format!("{}", ppu.cycles));

                    ui.label_text("NMI enabled", format!("{}", ppu_bus.ctrl.nmi_enabled()));
                    ui.label_text("Sprite Height", format!("{}", ppu_bus.ctrl.spr_height()));

                    ui.label_text("Show BG", format!("{}", ppu_bus.mask.show_bg()));
                    ui.label_text("Show SPR", format!("{}", ppu_bus.mask.show_spr()));
                    ui.label_text("Show BG left", format!("{}", ppu_bus.mask.show_bg_left()));
                    ui.label_text("Show SPR left", format!("{}", ppu_bus.mask.show_spr_left()));

                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    fn apu_state_window(&self, ui: &Ui, emulator: &mut Emulator) {
        ui.window("APU state")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([20.0, 450.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if emulator.rom_manager.bus.is_some() {
                    let _ = ui.slider("Master Volume", 0.0, 1.0, &mut emulator.audio_player.master_volume);

                    let apu = &mut emulator.cpu.apu;

                    Self::apu_channel_state(ui, "Pulse 1", &mut apu.pulse1_enabled, &apu.pulse1_samples);
                    Self::apu_channel_state(ui, "Pulse 2", &mut apu.pulse2_enabled, &apu.pulse2_samples);
                    Self::apu_channel_state(ui, "Triangle", &mut apu.triangle_enabled, &apu.triangle_samples);
                    Self::apu_channel_state(ui, "Noise", &mut apu.noise_enabled, &apu.noise_samples);
                    Self::apu_channel_state(ui, "DMC", &mut apu.dmc_enabled, &apu.dmc_samples);

                    apu.pulse1_samples.clear();
                    apu.pulse2_samples.clear();
                    apu.triangle_samples.clear();
                    apu.noise_samples.clear();
                    apu.dmc_samples.clear();
                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    fn apu_channel_state(ui: &Ui, channel_name: &str, enabled: &mut bool, samples: &[f32]) {
        let _ = ui.checkbox(channel_name, enabled);
        ui.plot_lines(channel_name, samples)
            .scale_min(0.0)
            .scale_max(15.0)
            .build();
    }

    // TODO: add dissasembly tab 
    fn cpu_state_window(&self, ui: &Ui, emulator: &mut Emulator) {
        ui.window("CPU state")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([900.0, 20.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if emulator.rom_manager.bus.is_some() {
                    let cpu = &emulator.cpu;

                    ui.label_text("A", format!("0x{:02X}", cpu.accumulator));
                    ui.label_text("X", format!("0x{:02X}", cpu.x_index_reg));
                    ui.label_text("Y", format!("0x{:02X}", cpu.y_index_reg));
                    ui.label_text("P", format!("0x{:02X}", cpu.processor_status));
                    ui.label_text("PC", format!("0x{:04X}", cpu.program_counter));
                    ui.label_text("SP", format!("0x{:04X}", cpu.stack_pointer));
                    ui.label_text("CPU cycles", format!("{}", cpu.total_cycles));
                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    pub fn show_options(&self, emulator: &mut Emulator, ui: &Ui, display: &mut Display<WindowSurface>, renderer: &mut Renderer, logger: &mut Logger) {
        ui.window("Emulation Options")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([20.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if ui.button(if emulator.paused {"Unpause"} else {"Pause"}) {
                    emulator.paused = !emulator.paused
                }
                if ui.button("Restart") {
                    emulator.reset();
                }
                if ui.button("Stop") {
                    emulator.stop_emulation(logger, display, renderer);
                }

                if ui.slider("Game Speed", 0.5, 2.0, &mut emulator.game_speed) {
                    emulator.cpu.apu.adjust_cpu_clock_rate(emulator.game_speed);
                }

                if ui.button("Default Game Speed") {
                    emulator.game_speed = 1.0;
                    emulator.cpu.apu.adjust_cpu_clock_rate(1.0);
                }
            });
    }

    pub fn main_menu(&mut self, emulator: &mut Emulator, ui: &Ui) {
        ui.main_menu_bar(|| {
            if ui.menu_item("Settings") {
                ui.open_popup("Settings");
            }

            ui.menu("Emulation", || {

                if ui.menu_item("Show CPU state") {
                    self.cpu_window = !self.cpu_window;
                }

                if ui.menu_item("Show PPU state") {
                    self.ppu_window = !self.ppu_window;
                }

                if ui.menu_item("Show APU state") {
                    self.apu_window = !self.apu_window
                }          
            });

            self.settings_popup(emulator, ui);
        });
    }

    fn settings_popup(&mut self, emulator: &mut Emulator, ui: &Ui) {
        ui.modal_popup_config("Settings")
            .build(|| {
                ui.child_window("Settings Child")
                    .size([600.0, 400.0])
                    .build(|| {
                        if let Some(_) = ui.tab_bar("Settings Tab") {  
                            TabItem::new("General").build(ui, || {
                                ui.checkbox("Enable Autosave", &mut emulator.rom_manager.auto_save);

                                ui.input_scalar("Audio Sample Rate", &mut self.new_sample_rate).build();
                                if ui.is_item_deactivated_after_edit() {
                                    emulator.adjust_sample_rate(self.new_sample_rate);
                                }
                            });

                            TabItem::new("Controls").build(ui, || {
                                emulator.joypad.show_key_settings(ui);
                                if ui.button("Reset Keys to Default") {
                                    emulator.joypad.reset_keys();
                                    emulator.rom_manager.auto_save = true;

                                    emulator.adjust_sample_rate(DEFAULT_SAMPLE_RATE);
                                    self.new_sample_rate = DEFAULT_SAMPLE_RATE;
                                }
                            });
                        };
                    });

                if ui.button("Close") {
                    ui.close_current_popup();
                }

                ui.same_line_with_spacing(10.0, 80.0);
                if ui.button("Reset All Settings to Default") {
                    emulator.joypad.reset_keys();
                }
            });
    }
}