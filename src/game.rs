use std::sync::{Mutex, Arc};

use crate::chunk::ChunkSystem;
use crate::worldgeometry::WorldGeometry; 
use crate::shader::Shader;

pub struct Game {
    chunksys: ChunkSystem,
    shader0: Shader,
}


impl Game {

    pub fn new() -> Game {
        let shader0 = Shader::new("assets/vert.glsl", "assets/frag.glsl");
        Game {
            chunksys: ChunkSystem::new(8),
            shader0
        }
    }

    pub fn update(&mut self) {
        self.draw();
    }

    pub fn draw(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::BindVertexArray(self.shader0.vao);
        }
        match self.chunksys.geoqueue.pop() {
            Some(index) => {
                WorldGeometry::bind_geometry(
                    self.chunksys.geobank[index].vbo32,
                    self.chunksys.geobank[index].vbo8, true,
                    &self.shader0, &self.chunksys.geobank[index]);
            },
            None => {

            }
        }
        static mut c_pos_loc: i32 = 0;
        unsafe {
            if c_pos_loc == 0 {
                c_pos_loc = gl::GetUniformLocation(self.shader0.shader_id, b"chunkpos\0".as_ptr() as *const i8);
            }
        }
        for cf in &self.chunksys.chunks {
            if cf.used {
                let cgeo = &self.chunksys.geobank[cf.geo_index];
                WorldGeometry::bind_geometry(cgeo.vbo32, cgeo.vbo8, false, &self.shader0, cgeo);
                unsafe {
                    gl::Uniform2f(c_pos_loc, cgeo.pos.x as f32, cgeo.pos.y as f32);
                    gl::DrawArrays(gl::TRIANGLES, 0, cgeo.data32.len() as i32);
                }
            }
        }
    }
}