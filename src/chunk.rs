use std::collections::HashMap;

use num_enum::FromPrimitive;
use std::sync::{Arc, Mutex};

use noise::{NoiseFn, Perlin};

use crate::cube::Cube;
use crate::cube::CubeSide;
use crate::packedvertex::PackedVertex;
use crate::vec::{self, IVec2};

use crate::blockinfo::Blocks;
pub struct ChunkGeo {
    pub data32: Vec<u32>,
    pub data8: Vec<u8>,
    pub pos: vec::IVec2,
    pub vbo32: gl::types::GLuint,
    pub vbo8: gl::types::GLuint,
}
impl ChunkGeo {
    pub fn new() -> ChunkGeo {
        let mut vbo32: gl::types::GLuint = 0;
        let mut vbo8: gl::types::GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut vbo32);
            gl::CreateBuffers(1, &mut vbo8);
            let error = unsafe { gl::GetError() };
            if error != gl::NO_ERROR {
                println!(
                    "OpenGL Error after creating chunk system buffers: {}",
                    error
                );
            }
        }

        ChunkGeo {
            data32: Vec::new(),
            data8: Vec::new(),
            pos: IVec2 {
                x: 999999,
                y: 999999,
            },
            vbo32,
            vbo8,
        }
    }
    pub fn clear(&mut self) {
        self.data32.clear();
        self.data8.clear();
    }
}

#[derive(Clone, Copy)]
pub struct ChunkFacade {
    pub geo_index: usize,
    pub used: bool,
    pub pos: vec::IVec2,
}

static CW: i32 = 15;
static CH: i32 = 128;

pub struct ChunkSystem {
    pub chunks: Vec<Arc<Mutex<ChunkFacade>>>,
    pub geobank: Vec<Arc<Mutex<ChunkGeo>>>,
    pub takencare: Arc<Mutex<HashMap<vec::IVec2, ChunkFacade>>>,
    pub geoqueue: Arc<lockfree::queue::Queue<usize>>,
    pub radius: u8,
    pub perlin: Perlin,
}

impl ChunkSystem {
    pub fn new(radius: u8) -> ChunkSystem {
        let mut cs = ChunkSystem {
            chunks: Vec::new(),
            geobank: Vec::new(),
            takencare: Arc::new(Mutex::new(HashMap::new())),
            geoqueue: Arc::new(lockfree::queue::Queue::new()),
            radius,
            perlin: Perlin::new(1),
        };

        for _ in 0..radius * 2 + 5 {
            for _ in 0..radius * 2 + 5 {
                cs.chunks.push(Arc::new(Mutex::new(ChunkFacade {
                    geo_index: cs.geobank.len(),
                    used: false,
                    pos: IVec2 {
                        x: 999999,
                        y: 999999,
                    },
                })));
                cs.geobank.push(Arc::new(Mutex::new(ChunkGeo::new())));
            }
        }

        println!("Amount of chunkgeo buffers: {}", 2 * cs.geobank.len());

        cs
    }
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

        let mut memo: HashMap<vec::IVec3, u32> = HashMap::new();

        for i in 0..CW {
            for k in 0..CW {
                for j in 0..CH {
                    let spot = vec::IVec3 {
                        x: (chunklock.pos.x * CW) + i,
                        y: j,
                        z: (chunklock.pos.y * CW) + k,
                    };
                    let block = self.blockatmemo(spot, &mut memo);
                    if block != 0 {
                        for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                            let neigh_block = self.blockatmemo(spot + *neigh, &mut memo);
                            let cubeside = CubeSide::from_primitive(indie);

                            if neigh_block == 0 {
                                let side = Cube::get_side(cubeside);
                                let mut packed32: [u32; 6] = [0, 0, 0, 0, 0, 0];
                                let mut packed8: [u8; 6] = [0, 0, 0, 0, 0, 0];

                                let texcoord = Blocks::get_tex_coords(block, cubeside);
                                for (ind, v) in side.chunks(4).enumerate() {
                                    let pack = PackedVertex::pack(
                                        i as u8 + v[0],
                                        j as u8 + v[1],
                                        k as u8 + v[2],
                                        ind as u8,
                                        v[3],
                                        0,
                                        texcoord.0,
                                        texcoord.1,
                                    );
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
        let gqarc = self.geoqueue.clone();
        gqarc.push(index);

        let takencare = self.takencare.clone();
        let mut takencarelock = takencare.lock().unwrap();
        #[cfg(feature = "yap_about_chunks")]
        println!("Got past tclock 2");
        if !takencarelock.contains_key(&chunklock.pos) {
            takencarelock.insert(chunklock.pos, *chunklock);
            #[cfg(feature = "yap_about_chunks")]
            println!("Inserting into taken care ");
        }
    }

    fn mix(a: f64, b: f64, t: f64) -> f64 {
        a * (1.0 - t) + b * t
    }

    pub fn noise_func(&self, spot: vec::IVec3) -> f64 {
        let mut y = spot.y - 20;

        let noise1 = f64::max(
            0.0,
            20.0 + self.perlin.get([
                spot.x as f64 / 25.35,
                y as f64 / 20.35,
                spot.z as f64 / 25.35,
            ]) * 5.0
                - f64::max(
                    y as f64 / 2.0
                        + self
                            .perlin
                            .get([spot.x as f64 / 65.0, spot.z as f64 / 65.0])
                            * 10.0,
                    0.0,
                ),
        );

        y += 60;

        let noise2 = f64::max(
            0.0,
            50.0 + self.perlin.get([
                spot.x as f64 / 55.35,
                y as f64 / 25.35,
                spot.z as f64 / 55.35,
            ]) * 10.0
                + self.perlin.get([
                    spot.x as f64 / 25.35,
                    y as f64 / 65.35,
                    spot.z as f64 / 25.35,
                ]) * 20.0
                - f64::max(y as f64 / 3.0, 0.0),
        );

        let mut p = self
            .perlin
            .get([spot.x as f64 / 500.0, spot.z as f64 / 500.0])
            * 10.0;

        p = f64::max(p, 0.0);
        p = f64::min(p, 1.0);

        // Mixing noise1 and noise2 based on p, assuming `mix` is a function that blends the two values
        // Rust doesn't have a direct `mix` function, but you can create one or use a linear interpolation
        ChunkSystem::mix(noise1, noise2, p * 0.5)
    }

    pub fn blockatmemo(&self, spot: vec::IVec3, memo: &mut HashMap<vec::IVec3, u32>) -> u32 {
        match memo.get(&spot) {
            Some(b) => {
                return *b;
            }
            None => {
                let b = self.blockat(spot);
                memo.insert(spot, b);
                return b;
            }
        }

        // if memo.contains_key(&spot) {
        //     return *memo.get(&spot).unwrap();
        // } else {
        //     let b = self.blockat(spot);
        //     memo.insert(spot, b);
        //     return b;
        // }
    }

    pub fn blockat(&self, spot: vec::IVec3) -> u32 {
        static WL: f32 = 40.0;

        if self.noise_func(spot) > 10.0 {
            if self.noise_func(spot + vec::IVec3 { x: 0, y: 10, z: 0 }) > 10.0 {
                return 5;
            }
            if spot.y > (WL + 2.0) as i32
                || self.noise_func(spot + vec::IVec3 { x: 0, y: 5, z: 0 }) > 10.0
            {
                if self.noise_func(spot + vec::IVec3 { x: 0, y: 1, z: 0 }) < 10.0 {
                    return 3;
                }
                return 4;
            } else {
                return 1;
            }
        } else {
            if spot.y < WL as i32 {
                return 2;
            } else {
                return 0;
            }
        }
    }
}
