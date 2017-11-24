use sound::SDLPlayer;
use glium::backend::glutin_backend::GlutinFacade;
use glium::DisplayBuild;
use glium::glutin::{VirtualKeyCode, ElementState};

use gb::{
    Hardware,
    Emulator,
    Key,
    Interrupt,
};

use gpu::renderer::GLRenderer;

use glium;
use glium::glutin;

pub struct Controller {
    display: GlutinFacade,
    renderer: GLRenderer,
    player: SDLPlayer,
}

pub enum Event {
    Quit,
    Break,
    ToggleSpeed,
    Continue,
}

impl Controller {
    pub fn new(x: u32, y: u32) -> Controller {
        let mut display = glium::glutin::WindowBuilder::new()
            .with_title("gb-rust")
            .with_dimensions(x, y)
            .build_glium()
            .unwrap();

        let renderer = GLRenderer::new(&mut display);

        Controller {
            display: display,
            renderer: renderer,
            player: SDLPlayer::new()
        }
    }

    pub fn refresh(&mut self, emulator: &mut Emulator) {
        {
            let pixels = emulator.cpu.handler_holder.get_screen_buffer();

            let mut frame = self.display.draw();
            self.renderer.refresh(&mut frame, pixels);
            frame.finish().unwrap();
        }

        {
            let sound_buffer = emulator.cpu.handler_holder.get_audio_buffer();
            self.player.refresh(sound_buffer);
        }
    }

    pub fn check_events(&mut self, emulator: &mut Emulator) -> Event {
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
                            },
                            Some(VirtualKeyCode::F2) => {
                                if state == ElementState::Pressed {
                                    return Event::ToggleSpeed;
                                }
                            },
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
                            ElementState::Pressed => { emulator.cpu.key_down(k) },
                            ElementState::Released => { emulator.cpu.key_up(k) },
                        }
                        emulator.cpu.interrupt(Interrupt::Joypad);
                    };
                },
                _ => {}
            }
        };

        Event::Continue
    }
}
