
use std::sync::*;

use gl::types::{GLsizei, GLuint, GLvoid};
use glam::Vec2;
use glfw::PWindow;

use crate::shader::Shader;
use crate::textureface::TextureFace;
use crate::vec::{self, IVec3};
use crate::windowandkey;

pub enum HighlightedSlot {
    ChestSlot(i32),
    InvSlot(i32),
    None
}

#[derive(Clone)]
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

    pub fn overlaps(&self, x: f64, y: f64) -> bool {
        unsafe {
            let xnorm = x / windowandkey::WINDOWWIDTH as f64;
            let ynorm = y / windowandkey::WINDOWHEIGHT as f64;

            let ndcx = 2.0 * xnorm - 1.0;
            let ndcy = 1.0 - 2.0 * ynorm;

            if ndcx >= self.normalized_pos.x as f64 && ndcx <= self.normalized_pos.x as f64 + self.size.x as f64 {
                if ndcy <= self.normalized_pos.y as f64 && ndcy >= self.normalized_pos.y as f64 - self.size.y as f64 {
                    return true;
                }
            }
            return false;
        }
        
    }
}


pub struct Hud {
    pub vbo: GLuint,
    pub chestvbo: GLuint,
    pub shader: Shader,
    pub window: Arc<RwLock<PWindow>>,
    pub dirty: bool,
    pub elements: Vec<HudElement>,
    pub chestelements: Vec<HudElement>,
    pub count: i32,
    pub chestcount: i32,
    pub bumped_slot: usize,

    //Chest inventory stuff
    pub current_chest: vec::IVec3,
    pub chest_open: bool,
    pub chestvao: GLuint,
    pub chestdirty: bool,
    pub highlightedslot: HighlightedSlot
}

impl Hud {
    pub fn new(window: &Arc<RwLock<PWindow>>, texture: GLuint) -> Hud {
        let mut vbo: GLuint = 0;
        let mut chestvbo: GLuint = 0;
        let shader = Shader::new("assets/menuvert.glsl", "assets/menufrag.glsl");
        let mut chestvao: GLuint = 0;
        unsafe {
            gl::BindVertexArray(shader.vao);
            gl::CreateVertexArrays(1, &mut chestvao);
            gl::CreateBuffers(1, &mut vbo);
            gl::CreateBuffers(1, &mut chestvbo);
            gl::BindTextureUnit(0, texture);
        }
        Hud {
            vbo,
            chestvbo,
            shader,
            window: window.clone(),
            dirty: true,
            elements: Vec::new(),
            chestelements: Vec::new(),
            count: 0,
            chestcount: 0,
            bumped_slot: 2,
            current_chest: IVec3::new(0,0,0),
            chest_open: false,
            chestvao,
            chestdirty: false,
            highlightedslot: HighlightedSlot::None
        }
    }
    pub fn update(&mut self) { 
        if self.dirty {

            fn bindthisgeo(vbo: GLuint, elements: &Vec<HudElement>, vao: GLuint, bumped_slot: i32, winsize: (i32, i32)) -> i32 {
                let mut allgeo = Vec::new();
                for (index, element) in elements.iter().enumerate() {

                    let (width,height) = winsize;

                    let realsize = (element.size*800.0) / Vec2::new(width as f32, height as f32);

                    let mut realpos = element.normalized_pos;
                    if bumped_slot != -1 
                    {if bumped_slot as usize == index || bumped_slot as usize + 5 == index {
                        realpos += Vec2::new(0.0, 0.05);
                    } }

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
                

                unsafe {
                    gl::BindVertexArray(vao);
                    gl::NamedBufferData(vbo, (allgeo.len() * std::mem::size_of::<f32>()) as isize, allgeo.as_ptr() as *const GLvoid, gl::STATIC_DRAW);
                    
                    gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, (5 * std::mem::size_of::<f32>()) as i32);
                    gl::EnableVertexArrayAttrib(vao, 0);
                    gl::VertexArrayAttribFormat(vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
                    gl::VertexArrayAttribBinding(vao, 0, 0);

                    gl::EnableVertexArrayAttrib(vao, 1);
                    gl::VertexArrayAttribFormat(vao, 1, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<f32>() as u32);
                    gl::VertexArrayAttribBinding(vao, 1, 0);

                    gl::EnableVertexArrayAttrib(vao, 2);
                    gl::VertexArrayAttribFormat(vao, 2, 1, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as u32);
                    gl::VertexArrayAttribBinding(vao, 2, 0);

                }
                
                (allgeo.len() / 5) as i32
            }

            let vao1 = self.shader.vao.clone();
            let vao2 = self.chestvao.clone();
            let vbo = self.vbo.clone();
            let chestvbo = self.chestvbo.clone();
            let elements1 = self.elements.clone();

            let elements2 = self.chestelements.clone();

            let winsize = self.window.read().unwrap().get_size();
            self.count = bindthisgeo( vbo, &elements1, vao1, self.bumped_slot as i32, winsize);
            self.chestcount = bindthisgeo( chestvbo, &elements2, vao2, -1, winsize);
            self.dirty = false;
        }
    }
    pub fn set_current_chest(&mut self, newspot: vec::IVec3) {
        self.current_chest = newspot;
        self.chestdirty = true;
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
            if self.chest_open {
                gl::BindVertexArray(self.chestvao);
                let tex_loc = gl::GetAttribLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
                gl::Uniform1i(tex_loc, 0);
                gl::DrawArrays(gl::TRIANGLES, 0, self.chestcount);
            }
            

            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}