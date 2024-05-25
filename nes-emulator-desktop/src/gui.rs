use std::{io, num::NonZeroU32, rc::Rc};

use glium::{texture::RawImage2d, uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior}, Surface, Texture2d};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use imgui::Image;
use imgui_glium_renderer::Texture;
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

use nesemulib::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::emulator::Emulator;

pub struct App {
    emulator: Option<Emulator>,
}

impl App {
    pub fn new() -> Result<Self, io::Error> { 
        let emulator = Emulator::new()?;

        Ok(Self {
            emulator: Some(emulator),
        })
    }

    pub fn run_app(&mut self, title: &str) {
        let (event_loop, window, display) = App::create_window(title);
        let (mut winit_platform, mut imgui_context) = App::imgui_init(&window);

        // Create renderer from this crate
        let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display)
            .expect("Failed to initialize renderer");

        // Timer for FPS calculation
        let mut last_frame = std::time::Instant::now();
        let mut last_emulation = std::time::Instant::now();

        let mut paused = false;

        let width = DISPLAY_WIDTH as u32;
        let height = DISPLAY_HEIGHT as u32;
        let image = RawImage2d::from_raw_rgba([0; 4 * DISPLAY_WIDTH * DISPLAY_HEIGHT].to_vec(), 
            (width, height));
        let texture = Rc::new(Texture2d::new(&display, image).unwrap());
        let texture_id = renderer.textures().insert(Texture { texture: Rc::clone(&texture), sampler: SamplerBehavior::default() });
        let sampler = SamplerBehavior {
            magnify_filter: MagnifySamplerFilter::Nearest,
            minify_filter: MinifySamplerFilter::Nearest,
            ..Default::default()
        };
        
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

                    if let Some(emulator) = &mut self.emulator {
                        if !paused {
                            let now = std::time::Instant::now();
                            emulator.run_for_duration(now - last_emulation);
                            last_emulation = now;

                            if let Some(frame) = emulator.get_updated_frame() {
                                let image = RawImage2d::from_raw_rgba(frame.to_vec(), (width, height));
                                let new_texture = Rc::new(Texture2d::new(&display, image).unwrap());
                                renderer.textures().replace(texture_id, Texture { texture: new_texture, sampler });
                            }
                        }
                    }

                    // renders the screen
                    Image::new(texture_id, [3.0 * width as f32, 3.0 * height as f32]).build(&ui);

                    ui.window("Hello world")
                        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            if ui.button("Pause / Unpause") {
                                paused = !paused
                            }
                            if ui.button("Stop") {
                                paused = true;
                                self.emulator = None;
                            }
                            if ui.button("Restart") {
                                if let Ok(emulator) = Emulator::new() {
                                    self.emulator = Some(emulator);
                                    paused = false;
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
                } => if let Some(emulator) = &mut self.emulator {
                    emulator.update_joypad(physical_key, state)
                },
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

    fn create_window(title: &str) -> (EventLoop<()>, Window, glium::Display<WindowSurface>) {
        let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    
        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(1024, 768));
    
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
            NonZeroU32::new(1024).unwrap(),
            NonZeroU32::new(768).unwrap(),
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
    
        let dpi_mode = imgui_winit_support::HiDpiMode::Default;
    
        winit_platform.attach_window(imgui_context.io_mut(), window, dpi_mode);
    
        imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    
        (winit_platform, imgui_context)
    }
}