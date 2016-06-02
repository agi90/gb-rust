extern crate ncurses;
extern crate nalgebra;

use gb_proc::video_controller::GrayShade;
use glium::{DrawParameters, DisplayBuild, Surface, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::{ ClientFormat, MipmapsOption, PixelValue, TextureCreationError, UncompressedFloatFormat };
use glium::texture::texture2d::Texture2d;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use glium::backend::glutin_backend::GlutinFacade;
use self::nalgebra::{Mat4, Vec4, Diag};
use glium;

const COLOR_BLACK: i16 = 232;
const COLOR_DARK_GRAY: i16 = 239;
const COLOR_LIGHT_GRAY: i16 = 250;
const COLOR_WHITE: i16 = 255;

const COLOR_PAIR_BLACK: i16 = 4;
const COLOR_PAIR_DARK_GRAY: i16 = 5;
const COLOR_PAIR_LIGHT_GRAY: i16 = 6;
const COLOR_PAIR_WHITE: i16 = 7;

const TEXTURE_WIDTH: u32 = 256;
const TEXTURE_HEIGHT: u32 = 256;
const TEX_OFFSET_X: f32 = 160 as f32 / TEXTURE_WIDTH as f32;
const TEX_OFFSET_Y: f32 = 144 as f32 / TEXTURE_HEIGHT as f32;

pub trait Renderer {
    fn print_pixel(&mut self, pixel: GrayShade, x: i32, y: i32);
    fn refresh(&mut self);
}

pub struct NCursesRenderer; impl NCursesRenderer {
    pub fn new() -> NCursesRenderer {
        ncurses::initscr();
        ncurses::start_color();
        ncurses::init_pair(COLOR_PAIR_BLACK, COLOR_BLACK, COLOR_BLACK);
        ncurses::init_pair(COLOR_PAIR_DARK_GRAY, COLOR_DARK_GRAY, COLOR_DARK_GRAY);
        ncurses::init_pair(COLOR_PAIR_LIGHT_GRAY, COLOR_LIGHT_GRAY, COLOR_LIGHT_GRAY);
        ncurses::init_pair(COLOR_PAIR_WHITE, COLOR_WHITE, COLOR_WHITE);

        println!("Inizializing video engine...");
        NCursesRenderer
    }
}

impl Renderer for NCursesRenderer {
    fn print_pixel(&mut self, pixel: GrayShade, x: i32, y: i32) {
        let color = match pixel {
            GrayShade::C11 => ncurses::COLOR_PAIR(COLOR_PAIR_BLACK),
            GrayShade::C10 => ncurses::COLOR_PAIR(COLOR_PAIR_DARK_GRAY),
            GrayShade::C01 => ncurses::COLOR_PAIR(COLOR_PAIR_LIGHT_GRAY),
            GrayShade::C00 => ncurses::COLOR_PAIR(COLOR_PAIR_WHITE),
        };

        ncurses::attron(color);
        ncurses::mvprintw(y as i32, x as i32, " ");
        ncurses::attroff(color);
    }

    fn refresh(&mut self) {
        ncurses::refresh();
    }
}

pub struct GLRenderer {
    buffer: PixelBuffer<u8>,
    texture: Texture2d,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: glium::Program,
    matrix: Mat4<f32>,
    palette: Mat4<f32>,
    display: GlutinFacade,
    screen_buffer: [u8; 160 * 144],
}

#[derive(Copy, Clone)]
pub struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, tex_coords);

impl GLRenderer {
    pub fn new() -> GLRenderer {
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(640, 576)
            .build_glium()
            .unwrap();

        let vertexes = [
            Vertex { position: [-1.0, -1.0], tex_coords: [0.0,          TEX_OFFSET_Y] },
            Vertex { position: [-1.0,  1.0], tex_coords: [0.0,          0.0] },
            Vertex { position: [ 1.0,  1.0], tex_coords: [TEX_OFFSET_X, 0.0] },
            Vertex { position: [ 1.0, -1.0], tex_coords: [TEX_OFFSET_X, TEX_OFFSET_Y] }
        ];

        let vertex_buffer = VertexBuffer::immutable(&display, &vertexes).unwrap();

        let index_buffer = (IndexBuffer::immutable(
                &display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3])).unwrap();


        let vertex_shader_src = r#"
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec2 tex_coords;

            varying vec2 v_tex_coords;

            void main() {
              gl_Position = matrix * vec4(position, 0.0, 1.0);
              v_tex_coords = tex_coords;
            }
        "#;

        let fragment_shader_src = r#"
            #version 110

            uniform sampler2D tex;
            uniform mat4 palette;

            varying vec2 v_tex_coords;

            void main() {
              float color = texture2D(tex, v_tex_coords).x;
              gl_FragColor = palette[int(color * 255.0 + 0.5)];
            }
        "#;

        let program = glium::Program::from_source(
            &display,
            vertex_shader_src,
            fragment_shader_src,
            None).unwrap();

        let pixel_buffer = PixelBuffer::new_empty(&display, 160 * 144);

        let mut texture = Texture2d::empty_with_format(&display,
                                                       UncompressedFloatFormat::U8,
                                                       MipmapsOption::NoMipmap,
                                                       TEXTURE_WIDTH, TEXTURE_HEIGHT).unwrap();

        let matrix = Mat4::from_diag(&Vec4::new(1.0, 1.0, 1.0, 1.0));

        let palette = Mat4::new(255.0, 181.0, 107.0, 33.0,
                                247.0, 174.0, 105.0, 32.0,
                                123.0, 74.0,  49.0,  16.0,
                                1.0,   1.0,   1.0,   1.0) / 255.0;

        GLRenderer {
            buffer: pixel_buffer,
            texture: texture,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            program: program,
            matrix: matrix,
            palette: palette,
            screen_buffer: [4; 160 * 144],
            display: display,
        }
    }

    fn update_pixels(&mut self) {
      self.texture.main_level().raw_upload_from_pixel_buffer(
        self.buffer.as_slice(), 0..160, 0..144, 0 .. 1);
    }
}

impl Renderer for GLRenderer {
    fn print_pixel(&mut self, pixel: GrayShade, x: i32, y: i32) {
        if x < 0 || y < 1 || x > 159 || y > 144 {
            return;
        }

        let color = match pixel {
            GrayShade::C11 => 0x04,
            GrayShade::C10 => 0x03,
            GrayShade::C01 => 0x02,
            GrayShade::C00 => 0x01,
        };

        self.screen_buffer[(y as usize - 1) * 160 + x as usize] = color;
    }

    fn refresh(&mut self) {
        self.buffer.write(&self.screen_buffer);
        self.update_pixels();

        let uniforms = uniform! {
            matrix: self.matrix.as_ref().clone(),
            palette: self.palette.as_ref().clone(),
            tex: self.texture.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest)
        };

        let params = DrawParameters {
            .. Default::default()
        };

        let mut frame = self.display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &params).unwrap();
        frame.finish().unwrap();
    }
}
