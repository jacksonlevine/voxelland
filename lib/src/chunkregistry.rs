

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

    pub wvvbo: GLuint,
    pub wuvvbo: GLuint,

    pub vbo8rgb: GLuint,
    pub tvbo8rgb: GLuint,

    pub length: i32,
    pub tlength: i32,
    pub vlength: i32,
    pub wvlength: i32,

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

            wvvbo: geo.wvvbo,
            wuvvbo: geo.wuvvbo,

            vbo8rgb: geo.vbo8rgb,
            tvbo8rgb: geo.tvbo8rgb,

            length: 0,
            tlength: 0,
            vlength: 0,
            wvlength: 0,
            pos: *geo.pos.lock()
        };
        //info!("ChunkMemory: {} {} {} {} {} {} {} {} {}", cm.used, cm.vbo8, cm.vbo32, cm.tvbo8, cm.tvbo32, cm.length, cm.tlength, cm.pos.x, cm.pos.y);

        cm
    }
}

pub struct ChunkRegistry {
    pub memories: Vec<ChunkMemory>
}