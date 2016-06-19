use glium::backend::glutin_backend::GlutinFacade;
use glium::DisplayBuild;
use glium::glutin::{VirtualKeyCode, ElementState};
use gpu::renderer::{GLRenderer, Renderer};
use gb_proc::handler_holder::Key;
use gb_proc::video_controller::ScreenBuffer;
use gb_proc::cpu::Interrupt;

use glium;
use glium::glutin;

pub struct Controller {
    display: GlutinFacade,
    renderer: GLRenderer,
}

pub enum Event {
    Quit,
    Break,
    Continue,
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

    pub fn check_events(&mut self, hardware: &mut Hardware) -> Event {
        for event in self.display.poll_events() {
            match event {
                glutin::Event::Closed => {
                    return Event::Quit;
                },
                glutin::Event::KeyboardInput(state, _, virtual_key) => {
                    if state == ElementState::Pressed {
                        match virtual_key {
                            Some(VirtualKeyCode::F1) => {
                                return Event::Break;
                            }
                            _ => {},
                        }
                    }

                    let key = virtual_key.and_then(|k| match k {
                        VirtualKeyCode::Left  => Some(Key::Left),
                        VirtualKeyCode::Right => Some(Key::Right),
                        VirtualKeyCode::Up    => Some(Key::Up),
                        VirtualKeyCode::Down  => Some(Key::Down),
                        VirtualKeyCode::A     => Some(Key::A),
                        VirtualKeyCode::S     => Some(Key::B),
                        VirtualKeyCode::D     => Some(Key::Start),
                        VirtualKeyCode::F     => Some(Key::Select),
                        _ => None,
                    });

                    if let Some(k) = key {
                        match state {
                            ElementState::Pressed => { hardware.key_down(k) },
                            ElementState::Released => { hardware.key_up(k) },
                        }
                        hardware.interrupt(Interrupt::Joypad);
                    };
                },
                _ => {}
            }
        };

        Event::Continue
    }
}

impl Renderer for Controller {
    fn refresh(&mut self, pixels: &ScreenBuffer) {
        let mut frame = self.display.draw();
        self.renderer.refresh(&mut frame, pixels);
        frame.finish().unwrap();
    }
}

pub trait Hardware {
    fn get_screen_buffer(&self) -> &ScreenBuffer;
    fn interrupt(&mut self, interrupt: Interrupt);
    fn key_up(&mut self, key: Key);
    fn key_down(&mut self, key: Key);
    fn next(&mut self);
}
