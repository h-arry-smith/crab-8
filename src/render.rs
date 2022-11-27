use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas, AudioSubsystem, EventPump};

use crate::display::Display;

pub struct Renderer {
    pub canvas: WindowCanvas,
    pub event_pump: EventPump,
    width: u32,
    cell_size: u32,
    pub audio_subsystem: AudioSubsystem,
}

impl Renderer {
    pub fn new(width: u32, height: u32, cell_size: u32) -> Self {
        let window_width = width * cell_size;
        let window_height = height * cell_size;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Flake-8", window_width, window_height)
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let event_pump = sdl_context.event_pump().unwrap();

        let audio_subsystem = sdl_context.audio().unwrap();

        Self {
            canvas,
            event_pump,
            width,
            cell_size,
            audio_subsystem,
        }
    }

    pub fn render(&mut self, display: &Display) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (i, pixel) in display.memory.iter().enumerate() {
            if !pixel {
                continue;
            }

            let x = i as u32 % self.width;
            let y = i as u32 / self.width;

            self.canvas
                .fill_rect(Rect::new(
                    (x * self.cell_size) as i32,
                    (y * self.cell_size) as i32,
                    self.cell_size,
                    self.cell_size,
                ))
                .unwrap();
        }

        self.canvas.present();
    }
}
