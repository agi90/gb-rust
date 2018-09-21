use sound::SDLPlayer;
use glium::glutin::{
    ContextBuilder,
    ElementState,
    EventsLoop,
    VirtualKeyCode,
};
use glium::glutin::dpi::LogicalSize;
use glium::backend::glutin::Display;

use gb::{
    Emulator,
    Hardware,
    Interrupt,
    Key,
};

use gpu::renderer::GLRenderer;

use glium;
use glium::glutin;

pub struct Controller {
    display: Display,
    renderer: GLRenderer,
    player: SDLPlayer,
    events_loop: EventsLoop,
}

#[derive(Debug)]
pub enum Event {
    Quit,
    Break,
    ToggleSpeed,
    Continue,
}

impl Controller {
    pub fn new(x: f64, y: f64) -> Controller {
        let events_loop = EventsLoop::new();
        let window_builder = glium::glutin::WindowBuilder::new()
            .with_title("gb-rust")
            .with_dimensions(LogicalSize::new(x, y));
        let mut display = Display::new(window_builder,
                ContextBuilder::new(),
                &events_loop).unwrap();
        let renderer = GLRenderer::new(&mut display);

        Controller {
            display: display,
            renderer: renderer,
            player: SDLPlayer::new(),
            events_loop: events_loop,
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

    fn handle_event(event: glutin::Event, emulator: &mut Emulator) -> Event {
        match event {
            glutin::Event::WindowEvent{window_id: _, event: window_event} => {
                match window_event {
                    glutin::WindowEvent::CloseRequested =>
                        return Event::Quit,
                    glutin::WindowEvent::KeyboardInput{device_id: _, input: keyboard_input} => {
                        if keyboard_input.state == ElementState::Pressed {
                            match keyboard_input.virtual_keycode {
                                Some(VirtualKeyCode::F1) => {
                                    return Event::Break;
                                },
                                Some(VirtualKeyCode::F2) => {
                                    return Event::ToggleSpeed;
                                },
                                _ => {},
                            }
                        }

                        let key = keyboard_input.virtual_keycode.and_then(|k| match k {
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
                            match keyboard_input.state {
                                ElementState::Pressed => { emulator.cpu.key_down(k) },
                                ElementState::Released => { emulator.cpu.key_up(k) },
                            }
                            emulator.cpu.interrupt(Interrupt::Joypad);
                        };
                    },
                    _ => {},
                }
            },
            glutin::Event::DeviceEvent{device_id: _, event: device_event} => {
                match device_event {
                    _ => {}
                };
            },
            _ => {}
        }

        Event::Continue
    }

    pub fn check_events(&mut self, emulator: &mut Emulator) -> Event {
        let mut event = Event::Continue;
        self.events_loop.poll_events(|glutin_event| {
            event = Self::handle_event(glutin_event, emulator);
        });

        event
    }
}
