use sdl2::{pixels::PixelFormatEnum, render::{Canvas, Texture}, video::Window};

use super::palette::Colour;


pub trait NesScreen {
    fn place_pixel(&mut self, x: usize, y: usize, colour: Colour);

    fn draw_frame(&mut self);
}

pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 240;

pub struct SdlScreen {
    frame: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4],
    canvas: Canvas<Window>,
    texture: Texture,
}

impl NesScreen for SdlScreen {
    fn place_pixel(&mut self, x: usize, y: usize, colour: Colour) {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return;
        }

        self.frame[4 * (y * DISPLAY_WIDTH + x) + 0] = colour.2;
        self.frame[4 * (y * DISPLAY_WIDTH + x) + 1] = colour.1;
        self.frame[4 * (y * DISPLAY_WIDTH + x) + 2] = colour.0;
    }
    
    fn draw_frame(&mut self) {
        self.texture
            .update(None, &self.frame, 4 * DISPLAY_WIDTH as usize)
            .expect("texture update failed");

        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}

impl SdlScreen {
    pub fn new(canvas: Canvas<Window>) -> Self {

        let creator = canvas.texture_creator();
        let texture = creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
            .map_err(|e| e.to_string())
            .unwrap();

        let mut frame = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4];
        
        for i in 0..(DISPLAY_WIDTH * DISPLAY_HEIGHT) {
            frame[4 * i + 3] = 0xFF;
        }

        Self {
            frame,
            canvas,
            texture,
        }
    }
}