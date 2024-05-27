use std::{rc::Rc, time::{Duration, Instant}};

use glium::{texture::RawImage2d, uniforms, Display, Texture2d};
use glutin::surface::WindowSurface;
use imgui::{Image, TextureId, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use nesemulib::{Colour, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const SCREEN_SCALE: f32 = 2.5;
const SCREEN_MARGIN: f32 = 20.0;

pub struct Screen {
    width: u32,
    height: u32,
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

        let image = RawImage2d::from_raw_rgba([0; 4 * DISPLAY_WIDTH * DISPLAY_HEIGHT].to_vec(), 
            (width, height));
        let sampler = uniforms::SamplerBehavior {
            magnify_filter: uniforms::MagnifySamplerFilter::Nearest,
            minify_filter: uniforms::MinifySamplerFilter::Nearest,
            ..Default::default()
        };

        let texture = Rc::new(Texture2d::new(display, image).unwrap());
        let texture_id = renderer.textures().insert(Texture { texture: Rc::clone(&texture), sampler });

        Self {
            width,
            height,
            texture_id,
            sampler,

            fps: 0.0,
            last_frame_update: Instant::now(),
            last_total_frames: 0,
            total_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.last_frame_update = Instant::now();
        self.last_total_frames = 0;
        self.total_frames = 0;
    }

    pub fn update(&mut self, colours: Option<&[Colour; DISPLAY_WIDTH * DISPLAY_HEIGHT]>, display: &mut Display<WindowSurface>, renderer: &mut Renderer, ui: &mut Ui) {
        if let Some(colours) = colours {
            let mut frame = [0xFF; 4 * DISPLAY_WIDTH * DISPLAY_HEIGHT];

            for i in 0..DISPLAY_WIDTH * DISPLAY_HEIGHT {
                frame[4 * i + 0] = colours[i].0;
                frame[4 * i + 1] = colours[i].1;
                frame[4 * i + 2] = colours[i].2;
            }
            
            let image = RawImage2d::from_raw_rgba(frame.to_vec(), (self.width, self.height));
            let new_texture = Rc::new(Texture2d::new(display, image).unwrap());
            renderer.textures().replace(self.texture_id, Texture { texture: new_texture, sampler: self.sampler });

            self.total_frames += 1;
        }

        if Instant::now() - self.last_frame_update >= Duration::from_secs(1) {
            let elapsed = (Instant::now() - self.last_frame_update).as_secs_f32();
            self.fps = (self.total_frames - self.last_total_frames) as f32 / elapsed;
            self.last_total_frames = self.total_frames;
            self.last_frame_update = Instant::now();
        }

        ui.window("Screen")
            .size([SCREEN_MARGIN + SCREEN_SCALE * self.width as f32, 2.0 * SCREEN_MARGIN + SCREEN_SCALE * self.height as f32], imgui::Condition::FirstUseEver)
            .position([300.0, 100.0], imgui::Condition::Always)
            .build(|| {
                ui.text(format!("FPS: {}", self.fps));
                ui.separator();
                Image::new(self.texture_id, [SCREEN_SCALE * self.width as f32, SCREEN_SCALE * self.height as f32])
                    .build(&ui);
            });
    }
}