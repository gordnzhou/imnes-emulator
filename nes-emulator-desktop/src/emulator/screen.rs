use std::{rc::Rc, time::{Duration, Instant}};

use glium::{texture::RawImage2d, uniforms, Display, Texture2d};
use glutin::surface::WindowSurface;
use imgui::{Image, TextureId, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use nesemulib::{Colour, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const DEFAULT_WINDOW_SIZE: [f32; 2] = [583.0, 568.0];
const SCREEN_MARGIN: f32 = 10.0;

const FRAME_LENGTH: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;

pub struct Screen {
    texture_id: TextureId,
    sampler: uniforms::SamplerBehavior,

    fps: f32,
    last_frame_update: Instant,
    last_total_frames: u64,
    total_frames: u64,
}

impl Screen {
    pub fn new(renderer: &mut Renderer, display: &mut Display<WindowSurface>) -> Self {
        let width = DISPLAY_WIDTH as u32;
        let height = DISPLAY_HEIGHT as u32;

        let image = RawImage2d::from_raw_rgba([0; FRAME_LENGTH].to_vec(), 
            (width, height));
        let sampler = uniforms::SamplerBehavior {
            magnify_filter: uniforms::MagnifySamplerFilter::Nearest,
            minify_filter: uniforms::MinifySamplerFilter::Nearest,
            ..Default::default()
        };

        let texture = Rc::new(Texture2d::new(display, image).unwrap());
        let texture_id = renderer.textures().insert(Texture { texture: Rc::clone(&texture), sampler });

        Self {
            texture_id,
            sampler,

            fps: 0.0,
            last_frame_update: Instant::now(),
            last_total_frames: 0,
            total_frames: 0,
        }
    }

    pub fn draw(&mut self, colours: Option<&[Colour; DISPLAY_WIDTH * DISPLAY_HEIGHT]>, display: &mut Display<WindowSurface>, renderer: &mut Renderer, ui: &Ui, name: &Option<String>) {
        if let Some(colours) = colours {
            let mut frame = [0xFF; FRAME_LENGTH];

            for i in 0..DISPLAY_WIDTH * DISPLAY_HEIGHT {
                frame[4 * i + 0] = colours[i].0;
                frame[4 * i + 1] = colours[i].1;
                frame[4 * i + 2] = colours[i].2;
            }
            
            self.update_screen(frame, display, renderer);
            self.total_frames += 1;
        }

        if Instant::now() - self.last_frame_update >= Duration::from_secs(1) {
            let elapsed = (Instant::now() - self.last_frame_update).as_secs_f32();
            self.fps = (self.total_frames - self.last_total_frames) as f32 / elapsed;
            self.last_total_frames = self.total_frames;
            self.last_frame_update = Instant::now();
        };

        ui.window("Screen")
            .size(DEFAULT_WINDOW_SIZE, imgui::Condition::FirstUseEver)
            .position([300.0, 20.0], imgui::Condition::Always)
            .build(|| {
                let text = if let Some(name) = name {
                    format!("{} (FPS: {:.3})", name, self.fps)
                } else {
                    format!("NO ROM DETECTED")
                };

                // ensure correct aspect ratio
                let mut window_size = ui.window_size();
                if window_size[0] < window_size[1] {
                    window_size[1] = DISPLAY_HEIGHT as f32 * window_size[0] / DISPLAY_WIDTH as f32;
                } else {
                    window_size[0] = DISPLAY_WIDTH as f32 * window_size[1] / DISPLAY_HEIGHT as f32;
                }

                let screen_width = window_size[0] - 2.0 * SCREEN_MARGIN;
                let screen_height = window_size[1] - 2.0 * SCREEN_MARGIN;
                        
                ui.text(text);
                ui.separator();
                Image::new(self.texture_id, [screen_width, screen_height])
                    .build(&ui);
            });
    }

    pub fn clear_screen(&mut self, display: &mut Display<WindowSurface>, renderer: &mut Renderer) {
        self.update_screen([0; FRAME_LENGTH], display, renderer)
    }

    pub fn reset(&mut self) {
        self.last_frame_update = Instant::now();
        self.last_total_frames = 0;
        self.total_frames = 0;
    }

    #[inline]
    fn update_screen(&mut self, frame: [u8; FRAME_LENGTH], display: &mut Display<WindowSurface>, renderer: &mut Renderer) {
        let image = RawImage2d::from_raw_rgba(frame.to_vec(), (DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32));
        let new_texture = Rc::new(Texture2d::new(display, image).unwrap());
        renderer.textures().replace(self.texture_id, Texture { texture: new_texture, sampler: self.sampler });
    }
}