use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU8;
use std::sync::RwLock;

use dashmap::DashMap;
use dashmap::DashSet;
use glam::Vec3;
use num_enum::FromPrimitive;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use vox_format::data::VoxModels;
use vox_format::types::Model;
use walkdir::WalkDir;
use std::sync::{Arc, Mutex};

use noise::{NoiseFn, Perlin};

use crate::chunkregistry::ChunkMemory;
use crate::chunkregistry::ChunkRegistry;
use crate::cube::Cube;
use crate::cube::CubeSide;
use crate::packedvertex::PackedVertex;
use crate::planetinfo::Planets;
use crate::shader::Shader;
use crate::vec::IVec3;
use crate::vec::{self, IVec2};

use crate::blockinfo::Blocks;
use crate::voxmodel::JVoxModel;
use crate::worldgeometry::WorldGeometry;

use std::str::FromStr;
use std::io::Write;
pub struct ChunkGeo {
    pub data32: Mutex<Vec<u32>>,
    pub data8: Mutex<Vec<u8>>,
    pub pos: Mutex<vec::IVec2>,
    pub vbo32: gl::types::GLuint,
    pub vbo8: gl::types::GLuint,
    pub tdata32: Mutex<Vec<u32>>,
    pub tdata8: Mutex<Vec<u8>>,
    pub tvbo32: gl::types::GLuint,
    pub tvbo8: gl::types::GLuint
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
            tvbo8
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



pub struct ReadyMesh {
    pub geo_index: usize,
    pub newpos: vec::IVec2, 
    pub newlength: i32,
    pub newtlength: i32
}

impl ReadyMesh {
    pub fn new(index: usize, newpos: &vec::IVec2, newlength: i32, newtlength: i32) -> ReadyMesh {
        ReadyMesh {
            geo_index: index,
            newpos: *newpos,
            newlength,
            newtlength
        }
    }
}

pub struct ChunkSystem {
    pub chunks: Vec<Arc<Mutex<ChunkFacade>>>,
    pub geobank: Vec<Arc<ChunkGeo>>,
    pub takencare: Arc<DashMap<vec::IVec2, ChunkFacade>>,
    pub finished_user_geo_queue: Arc<lockfree::queue::Queue<ReadyMesh>>,
    pub finished_geo_queue: Arc<lockfree::queue::Queue<ReadyMesh>>,
    pub user_rebuild_requests: lockfree::queue::Queue<usize>,
    pub gen_rebuild_requests: lockfree::queue::Queue<usize>,
    pub background_rebuild_requests: lockfree::queue::Queue<usize>,
    pub userdatamap: DashMap<vec::IVec3, u32>,
    pub nonuserdatamap: DashMap<vec::IVec3, u32>,
    pub justcollisionmap: DashMap<vec::IVec3, u8>,
    pub radius: u8,
    pub perlin: Perlin,
    pub voxel_models: Option<Arc<Vec<JVoxModel>>>,
    pub chunk_memories: Mutex<ChunkRegistry>,
    pub planet_type: u8,
    pub currentseed: RwLock<u32>
}

impl ChunkSystem {

    pub fn save_current_world_to_file(&self, path: String) {

        fs::create_dir_all(&path).unwrap();
        
        let mut file = File::create(path.clone() + "/udm").unwrap();
        for entry in self.userdatamap.iter() {
            writeln!(file, "{} {}", entry.key(), entry.value()).unwrap();
        }

        let mut file = File::create(path.clone() + "/seed").unwrap();
        writeln!(file, "{}", self.currentseed.read().unwrap()).unwrap();

    }

    pub fn load_world_from_file(&self, path: String) {
        self.userdatamap.clear();

        match File::open(format!("{}/udm", path.clone())) {
            Ok(_) => {

            }
            Err(_) => {
                fs::create_dir_all(&path.clone()).unwrap();
                self.save_current_world_to_file(path.clone());
            }
        }

        let file = File::open(format!("{}/udm", path)).unwrap();
        let reader = BufReader::new(file);
    
        for line in reader.lines() {
            let line = line.unwrap();
            let mut parts = line.splitn(4, ' ');
            if let (Some(x), Some(y), Some(z), Some(value)) = (parts.next(), parts.next(), parts.next(), parts.next()) {
                let key = format!("{} {} {}", x, y, z);
                self.userdatamap.insert(vec::IVec3::from_str(&key).unwrap(), value.parse::<u32>().unwrap());
            }
        }
    
        let file = File::open(format!("{}/seed", path)).unwrap();
        let reader = BufReader::new(file);
    
        for line in reader.lines() {
            let line = line.unwrap();
            let mut parts = line.splitn(2, ' ');
            if let Some(seed) = parts.next() {
                *self.currentseed.write().unwrap() = seed.parse::<u32>().unwrap();
            }
        }
    }

