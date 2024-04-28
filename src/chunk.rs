use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicU8;

use dashmap::DashMap;
use dashmap::DashSet;
use num_enum::FromPrimitive;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use vox_format::data::VoxModels;
use vox_format::types::Model;
use walkdir::WalkDir;
use std::sync::{Arc, Mutex};

use noise::{NoiseFn, Perlin};

use crate::cube::Cube;
use crate::cube::CubeSide;
use crate::packedvertex::PackedVertex;
use crate::vec::IVec3;
use crate::vec::{self, IVec2};

use crate::blockinfo::Blocks;
use crate::voxmodel::JVoxModel;
pub struct ChunkGeo {
    pub data32: Mutex<Vec<u32>>,
    pub data8: Mutex<Vec<u8>>,
    pub pos: Mutex<vec::IVec2>,
    pub vbo32: gl::types::GLuint,
    pub vbo8: gl::types::GLuint,
    pub tdata32: Mutex<Vec<u32>>,
    pub tdata8: Mutex<Vec<u8>>,
    pub tvbo32: gl::types::GLuint,
    pub tvbo8: gl::types::GLuint,
}
impl ChunkGeo {
    pub fn new() -> ChunkGeo {
        let mut vbo32: gl::types::GLuint = 0;
        let mut vbo8: gl::types::GLuint = 0;
        let mut tvbo32: gl::types::GLuint = 0;
        let mut tvbo8: gl::types::GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut vbo32);
            gl::CreateBuffers(1, &mut vbo8);
            gl::CreateBuffers(1, &mut tvbo32);
            gl::CreateBuffers(1, &mut tvbo8);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!(
                    "OpenGL Error after creating chunk system buffers: {}",
                    error
                );
            }
        }

        ChunkGeo {
            data32: Mutex::new(Vec::new()),
            data8: Mutex::new(Vec::new()),
            pos: Mutex::new(IVec2 {
                x: 999999,
                y: 999999,
            }),
            vbo32,
            vbo8,
            tdata32: Mutex::new(Vec::new()),
            tdata8: Mutex::new(Vec::new()),
            tvbo32,
            tvbo8,
        }
    }
    pub fn clear(&self) {
        self.data32.lock().unwrap().clear();
        self.data8.lock().unwrap().clear();
        self.tdata32.lock().unwrap().clear();
        self.tdata8.lock().unwrap().clear();
    }
    pub fn solids(&self) -> (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>) {
        return (&self.data32, &self.data8);
    }
    pub fn transparents(&self) -> (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>) {
        return (&self.tdata32, &self.tdata8);
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

pub struct DoubleChunkGeo {
    pub geos: [Mutex<ChunkGeo>; 2],
    pub num: AtomicU8
}

impl DoubleChunkGeo {
    pub fn new() -> DoubleChunkGeo {
        DoubleChunkGeo {
            geos: [ Mutex::new(ChunkGeo::new()),
                    Mutex::new(ChunkGeo::new())],
            num: AtomicU8::new(1)
        }
    }
}

pub struct ChunkSystem {
    pub chunks: Vec<Arc<Mutex<ChunkFacade>>>,
    pub geobank: Vec<Arc<ChunkGeo>>,
    pub takencare: Arc<DashMap<vec::IVec2, ChunkFacade>>,
    pub finished_geo_queue: Arc<lockfree::queue::Queue<usize>>,
    pub user_rebuild_requests: lockfree::queue::Queue<usize>,
    pub userdatamap: DashMap<vec::IVec3, u32>,
    pub radius: u8,
    pub perlin: Perlin,
    pub voxel_models: Vec<JVoxModel>,
    pub generated: DashSet<vec::IVec2>,
}

impl ChunkSystem {
    pub fn new(radius: u8) -> ChunkSystem {
        let mut cs = ChunkSystem {
            chunks: Vec::new(),
            geobank: Vec::new(),
            takencare: Arc::new(DashMap::new()),
            finished_geo_queue: Arc::new(lockfree::queue::Queue::new()),
            user_rebuild_requests: lockfree::queue::Queue::new(),
            userdatamap: DashMap::new(),
            radius,
            perlin: Perlin::new(1),
            voxel_models: Vec::new(),
            generated: DashSet::new()
        };

        let directory_path = "assets/voxelmodels/";

        for entry in WalkDir::new(directory_path) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy().into_owned();
                cs.voxel_models.push(JVoxModel::new(Box::leak(path_str.into_boxed_str())));
            }
        }

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
                cs.geobank.push(Arc::new(ChunkGeo::new()));
            }
        }

        //tracing::info!("Amount of chunkgeo buffers: {}", 4 * cs.geobank.len());

        cs
    }
    pub fn spot_to_chunk_pos(spot: &vec::IVec3) -> vec::IVec2 {
        return vec::IVec2{
            x: (spot.x as f32 / 15.0).floor() as i32,
            y: (spot.z as f32 / 15.0).floor() as i32,
        }
    }
    pub fn set_block_and_queue_rerender(&self, spot: vec::IVec3, block: u32, neighbors: bool) {
        self.set_block(spot, block);
        let chunk_key = &Self::spot_to_chunk_pos(&spot);
        if neighbors {
            static NEIGHBORS: [vec::IVec2; 9] = [
                vec::IVec2{x: -1, y: -1},
                vec::IVec2{x: -1, y: 0},
                vec::IVec2{x: -1, y: 1},
                vec::IVec2{x: 0, y: -1},
                vec::IVec2{x: 0, y: 0},
                vec::IVec2{x: 0, y: 1},
                vec::IVec2{x: 1, y: -1},
                vec::IVec2{x: 1, y: 0},
                vec::IVec2{x: 1, y: 1},
            ];
            for i in NEIGHBORS {
                let here = *chunk_key + i;
                match self.takencare.get(&here) {
                    Some(cf) => {
                        self.user_rebuild_requests.push(cf.geo_index);
                    }
                    None => {}
                }
            }
        } else {
            
            match self.takencare.get(chunk_key) {
                Some(cf) => {
                    self.user_rebuild_requests.push(cf.geo_index);
                }
                None => {}
            }
        }
        
    }
    pub fn set_block(&self, spot: vec::IVec3, block: u32) {
        self.userdatamap.insert(spot, block);
    }
    pub fn move_and_rebuild(&self, index: usize, cpos: vec::IVec2) {
        let tc = self.takencare.clone();

        if !tc.contains_key(&cpos) {
            let chunkarc = self.chunks[index].clone();
            let mut chunklock = chunkarc.lock().unwrap();

            if tc.contains_key(&chunklock.pos) {
                tc.remove(&chunklock.pos);
            }
            chunklock.pos = cpos;
            drop(chunklock);
            let chunkgeoarc = self.geobank[index].clone();

            // let mut chunkgeolock = chunkgeoarc.geos[num as usize].lock().unwrap();
            // chunkgeolock.pos = cpos;
            // drop(chunkgeolock);

            // if num == 0 { num = 1; } else { num = 0; }

            chunkgeoarc.pos.lock().unwrap().clone_from(&cpos);

            //#[cfg(feature="structures")]
            self.generate_chunk(&chunkgeoarc.pos.lock().unwrap());

            

            self.rebuild_index(index);
        } else {
            println!("This path");
            let ind = tc.get(&cpos).unwrap().geo_index;
            self.rebuild_index(ind);
        }
    }

    pub fn rebuild_index(&self, index: usize) {



        let chunkarc = self.chunks[index].clone();
        let mut chunklock = chunkarc.lock().unwrap();

        chunklock.used = true;


        let geobankarc = self.geobank[index].clone();
        // if num == 0 { num = 1; } else { num = 0; }

        
        

        geobankarc.clear();


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
                        if Blocks::is_transparent(block) || Blocks::is_semi_transparent(block) {
                            for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                let neigh_block = self.blockatmemo(spot + *neigh, &mut memo);
                                let cubeside = CubeSide::from_primitive(indie);
                                let neigh_semi_trans = Blocks::is_semi_transparent(neigh_block);
                                let water_bordering_transparent = block == 2 && neigh_block != 2 && Blocks::is_transparent(neigh_block);
    
                                if neigh_block == 0 || neigh_semi_trans || water_bordering_transparent {
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
    
                                    geobankarc.tdata32.lock().unwrap().extend_from_slice(packed32.as_slice());
                                    geobankarc.tdata8.lock().unwrap().extend_from_slice(packed8.as_slice());
                                }
                            }
                        } else {
                            for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                let neigh_block = self.blockatmemo(spot + *neigh, &mut memo);
                                let cubeside = CubeSide::from_primitive(indie);
                                let neighbor_transparent = Blocks::is_transparent(neigh_block) || Blocks::is_semi_transparent(neigh_block);
                                
                                if neigh_block == 0 || neighbor_transparent {
                                    let side = Cube::get_side(cubeside);
                                    let mut packed32: [u32; 6] = [0, 0, 0, 0, 0, 0];
                                    let mut packed8: [u8; 6] = [0, 0, 0, 0, 0, 0];
    
                                    let texcoord = Blocks::get_tex_coords(block, cubeside);
                                    for (ind, v) in side.chunks(4).enumerate() {
                                        static AMB_CHANGES: [u8; 4] = [
                                            0, 3, 6, 10
                                        ];

                                        let amb_spots: &[vec::IVec3; 3] = Cube::get_amb_occul_spots(cubeside, ind as u8);

                                        let amb_change = amb_spots.iter()
                                                                  .map(|vec| self.blockatmemo(*vec + spot, &mut memo))
                                                                  .filter(|&result| result != 0)
                                                                  .count();

                                        
                                        
                                        let pack = PackedVertex::pack(
                                            i as u8 + v[0],
                                            j as u8 + v[1],
                                            k as u8 + v[2],
                                            ind as u8,
                                            (v[3] - AMB_CHANGES[amb_change]).clamp(0, 16),
                                            0,
                                            texcoord.0,
                                            texcoord.1,
                                        );
                                        packed32[ind] = pack.0;
                                        packed8[ind] = pack.1;
                                    }
    
                                    geobankarc.data32.lock().unwrap().extend_from_slice(packed32.as_slice());
                                    geobankarc.data8.lock().unwrap().extend_from_slice(packed8.as_slice());
                                }
                            }
                        }
                        
                    }
                }
            }
        }
        
        let gqarc = self.finished_geo_queue.clone();
        gqarc.push(index);
        

        let tc = self.takencare.clone();

        if !tc.contains_key(&chunklock.pos) {
            tc.insert(chunklock.pos, *chunklock);

        }
    }

    pub fn stamp_here(&self, spot: &vec::IVec3, model: &JVoxModel, implicated: Option<&mut HashSet<IVec2>>) {
        let mut colorset = HashSet::new();
        let mut local_implicated_chunks; // Declare a mutable local HashSet for when None is provided
        let implicated_chunks; // This will be the reference used throughout the function
        let mut implicated_provided = false;

        // Determine which HashSet to use
        match implicated {
            Some(hs) => {
                implicated_chunks = hs;
                implicated_provided = true;
            },
            None => {
                local_implicated_chunks = HashSet::new(); // Create a new HashSet when None is provided
                implicated_chunks = &mut local_implicated_chunks; // Use the local HashSet
            }
        };
        
        for i in &model.model.models {
            for v in &i.voxels {
                let rearr_point = IVec3::new(v.point.x as i32, (v.point.z as i32) - 40, v.point.y as i32);
                colorset.insert(v.color_index);
                let c_pos = IVec2{x: ((spot.x + rearr_point.x) as f32 / 15.0).floor() as i32, y: ((spot.z + rearr_point.z) as f32 / 15.0).floor() as i32};
                implicated_chunks.insert(c_pos);
                self.set_block(
                    *spot + IVec3::new(spot.x + rearr_point.x, spot.y + rearr_point.y, spot.z + rearr_point.z),
                    (1 + colorset.len()).clamp(0, 10) as u32,
                )
            }
        }
        if !implicated_provided {
            for c in implicated_chunks.iter() {
                match self.takencare.get(&c) {
                    Some(cf) => {
                        self.user_rebuild_requests.push(cf.geo_index);
                    }
                    None => {}
                }
            }
        }
    }

    pub fn generate_chunk(&self, cpos: &vec::IVec2) {
            // Seed for the RNG.
        let seed: [u8; 32] = [
            (cpos.x % 255) as u8 , (cpos.y % 255) as u8 , 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]; // This needs to be a fixed-size array of bytes (u8).

        // Create a new RNG instance with the seed.
        let mut rng = StdRng::from_seed(seed);

        // // Generate some random numbers
        // let rand_number1: u32 = rng.gen();
        // let rand_number2: u32 = rng.gen();
        let mut implicated: HashSet<vec::IVec2> = HashSet::new();

        let mut should_break = false;
        
        for x in 0..CW {
            for z in 0..CW {
                for y in (0..CH).rev() {
                    let coord = IVec3::new(cpos.x * CW + x, y, cpos.y * CW + z);
                    if self.blockat(coord) == 3 {
                        let rand_number1: u32 = rng.gen_range(0..25);
                        if rand_number1 < self.voxel_models.len() as u32 {
                            self.stamp_here(&coord, &self.voxel_models[rand_number1 as usize], Some(&mut implicated));
                        }
                        should_break = true;
                        break;
                    }
                    
                }
                if should_break {
                    break;
                }
            }
            if should_break {
                break;
            }
        }

        for c in implicated.iter() {
            match self.takencare.get(&c) {
                Some(cf) => {
                    self.user_rebuild_requests.push(cf.geo_index);
                }
                None => {}
            }
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
    
        // return memo.get(&spot).map_or_else(|| {
        //     let b = self.blockat(spot);
        //     memo.insert(spot, b);
        //     b
        // }, |b| *b);

        return match memo.get(&spot) {
            Some(b) => {
                *b
            }
            None => {
                let b = self.blockat(spot);
                memo.insert(spot, b);
                b
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

        match self.userdatamap.get(&spot) {
            Some(id) => {
                return *id;
            }
            None => {}
        }

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
