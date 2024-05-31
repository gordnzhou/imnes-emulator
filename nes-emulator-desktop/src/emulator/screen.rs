use std::time::{Duration, Instant};

use glium::Display;
use glutin::surface::WindowSurface;
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use nesemulib::{Colour, DISPLAY_HEIGHT, DISPLAY_WIDTH};

use crate::ui::PixelFrame;

const DEFAULT_WINDOW_SIZE: [f32; 2] = [583.0, 568.0];
const SCREEN_MARGIN: f32 = 10.0;

const FRAME_LENGTH: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;

pub struct Screen {
    screen_frame: PixelFrame,

    fps: f32,
    last_frame_update: Instant,
    last_total_frames: u64,
    total_frames: u64,
}

impl Screen {
    pub fn new(renderer: &mut Renderer, display: &mut Display<WindowSurface>) -> Self {
        let width = DISPLAY_WIDTH as u32;
        let height = DISPLAY_HEIGHT as u32;

        Self {
            screen_frame: PixelFrame::new(width, height, renderer, display),
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
            
            self.total_frames += 1;
            self.screen_frame.update_frame(frame.to_vec(), display, renderer);
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
                        
                ui.text(text);
                ui.separator();
                
                self.screen_frame.build(ui, SCREEN_MARGIN);
            });
    }

    pub fn clear_screen(&mut self, display: &mut Display<WindowSurface>, renderer: &mut Renderer) {
        self.screen_frame.update_frame(vec![0; FRAME_LENGTH], display, renderer);
    }

    pub fn reset(&mut self) {
        self.last_frame_update = Instant::now();
        self.last_total_frames = 0;
        self.total_frames = 0;
    }
}