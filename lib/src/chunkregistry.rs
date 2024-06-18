use std::fmt::Display;

use gl::types::GLuint;

use crate::{chunk::ChunkGeo, vec};



pub struct ChunkMemory {
    pub used: bool,

    pub vbo8: gl::types::GLuint,
    pub vbo32: gl::types::GLuint, 
    pub tvbo8: gl::types::GLuint,
    pub tvbo32: gl::types::GLuint, 
    pub vvbo: GLuint,
    pub uvvbo: GLuint,

    pub length: i32,
    pub tlength: i32,
    pub vlength: i32,

    pub pos: vec::IVec2
}

impl ChunkMemory {
    pub fn new(geo: &ChunkGeo) -> ChunkMemory {
        let cm = ChunkMemory {
            used: false,
            vbo8: geo.vbo8,
            vbo32: geo.vbo32,
            tvbo8: geo.tvbo8,
            tvbo32: geo.tvbo32,
            vvbo: geo.vvbo,
            uvvbo: geo.uvvbo,
            length: 0,
            tlength: 0,
            vlength: 0,
            pos: *geo.pos.lock().unwrap()
        };
        //println!("ChunkMemory: {} {} {} {} {} {} {} {} {}", cm.used, cm.vbo8, cm.vbo32, cm.tvbo8, cm.tvbo32, cm.length, cm.tlength, cm.pos.x, cm.pos.y);

        cm
    }
}

pub struct ChunkRegistry {
    pub memories: Vec<ChunkMemory>
}