    pub fn collision_predicate(&self, vec: vec::IVec3) -> bool {
        return self.blockat(vec.clone()) != 0 || self.justcollisionmap.contains_key(&vec);
    }
    

    pub fn start_with_seed(seed: u32) {

    }

    pub fn reset(&mut self, radius: u8, seed: u32, noisetype: usize) {
        for cg in &self.geobank {
            unsafe {
                gl::DeleteBuffers(1, &cg.vbo32);
                gl::DeleteBuffers(1, &cg.tvbo32);
                gl::DeleteBuffers(1, &cg.vbo8);
                gl::DeleteBuffers(1, &cg.tvbo8);
            }
        }
        self.chunks.clear();
        self.geobank.clear();
        self.chunk_memories.lock().unwrap().memories.clear();
        self.takencare.clear();
        while let Some(_) = self.finished_geo_queue.pop() {}
        while let Some(_) = self.finished_user_geo_queue.pop() {}
        while let Some(_) = self.user_rebuild_requests.pop() {}
        while let Some(_) = self.gen_rebuild_requests.pop() {}
        while let Some(_) = self.background_rebuild_requests.pop() {}
        self.userdatamap.clear();
        self.nonuserdatamap.clear();
        self.justcollisionmap.clear();
        self.radius = radius;
        self.perlin = Perlin::new(seed);
        self.voxel_models = None;
        self.planet_type = noisetype as u8;
        (*self.currentseed.write().unwrap()) = seed;

        for _ in 0..radius * 2 + 5 {
            for _ in 0..radius * 2 + 5 {
                self.chunks.push(Arc::new(Mutex::new(ChunkFacade {
                    geo_index: self.geobank.len(),
                    used: false,
                    pos: IVec2 {
                        x: 999999,
                        y: 999999,
                    },
                })));
                
                self.geobank.push(Arc::new(ChunkGeo::new()));
                self.chunk_memories.lock().unwrap().memories.push(ChunkMemory::new(&self.geobank[self.geobank.len() - 1]));
            }
        }
    }
    
    pub fn new(radius: u8, seed: u32, noisetype: usize) -> ChunkSystem {
        let mut cs = ChunkSystem {
            chunks: Vec::new(),
            geobank: Vec::new(),
            takencare: Arc::new(DashMap::new()),
            finished_user_geo_queue: Arc::new(lockfree::queue::Queue::new()),
            finished_geo_queue: Arc::new(lockfree::queue::Queue::new()),
            user_rebuild_requests: lockfree::queue::Queue::new(),
            gen_rebuild_requests: lockfree::queue::Queue::new(),
            background_rebuild_requests: lockfree::queue::Queue::new(),
            userdatamap: DashMap::new(),
            nonuserdatamap: DashMap::new(),
            justcollisionmap: DashMap::new(),
            radius,
            perlin: Perlin::new(seed),
            voxel_models: None,
            chunk_memories: Mutex::new(ChunkRegistry{
                memories: Vec::new()
            }),
            planet_type: noisetype as u8,
            currentseed: RwLock::new(0)
        };


        // let directory_path = "assets/voxelmodels/";

        // for entry in WalkDir::new(directory_path) {
        //     let entry = entry.unwrap();
        //     if entry.file_type().is_file() {
        //         let path_str = entry.path().to_string_lossy().into_owned();
        //         let jv = JVoxModel::new(Box::leak(path_str.into_boxed_str()));
        //         //println!("{:#?}", jv.model);
        //         cs.voxel_models.push(jv);
        //     }
        // }

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
                cs.chunk_memories.lock().unwrap().memories.push(ChunkMemory::new(&cs.geobank[cs.geobank.len() - 1]));
            }
        }

        //tracing::info!("Amount of chunkgeo buffers: {}", 4 * cs.geobank.len());

