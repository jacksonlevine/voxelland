
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    pub chunks: Vec<Arc<Mutex<ChunkFacade>>>,
    pub geobank: Vec<Arc<Mutex<ChunkGeo>>>,
    pub takencare: Arc<Mutex<HashMap<vec::IVec2, ChunkFacade>>>,
    pub geoqueue: Arc<lockfree::queue::Queue<usize>>,
    pub radius: u8
}

impl ChunkSystem {
    pub fn move_and_rebuild(&self, index: usize, cpos: vec::IVec2) {
        let takencare = self.takencare.clone();
        let mut takencarelock = takencare.lock().unwrap();

        if !takencarelock.contains_key(&cpos) {
            let chunkarc = self.chunks[index].clone();
            let mut chunklock = chunkarc.lock().unwrap();

            if takencarelock.contains_key(&chunklock.pos) {
                takencarelock.remove(&chunklock.pos);
            }
            chunklock.pos = cpos;
            drop(chunklock);
            drop(takencarelock);
            let chunkgeoarc = self.geobank[index].clone();
            let mut chunkgeolock = chunkgeoarc.lock().unwrap();
            chunkgeolock.pos = cpos;
            drop(chunkgeolock);
            self.rebuild_index(index);
        } else {
            let ind = takencarelock.get(&cpos).unwrap().geo_index;
            drop(takencarelock);
            self.rebuild_index(ind);
        }
    }

    pub fn rebuild_index(&self, index: usize) {
        #[cfg(feature = "yap_about_chunks")]
        println!("Rebuild index {}", index);
        
        let chunkarc = self.chunks[index].clone();
        let mut chunklock = chunkarc.lock().unwrap();
        chunklock.used = true;
        #[cfg(feature = "yap_about_chunks")]
        println!("Got past chunk lock");
        let geobankarc = self.geobank[index].clone();
        let mut geobanklock = geobankarc.lock().unwrap();
        geobanklock.clear();
        #[cfg(feature = "yap_about_chunks")]
        println!("Got past clearing geobank");
        for i in 0..CW {
            for k in 0..CW {
                for j in 0..CH {
                    let spot = vec::IVec3{x:(chunklock.pos.x * CW)+i, y:j, z:(chunklock.pos.y * CW)+k };
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

                                geobanklock.data32.extend_from_slice(packed32.as_slice());
                                geobanklock.data8.extend_from_slice(packed8.as_slice());
                            }
                        }
                    }
                }
            }
        }
        #[cfg(feature = "yap_about_chunks")]
        println!("Got past traversal");
        let mut gqarc = self.geoqueue.clone();
        gqarc.push(index);

        let takencare = self.takencare.clone();
        let mut takencarelock = takencare.lock().unwrap();
        #[cfg(feature = "yap_about_chunks")]
        println!("Got past tclock 2");
        if !takencarelock.contains_key(&chunklock.pos) {
            takencarelock.insert(chunklock.pos,*chunklock);
            #[cfg(feature = "yap_about_chunks")]
            println!("Inserting into taken care ");
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
            takencare: Arc::new(Mutex::new(HashMap::new())),
            geoqueue: Arc::new(lockfree::queue::Queue::new()),
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
                cs.chunks.push(Arc::new(Mutex::new(ChunkFacade {
                    geo_index: cs.geobank.len(),
                    used: false,
                    pos: IVec2{x:0, y:0},
                })));
                cs.geobank.push(Arc::new(Mutex::new(ChunkGeo {
                    data32: Vec::new(),
                    data8: Vec::new(),
                    pos: IVec2{x:0, y:0},
                    vbo32,
                    vbo8
                })));
            }
        }

        cs
        
    }
}