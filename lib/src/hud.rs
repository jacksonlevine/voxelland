
use std::sync::*;
use parking_lot::{Mutex, RwLock};
use atomic::{AtomicI32, AtomicI8};
use gl::types::{GLuint, GLvoid};
use glam::Vec2;
use glfw::PWindow;

use crate::game::ROWLENGTH;
use crate::shader::Shader;
use crate::textureface::{TextureFace, ONE_OVER_16};
use crate::vec::{self, IVec3};
use crate::windowandkey::{WINDOWHEIGHT, WINDOWWIDTH};
use crate::{game, windowandkey};

#[derive(Clone)]
pub enum SlotIndexType {
    ChestSlot(i32),
    InvSlot(i32),
    None
}

#[derive(Clone)]
pub struct HudElement {
    pub normalized_pos: Vec2,
    pub size: Vec2,
    pub uvs: [f32; 12],
    pub ass_slot: SlotIndexType,
    pub translation: Vec2
}

impl HudElement {
    pub fn new(pos: Vec2, size: Vec2, uvs: [f32; 12], ass_slot: SlotIndexType) -> HudElement {
        HudElement {
            normalized_pos: pos,
            size,
            uvs,
            ass_slot,
            translation: Vec2::ZERO
        }
    }

    pub fn overlaps(&self, x: f64, y: f64) -> bool {
        unsafe {
            let xnorm = x / windowandkey::WINDOWWIDTH as f64;
            let ynorm = y / windowandkey::WINDOWHEIGHT as f64;

            let ndcx = 2.0 * xnorm - 1.0;
            let ndcy = 1.0 - 2.0 * ynorm;

            if ndcx >= self.normalized_pos.x as f64 - (self.size.x as f64 / 2.0) && ndcx <= self.normalized_pos.x as f64 + (self.size.x as f64 / 2.0)  {
                if ndcy <= self.normalized_pos.y as f64 + (self.size.y as f64 / 2.0) && ndcy >= self.normalized_pos.y as f64 - (self.size.y as f64 / 2.0) {
                    return true;
                }
            }
            return false;
        }
        
    }

    pub fn xytondc(x: f64, y: f64) -> Vec2 {
        unsafe {
            let xnorm = x / windowandkey::WINDOWWIDTH as f64;
            let ynorm = y / windowandkey::WINDOWHEIGHT as f64;

            let ndcx = 2.0 * xnorm - 1.0;
            let ndcy = 1.0 - 2.0 * ynorm;

            return Vec2::new(ndcx as f32, ndcy as f32);
        }
        
    }

    pub fn element_ass_slot_to_shader_float(&self) -> f32 {
        return match self.ass_slot {
            SlotIndexType::ChestSlot(n) => {
                n + 60
            },
            SlotIndexType::InvSlot(n) => {
                n + 1
            },
            SlotIndexType::None => {
                0
            },
        } as f32
    }

    pub fn ass_slot_to_shader_float(slot: &SlotIndexType) -> f32 {
        return match slot {
            SlotIndexType::ChestSlot(n) => {
                n + 60
            },
            SlotIndexType::InvSlot(n) => {
                n + 1
            },
            SlotIndexType::None => {
                0
            },
        } as f32
    }
}


pub struct Hud {
    pub vbo: GLuint,
    pub chestvbo: GLuint,

    pub healthvbo: GLuint,
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
    pub healthvao: GLuint,
    pub chestdirty: bool,
    pub highlightedslot: SlotIndexType,
    pub mousetrans: Vec2,
    pub health: Arc<AtomicI8>,
    pub stamina: Arc<AtomicI32>
}

