use std::{borrow::Cow, env, rc::Rc};

use imgui::{Image, TabItem, TextureId, Ui};
use glium::{texture::RawImage2d, uniforms, Display, Texture2d};
use glutin::surface::WindowSurface;
use imgui_glium_renderer::{Renderer, Texture};
use native_dialog::FileDialog;

use nesemulib::{PATTERN_TABLE_LENGTH, PATTERN_TABLE_W_H};
use crate::{emulator::Emulator, logger::Logger};


pub struct EmulatorUi {
    cpu_window: bool,
    apu_window: bool,
    ppu_window: bool,

    pattern_table_frame: PixelFrame,
    selected_palette: usize,
}

impl EmulatorUi {
    pub fn new(renderer: &mut Renderer, display: &mut Display<WindowSurface>) -> Self {     
        Self {
            cpu_window: true,
            apu_window: true,
            ppu_window: true,

            pattern_table_frame: PixelFrame::new(2 * PATTERN_TABLE_W_H as u32, PATTERN_TABLE_W_H as u32, renderer, display),
            selected_palette: 0,
        }
    }

    pub fn render_emulation(&mut self, emulator: &mut Emulator, ui: &Ui, logger: &mut Logger, renderer: &mut Renderer) {
        
        self.show_options(emulator, ui, renderer, logger);

        self.emulation_state_windows(ui, emulator, renderer);

        self.rom_window(ui, logger, emulator);

        self.main_menu(emulator, ui);
    }

    fn emulation_state_windows(&mut self, ui: &Ui, emulator: &mut Emulator, renderer: &mut Renderer) {
        if self.apu_window {
            self.apu_state_window(ui, emulator);
        }
        
        if self.ppu_window {
            self.ppu_state_window(ui, emulator, renderer);
        }
        
        if self.cpu_window {
            self.cpu_state_window(ui, emulator);
        }
    }