        cs
    }
    pub fn spot_to_chunk_pos(spot: &vec::IVec3) -> vec::IVec2 {
        return vec::IVec2{
            x: (spot.x as f32 / CW as f32).floor() as i32,
            y: (spot.z as f32 / CW as f32).floor() as i32,
        }
    }
    pub fn initial_rebuild_on_main_thread(csys: &Arc<RwLock<ChunkSystem>>, shader: &Shader, campos: &Vec3) {

        // unsafe {
        //     gl::BindVertexArray(shader.vao);
        //     gl::UseProgram(shader.shader_id);
        // }

        let user_cpos = IVec2 {
            x: (campos.x / CW as f32).floor() as i32,
            y: (campos.z / CW as f32).floor() as i32,
        };

        let mut neededspots = Vec::new();

        let csys = csys.read().unwrap();

        for i in -(csys.radius as i32)..(csys.radius as i32) {
            for k in -(csys.radius as i32)..(csys.radius as i32) {
                let this_spot = IVec2 {
                    x: user_cpos.x + i as i32,
                    y: user_cpos.y + k as i32,
                };
                neededspots.push(this_spot);
            }
        }


        for (index, cpos) in neededspots.iter().enumerate() {
            csys.move_and_rebuild(index, *cpos);
        }


        // let mut genstuff = true;
        // while genstuff {
        //     match csys.gen_rebuild_requests.pop() {
        //         Some(index) => {
        //             csys.rebuild_index(index, true);
        //         }
        //         None => {
        //             genstuff = false;
        //         }
        //     }
        // }

        // let mut more_in_queue = true;
        // while more_in_queue {
        //     match csys.finished_user_geo_queue.pop() {
        //         Some(ready) => {
        //             //println!("Some user queue");
        //            // println!("Weird!");
    
        //             let bankarc = csys.geobank[ready.geo_index].clone();
    
        //             let mut cmemlock = csys.chunk_memories.lock().unwrap();
    
        //             cmemlock.memories[ready.geo_index].length = ready.newlength;
        //             cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
        //             cmemlock.memories[ready.geo_index].pos = ready.newpos;
        //             cmemlock.memories[ready.geo_index].used = true;
    
        //             // println!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
        //             // println!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
        //             // //if num == 0 { num = 1; } else { num = 0; }
        //             //bankarc.num.store(num, std::sync::atomic::Ordering::Release);
        //             // if num == 0 {
        //             //     bankarc.num.store(1, Ordering::Relaxed);
        //             //     num = 1;
        //             // } else {
        //             //     bankarc.num.store(0, Ordering::Relaxed);
        //             //     num = 0;
        //             // };
    
        //             // let v32 = cmemlock.memories[ready.geo_index].vbo32;
        //             // let v8 = cmemlock.memories[ready.geo_index].vbo8;
        //             // let tv32 = cmemlock.memories[ready.geo_index].tvbo32;
        //             // let tv8 = cmemlock.memories[ready.geo_index].tvbo8;
                    
    
        //             // WorldGeometry::bind_geometry(v32, v8, true, shader, bankarc.solids());
        //             // WorldGeometry::bind_geometry(
        //             //     tv32,
        //             //     tv8,
        //             //     true,
        //             //     shader,
        //             //     bankarc.transparents(),
        //             // );
        //         }
        //         None => {
        //             more_in_queue = false;
        //         }
        //     }
        // }
        

    }
    pub fn queue_rerender(&self, spot: vec::IVec3, user_power: bool) {
        let chunk_key = &Self::spot_to_chunk_pos(&spot);
        match self.takencare.get(chunk_key) {
            Some(cf) => {
                match user_power {
                    true => {
                        self.user_rebuild_requests.push(cf.geo_index);
                    }
                    false => {
                        self.background_rebuild_requests.push(cf.geo_index);
                    }
                }
                
            }
            None => {}
        }
    }
    pub fn queue_rerender_with_key(&self, chunk_key: IVec2, user_power: bool) {

        match self.takencare.get(&chunk_key) {
            Some(cf) => {
                match user_power {
                    true => {
                        self.user_rebuild_requests.push(cf.geo_index);
                    }
                    false => {
                        self.background_rebuild_requests.push(cf.geo_index);
                    }
                }
                
            }
            None => {}
        }
    }
    pub fn set_block_and_queue_rerender(&self, spot: vec::IVec3, block: u32, neighbors: bool, user_power: bool) {
        self.set_block(spot, block, user_power);
        if neighbors {

            let mut neighbs: HashSet<vec::IVec2> = HashSet::new();

            for i in Cube::get_neighbors() {
                let thisspot = spot + *i;
                neighbs.insert(ChunkSystem::spot_to_chunk_pos(&thisspot));
            }
            for i in neighbs {
                let here = i;
                self.queue_rerender_with_key(here, user_power);
            }
        } else {
            self.queue_rerender(spot, user_power);
        }
        
    }

    pub fn set_block(&self, spot: vec::IVec3, block: u32, user_power: bool) {
        match user_power {
            true => {
                //println!("Has user power, set block to {block}");
                self.userdatamap.insert(spot, block);
            }
            false => {
                //println!("Non user power");
                self.nonuserdatamap.insert(spot, block);
            }
        }
        
    }
    pub fn move_and_rebuild(&self, index: usize, cpos: vec::IVec2) {
        //println!("MBeing asked to move and rebuild to {} {}", cpos.x, cpos.y);
        let tc = self.takencare.clone();

        if !tc.contains_key(&cpos) {

            let chunkgeoarc = self.geobank[index].clone();

            if tc.contains_key(&chunkgeoarc.pos.lock().unwrap()) {
                tc.remove(&chunkgeoarc.pos.lock().unwrap());
            }


            // let mut chunkgeolock = chunkgeoarc.geos[num as usize].lock().unwrap();
            // chunkgeolock.pos = cpos;
            // drop(chunkgeolock);

            // if num == 0 { num = 1; } else { num = 0; }

            chunkgeoarc.pos.lock().unwrap().clone_from(&cpos);
            let lo = chunkgeoarc.pos.lock().unwrap();

            let mut chunklock = self.chunks[index].lock().unwrap();

            chunklock.pos = cpos;

            drop(chunklock);

            //println!("Chunkgeoarc pos set to {} {}", lo.x, lo.y);

            //#[cfg(feature="structures")]
            self.generate_chunk(&lo);

            

            self.rebuild_index(index, false);
        } else {
            println!("This path");
            let ind = tc.get(&cpos).unwrap().geo_index;
            self.rebuild_index(ind, false);
        }
    }

    pub fn rebuild_index(&self, index: usize, user_power: bool) {


        //println!("Rebuilding!");
        let chunkarc = self.chunks[index].clone();
        let mut chunklock = chunkarc.lock().unwrap();

        chunklock.used = true;


        let geobankarc = self.geobank[index].clone();
        // if num == 0 { num = 1; } else { num = 0; }

        
        

        geobankarc.clear();


        let mut memo: HashMap<vec::IVec3, u32> = HashMap::new();

        let mut data32 = geobankarc.data32.lock().unwrap();
        let mut data8 = geobankarc.data8.lock().unwrap();
        let mut tdata32 = geobankarc.tdata32.lock().unwrap();
        let mut tdata8 = geobankarc.tdata8.lock().unwrap();

        let mut tops: HashMap<vec::IVec2, i32> = HashMap::new();

        for i in 0..CW {
            for k in 0..CW {
                let mut hit_block = false;
                for j in (0..CH).rev() {
                    let spot = vec::IVec3 {
                        x: (chunklock.pos.x * CW) + i,
                        y: j,
                        z: (chunklock.pos.y * CW) + k,
                    };
                    let block = self.blockatmemo(spot, &mut memo);
                    // if self.justcollisionmap.contains_key(&spot) {
                    //     for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                    //         let cubeside = CubeSide::from_primitive(indie);
                    //         let side = Cube::get_side(cubeside);
                    //                 let mut packed32: [u32; 6] = [0, 0, 0, 0, 0, 0];
                    //                 let mut packed8: [u8; 6] = [0, 0, 0, 0, 0, 0];
    
                    //                 let texcoord = Blocks::get_tex_coords(1, cubeside);
                    //                 for (ind, v) in side.chunks(4).enumerate() {


                    //                     static AMB_CHANGES: [u8; 4] = [
                    //                         0, 3, 6, 10
                    //                     ];

                    //                     let amb_spots: &[vec::IVec3; 3] = Cube::get_amb_occul_spots(cubeside, ind as u8);

                    //                     let amb_change = amb_spots.iter()
                    //                                               .map(|vec| self.blockatmemo(*vec + spot, &mut memo))
                    //                                               .filter(|&result| result != 0)
                    //                                               .count();

                    //                     let base_light: i32 = v[3] as i32 - AMB_CHANGES[amb_change] as i32; // Perform calculations as i32
                    //                     let adjusted_light: i32 = if hit_block {
                    //                         base_light - 3
                    //                     } else {
                    //                         base_light
                    //                     };
                    //                     let clamped_light: u8 = adjusted_light.clamp(0, 15) as u8; // Clamp in i32 context, then cast to u8
                                        


                    //                     let pack = PackedVertex::pack(
                    //                         i as u8 + v[0],
                    //                         j as u8 + v[1],
                    //                         k as u8 + v[2],
                    //                         ind as u8,
                    //                         clamped_light,
                    //                         0,
                    //                         texcoord.0,
                    //                         texcoord.1,
                    //                     );
                    //                     packed32[ind] = pack.0;
                    //                     packed8[ind] = pack.1;
                    //                 }
    
                    //                 tdata32.extend_from_slice(packed32.as_slice());
                    //                 tdata8.extend_from_slice(packed8.as_slice());
                    //     }
                    // }
                    if block != 0 {
 
                        
                        if Blocks::is_transparent(block) || Blocks::is_semi_transparent(block) {
                            for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                let neigh_block = self.blockatmemo(spot + *neigh, &mut memo);
                                let cubeside = CubeSide::from_primitive(indie);
                                let neigh_semi_trans = Blocks::is_semi_transparent(neigh_block);
                                let water_bordering_transparent = block == 2 && neigh_block != 2 && Blocks::is_transparent(neigh_block);

                                hit_block = match tops.get(&vec::IVec2{x: i + neigh.x, y: k + neigh.z}) {
                                    Some(t) => {
                                        *t > j + neigh.y
                                    }
                                    None => { false }
                                };
    
                                if neigh_block == 0 || neigh_semi_trans || water_bordering_transparent {
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

                                        let base_light: i32 = v[3] as i32 - AMB_CHANGES[amb_change] as i32; // Perform calculations as i32
                                        let adjusted_light: i32 = if hit_block {
                                            base_light - 3
                                        } else {
                                            base_light
                                        };
                                        let clamped_light: u8 = adjusted_light.clamp(0, 15) as u8; // Clamp in i32 context, then cast to u8
                                        


                                        let pack = PackedVertex::pack(
                                            i as u8 + v[0],
                                            j as u8 + v[1],
                                            k as u8 + v[2],
                                            ind as u8,
                                            clamped_light,
                                            0,
                                            texcoord.0,
                                            texcoord.1,
                                        );
                                        packed32[ind] = pack.0;
                                        packed8[ind] = pack.1;
                                    }
    
                                    tdata32.extend_from_slice(packed32.as_slice());
                                    tdata8.extend_from_slice(packed8.as_slice());
                                } else {
                                    tops.insert(vec::IVec2{x: i + neigh.x, y: k + neigh.z}, j + neigh.y);
                                }
                            }
                        } else {

                            for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                let neigh_block = self.blockatmemo(spot + *neigh, &mut memo);
                                let cubeside = CubeSide::from_primitive(indie);
                                let neighbor_transparent = Blocks::is_transparent(neigh_block) || Blocks::is_semi_transparent(neigh_block);
                                
                                hit_block = match tops.get(&vec::IVec2{x: i + neigh.x, y: k + neigh.z}) {
                                    Some(t) => {
                                        *t > j + neigh.y
                                    }
                                    None => { false }
                                };

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

                                        let base_light: i32 = v[3] as i32 - AMB_CHANGES[amb_change] as i32; // Perform calculations as i32
                                        let adjusted_light: i32 = if hit_block {
                                            base_light - 3
                                        } else {
                                            base_light
                                        };
                                        let clamped_light: u8 = adjusted_light.clamp(0, 15) as u8; // Clamp in i32 context, then cast to u8
                                        
                                        let pack = PackedVertex::pack(
                                            i as u8 + v[0],
                                            j as u8 + v[1],
                                            k as u8 + v[2],
                                            ind as u8,
                                            clamped_light,
                                            0,
                                            texcoord.0,
                                            texcoord.1,
                                        );
                                        packed32[ind] = pack.0;
                                        packed8[ind] = pack.1;
                                    }
    
                                    data32.extend_from_slice(packed32.as_slice());
                                    data8.extend_from_slice(packed8.as_slice());

                                    if Blocks::is_semi_transparent(neigh_block) {
                                        tops.insert(vec::IVec2{x: i + neigh.x, y: k + neigh.z}, j + neigh.y);
                                    }
                                } else {
                                    tops.insert(vec::IVec2{x: i + neigh.x, y: k + neigh.z}, j + neigh.y);
                                }
                            }
                        }
                        
                    }
                }
            }
        }

        let rm = ReadyMesh::new(index, &chunklock.pos, data32.len() as i32, tdata32.len() as i32);
        let ugqarc = self.finished_user_geo_queue.clone();
        let gqarc = self.finished_geo_queue.clone();

        match user_power {
            true => {
                ugqarc.push(rm);
            }
            false => {
                
                gqarc.push(rm);
            }
        }
        
        

        let tc = self.takencare.clone();

        if !tc.contains_key(&chunklock.pos) {
            tc.insert(chunklock.pos, *chunklock);

        }
    }

    pub fn stamp_here(&self, spot: &vec::IVec3, model: &JVoxModel, implicated: Option<&mut HashSet<IVec2>>) {

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
            let size = i.size;
            for v in &i.voxels {
                let rearr_point = IVec3::new(v.point.x as i32  - (size.x / 2) as i32, v.point.z as i32, v.point.y  as i32  - (size.y / 2) as i32);

                let c_pos = ChunkSystem::spot_to_chunk_pos(&(*spot + rearr_point));
                implicated_chunks.insert(c_pos);
                self.set_block(
                    IVec3::new(spot.x + rearr_point.x, spot.y + rearr_point.y, spot.z + rearr_point.z),
                    (v.color_index.0).clamp(0, Blocks::get_texs_length() as u8) as u32,
                    false
                )
            }
        }
        if !implicated_provided {
            for c in implicated_chunks.iter() {
                match self.takencare.get(&c) {
                    Some(cf) => {
                        self.background_rebuild_requests.push(cf.geo_index);
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

        let dim_floor = Planets::get_floor_block(self.planet_type as u32);

        let dim_range = Planets::get_voxel_model_index_range(self.planet_type as u32);

        //Two rng per chunk! 
        //let spot: u32 = rng.gen_range(0..(CW as u32 * CW as u32)*(CH-40) as u32);
        //let item: u32 = rng.gen_range(dim_range.0 as u32..dim_range.1 as u32);


        //let mut index = 0;
        
        for x in 0..CW {
            for z in 0..CW {
                for y in (0..CH-40).rev() {
                    let coord = IVec3::new(cpos.x * CW + x, y, cpos.y * CW + z);
                    //if index == spot {
                        if self.natural_blockat(coord) == dim_floor {

                            let item: u32 = rng.gen_range(dim_range.0 as u32..dim_range.1 as u32 * 3);
                            if item <= dim_range.1 as u32 && item >= dim_range.0 as u32 {

                                self.stamp_here(&coord, &self.voxel_models.as_ref().unwrap()[item as usize], Some(&mut implicated));
                                
                            }

                                
                            should_break = true;
                            break;
                        }
                    //}
                    
                    //index += 1;
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
                    self.gen_rebuild_requests.push(cf.geo_index);
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

    pub fn noise_func2(&self, spot: vec::IVec3) -> f64 {

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
                    spot.x  as f64 / 25.35,
                    y as f64 / 65.35,
                    spot.z  as f64 / 25.35,
                ]) * 20.0
                - f64::max(y as f64 * 3.0, 0.0),
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

        match self.userdatamap.get(&spot) {
            Some(id) => {
                return *id;
            }
            None => {}
        }

        match self.nonuserdatamap.get(&spot) {
            Some(id) => {
                return *id;
            }
            None => { return self.natural_blockat(spot)}
        }

        
    }

    pub fn natural_blockat(&self, spot: vec::IVec3) -> u32 {
        if spot.y == 0 {
            return 15;
        }
        match self.planet_type {
            1 => {
                if self.noise_func2(spot) > 10.0 {
                    if self.noise_func2(spot + vec::IVec3 { x: 0, y: 1, z: 0 }) < 10.0 {
                        return 14;
                    }
                    return 1;
                } else {
                    return 0;
                }
            }
            _ => {
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
        
    }
}