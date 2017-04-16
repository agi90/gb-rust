extern crate nalgebra;

use self::nalgebra::{Mat4, Vec4, Diag};

use glium;
use glium::{DrawParameters, Surface, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::{ MipmapsOption, UncompressedFloatFormat };
use glium::texture::texture2d::Texture2d;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use glium::backend::glutin_backend::GlutinFacade;

use gb::ScreenBuffer;

const TEXTURE_WIDTH: u32 = 256;
const TEXTURE_HEIGHT: u32 = 256;
const TEX_OFFSET_X: f32 = 160 as f32 / TEXTURE_WIDTH as f32;
const TEX_OFFSET_Y: f32 = 144 as f32 / TEXTURE_HEIGHT as f32;

pub trait Renderer {
    fn refresh(&mut self, pixels: &ScreenBuffer);
}

pub struct GLRenderer {
    buffer: PixelBuffer<u8>,
    texture: Texture2d,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: glium::Program,
    matrix: Mat4<f32>,
    palette: Mat4<f32>,
}

#[derive(Copy, Clone)]
pub struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, tex_coords);

impl GLRenderer {
    pub fn new(display: &mut GlutinFacade) -> GLRenderer {
        let vertexes = [
            Vertex { position: [-1.0, -1.0], tex_coords: [0.0,          TEX_OFFSET_Y] },
            Vertex { position: [-1.0,  1.0], tex_coords: [0.0,          0.0] },
            Vertex { position: [ 1.0,  1.0], tex_coords: [TEX_OFFSET_X, 0.0] },
            Vertex { position: [ 1.0, -1.0], tex_coords: [TEX_OFFSET_X, TEX_OFFSET_Y] }
        ];

        let vertex_buffer = VertexBuffer::immutable(display, &vertexes).unwrap();

        let index_buffer = (IndexBuffer::immutable(
                display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3])).unwrap();

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
            display,
            vertex_shader_src,
            fragment_shader_src,
            None).unwrap();

        let pixel_buffer = PixelBuffer::new_empty(display, 160 * 144);

        let texture = Texture2d::empty_with_format(display,
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
        }
    }

    fn update_pixels(&mut self) {
      self.texture.main_level().raw_upload_from_pixel_buffer(
        self.buffer.as_slice(), 0..160, 0..144, 0 .. 1);
    }

    pub fn refresh(&mut self, frame: &mut glium::Frame, pixels: &ScreenBuffer) {
        let mut pixel_buffer = [0u8; 160 * 144];

        let mut index = 0;
        for y in 0..144 {
            for x in 0..160 {
                pixel_buffer[index] = pixels[y][x] as u8 + 1;
                index += 1;
            }
        }

        self.buffer.write(&pixel_buffer);

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

        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &params).unwrap();
    }
}