    pub fn rom_window(&self, ui: &Ui, logger: &mut Logger, emulator: &mut Emulator) {
        ui.window("ROMs")
            .size([300.0, 200.0], imgui::Condition::Always)
            .position([0.0, 20.0], imgui::Condition::Always)
            .build(|| {
                let mut file_name = None;

                {
                    let rm = &mut emulator.rom_manager;

                    ui.text(format!("Select a ROM file from: {}", rm.roms_folder));
                    ui.combo("##combo", &mut rm.selected_file, &rm.file_names, |i| {
                        Cow::Borrowed(i)
                    });
                    ui.same_line();
                    if ui.button("Refresh...") {
                        rm.refresh_file_names();
                    }

                    if ui.button("Load Selected File") {
                        file_name = Some(format!("{}{}", rm.roms_folder, &rm.file_names[rm.selected_file]));  
                    }
                }

                // Try to load a ROM if a file was chosen and confirmed by button
                if let Some(file_name) = file_name {
                    match emulator.load_ines_cartridge(&file_name, logger) {
                        Err(e) => logger.log_error(&format!("Unable to load ROM from {}: {}", file_name, e)),
                        _ => {}
                    }
                }

                if let Some(bus) = &emulator.rom_manager.bus {
                    ui.separator();

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

    fn ppu_state_window(&mut self, ui: &Ui, emulator: &mut Emulator, renderer: &mut Renderer) {
        ui.window("PPU State")
            .size([300.0, 350.0], imgui::Condition::FirstUseEver)
            .position([900.0, 370.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if let Some(bus) = &emulator.rom_manager.bus {
                    let ppu = &emulator.ppu;
                    let ppu_bus = &bus.ppu_bus;

                    let register_label = |name: &str, value: &dyn std::fmt::Display| {
                        ui.label_text(format!("{}", value), format!("{}:", name));
                    };

                    if let Some(_) = ui.tab_bar("PPU State Tab") {  
                        TabItem::new("Registers").build(ui, || {

                            register_label("PPU Scanline", &ppu.scanline);
                            register_label("PPU Dot", &ppu.cycles);
                            register_label("Vertical Blank", &ppu_bus.status.in_vblank());
                            register_label("Sprite Overflow", &ppu_bus.status.spr_overflow());
                            register_label("Sprite 0 Hit", &ppu_bus.status.spr_0_hit());
                            register_label("NMI Enabled", &ppu_bus.ctrl.nmi_enabled());
                            register_label("Sprite Height", &ppu_bus.ctrl.spr_height());
                            register_label("Show BG", &ppu_bus.mask.show_bg());
                            register_label("Show SPR", &ppu_bus.mask.show_spr());
                            register_label("Show BG left", &ppu_bus.mask.show_bg_left());
                            register_label("Show SPR left", &ppu_bus.mask.show_spr_left());
                        });

                        TabItem::new("Pattern Table").build(ui, || {
                            let mut frame = [0xFF; 2 * 4 * PATTERN_TABLE_LENGTH];

                            for table in 0..2 {
                                let patt_table = emulator.ppu.get_pattern_table(&bus, table, self.selected_palette);
                                let offset = PATTERN_TABLE_W_H * table;

                                for y in 0..PATTERN_TABLE_W_H {
                                    for x in 0..PATTERN_TABLE_W_H {
                                        let frame_i = offset + x + y * 2 * PATTERN_TABLE_W_H;
                                        let table_i = x + y * PATTERN_TABLE_W_H;

                                        frame[4 * frame_i + 0] = patt_table[table_i].0;
                                        frame[4 * frame_i + 1] = patt_table[table_i].1;
                                        frame[4 * frame_i + 2] = patt_table[table_i].2;
                                    }
                                }
                            }
                            
                            self.pattern_table_frame.update_frame(frame.to_vec(), renderer);
                            self.pattern_table_frame.build(ui, 10.0);
                            
                            ui.text(format!("Viewing Palette: {}", self.selected_palette));
                            for i in 0..8 {  
                                if ui.button(format!("{}", i)) {
                                    self.selected_palette = i;
                                }
                                if i != 7 {
                                    ui.same_line();
                                }
                            }
                        });
                    }

                } else {
                    ui.text("(No currently running ROM)");
                }
            });
    }

    fn apu_state_window(&self, ui: &Ui, emulator: &mut Emulator) {

        let channel_sound_plot = |channel_name: &str, enabled: &mut bool, samples: &[f32]| {
            let _ = ui.checkbox(channel_name, enabled);
            ui.plot_lines(channel_name, samples)
                .scale_min(0.0)
                .scale_max(15.0)
                .build();
        };

        ui.window("APU State")
            .size([300.0, 380.0], imgui::Condition::Always)
            .position([0.0, 420.0], imgui::Condition::Always)
            .build(|| {
                if emulator.rom_manager.bus.is_some() {
                    ui.text("Master Volume");
                    let _ = ui.slider("##slider", 0.0, 1.0, &mut emulator.audio_player.master_volume);
                    ui.separator();

                    let apu = &mut emulator.cpu.apu;

                    channel_sound_plot("Pulse 1", &mut apu.pulse1_enabled, &apu.pulse1_samples);
                    channel_sound_plot("Pulse 2", &mut apu.pulse2_enabled, &apu.pulse2_samples);
                    channel_sound_plot("Triangle", &mut apu.triangle_enabled, &apu.triangle_samples);
                    channel_sound_plot("Noise", &mut apu.noise_enabled, &apu.noise_samples);
                    channel_sound_plot("DMC", &mut apu.dmc_enabled, &apu.dmc_samples);

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

    fn cpu_state_window(&self, ui: &Ui, emulator: &mut Emulator) {
        let style = ui.push_style_var(imgui::StyleVar::ItemInnerSpacing([0.0, 0.0]));

        let register_label = |value: u8, name: &str| {
            ui.text(&format!("{:<7}{}", format!("{}:", name), format!("0x{:02X}", value)));
        };

        ui.window("CPU State")
            .size([320.0, 350.0], imgui::Condition::FirstUseEver)
            .position([900.0, 20.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if let Some(bus) = &mut emulator.rom_manager.bus {
                    let cpu = &emulator.cpu;

                    register_label(cpu.accumulator, "A");
                    ui.same_line();
                    register_label(cpu.stack_pointer, "SP");

                    register_label(cpu.x_index_reg, "X");
                    ui.same_line();
                    register_label(cpu.y_index_reg, "Y");

                    register_label(cpu.processor_status, "P");
                    ui.same_line();
                    ui.text(&format!("{:<7}{}", "PC:", format!("0x{:04X}", cpu.program_counter)));

                    ui.text_wrapped(format!("Total CPU cycles: {}", cpu.total_cycles));
                    
                    ui.separator();
                    ui.separator();
                    for instruction in emulator.cpu.get_disassembly(bus, 10) {
                        ui.text(instruction);
                    }
                    
                } else {
                    ui.text("(No currently running ROM)");
                }
            });

        style.end();
    }

    pub fn show_options(&self, emulator: &mut Emulator, ui: &Ui, renderer: &mut Renderer, logger: &mut Logger) {
        ui.window("Emulation Options")
            .size([300.0, 200.0], imgui::Condition::Always)
            .position([0.0, 220.0], imgui::Condition::Always)
            .build(|| {
                if ui.button(if emulator.paused {"Unpause"} else {"Pause"}) {
                    emulator.paused = !emulator.paused
                }
                ui.same_line();
                if ui.button("Restart") {
                    emulator.reset();
                }
                ui.same_line();
                if ui.button("Stop") {
                    emulator.stop_emulation(logger, renderer);
                }

                ui.separator();

                if ui.slider("Game Speed", 0.5, 2.0, &mut emulator.game_speed) {
                    emulator.cpu.apu.adjust_cpu_clock_rate(emulator.game_speed);
                }

                ui.spacing();

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

                if ui.menu_item("Show CPU State") {
                    self.cpu_window = !self.cpu_window;
                }

                if ui.menu_item("Show PPU State") {
                    self.ppu_window = !self.ppu_window;
                }

                if ui.menu_item("Show APU State") {
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
                                let v_space = ui.push_style_var(imgui::StyleVar::ItemSpacing([10.0, 30.0]));

                                ui.checkbox("Enable Autosave", &mut emulator.rom_manager.auto_save);

                                ui.checkbox("Skip Illegal CPU Opcodes", &mut emulator.cpu.skip_illegal_opcodes);

                                ui.text(format!("Current ROMs Folder: {}", emulator.rom_manager.roms_folder));
                                ui.same_line();
                                if ui.button("Change...") {
                                    match FileDialog::new().show_open_single_dir() {
                                        Ok(Some(selected_path)) => {

                                            let mut rom_folder = selected_path.to_string_lossy().into_owned();

                                            if let Ok(current_folder) = env::current_dir() {  
                                                if let Ok(folder) = selected_path.strip_prefix(current_folder) {
                                                    rom_folder = folder.to_string_lossy().into_owned()
                                                }
                                            }

                                            emulator.rom_manager.roms_folder = format!("{}/", rom_folder);
                                            emulator.rom_manager.refresh_file_names();
                                        }
                                        Ok(None) => {}
                                        Err(e) => eprintln!("Error: {}", e),
                                    }
                                }
                                
                                ui.text(format!("AUDIO DETAILS\nSample Rate: {}Hz\nBuffer Size: {}", 
                                    emulator.audio_player.get_sample_rate(), 
                                    emulator.audio_player.get_buffer_size()));

                                v_space.pop();
                            });

                            TabItem::new("Controls").build(ui, || {
                                emulator.joypad.show_key_settings(ui);
                                if ui.button("Reset Keys to Default") {
                                    emulator.joypad.reset_keys();
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
                    emulator.rom_manager.auto_save = true;
                    emulator.cpu.skip_illegal_opcodes = false;
                }
            });
    }
}


pub struct PixelFrame {
    texture: Rc<Texture2d>,
    texture_id: TextureId,
    sampler: uniforms::SamplerBehavior,
    width: u32,
    height: u32,
}

impl PixelFrame {
    pub fn new(width: u32, height: u32, renderer: &mut Renderer, display: &mut Display<WindowSurface>) -> Self {
        let image = RawImage2d::from_raw_rgba(vec![0; (4 * width * height) as usize], (width, height));
        
        let sampler = uniforms::SamplerBehavior {
            magnify_filter: uniforms::MagnifySamplerFilter::Nearest,
            minify_filter: uniforms::MinifySamplerFilter::Nearest,
            ..Default::default()
        };

        let texture = Rc::new(Texture2d::new(display, image).unwrap());
        let texture_id = renderer.textures().insert(Texture { texture: Rc::clone(&texture), sampler });

        Self {
            texture,
            texture_id,
            sampler,
            width,
            height,
        }
    }

    pub fn build(&self, ui: &Ui, window_margin: f32) {
        let mut size = ui.window_size();

        // ensure proper aspect ratio
        if size[0] * (self.height as f32) < (self.width as f32) * size[1] {
            size[1] = self.height as f32 * size[0] / self.width as f32;
        } else {
            size[0] = self.width as f32 * size[1] / self.height as f32;
        }

        size[0] -= 2.0 * window_margin;
        size[1] -= 2.0 * window_margin;

        Image::new(self.texture_id, size)
            .build(&ui);
    }

    pub fn update_frame(&mut self, frame: Vec<u8>, renderer: &mut Renderer) {
        let image = RawImage2d::from_raw_rgba(frame, (self.width, self.height));

        self.texture.write(glium::Rect {
            left: 0,
            bottom: 0,
            width: self.width,
            height: self.height,
        }, image);

        renderer.textures().replace(self.texture_id, Texture { texture: Rc::clone(&self.texture), sampler: self.sampler });
    }
}