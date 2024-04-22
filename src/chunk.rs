
use std::collections::HashMap;
use num_enum::FromPrimitive;
use lockfree::queue;
use crate::worldgeometry::WorldGeometry;

use crate::cube::Cube;
use crate::vec::{self, IVec2};
use crate::packedvertex::{self, PackedVertex};
use crate::cube::CubeSide;
use rand::Rng;
pub struct ChunkGeo {
    pub data32: Vec<u32>,
    pub data8: Vec<u8>,
    pub pos: vec::IVec2,
    pub vbo32: gl::types::GLuint,
    pub vbo8: gl::types::GLuint,
}
impl ChunkGeo {
    pub fn clear(&mut self) {
        self.data32.clear();
        self.data8.clear();
    }
}

#[derive(Clone, Copy)]
pub struct ChunkFacade {
    
    pub geo_index: usize,
    pub used: bool,
    pub pos: vec::IVec2
}



static CW: i32 = 15;
static CH: i32 = 128;


pub struct ChunkSystem {
    pub chunks: Vec<ChunkFacade>,
    pub geobank: Vec<ChunkGeo>,
    pub takencare: HashMap<vec::IVec2, ChunkFacade>,
    pub geoqueue: lockfree::queue::Queue<usize>,
    pub radius: u8
}

impl ChunkSystem {
    pub fn move_and_rebuild(&mut self, index: usize, cpos: vec::IVec2) {
        if !self.takencare.contains_key(&cpos) {
            let chunk: &mut ChunkFacade = &mut self.chunks[index];
            if self.takencare.contains_key(&chunk.pos) {
                self.takencare.remove(&chunk.pos);
            }
            chunk.pos = cpos;
            self.rebuild_index(index);
        } else {
            self.rebuild_index(self.takencare.get(&cpos).unwrap().geo_index);
        }
    }

    pub fn rebuild_index(&mut self, index: usize) {
        let chunk: &mut ChunkFacade = &mut self.chunks[index];
        chunk.used = true;

        self.geobank[index].clear();

        for i in 0..CW {
            for k in 0..CW {
                for j in 0..CH {
                    let spot = vec::IVec3{x:(chunk.pos.x * CW)+i, y:j, z:(chunk.pos.y * CW)+k };
                    let block =  ChunkSystem::blockat(spot);
                    if block != 0 {
                        for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                            let neigh_block = ChunkSystem::blockat(spot + *neigh);
                            let cubeside = CubeSide::from_primitive(indie);

                            if neigh_block == 0 {
                                let side = Cube::get_side(cubeside);
                                let mut packed32: [u32; 6] = [0,0,0,0,0,0];
                                let mut packed8: [u8; 6] = [0,0,0,0,0,0];
                                for (ind, v) in side.chunks(3).enumerate() {
                                    let pack = PackedVertex::pack(i as u8 + v[0], j as u8 + v[1], k as u8 + v[2], ind as u8, 15, 0, 1, 0);
                                    packed32[ind] = pack.0;
                                    packed8[ind] = pack.1;
                                }
                                self.geobank[index].data32.extend_from_slice(packed32.as_slice());
                                self.geobank[index].data8.extend_from_slice(packed8.as_slice());
                            }
                        }
                    }
                }
            }
        }
        self.geoqueue.push(index);
        
        if !self.takencare.contains_key(&chunk.pos) {
            self.takencare.insert(chunk.pos,*chunk);
        }
    }

    pub fn blockat(_spot: vec::IVec3) -> u32 {
        //Random for now
        let mut rng = rand::thread_rng();

        let n1: u8 = rng.gen();
        if n1 > 128 {
            1
        } else {
            0
        }
    }
    pub fn new(radius: u8) -> ChunkSystem {
        let mut cs = ChunkSystem {
            chunks: Vec::new(),
            geobank: Vec::new(),
            takencare: HashMap::new(),
            geoqueue: lockfree::queue::Queue::new(),
            radius
        };

        let mut vbo32: gl::types::GLuint = 0;
        let mut vbo8: gl::types::GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut vbo32);
            gl::CreateBuffers(1, &mut vbo8);
        }
        
        for _ in 0..=radius*2 {
            for _ in 0..=radius*2 {
                cs.chunks.push(ChunkFacade {
                    geo_index: cs.geobank.len(),
                    used: false,
                    pos: IVec2{x:0, y:0},
                });
                cs.geobank.push(ChunkGeo {
                    data32: Vec::new(),
                    data8: Vec::new(),
                    pos: IVec2{x:0, y:0},
                    vbo32,
                    vbo8
                });
            }
        }

        cs
        
    }
}