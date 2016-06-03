use glium::backend::glutin_backend::GlutinFacade;
use glium::DisplayBuild;
use gpu::renderer::{GLRenderer, Renderer};
use gb_proc::video_controller::GrayShade;
use gb_proc::cpu::Interrupt;

use glium;

pub struct Controller {
    display: GlutinFacade,
    renderer: GLRenderer,
}

impl Controller {
    pub fn new() -> Controller {
        let mut display = glium::glutin::WindowBuilder::new()
            .with_dimensions(640, 576)
            .build_glium()
            .unwrap();

        let renderer = GLRenderer::new(&mut display);

        Controller {
            display: display,
            renderer: renderer,
        }
    }
}

impl Renderer for Controller {
    fn refresh(&mut self, pixels: &[[GrayShade; 160]; 144]) {
        let mut frame = self.display.draw();
        self.renderer.refresh(&mut frame, pixels);
        frame.finish().unwrap();
    }
}

pub enum Key {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
}

pub trait Hardware {
    fn interrupt(&mut self, interrupt: Interrupt);
    fn key_up(&mut self, key: Key);
    fn key_down(&mut self, key: Key);
    fn next(&mut self);
}
