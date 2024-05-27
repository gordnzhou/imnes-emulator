use std::{borrow::Cow, fs, num::NonZeroU32};

use glium::Surface;
use glutin::{
    config::ConfigTemplateBuilder, context::{ContextAttributesBuilder, NotCurrentGlContext}, display::{GetGlDisplay, GlDisplay}, surface::{SurfaceAttributesBuilder, WindowSurface}
};
use imgui_winit_support::winit::{
    dpi::LogicalSize, 
    event_loop::EventLoop, 
    window::WindowBuilder
};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, KeyEvent, WindowEvent}, 
    window::Window
};

use crate::emulator::{Emulator, Screen};

pub struct App {
    width: u32,
    height: u32,
}

const ROMS_FOLDER: &str = "roms/";

impl App {
    pub fn new(width: u32, height: u32) -> Self { 
        Self {
            width,
            height,
        }
    }

    pub fn run_app(&mut self, title: &str) {
        let (event_loop, window, mut display) = self.create_window(title);
        let (mut winit_platform, mut imgui_context) = App::imgui_init(&window);

        let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display)
            .expect("Failed to initialize renderer");

        let screen = Screen::new(&mut renderer, &mut display);

        let mut emulator = Emulator::new(screen);
        emulator.reset();

        let mut last_frame = std::time::Instant::now();
        let mut last_emulation = std::time::Instant::now();

        let mut selected_file = 0;
        let mut file_names: Vec<String> = fs::read_dir(ROMS_FOLDER).unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().unwrap().is_file())
            .map(|e| e.file_name().into_string().unwrap())
            .collect();
        file_names.sort(); 
        
        event_loop.run(move |event, window_target| {
            match event {
                Event::NewEvents(_) => {
                    let now = std::time::Instant::now();
                    imgui_context.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                Event::AboutToWait => {
                    winit_platform
                        .prepare_frame(imgui_context.io_mut(), &window)
                        .expect("Failed to prepare frame");
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let ui = imgui_context.frame();

                    let now = std::time::Instant::now();
                    emulator.run_for_duration(now - last_emulation);
                    last_emulation = now;

                    emulator.update_screen(&mut display, &mut renderer, ui);
                    emulator.show_options(ui);
                    emulator.show_cpu_state(ui);
                    emulator.show_ppu_state(ui);
                    emulator.show_apu_state(ui);

                    ui.window("ROMs")
                        .size([300.0, 200.0], imgui::Condition::FirstUseEver)
                        .position([20.0, 20.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.combo("Select a ROM file", &mut selected_file, &file_names, |i| {
                                Cow::Borrowed(i)
                            });

                            if ui.button("Load Selected File") {
                                let file_name = format!("{}{}", ROMS_FOLDER, &file_names[selected_file as usize]);
                                match emulator.load_ines_cartridge(&file_name) {
                                    Err(e) => println!("Error loading ROM: {}", e),
                                    _ => {}
                                }
                            }
                        });

                    // Setup for drawing
                    let mut target = display.draw();

                    // Renderer doesn't automatically clear window
                    target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);

                    // Perform rendering
                    winit_platform.prepare_render(ui, &window);
                    let draw_data = imgui_context.render();
                    renderer
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");
                    target.finish().expect("Failed to swap buffers");
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => window_target.exit(),
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state,
                            physical_key,
                            ..
                        },
                        ..
                    },
                    ..
                } => emulator.update_joypad(physical_key, state),
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(new_size),
                    ..
                } => {
                    if new_size.width > 0 && new_size.height > 0 {
                        display.resize((new_size.width, new_size.height));
                    }
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }
                event => {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }
            }
        }).expect("EventLoop error");
    }

    fn create_window(&self, title: &str) -> (EventLoop<()>, Window, glium::Display<WindowSurface>) {
        let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    
        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(self.width, self.height));
    
        let (window, cfg) = glutin_winit::DisplayBuilder::new()
            .with_window_builder(Some(window_builder))
            .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
                configs.next().unwrap()
            })
            .expect("Failed to create OpenGL window");
        let window = window.unwrap();
    
        let context_attribs = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
        let context = unsafe {
            cfg.display()
                .create_context(&cfg, &context_attribs)
                .expect("Failed to create OpenGL context")
        };
    
        let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.raw_window_handle(),
            NonZeroU32::new(self.width).unwrap(),
            NonZeroU32::new(self.height).unwrap(),
        );
        let surface = unsafe {
            cfg.display()
                .create_window_surface(&cfg, &surface_attribs)
                .expect("Failed to create OpenGL surface")
        };
    
        let context = context
            .make_current(&surface)
            .expect("Failed to make OpenGL context current");
    
        let display = glium::Display::from_context_surface(context, surface)
            .expect("Failed to create glium Display");
    
        (event_loop, window, display)
    }
    
    fn imgui_init(window: &Window) -> (imgui_winit_support::WinitPlatform, imgui::Context) {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);
    
        let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        winit_platform.attach_window(imgui_context.io_mut(), window, imgui_winit_support::HiDpiMode::Default);
    
        imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    
        (winit_platform, imgui_context)
    }
}