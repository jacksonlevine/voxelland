
use std::sync::*;

use gl::types::{GLsizei, GLuint, GLvoid};
use glam::Vec2;
use glfw::PWindow;

use crate::shader::Shader;
use crate::textureface::TextureFace;

pub struct HudElement {
    pub normalized_pos: Vec2,
    pub size: Vec2,
    pub uvs: [f32; 12]
}

impl HudElement {
    pub fn new(pos: Vec2, size: Vec2, uvs: [f32; 12]) -> HudElement {
        HudElement {
            normalized_pos: pos,
            size,
            uvs
        }
    }
}


pub struct Hud {
    pub vbo: GLuint,
    pub shader: Shader,
    pub window: Arc<RwLock<PWindow>>,
    pub dirty: bool,
    pub elements: Vec<HudElement>,
    pub count: i32,
    pub bumped_slot: usize
}

impl Hud {
    pub fn new(window: &Arc<RwLock<PWindow>>, texture: GLuint) -> Hud {
        let mut vbo: GLuint = 0;
        let shader = Shader::new("assets/menuvert.glsl", "assets/menufrag.glsl");
        unsafe {
            gl::BindVertexArray(shader.vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindTextureUnit(0, texture);
        }
        Hud {
            vbo,
            shader,
            window: window.clone(),
            dirty: true,
            elements: Vec::new(),
            count: 0,
            bumped_slot: 2
        }
    }
    pub fn update(&mut self) { 
        if self.dirty {
            let mut allgeo = Vec::new();
            for (index, element) in self.elements.iter().enumerate() {

                let (width,height) = self.window.read().unwrap().get_size();
                let realsize = (element.size*800.0) / Vec2::new(width as f32, height as f32);

                let mut realpos = element.normalized_pos;
                if self.bumped_slot == index {
                    realpos += Vec2::new(0.0, 0.05);
                } 

                let bl = realpos - (realsize*0.5);
                let br = realpos - (realsize*0.5) + Vec2::new(realsize.x, 0.0);
                let tr = realpos + (realsize*0.5);
                let tl: Vec2 = realpos + (realsize*0.5) - Vec2::new(realsize.x, 0.0);

                allgeo.extend_from_slice(&[
                    bl.x, bl.y, element.uvs[0], element.uvs[1], -1.0,
                    br.x, br.y, element.uvs[2], element.uvs[3], -1.0,
                    tr.x, tr.y, element.uvs[4], element.uvs[5], -1.0,

                    tr.x, tr.y, element.uvs[6], element.uvs[7], -1.0,
                    tl.x, tl.y, element.uvs[8], element.uvs[9], -1.0,
                    bl.x, bl.y, element.uvs[10], element.uvs[11], -1.0,
                ]);
            }
            self.count = (allgeo.len() / 5) as i32;
            unsafe {
                gl::BindVertexArray(self.shader.vao);
                gl::NamedBufferData(self.vbo, (allgeo.len() * std::mem::size_of::<f32>()) as isize, allgeo.as_ptr() as *const GLvoid, gl::STATIC_DRAW);
                
                gl::VertexArrayVertexBuffer(self.shader.vao, 0, self.vbo, 0, (5 * std::mem::size_of::<f32>()) as i32);
                gl::EnableVertexArrayAttrib(self.shader.vao, 0);
                gl::VertexArrayAttribFormat(self.shader.vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribBinding(self.shader.vao, 0, 0);

                gl::EnableVertexArrayAttrib(self.shader.vao, 1);
                gl::VertexArrayAttribFormat(self.shader.vao, 1, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<f32>() as u32);
                gl::VertexArrayAttribBinding(self.shader.vao, 1, 0);

                gl::EnableVertexArrayAttrib(self.shader.vao, 2);
                gl::VertexArrayAttribFormat(self.shader.vao, 2, 1, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as u32);
                gl::VertexArrayAttribBinding(self.shader.vao, 2, 0);

            }
            self.dirty = false;
        }
    }
    pub fn draw(&self) {
        unsafe {

            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::BindVertexArray(self.shader.vao);
            gl::UseProgram(self.shader.shader_id);

            let tex_loc = gl::GetAttribLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
            gl::Uniform1i(tex_loc, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, self.count);

            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}