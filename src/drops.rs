use std::{ops::Bound, sync::*};

use gl::types::{GLuint, GLvoid};
use glam::{Mat4, Vec3};
use glfw::ffi::glfwGetTime;
use vox_format::chunk::Chunk;

use crate::{camera::Camera, chunk::ChunkSystem, collisioncage::{BoundBox, CollCage, Side}, shader::Shader, vec};





pub struct Drop {
    position: Vec3,
    block_id: u32,
    coll_cage: CollCage,
    grounded: bool,
    time_falling_scalar: f32,
    velocity: Vec3,
    bound_box: BoundBox,
    to_be_deleted: bool
}

impl Drop {
    pub fn new(block_id: u32, position: Vec3, csys: &Arc<ChunkSystem>) -> Drop {

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool> = {
            let csys_arc = Arc::clone(&csys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.collision_predicate(v);
            })
        };

        Drop {
            position,
            block_id,
            coll_cage: CollCage::new(solid_pred),
            grounded: false,
            time_falling_scalar: 1.0,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            bound_box: BoundBox::new(position),
            to_be_deleted: false
        }
    }
}




pub struct Drops {
    pub shader: Shader,
    pub drops: Vec<Drop>,
    pub texture: GLuint,
    pub cam: Arc<Mutex<Camera>>,
    pub csys: Arc<ChunkSystem>
}

impl Drops {