impl Hud {
    pub fn new(window: &Arc<RwLock<PWindow>>, texture: GLuint, health: Arc<AtomicI8>, stamina: Arc<AtomicI32>) -> Hud {
        let mut vbo: GLuint = 0;
        let mut chestvbo: GLuint = 0;
        let mut healthvbo: GLuint = 0;
        let shader = Shader::new(path!("assets/menuvert.glsl"), path!("assets/menufrag.glsl"));
        let mut chestvao: GLuint = 0;
        let mut healthvao: GLuint = 0;
        #[cfg(feature = "glfw")]
        unsafe {
            gl::BindVertexArray(shader.vao);
            gl::CreateVertexArrays(1, &mut chestvao);
            gl::CreateVertexArrays(1, &mut healthvao);
            gl::CreateBuffers(1, &mut vbo);
            gl::CreateBuffers(1, &mut chestvbo);
            gl::CreateBuffers(1, &mut healthvbo);
            gl::BindTextureUnit(0, texture);
        }
        Hud {
            vbo,
            chestvbo,
            healthvbo,
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
            healthvao,
            chestdirty: false,
            highlightedslot: SlotIndexType::None,
            mousetrans: Vec2::ZERO,
            health: health.clone(),
            stamina
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
                    let mut xoff = 0.0;
                    if bumped_slot != -1 
                    {
                        if bumped_slot as usize == index || bumped_slot as usize + ROWLENGTH as usize == index {
                            if bumped_slot as usize == index {
                                xoff = ONE_OVER_16;
                            }
                                realpos += Vec2::new(0.0, 0.03);
                            
                        }
                    }

                    let bl = realpos - (realsize*0.5);
                    let br = realpos - (realsize*0.5) + Vec2::new(realsize.x, 0.0);
                    let tr = realpos + (realsize*0.5);
                    let tl: Vec2 = realpos + (realsize*0.5) - Vec2::new(realsize.x, 0.0);

                    let element_id = element.element_ass_slot_to_shader_float();
                    //info!("Putting e id {}", element_id);

                   

                    allgeo.extend_from_slice(&[
                        bl.x, bl.y, element.uvs[0] + xoff, element.uvs[1], element_id,
                        br.x, br.y, element.uvs[2] + xoff, element.uvs[3], element_id,
                        tr.x, tr.y, element.uvs[4] + xoff, element.uvs[5], element_id,

                        tr.x, tr.y, element.uvs[6] + xoff, element.uvs[7], element_id,
                        tl.x, tl.y, element.uvs[8] + xoff, element.uvs[9], element_id,
                        bl.x, bl.y, element.uvs[10] + xoff, element.uvs[11], element_id,
                    ]);
                }
                
                #[cfg(feature = "glfw")]
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

            let winsize = self.window.read().get_size();
            self.count = bindthisgeo( vbo, &elements1, vao1, self.bumped_slot as i32, winsize);
            self.chestcount = bindthisgeo( chestvbo, &elements2, vao2, -1, winsize);
            self.dirty = false;
        }
    }
    pub fn set_current_chest(&mut self, newspot: vec::IVec3) {
        self.current_chest = newspot;
        self.chestdirty = true;
    }

    pub fn draw_health(&self) {

        static mut LASTHEALTH: i8 = -99;
        static mut LASTSTAMINA: i32 = -999;

        let redface   = TextureFace::new(0, 5);
        let blackface = TextureFace::new(0, 6);

        let wwf = unsafe { WINDOWWIDTH } as f32 / 100.0;
        #[cfg(feature = "glfw")]
        unsafe {

            let height = (20.0 / WINDOWHEIGHT as f32) as f32;
            let width = ((20.0 * wwf) / WINDOWWIDTH as f32) as f32;

            let ythickness = (5.0 / WINDOWHEIGHT as f32) as f32;
            let xthickness = (5.0 / WINDOWWIDTH as f32) as f32;

            

            gl::BindVertexArray(self.healthvao);

            gl::UseProgram(self.shader.shader_id);
            
            let tex_loc = gl::GetAttribLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
            gl::Uniform1i(tex_loc, 0);

            let moused_slot_loc = gl::GetUniformLocation(self.shader.shader_id, b"mousedSlot\0".as_ptr() as *const i8);
            gl::Uniform1f(moused_slot_loc, HudElement::ass_slot_to_shader_float(&game::MOUSED_SLOT));

            let trans_loc = gl::GetUniformLocation(self.shader.shader_id, b"translation\0".as_ptr() as *const i8);
            gl::Uniform2f(trans_loc, self.mousetrans.x, self.mousetrans.y);


            let h = self.health.load(atomic::Ordering::Relaxed);

            let redwidth = (h as f32 * wwf) / WINDOWWIDTH as f32;

            let stam = self.stamina.load(atomic::Ordering::Relaxed);

            static mut count: usize = 0;

            if h != LASTHEALTH || stam != LASTSTAMINA {

                let startx = -0.25;
                let starty = -0.70;

                let mut allgeo: Vec<f32> /*60 / 5 = 12*/ = vec![
                    startx - xthickness ,                                  starty - ythickness,                                      blackface.blx, blackface.bly, -1.0,  
                    startx - xthickness + width + xthickness + xthickness, starty - ythickness,                                      blackface.brx, blackface.bry, -1.0, 
                    startx - xthickness + width + xthickness + xthickness, starty - ythickness + height  + ythickness + ythickness,  blackface.trx, blackface.tr_y, -1.0, 

                    startx - xthickness + width + xthickness + xthickness, starty - ythickness + height  + ythickness + ythickness,  blackface.trx, blackface.tr_y, -1.0, 
                    startx - xthickness ,                                  starty - ythickness + height  + ythickness + ythickness,  blackface.tlx, blackface.tly, -1.0, 
                    startx - xthickness ,                                  starty - ythickness,                                     blackface.blx, blackface.bly, -1.0,


                    startx,                      starty,                         redface.blx, redface.bly, -1.0,  
                    startx + redwidth,             starty,                         redface.brx, redface.bry, -1.0, 
                    startx + redwidth,             starty + height,                redface.trx, redface.tr_y, -1.0, 

                    startx + redwidth,              starty + height,                redface.trx, redface.tr_y, -1.0, 
                    startx ,                     starty + height,                redface.tlx, redface.tly, -1.0, 
                    startx ,                     starty,                         redface.blx, redface.bly, -1.0,
                ];


                let redwidth = ((stam as f32 * 0.2) * wwf) / WINDOWWIDTH as f32;
                let redface   = TextureFace::new(0, 7);
                let startx = 0.05;

                allgeo.extend_from_slice(&[
                    startx - xthickness ,                                  starty - ythickness,                                      blackface.blx, blackface.bly, -1.0,  
                    startx - xthickness + width + xthickness + xthickness, starty - ythickness,                                      blackface.brx, blackface.bry, -1.0, 
                    startx - xthickness + width + xthickness + xthickness, starty - ythickness + height  + ythickness + ythickness,  blackface.trx, blackface.tr_y, -1.0, 

                    startx - xthickness + width + xthickness + xthickness, starty - ythickness + height  + ythickness + ythickness,  blackface.trx, blackface.tr_y, -1.0, 
                    startx - xthickness ,                                  starty - ythickness + height  + ythickness + ythickness,  blackface.tlx, blackface.tly, -1.0, 
                    startx - xthickness ,                                  starty - ythickness,                                     blackface.blx, blackface.bly, -1.0,


                    startx,                      starty,                         redface.blx, redface.bly, -1.0,  
                    startx + redwidth,             starty,                         redface.brx, redface.bry, -1.0, 
                    startx + redwidth,             starty + height,                redface.trx, redface.tr_y, -1.0, 

                    startx + redwidth,              starty + height,                redface.trx, redface.tr_y, -1.0, 
                    startx ,                     starty + height,                redface.tlx, redface.tly, -1.0, 
                    startx ,                     starty,                         redface.blx, redface.bly, -1.0,
                ]);

                unsafe {
                    count = allgeo.len();
                }


                let vao = self.healthvao;
                let vbo = self.healthvbo;


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


                LASTHEALTH = h;
            }

            gl::DrawArrays(gl::TRIANGLES, 0, count as i32 / 5);
        }
        
    }
    #[cfg(feature = "glfw")]
    pub fn draw(&self) {
        unsafe {

            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::BindVertexArray(self.shader.vao);
            gl::UseProgram(self.shader.shader_id);

            

            let tex_loc = gl::GetAttribLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
            gl::Uniform1i(tex_loc, 0);

            let moused_slot_loc = gl::GetUniformLocation(self.shader.shader_id, b"mousedSlot\0".as_ptr() as *const i8);
            gl::Uniform1f(moused_slot_loc, HudElement::ass_slot_to_shader_float(&game::MOUSED_SLOT));

            let trans_loc = gl::GetUniformLocation(self.shader.shader_id, b"translation\0".as_ptr() as *const i8);
            gl::Uniform2f(trans_loc, self.mousetrans.x, self.mousetrans.y);


            gl::DrawArrays(gl::TRIANGLES, 0, self.count);


            if self.chest_open {
                gl::BindVertexArray(self.chestvao);
                gl::UseProgram(self.shader.shader_id);
                let tex_loc = gl::GetAttribLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
                gl::Uniform1i(tex_loc, 0);
                
                let moused_slot_loc = gl::GetUniformLocation(self.shader.shader_id, b"mousedSlot\0".as_ptr() as *const i8);

                gl::Uniform1f(moused_slot_loc, HudElement::ass_slot_to_shader_float(&game::MOUSED_SLOT));
                gl::DrawArrays(gl::TRIANGLES, 0, self.chestcount);
            }

            // if self.inv_open {
            //     gl::BindVertexArray(self.invvao);
            //     gl::UseProgram(self.shader.shader_id);
            //     let tex_loc = gl::GetAttribLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
            //     gl::Uniform1i(tex_loc, 0);
                
            //     let moused_slot_loc = gl::GetUniformLocation(self.shader.shader_id, b"mousedSlot\0".as_ptr() as *const i8);

            //     gl::Uniform1f(moused_slot_loc, HudElement::ass_slot_to_shader_float(&game::MOUSED_SLOT));
            //     gl::DrawArrays(gl::TRIANGLES, 0, self.chestcount);
            // }

            self.draw_health();
            

            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}
