use std::str::FromStr;

use gl::types::{GLuint, GLvoid};
use glam::Vec2;
use glfw::PWindow;

use crate::glyphface::GlyphFace;




pub struct Text {
    vao: GLuint,
    str: String,
    laststr: String,
    vbo: GLuint,
    shader: GLuint,
    count: i32,
}

impl Text {
    pub fn new(str: &'static str, window: &PWindow, shader: GLuint, pos: Vec2, texture: GLuint) -> Text {
        unsafe {
            let mut vao: GLuint = 0;
            let mut vbo: GLuint = 0;
            
            gl::CreateVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindTextureUnit(0, texture);

            let mut text = Text {
                vao,
                str: String::from_str(str).unwrap(),
                laststr: String::new(),
                vbo,
                shader,
                count: 0
            };

            text.update_geo(str, window, pos.x as f64, pos.y as f64);
            text
        }
    }

    pub fn update_geo(&mut self, newtext: &'static str, window: &PWindow, xpos: f64, ypos: f64) {

        self.str = String::from(newtext);

        let (window_width, window_height) = window.get_size();

        let normalized_x = xpos / window_width as f64;
        let normalized_y = 1.0 - (ypos / window_height as f64);

        let realx = (normalized_x * 2.0 - 1.0) as f32;
        let realy = (normalized_y * 2.0 - 1.0) as f32;

        let gwidth = 32.0/window_width as f32;
        let gheight = 32.0/window_height as f32;

        let mut letters_geometry: Vec<f32> = Vec::new();
        let mut g = GlyphFace::new(0);
        self.count = self.str.len() as i32 * 6;

        for i in 0..self.str.len() {
            g.set_char(self.str.as_bytes()[i]);
            
            letters_geometry.extend_from_slice(&[
                i as f32 * gwidth + realx,          realy,            g.blx, g.bly,  -1.0,
                i as f32 * gwidth + realx,          realy + gheight,  g.tlx, g.tly,  -1.0,
                i as f32 * gwidth + realx + gwidth, realy + gheight,  g.trx, g.tr_y, -1.0,

                i as f32 * gwidth + realx + gwidth, realy + gheight,  g.trx, g.tr_y, -1.0,
                i as f32 * gwidth + realx + gwidth, realy,            g.brx, g.bry,  -1.0,
                i as f32 * gwidth + realx,          realy,            g.blx, g.bly,  -1.0,
            ]);
        }
        unsafe {
            gl::NamedBufferData(self.vbo, (letters_geometry.len() * std::mem::size_of::<f32>()) as isize, letters_geometry.as_ptr() as *const GLvoid, gl::STATIC_DRAW);

                gl::VertexArrayVertexBuffer(self.vao, 0, self.vbo, 0, (5 * std::mem::size_of::<f32>()) as i32);
                gl::EnableVertexArrayAttrib(self.vao, 0);
                gl::VertexArrayAttribFormat(self.vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribBinding(self.vao, 0, 0);

                gl::EnableVertexArrayAttrib(self.vao, 1);
                gl::VertexArrayAttribFormat(self.vao, 1, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<f32>() as u32);
                gl::VertexArrayAttribBinding(self.vao, 1, 0);

                gl::EnableVertexArrayAttrib(self.vao, 2);
                gl::VertexArrayAttribFormat(self.vao, 2, 1, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as u32);
                gl::VertexArrayAttribBinding(self.vao, 2, 0);

            
        }
    }

    pub fn draw(&self) {
        unsafe {
            
            gl::BindVertexArray(self.vao);
            gl::UseProgram(self.shader);

            let tex_loc = gl::GetAttribLocation(self.shader, b"ourTexture\0".as_ptr() as *const i8);
            gl::Uniform1i(tex_loc, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, self.count);
        }
    }
}