    pub fn new(texture: GLuint, cam: &Arc<Mutex<Camera>>, csys: &Arc<ChunkSystem>) -> Drops {

        let shader = Shader::new("assets/dropvert.glsl", "assets/dropfrag.glsl");
        let mut vbo: GLuint = 0;
        unsafe {
            gl::BindVertexArray(shader.vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindTextureUnit(0, texture);
            let faces: [f32; 144] = [

            
            -0.5, -0.5, 0.5, 0.0,
            -0.5, -0.5, -0.5, 1.0,
            -0.5, 0.5, -0.5, 2.0,
    
            -0.5, 0.5, -0.5, 2.0,
            -0.5, 0.5, 0.5, 3.0,
            -0.5, -0.5, 0.5, 0.0,
    
                    0.5, -0.5, -0.5, 0.0,
                    0.5, -0.5, 0.5, 1.0,
                    0.5, 0.5, 0.5,2.0,
    
                    0.5, 0.5, 0.5, 2.0,
                    0.5, 0.5, -0.5, 3.0,
                    0.5, -0.5, -0.5,0.0,
    
            0.5, -0.5, 0.5, 0.0,
            -0.5, -0.5, 0.5,1.0,
            -0.5, 0.5, 0.5, 2.0,
    
            -0.5, 0.5, 0.5, 2.0,
            0.5, 0.5, 0.5,  3.0,
            0.5, -0.5, 0.5, 0.0,
    
                    -0.5, -0.5, -0.5, 0.0,
                    0.5, -0.5, -0.5,1.0,
                    0.5, 0.5, -0.5,2.0,
    
                    0.5, 0.5, -0.5, 2.0,
                    -0.5, 0.5, -0.5, 3.0,
                    -0.5, -0.5, -0.5,0.0,
    
            -0.5, 0.5, -0.5, 0.0,
            0.5, 0.5, -0.5,1.0,
            0.5, 0.5, 0.5,2.0,
    
            0.5, 0.5, 0.5, 2.0,
            -0.5, 0.5, 0.5, 3.0,
            -0.5, 0.5, -0.5,0.0,
    
                    0.5, -0.5, -0.5, 0.0,
                    -0.5, -0.5, -0.5,1.0,
                    -0.5, -0.5, 0.5,2.0,
    
                    -0.5, -0.5, 0.5, 2.0,
                    0.5, -0.5, 0.5, 3.0,
                    0.5, -0.5, -0.5,0.0,
        
            ];

            gl::NamedBufferData(vbo, (faces.len() * std::mem::size_of::<f32>()) as isize, faces.as_ptr() as *const GLvoid, gl::STATIC_DRAW);
            
            gl::VertexArrayVertexBuffer(shader.vao, 0, vbo, 0, (4 * std::mem::size_of::<f32>()) as i32);
            gl::EnableVertexArrayAttrib(shader.vao, 0);
            gl::VertexArrayAttribFormat(shader.vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(shader.vao, 0, 0);

            gl::EnableVertexArrayAttrib(shader.vao, 1);
            gl::VertexArrayAttribFormat(shader.vao, 1, 1, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as u32);
            gl::VertexArrayAttribBinding(shader.vao, 1, 0);

        }
        Drops {
            shader,
            drops: Vec::new(),
            texture,
            cam: cam.clone(),
            csys: csys.clone()
        }
    }

    pub fn add_drop(&mut self, pos: Vec3, block_id: u32) {
        let drop = Drop::new(block_id, pos, &self.csys);
        self.drops.push(drop);
    }
    pub fn update_and_draw_drops(&mut self, delta_time: &f32, mvp: &Mat4) {
        self.update_drops(delta_time);
        self.draw_drops(mvp);
    }

    pub fn draw_drops(&self, mvp: &Mat4) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::BindVertexArray(self.shader.vao);
            gl::UseProgram(self.shader.shader_id);
            //println!("Drawing {} drops", self.drops.len());
            for drop in &self.drops {
                let pos_loc = gl::GetUniformLocation(self.shader.shader_id, b"pos\0".as_ptr() as *const i8);
                let time_loc = gl::GetUniformLocation(self.shader.shader_id, b"time\0".as_ptr() as *const i8);
                let blockid_loc = gl::GetUniformLocation(self.shader.shader_id, b"blockID\0".as_ptr() as *const i8);
                let tex_loc = gl::GetUniformLocation(self.shader.shader_id, b"ourTexture\0".as_ptr() as *const i8);
                //println!("Drop at {} {} {}", drop.position.x, drop.position.y, drop.position.z);
                let mvp_loc = gl::GetUniformLocation(self.shader.shader_id, b"mvp\0".as_ptr() as *const i8);

                gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, mvp.to_cols_array().as_ptr());

                gl::Uniform3f(pos_loc, drop.position.x, drop.position.y, drop.position.z);
                gl::Uniform1f(time_loc, glfwGetTime() as f32);
                gl::Uniform1f(blockid_loc, drop.block_id as f32);
                gl::Uniform1i(tex_loc, 0);
                gl::DrawArrays(gl::TRIANGLES, 0, 144/4);
            }
            gl::Enable(gl::CULL_FACE);
        }
        
    }

    pub fn update_drops(&mut self, delta_time: &f32) {
        let mut to_remove_indices = Vec::new();
        for (index, drop) in self.drops.iter_mut().enumerate() {
            if !drop.coll_cage.solid.contains(&Side::FLOOR) {
                drop.grounded = false;
            }

            const GRAV: f32 = 9.8;

            if !drop.grounded  {
                drop.time_falling_scalar = (drop.time_falling_scalar + delta_time * 5.0).min(3.0);
            } else {
                drop.time_falling_scalar = 1.0;
            }
    
            if !drop.grounded {
                drop.velocity +=
                    Vec3::new(0.0, -GRAV * drop.time_falling_scalar * delta_time, 0.0);
            }
    


            let cc_center = drop.position;
            drop.coll_cage.update_readings(cc_center);
            
            let campos = self.cam.lock().unwrap().position - Vec3::new(0.0, 1.0, 0.0);
            if (drop.position).distance(campos) < 4.0 {
                let diff = campos - drop.position;

                let diffmag = campos.distance(drop.position);

                let diffnorm = diff.normalize();

                let pull = diffnorm * (1.0 - (diffmag / 4.0));

                drop.velocity += pull * *delta_time * 10.0;
            }

            if (drop.position).distance(campos) < 1.0 {
                to_remove_indices.push(index);
            }
            
            let mut proposed = if drop.velocity.length() > 0.0 {
                let amt_to_subtract = drop.velocity * *delta_time * 5.0;
                drop.velocity -= amt_to_subtract;
    
                drop.position + amt_to_subtract
            } else {
                drop.position
            };

            drop.bound_box
                .set_center(proposed, 0.2, 0.2);

            drop.coll_cage.update_colliding(&drop.bound_box);

            let mut corr_made: Vec<Vec3> = Vec::new();
            if drop.coll_cage.colliding.len() > 0 {
                for side in &drop.coll_cage.colliding {
                    if !corr_made.contains(&drop.coll_cage.normals[*side as usize]) {
                        proposed += drop.coll_cage.normals[*side as usize]
                            * drop.coll_cage.penetrations[*side as usize];
                        corr_made.push(drop.coll_cage.normals[*side as usize]);
                    }
                    if *side == Side::FLOOR {
                        drop.grounded = true;
                    }
                    if *side == Side::ROOF {

                        drop.grounded = false;
                    }
                }
            }
            drop.position = proposed;
        }
        for &index in to_remove_indices.iter().rev() {
            //just delete for now, add toinventory later
            self.drops.remove(index);
        }
    }




}

