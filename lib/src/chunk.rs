use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use std::sync::RwLock;

use dashmap::DashMap;

use gl::types::GLuint;
use glam::Vec3;
use num_enum::FromPrimitive;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rusqlite::params;
use rusqlite::Connection;

use std::sync::{Arc, Mutex};

use noise::{NoiseFn, Perlin};

use crate::audio::AudioPlayer;
use crate::chunkregistry::ChunkMemory;
use crate::chunkregistry::ChunkRegistry;
use crate::cube::Cube;
use crate::cube::CubeSide;
use crate::inventory::ChestInventory;

use crate::packedvertex::PackedVertex;
use crate::planetinfo::Planets;
use crate::shader::Shader;
use crate::specialblocks::chest::ChestInfo;
use crate::specialblocks::crafttable::CraftTableInfo;
use crate::specialblocks::door::DoorInfo;
use crate::specialblocks::ladder::LadderInfo;
use crate::specialblocks::tallgrass::TallGrassInfo;
use crate::textureface::TextureFace;
use crate::vec::IVec3;
use crate::vec::{self, IVec2};

use tracing::info;

use crate::blockinfo::Blocks;
use crate::voxmodel::JVoxModel;

use std::io::Write;

pub type LightColor = glam::U16Vec3;

pub struct LightRay {
    pub value: LightColor,
    pub origin: vec::IVec3,
    pub directions: Vec<CubeSide>,
}

pub struct LightSegment {
    pub rays: Vec<LightRay>,
}

impl LightSegment {
    pub fn sum(&self) -> LightColor {
        let mut res = LightColor::ZERO;
        for ray in &self.rays {
            res += ray.value;
        }
        return res.min(LightColor::new(15, 15, 15));
    }
}

pub struct ChunkGeo {
    pub data32: Mutex<Vec<u32>>,
    pub data8: Mutex<Vec<u8>>,
    pub data8rgb: Mutex<Vec<u16>>,

    pub pos: Mutex<vec::IVec2>,

    pub vbo32: gl::types::GLuint,
    pub vbo8: gl::types::GLuint,
    pub vbo8rgb: GLuint,

    pub tdata32: Mutex<Vec<u32>>,
    pub tdata8: Mutex<Vec<u8>>,
    pub tdata8rgb: Mutex<Vec<u16>>,

    pub tvbo32: gl::types::GLuint,
    pub tvbo8: gl::types::GLuint,
    pub tvbo8rgb: GLuint,

    pub vvbo: GLuint,
    pub uvvbo: GLuint,

    pub vdata: Mutex<Vec<f32>>,
    pub uvdata: Mutex<Vec<f32>>,
}
impl ChunkGeo {
    pub fn new() -> ChunkGeo {
        let mut vbo32: gl::types::GLuint = 0;
        let mut vbo8: gl::types::GLuint = 0;
        let mut tvbo32: gl::types::GLuint = 0;
        let mut tvbo8: gl::types::GLuint = 0;
        let mut vbo8rgb: GLuint = 0;
        let mut tvbo8rgb: GLuint = 0;

        let mut vvbo: gl::types::GLuint = 0;
        let mut uvvbo: gl::types::GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut vbo32);
            gl::CreateBuffers(1, &mut vbo8);
            gl::CreateBuffers(1, &mut tvbo32);
            gl::CreateBuffers(1, &mut tvbo8);

            gl::CreateBuffers(1, &mut vvbo);
            gl::CreateBuffers(1, &mut uvvbo);

            gl::CreateBuffers(1, &mut vbo8rgb);
            gl::CreateBuffers(1, &mut tvbo8rgb);

            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!(
                    "OpenGL Error after creating chunk system buffers: {}",
                    error
                );
            }
        }

        ChunkGeo {
            data32: Mutex::new(Vec::new()),
            data8: Mutex::new(Vec::new()),
            data8rgb: Mutex::new(Vec::new()),
            pos: Mutex::new(IVec2 {
                x: 999999,
                y: 999999,
            }),
            vbo32,
            vbo8,
            vbo8rgb,
            tdata32: Mutex::new(Vec::new()),
            tdata8: Mutex::new(Vec::new()),
            tdata8rgb: Mutex::new(Vec::new()),
            tvbo32,
            tvbo8,
            tvbo8rgb,

            vvbo,
            uvvbo,
            vdata: Mutex::new(Vec::new()),
            uvdata: Mutex::new(Vec::new()),
        }
    }

    pub fn clear(&self) {
        self.data32.lock().unwrap().clear();
        self.data8.lock().unwrap().clear();
        self.tdata32.lock().unwrap().clear();
        self.tdata8.lock().unwrap().clear();

        self.vdata.lock().unwrap().clear();
        self.uvdata.lock().unwrap().clear();

        self.data8rgb.lock().unwrap().clear();
        self.tdata8rgb.lock().unwrap().clear();
    }
    pub fn solids(&self) -> (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>, &Mutex<Vec<u16>>) {
        return (&self.data32, &self.data8, &self.data8rgb);
    }
    pub fn transparents(&self) -> (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>, &Mutex<Vec<u16>>) {
        return (&self.tdata32, &self.tdata8, &self.tdata8rgb);
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
    pub newtlength: i32,
    pub newvlength: i32,
}

impl ReadyMesh {
    pub fn new(
        index: usize,
        newpos: &vec::IVec2,
        newlength: i32,
        newtlength: i32,
        newvlength: i32,
    ) -> ReadyMesh {
        ReadyMesh {
            geo_index: index,
            newpos: *newpos,
            newlength,
            newtlength,
            newvlength,
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
    pub light_rebuild_requests: lockfree::queue::Queue<usize>,
    pub background_rebuild_requests: lockfree::queue::Queue<usize>,
    pub userdatamap: DashMap<vec::IVec3, u32>,
    pub nonuserdatamap: DashMap<vec::IVec3, u32>,
    pub justcollisionmap: DashMap<vec::IVec3, u8>,
    pub radius: u8,
    pub perlin: Perlin,
    pub voxel_models: Option<Arc<Vec<JVoxModel>>>,
    pub chunk_memories: Mutex<ChunkRegistry>,
    pub planet_type: u8,
    pub currentseed: RwLock<u32>,
    pub headless: bool,
    pub hashadinitiallightpass: Arc<Mutex<HashMap<vec::IVec2, bool>>>,
    pub lightmap: Arc<Mutex<HashMap<vec::IVec3, LightSegment>>>,
    pub audiop: Option<Arc<RwLock<AudioPlayer>>>,
    pub chest_registry: Arc<DashMap<vec::IVec3, ChestInventory>>,
    pub generated_chunks: Arc<DashMap<vec::IVec2, bool>>,
}

impl ChunkSystem {
    pub fn write_new_udm_entry(&self, spot: vec::IVec3, block: u32) {
        let seed = self.currentseed.read().unwrap();
        let table_name = format!("userdatamap_{}", seed);

        let conn = Connection::open("db").unwrap();

        // Insert userdatamap entries
        let mut stmt = conn
            .prepare(&format!(
                "INSERT OR REPLACE INTO {} (x, y, z, value) VALUES (?, ?, ?, ?)",
                table_name
            ))
            .unwrap();

        stmt.execute(params![spot.x, spot.y, spot.z, block])
            .unwrap();
    }

    pub fn save_current_world_to_file(&self, path: String) {
        let seed = self.currentseed.read().unwrap();
        let table_name = format!("userdatamap_{}", seed);

        let conn = Connection::open("db").unwrap();

        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    x INTEGER,
                    y INTEGER,
                    z INTEGER,
                    value INTEGER,
                    PRIMARY KEY (x, y, z)
                )",
                table_name
            ),
            (),
        )
        .unwrap();

        // Insert userdatamap entries
        let mut stmt = conn
            .prepare(&format!(
                "INSERT OR REPLACE INTO {} (x, y, z, value) VALUES (?, ?, ?, ?)",
                table_name
            ))
            .unwrap();
        for entry in self.userdatamap.iter() {
            stmt.execute(params![
                entry.key().x,
                entry.key().y,
                entry.key().z,
                *entry.value()
            ])
            .unwrap();
        }

        fs::create_dir_all(&path).unwrap();

        // let mut file = File::create(path.clone() + "/udm").unwrap();
        // for entry in self.userdatamap.iter() {
        //     writeln!(file, "{} {}", entry.key(), entry.value()).unwrap();
        // }

        let mut file = File::create(path.clone() + "/seed").unwrap();
        writeln!(file, "{}", self.currentseed.read().unwrap()).unwrap();

        let mut file = File::create(path.clone() + "/pt").unwrap();
        writeln!(file, "{}", self.planet_type).unwrap();
    }

    pub fn save_one_chest_to_file(&self, key: IVec3) {
        let seed = self.currentseed.read().unwrap();
        let table_name = format!("chest_registry_{}", seed);

        match Connection::open("chestdb") {
            Ok(conn) => {
                // Ensure the table exists
                conn.execute(
                    &format!(
                        "CREATE TABLE IF NOT EXISTS {} (
                            x INTEGER,
                            y INTEGER,
                            z INTEGER,
                            dirty BOOLEAN,
                            inventory BLOB,
                            PRIMARY KEY (x, y, z)
                        )",
                        table_name
                    ),
                    (),
                )
                .unwrap();

                // Get the chest inventory for the given key
                if let Some(chest_inventory) = self.chest_registry.get(&key) {
                    let inv_bin = bincode::serialize(&chest_inventory.inv).unwrap();

                    // Update the specific entry in the database
                    let mut stmt = conn.prepare(&format!(
                        "INSERT OR REPLACE INTO {} (x, y, z, dirty, inventory) VALUES (?, ?, ?, ?, ?)",
                        table_name
                    )).unwrap();

                    stmt.execute(params![key.x, key.y, key.z, chest_inventory.dirty, inv_bin])
                        .unwrap();
                } else {
                    info!("No chest inventory found for key {:?}", key);
                }
            }
            Err(_e) => {}
        };
    }

    pub fn save_current_chests_to_file(&self) {
        let seed = self.currentseed.read().unwrap();
        let table_name = format!("chest_registry_{}", seed);

        let conn = Connection::open("chestdb").unwrap();

        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                x INTEGER,
                y INTEGER,
                z INTEGER,
                dirty BOOLEAN,
                inventory BLOB,
                PRIMARY KEY (x, y, z)
            )",
                table_name
            ),
            (),
        )
        .unwrap();

        // Insert chest_registry entries
        let mut stmt = conn
            .prepare(&format!(
                "INSERT OR REPLACE INTO {} (x, y, z, dirty, inventory) VALUES (?, ?, ?, ?, ?)",
                table_name
            ))
            .unwrap();

        for entry in self.chest_registry.iter() {
            let key = entry.key();
            let chest_inventory = entry.value();
            let inv_bin = bincode::serialize(&chest_inventory.inv).unwrap();
            stmt.execute(params![key.x, key.y, key.z, chest_inventory.dirty, inv_bin])
                .unwrap();
        }
    }

    pub fn load_chests_from_file(&self) {
        let seed = self.currentseed.read().unwrap();
        let table_name = format!("chest_registry_{}", seed);

        let conn = Connection::open("chestdb").unwrap();

        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                x INTEGER,
                y INTEGER,
                z INTEGER,
                dirty BOOLEAN,
                inventory BLOB,
                PRIMARY KEY (x, y, z)
            )",
                table_name
            ),
            (),
        )
        .unwrap();

        let mut stmt = conn
            .prepare(&format!(
                "SELECT x, y, z, dirty, inventory FROM {}",
                table_name
            ))
            .unwrap();

        let chest_iter = stmt
            .query_map([], |row| {
                let x: i32 = row.get(0)?;
                let y: i32 = row.get(1)?;
                let z: i32 = row.get(2)?;
                let dirty: bool = row.get(3)?;
                let inventory: Vec<u8> = row.get(4)?;
                let inv: [(u32, u32); 20] = bincode::deserialize(&inventory).unwrap();
                Ok((IVec3 { x, y, z }, ChestInventory { dirty, inv }))
            })
            .unwrap();

        for chest in chest_iter {
            let (coords, chest_inventory) = chest.unwrap();
            self.chest_registry.insert(coords, chest_inventory);
        }
    }

    pub fn load_world_from_file(&mut self, path: String) {
        self.userdatamap.clear();
        self.nonuserdatamap.clear();

        match File::open(format!("{}/udm", path.clone())) {
            Ok(_) => {}
            Err(_) => {
                fs::create_dir_all(&path.clone()).unwrap();
                self.save_current_world_to_file(path.clone());
            }
        }

        let conn = Connection::open("db").unwrap();

        conn.execute_batch(
            "
            PRAGMA synchronous = OFF;
            PRAGMA journal_mode = WAL;
            PRAGMA cache_size = 10000;
        ",
        )
        .unwrap();

        // let file = File::open(format!("{}/udm", path)).unwrap();
        // let reader = BufReader::new(file);

        // for line in reader.lines() {
        //     let line = line.unwrap();
        //     let mut parts = line.splitn(4, ' ');
        //     if let (Some(x), Some(y), Some(z), Some(value)) = (parts.next(), parts.next(), parts.next(), parts.next()) {
        //         let key = format!("{} {} {}", x, y, z);
        //         self.userdatamap.insert(vec::IVec3::from_str(&key).unwrap(), value.parse::<u32>().unwrap());
        //     }
        // }
        let pa = format!("{}/seed2", path);
        if Path::new(&pa).exists() {
            let file = File::open(pa).unwrap();
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line.unwrap();
                let mut parts = line.splitn(2, ' ');
                if let Some(seed) = parts.next() {
                    let s = seed.parse::<u32>().unwrap();
                    info!("Seed Is {}", s);
                    self.perlin = Perlin::new(s);
                    *self.currentseed.write().unwrap() = s;
                }
            }
        } else {
            info!("Seed2 doesnt exist");
        }

        let seed = self.currentseed.read().unwrap();
        let table_name = format!("userdatamap_{}", seed);
        info!("LOADING FROM TABLENAME {}", table_name);

        // Query the userdatamap table
        let mut stmt = conn
            .prepare(&format!("SELECT x, y, z, value FROM {}", table_name))
            .unwrap();

        let userdatamap_iter = stmt
            .query_map([], |row| {
                Ok((
                    vec::IVec3::new(row.get(0)?, row.get(1)?, row.get(2)?),
                    row.get(3)?,
                ))
            })
            .unwrap();

        for entry in userdatamap_iter {
            let (key, value): (vec::IVec3, u32) = entry.unwrap();
            self.userdatamap.insert(key, value);
        }

        let file = File::open(format!("{}/pt", path)).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let mut parts = line.splitn(2, ' ');
            if let Some(pt) = parts.next() {
                self.planet_type = pt.parse::<u8>().unwrap();
            }
        }
    }

    pub fn collision_predicate(&self, vec: vec::IVec3) -> bool {
        let isntwater = (self.blockat(vec.clone()) & Blocks::block_id_bits()) != 2;
        let isnttallgrass = (self.blockat(vec.clone()) & Blocks::block_id_bits()) != 23;

        return isntwater && isnttallgrass && self.blockat(vec.clone()) != 0
            || self.justcollisionmap.contains_key(&vec);
    }

    pub fn start_with_seed(_seed: u32) {}

    pub fn exit(&mut self) {
        if !self.headless {
            for cg in &self.geobank {
                unsafe {
                    gl::DeleteBuffers(1, &cg.vbo32);
                    gl::DeleteBuffers(1, &cg.tvbo32);
                    gl::DeleteBuffers(1, &cg.vbo8);
                    gl::DeleteBuffers(1, &cg.tvbo8);
                }
            }
        }

        info!("After deleting buffers");
        self.chunks.clear();
        info!("After clearing chunks");
        self.geobank.clear();
        info!("After clearing geobank");
        self.chunk_memories.lock().unwrap().memories.clear();
        info!("After clearing memories");
        self.takencare.clear();
        info!("After clearing takencare");
        while let Some(_) = self.finished_geo_queue.pop() {}
        while let Some(_) = self.finished_user_geo_queue.pop() {}
        while let Some(_) = self.user_rebuild_requests.pop() {}
        while let Some(_) = self.gen_rebuild_requests.pop() {}
        while let Some(_) = self.background_rebuild_requests.pop() {}
        info!("After that whole popping thing");
        self.userdatamap.clear();
        self.nonuserdatamap.clear();
        self.justcollisionmap.clear();
        info!("After clearing the next 3 things");
    }

    pub fn reset(&mut self, radius: u8, seed: u32, noisetype: usize) {
        info!("Start of reset func");

        self.radius = radius;
        self.perlin = Perlin::new(seed);
        self.voxel_models = None;
        self.planet_type = noisetype as u8;
        (*self.currentseed.write().unwrap()) = seed;
        info!("After setting currentseed");

        if !self.headless {
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
                    self.chunk_memories
                        .lock()
                        .unwrap()
                        .memories
                        .push(ChunkMemory::new(&self.geobank[self.geobank.len() - 1]));
                }
            }
        }

        info!("After making new chunk stuff");
    }

    pub fn new(
        radius: u8,
        seed: u32,
        noisetype: usize,
        headless: bool,
        audiop: Option<Arc<RwLock<AudioPlayer>>>,
    ) -> ChunkSystem {
        let mut cs = ChunkSystem {
            chunks: Vec::new(),
            geobank: Vec::new(),
            takencare: Arc::new(DashMap::new()),
            finished_user_geo_queue: Arc::new(lockfree::queue::Queue::new()),
            finished_geo_queue: Arc::new(lockfree::queue::Queue::new()),
            user_rebuild_requests: lockfree::queue::Queue::new(),
            gen_rebuild_requests: lockfree::queue::Queue::new(),
            light_rebuild_requests: lockfree::queue::Queue::new(),
            background_rebuild_requests: lockfree::queue::Queue::new(),
            userdatamap: DashMap::new(),
            nonuserdatamap: DashMap::new(),
            justcollisionmap: DashMap::new(),
            radius,
            perlin: Perlin::new(seed),
            voxel_models: None,
            chunk_memories: Mutex::new(ChunkRegistry {
                memories: Vec::new(),
            }),
            planet_type: noisetype as u8,
            currentseed: RwLock::new(0),
            headless,
            hashadinitiallightpass: Arc::new(Mutex::new(HashMap::new())),
            lightmap: Arc::new(Mutex::new(HashMap::new())),
            audiop,
            chest_registry: Arc::new(DashMap::new()),
            generated_chunks: Arc::new(DashMap::new()),
        };

        // let directory_path = "assets/voxelmodels/";

        // for entry in WalkDir::new(directory_path) {
        //     let entry = entry.unwrap();
        //     if entry.file_type().is_file() {
        //         let path_str = entry.path().to_string_lossy().into_owned();
        //         let jv = JVoxModel::new(Box::leak(path_str.into_boxed_str()));
        //         //info!("{:#?}", jv.model);
        //         cs.voxel_models.push(jv);
        //     }
        // }
        if !cs.headless {
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
                    cs.chunk_memories
                        .lock()
                        .unwrap()
                        .memories
                        .push(ChunkMemory::new(&cs.geobank[cs.geobank.len() - 1]));
                }
            }
        }

        //tracing::info!("Amount of chunkgeo buffers: {}", 4 * cs.geobank.len());

        cs
    }
    pub fn spot_to_chunk_pos(spot: &vec::IVec3) -> vec::IVec2 {
        return vec::IVec2 {
            x: (spot.x as f32 / CW as f32).floor() as i32,
            y: (spot.z as f32 / CW as f32).floor() as i32,
        };
    }
    pub fn initial_rebuild_on_main_thread(
        csys: &Arc<RwLock<ChunkSystem>>,
        _shader: &Shader,
        campos: &Vec3,
    ) {
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
        //             //info!("Some user queue");
        //            // info!("Weird!");

        //             let bankarc = csys.geobank[ready.geo_index].clone();

        //             let mut cmemlock = csys.chunk_memories.lock().unwrap();

        //             cmemlock.memories[ready.geo_index].length = ready.newlength;
        //             cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
        //             cmemlock.memories[ready.geo_index].pos = ready.newpos;
        //             cmemlock.memories[ready.geo_index].used = true;

        //             // info!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
        //             // info!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
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
    pub fn queue_geoindex_rerender(&self, geo_index: usize, user_power: bool, light: bool) {
        if light {
            self.light_rebuild_requests.push(geo_index);
            //info!("Pushed light rebuild request");
        } else {
            match user_power {
                true => {
                    self.user_rebuild_requests.push(geo_index);
                }
                false => {
                    self.background_rebuild_requests.push(geo_index);
                }
            }
        }
    }
    pub fn queue_rerender(&self, spot: vec::IVec3, user_power: bool, light: bool) {
        let chunk_key = &Self::spot_to_chunk_pos(&spot);
        match self.takencare.get(chunk_key) {
            Some(cf) => {
                self.queue_geoindex_rerender(cf.geo_index, user_power, light);
            }
            None => {}
        }
    }
    pub fn queue_rerender_with_key(&self, chunk_key: IVec2, user_power: bool, light: bool) {
        match self.takencare.get(&chunk_key) {
            Some(cf) => self.queue_geoindex_rerender(cf.geo_index, user_power, light),
            None => {}
        }
    }
    pub fn set_block_and_queue_rerender(
        &self,
        spot: vec::IVec3,
        block: u32,
        neighbors: bool,
        user_power: bool,
    ) {
        let existingblock = self.blockat(spot);

        self.set_block(spot, block, user_power);

        let blockislight = Blocks::is_light(block);
        let blockwaslight = Blocks::is_light(existingblock);

        let light = blockislight || blockwaslight;

        if !light {
            //If not light still check if it intercepts any lights, we will need to update.

            let mut implicated = HashSet::new();
            for i in Cube::get_neighbors() {
                match self.lightmap.lock().unwrap().get(&(*i + spot)) {
                    Some(k) => {
                        for ray in &k.rays {
                            let chunkofthisraysorigin = ChunkSystem::spot_to_chunk_pos(&ray.origin);
                            // match self.takencare.get(&chunkofthisraysorigin) {
                            //     Some(chunk) => {
                            //         implicated.insert(chunk.geo_index);
                            //     }
                            //     None => {

                            //     }
                            // }
                            implicated.insert(chunkofthisraysorigin);
                        }
                    }
                    None => {}
                }
            }

            for i in implicated {
                self.queue_rerender_with_key(i, true, true);
            }
        }

        if neighbors {
            let mut neighbs: HashSet<vec::IVec2> = HashSet::new();

            for i in Cube::get_neighbors() {
                let thisspot = spot + *i;
                neighbs.insert(ChunkSystem::spot_to_chunk_pos(&thisspot));
            }
            for i in neighbs {
                let here = i;
                self.queue_rerender_with_key(here, user_power, light);
            }
        } else {
            self.queue_rerender(spot, user_power, light);
        }
    }

    pub fn set_block(&self, spot: vec::IVec3, block: u32, user_power: bool) {
        match user_power {
            true => {
                //info!("Has user power, set block to {block}");
                self.userdatamap.insert(spot, block);
            }
            false => {
                //info!("Non user power");
                self.nonuserdatamap.insert(spot, block);
            }
        }
        if !self.headless {
            if block == 0 {
                let wastherebits = self.blockat(spot) & Blocks::block_id_bits();
                self.audiop
                    .as_ref()
                    .unwrap()
                    .write()
                    .unwrap()
                    .play_next_in_series(
                        Blocks::get_place_series(wastherebits),
                        &Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32),
                        &Vec3::ZERO,
                        0.5,
                    );
            } else {
                self.audiop
                    .as_ref()
                    .unwrap()
                    .write()
                    .unwrap()
                    .play_next_in_series(
                        Blocks::get_place_series(block & Blocks::block_id_bits()),
                        &Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32),
                        &Vec3::ZERO,
                        0.5,
                    );
            }
        }
    }

    pub fn set_block_no_sound(&self, spot: vec::IVec3, block: u32, user_power: bool) {
        match user_power {
            true => {
                //info!("Has user power, set block to {block}");
                self.userdatamap.insert(spot, block);
            }
            false => {
                //info!("Non user power");
                self.nonuserdatamap.insert(spot, block);
            }
        }
    }
    pub fn move_and_rebuild(&self, index: usize, cpos: vec::IVec2) {
        //info!("MBeing asked to move and rebuild to {} {}", cpos.x, cpos.y);
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

            //info!("Chunkgeoarc pos set to {} {}", lo.x, lo.y);

            //#[cfg(feature="structures")]
            self.generate_chunk(&lo);

            let hashadlock = self.hashadinitiallightpass.lock().unwrap();
            let mut light = false;
            if !hashadlock.contains_key(&cpos) {
                light = true;
            }
            drop(hashadlock);
            self.rebuild_index(index, false, light);
        } else {
            info!("This path");
            let ind = tc.get(&cpos).unwrap().geo_index;

            let hashadlock = self.hashadinitiallightpass.lock().unwrap();
            let mut light = false;
            if !hashadlock.contains_key(&cpos) {
                light = true;
            }
            drop(hashadlock);
            self.rebuild_index(ind, false, light);
        }
    }

    pub fn depropagate_light_origin(&self, origin: vec::IVec3, imp: &mut HashSet<vec::IVec2>) {
        //info!("Starting depropagating light origin");
        let mut stack: Vec<vec::IVec3> = Vec::new();

        stack.push(origin);

        let lmarc = self.lightmap.clone();
        let mut lmlock = lmarc.lock().unwrap();

        while !stack.is_empty() {
            let spot = stack.pop().unwrap();

            let chunkcoordoforigin = Self::spot_to_chunk_pos(&origin);

            let chunkcoordhere = Self::spot_to_chunk_pos(&spot);

            if chunkcoordoforigin != chunkcoordhere {
                imp.insert(chunkcoordhere);
            }

            match lmlock.get_mut(&spot) {
                Some(k) => {
                    let mut i = 0;
                    while i < k.rays.len() {
                        if k.rays[i].origin == origin {
                            let directions = k.rays[i].directions.clone();
                            //info!("There are {} directions", directions.len());
                            k.rays.remove(i);
                            let neighbs = Cube::get_neighbors();
                            for dir in directions {
                                stack.push(neighbs[dir as usize] + spot);
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
                None => {}
            }
        }
        //info!("Got to end of depropagating light origin");
    }
    pub fn propagate_light_origin(
        &self,
        spot: vec::IVec3,
        origin: vec::IVec3,
        value: LightColor,
        imp: &mut HashSet<vec::IVec2>,
    ) {
        //info!("Starting propagating light origin");
        let mut stack: Vec<(LightColor, vec::IVec3)> = Vec::new();
        let mut visited: HashSet<vec::IVec3> = HashSet::new();

        stack.push((value, spot));
        visited.insert(spot);

        let lmarc = self.lightmap.clone();

        let mut lmlock = lmarc.lock().unwrap();

        while !stack.is_empty() {
            //info!("Stack size is now {}", stack.len());
            let n = stack.pop().unwrap();

            let blockbitshere = self.blockat(n.1);

            //info!("Got the block bits here: {}", blockbitshere);
            let blockidhere = blockbitshere & Blocks::block_id_bits();

            let goinghere = blockidhere == 0
                || Blocks::is_transparent(blockidhere)
                || Blocks::is_semi_transparent(blockidhere)
                || n.1 == origin;

            //info!("Goinghere: {}", goinghere);

            if goinghere {
                let chunkcoordoforigin = Self::spot_to_chunk_pos(&origin);
                let chunkcoordhere = Self::spot_to_chunk_pos(&n.1);

                //info!("Chunk coord of origin: {:?}", chunkcoordoforigin);
                //info!("Chunk coord here: {:?}", chunkcoordhere);

                if chunkcoordoforigin != chunkcoordhere {
                    imp.insert(chunkcoordhere);
                }

                let inner_light_seg;

                match lmlock.get_mut(&n.1) {
                    Some(k2) => inner_light_seg = k2,
                    None => {
                        lmlock.insert(n.1, LightSegment { rays: Vec::new() });
                        inner_light_seg = lmlock.get_mut(&n.1).unwrap();
                    }
                }

                let my_ray_here = match inner_light_seg.rays.iter_mut().find(|r| r.origin == origin)
                {
                    Some(k) => {
                        if k.value.x < n.0.x {
                            k.value.x = n.0.x;
                        }
                        if k.value.y < n.0.y {
                            k.value.y = n.0.y;
                        }
                        if k.value.z < n.0.z {
                            k.value.z = n.0.z;
                        }
                        k
                    }
                    None => {
                        inner_light_seg.rays.push(LightRay {
                            value: n.0,
                            origin,
                            directions: Vec::new(),
                        });
                        inner_light_seg.rays.last_mut().unwrap()
                    }
                };
                drop(my_ray_here);
                drop(inner_light_seg);

                if n.0.x > 1 || n.0.y > 1 || n.0.z > 1 {
                    let neighbs = Cube::get_neighbors();
                    for (index, neigh) in neighbs.iter().enumerate() {
                        let next = n.1 + *neigh;

                        //info!("Checking neighbor at index {}: {:?}", index, next);

                        let existing_new_light_seg;

                        let mut existing_next_value = LightColor::ZERO;

                        match lmlock.get(&next) {
                            Some(k2) => {
                                existing_new_light_seg = k2;

                                for (_ind, ray) in
                                    &mut existing_new_light_seg.rays.iter().enumerate()
                                {
                                    if ray.origin == origin {
                                        existing_next_value = ray.value; //Either this or we'll consider it zero
                                    }
                                }
                            }
                            None => {}
                        }

                        let reducedvalue = LightColor::new(
                            (n.0.x as i32 - 1).max(0) as u16,
                            (n.0.y as i32 - 1).max(0) as u16,
                            (n.0.z as i32 - 1).max(0) as u16,
                        );

                        if !visited.contains(&next)
                            || existing_next_value.x < reducedvalue.x
                            || existing_next_value.y < reducedvalue.y
                            || existing_next_value.z < reducedvalue.z
                        {
                            //info!("Pushing next onto stack: {:?}", next);

                            stack.push((reducedvalue, next));
                            visited.insert(next);

                            let inner_light_seg = lmlock.get_mut(&n.1).unwrap();

                            let my_ray_here = inner_light_seg
                                .rays
                                .iter_mut()
                                .find(|r| r.origin == origin)
                                .unwrap();

                            if !my_ray_here
                                .directions
                                .contains(&CubeSide::from_primitive(index))
                            {
                                my_ray_here.directions.push(CubeSide::from_primitive(index));
                            }
                        }
                    }
                }
            }
        }
        //info!("Got to end of propagating light origin");
    }

    pub fn lightpass_on_chunk(&self, pos: vec::IVec2) {
        //info!("Doing lightpass on chunk!");

        let hashadarc = self.hashadinitiallightpass.clone();
        let mut hashadlock = hashadarc.lock().unwrap();

        hashadlock.insert(pos, true);

        drop(hashadlock);

        let mut implicated: HashSet<vec::IVec2> = HashSet::new();

        let mut lightsources: HashSet<(vec::IVec3, u32)> = HashSet::new();

        let mut existingsources: HashSet<vec::IVec3> = HashSet::new();

        let lmarc = self.lightmap.clone();
        for x in 0..CW {
            for z in 0..CW {
                for y in 0..CH {
                    let blockcoord = IVec3::new(pos.x * CW + x, y, pos.y * CW + z);
                    let lmlock = lmarc.lock().unwrap();
                    match lmlock.get(&blockcoord) {
                        Some(k) => {
                            for ray in &k.rays {
                                let chunkcoord_of_origin = Self::spot_to_chunk_pos(&ray.origin);

                                if chunkcoord_of_origin == pos {
                                    let originweremoving = ray.origin;

                                    existingsources.insert(ray.origin);

                                    let id = self.blockat(originweremoving);

                                    if Blocks::is_light(id) {
                                        lightsources.insert((originweremoving, id));
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                    let id = self.blockat(blockcoord);
                    if Blocks::is_light(id) {
                        lightsources.insert((blockcoord, id));
                    }
                }
            }
        }

        for source in existingsources {
            self.depropagate_light_origin(source, &mut implicated);
        }

        for (source, id) in lightsources {
            self.propagate_light_origin(
                source,
                source,
                Blocks::get_light_color(id),
                &mut implicated,
            );
        }

        //info!("Implicated number: {}", implicated.len());

        for c in implicated.iter() {
            match self.takencare.get(&c) {
                Some(cf) => {
                    self.user_rebuild_requests.push(cf.geo_index);
                }
                None => {}
            }
        }

        //info!("Got to end of lightpass");
    }

    pub fn rebuild_index(&self, index: usize, user_power: bool, light: bool) {
        //info!("Rebuilding!");
        let chunkarc = self.chunks[index].clone();
        let mut chunklock = chunkarc.lock().unwrap();

        if light {
            self.lightpass_on_chunk(chunklock.pos);
        }

        chunklock.used = true;

        let doorbottomuvs = DoorInfo::get_door_uvs(TextureFace::new(11, 0));
        let doortopuvs = DoorInfo::get_door_uvs(TextureFace::new(11, 1));

        let geobankarc = self.geobank[index].clone();
        // if num == 0 { num = 1; } else { num = 0; }

        geobankarc.clear();

        let mut memo: HashMap<vec::IVec3, u32> = HashMap::new();

        let mut data32 = geobankarc.data32.lock().unwrap();
        let mut data8 = geobankarc.data8.lock().unwrap();
        let mut tdata32 = geobankarc.tdata32.lock().unwrap();
        let mut tdata8 = geobankarc.tdata8.lock().unwrap();

        let mut vdata = geobankarc.vdata.lock().unwrap();
        let mut uvdata = geobankarc.uvdata.lock().unwrap();

        let mut data8rgb = geobankarc.data8rgb.lock().unwrap();
        let mut tdata8rgb = geobankarc.tdata8rgb.lock().unwrap();

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
                    let combined = self.blockatmemo(spot, &mut memo);
                    let block = combined & Blocks::block_id_bits();
                    let flags = combined & Blocks::block_flag_bits();
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
                        if block == 19 {
                            let direction = Blocks::get_direction_bits(flags);
                            let open = DoorInfo::get_door_open_bit(flags);
                            let opposite = DoorInfo::get_opposite_door_bits(flags);

                            let mut modelindex: i32;
                            if opposite == 1 {
                                modelindex = direction as i32 - open as i32;
                                if modelindex < 0 {
                                    modelindex = 3;
                                }
                            } else {
                                modelindex = (direction as i32 + open as i32) % 4;
                            }

                            let doortop = DoorInfo::get_door_top_bit(flags);

                            let _blocklightval = 0.0;

                            let lmlock = self.lightmap.lock().unwrap();
                            let blocklighthere = match lmlock.get(&spot) {
                                Some(k) => k.sum(),
                                None => LightColor::ZERO,
                            };

                            let packedrgb = PackedVertex::pack_rgb(
                                blocklighthere.x,
                                blocklighthere.y,
                                blocklighthere.z,
                            );

                            let prgb: u32 =
                                0b0000_0000_0000_0000_0000_0000_0000_0000 | (packedrgb) as u32;
                            drop(lmlock);

                            for vert in
                                DoorInfo::door_model_from_index(modelindex as usize).chunks(5)
                            {
                                vdata.extend_from_slice(&[
                                    vert[0] + spot.x as f32,
                                    vert[1] + spot.y as f32,
                                    vert[2] + spot.z as f32,
                                    f32::from_bits(prgb),
                                    vert[4],
                                ])
                            }

                            if doortop != 0 {
                                uvdata.extend_from_slice(&doortopuvs);
                            } else {
                                uvdata.extend_from_slice(&doorbottomuvs);
                            }
                        } else if block == 20 {
                            let direction = Blocks::get_direction_bits(flags);

                            let modelindex: i32 = direction as i32;

                            let _blocklightval = 0.0;

                            let lmlock = self.lightmap.lock().unwrap();
                            let blocklighthere = match lmlock.get(&spot) {
                                Some(k) => k.sum(),
                                None => LightColor::ZERO,
                            };

                            let packedrgb = PackedVertex::pack_rgb(
                                blocklighthere.x,
                                blocklighthere.y,
                                blocklighthere.z,
                            );

                            let prgb: u32 =
                                0b0000_0000_0000_0000_0000_0000_0000_0000 | (packedrgb) as u32;
                            drop(lmlock);

                            for vert in
                                LadderInfo::ladder_model_from_index(modelindex as usize).chunks(5)
                            {
                                vdata.extend_from_slice(&[
                                    vert[0] + spot.x as f32,
                                    vert[1] + spot.y as f32,
                                    vert[2] + spot.z as f32,
                                    f32::from_bits(prgb),
                                    vert[4],
                                ])
                            }

                            uvdata.extend_from_slice(&LadderInfo::get_ladder_uvs());
                        } else if block == 21 {
                            let direction = Blocks::get_direction_bits(flags);

                            let modelindex: i32 = direction as i32;

                            let _blocklightval = 0.0;

                            let lmlock = self.lightmap.lock().unwrap();
                            let blocklighthere = match lmlock.get(&spot) {
                                Some(k) => k.sum(),
                                None => LightColor::ZERO,
                            };

                            let packedrgb = PackedVertex::pack_rgb(
                                blocklighthere.x,
                                blocklighthere.y,
                                blocklighthere.z,
                            );

                            let prgb: u32 =
                                0b0000_0000_0000_0000_0000_0000_0000_0000 | (packedrgb) as u32;
                            drop(lmlock);

                            for vert in
                                ChestInfo::chest_model_from_index(modelindex as usize).chunks(5)
                            {
                                vdata.extend_from_slice(&[
                                    vert[0] + spot.x as f32,
                                    vert[1] + spot.y as f32,
                                    vert[2] + spot.z as f32,
                                    f32::from_bits(prgb),
                                    vert[4],
                                ])
                            }

                            uvdata.extend_from_slice(&ChestInfo::get_chest_uvs());
                        } else if block == 23 {
                            let modelindex: i32 = 0;

                            let _blocklightval = 0.0;

                            let lmlock = self.lightmap.lock().unwrap();
                            let blocklighthere = match lmlock.get(&spot) {
                                Some(k) => k.sum(),
                                None => LightColor::ZERO,
                            };

                            let packedrgb = PackedVertex::pack_rgb(
                                blocklighthere.x,
                                blocklighthere.y,
                                blocklighthere.z,
                            );

                            let prgb: u32 =
                                0b0000_0000_0000_0000_0000_0000_0000_0000 | (packedrgb) as u32;
                            drop(lmlock);

                            for vert in
                                TallGrassInfo::tallgrass_model_from_index(modelindex as usize)
                                    .chunks(5)
                            {
                                vdata.extend_from_slice(&[
                                    vert[0] + spot.x as f32,
                                    vert[1] + spot.y as f32,
                                    vert[2] + spot.z as f32,
                                    f32::from_bits(prgb),
                                    vert[4],
                                ])
                            }

                            uvdata.extend_from_slice(&TallGrassInfo::get_tallgrass_uvs());
                        } else if block == 31 {
                            let direction = Blocks::get_direction_bits(flags);

                            let modelindex: i32 = direction as i32;

                            let _blocklightval = 0.0;

                            let lmlock = self.lightmap.lock().unwrap();
                            let blocklighthere = match lmlock.get(&spot) {
                                Some(k) => k.sum(),
                                None => LightColor::ZERO,
                            };

                            let packedrgb = PackedVertex::pack_rgb(
                                blocklighthere.x,
                                blocklighthere.y,
                                blocklighthere.z,
                            );

                            let prgb: u32 =
                                0b0000_0000_0000_0000_0000_0000_0000_0000 | (packedrgb) as u32;
                            drop(lmlock);

                            for vert in
                                CraftTableInfo::craft_table_model_from_index(modelindex as usize)
                                    .chunks(5)
                            {
                                vdata.extend_from_slice(&[
                                    vert[0] + spot.x as f32,
                                    vert[1] + spot.y as f32,
                                    vert[2] + spot.z as f32,
                                    f32::from_bits(prgb),
                                    vert[4],
                                ])
                            }

                            uvdata.extend_from_slice(&CraftTableInfo::get_craft_table_uvs());
                        } else {
                            if Blocks::is_transparent(block) || Blocks::is_semi_transparent(block) {
                                for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                    let neighspot = spot + *neigh;
                                    let neigh_block = self.blockatmemo(neighspot, &mut memo)
                                        & Blocks::block_id_bits();
                                    let cubeside = CubeSide::from_primitive(indie);
                                    let neigh_semi_trans = Blocks::is_semi_transparent(neigh_block);
                                    let water_bordering_transparent = block == 2
                                        && neigh_block != 2
                                        && Blocks::is_transparent(neigh_block);

                                    let lmlock = self.lightmap.lock().unwrap();

                                    let blocklighthere = match lmlock.get(&neighspot) {
                                        Some(k) => k.sum(),
                                        None => LightColor::ZERO,
                                    };

                                    // if blocklighthere != 0 {
                                    //     info!("Block light here: {}", blocklighthere);
                                    // }
                                    drop(lmlock);

                                    hit_block = match tops.get(&vec::IVec2 {
                                        x: i + neigh.x,
                                        y: k + neigh.z,
                                    }) {
                                        Some(t) => *t > j + neigh.y,
                                        None => false,
                                    };

                                    if neigh_block == 0
                                        || neigh_semi_trans
                                        || water_bordering_transparent
                                    {
                                        let side = Cube::get_side(cubeside);
                                        let mut packed32: [u32; 6] = [0, 0, 0, 0, 0, 0];
                                        let mut packed8: [u8; 6] = [0, 0, 0, 0, 0, 0];
                                        let mut packed8rgb: [u16; 6] = [0, 0, 0, 0, 0, 0];

                                        let texcoord = Blocks::get_tex_coords(block, cubeside);
                                        for (ind, v) in side.chunks(4).enumerate() {
                                            static AMB_CHANGES: [u8; 4] = [0, 3, 6, 10];

                                            let amb_spots: &[vec::IVec3; 3] =
                                                Cube::get_amb_occul_spots(cubeside, ind as u8);

                                            let amb_change = amb_spots
                                                .iter()
                                                .map(|vec| self.blockatmemo(*vec + spot, &mut memo))
                                                .filter(|&result| result != 0)
                                                .count();

                                            let base_light: i32 =
                                                v[3] as i32 - AMB_CHANGES[amb_change] as i32; // Perform calculations as i32
                                            let adjusted_light: i32 = if hit_block {
                                                base_light - 3
                                            } else {
                                                base_light
                                            };
                                            let clamped_light: u8 =
                                                adjusted_light.clamp(0, 15) as u8; // Clamp in i32 context, then cast to u8

                                            let pack = PackedVertex::pack(
                                                i as u8 + v[0],
                                                j as u8 + v[1],
                                                k as u8 + v[2],
                                                ind as u8,
                                                clamped_light,
                                                0u8, //TEMPORARY UNUSED
                                                texcoord.0,
                                                texcoord.1,
                                            );

                                            let packedcolor = PackedVertex::pack_rgb(
                                                blocklighthere.x,
                                                blocklighthere.y,
                                                blocklighthere.z,
                                            );

                                            packed32[ind] = pack.0;
                                            packed8[ind] = pack.1;
                                            packed8rgb[ind] = packedcolor;
                                        }

                                        tdata32.extend_from_slice(packed32.as_slice());
                                        tdata8.extend_from_slice(packed8.as_slice());
                                        tdata8rgb.extend_from_slice(packed8rgb.as_slice());
                                    } else {
                                        tops.insert(
                                            vec::IVec2 {
                                                x: i + neigh.x,
                                                y: k + neigh.z,
                                            },
                                            j + neigh.y,
                                        );
                                    }
                                }
                            } else {
                                for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                    let neighspot = spot + *neigh;
                                    let neigh_block = self.blockatmemo(neighspot, &mut memo)
                                        & Blocks::block_id_bits();

                                    let cubeside = CubeSide::from_primitive(indie);
                                    let neighbor_transparent = Blocks::is_transparent(neigh_block)
                                        || Blocks::is_semi_transparent(neigh_block);

                                    hit_block = match tops.get(&vec::IVec2 {
                                        x: i + neigh.x,
                                        y: k + neigh.z,
                                    }) {
                                        Some(t) => *t > j + neigh.y,
                                        None => false,
                                    };

                                    let lmlock = self.lightmap.lock().unwrap();

                                    let blocklighthere = match lmlock.get(&neighspot) {
                                        Some(k) => k.sum(),
                                        None => LightColor::ZERO,
                                    };
                                    // if blocklighthere != 0 {
                                    //     info!("Block light here: {}", blocklighthere);
                                    // }

                                    drop(lmlock);

                                    if neigh_block == 0 || neighbor_transparent {
                                        let side = Cube::get_side(cubeside);
                                        let mut packed32: [u32; 6] = [0, 0, 0, 0, 0, 0];
                                        let mut packed8: [u8; 6] = [0, 0, 0, 0, 0, 0];
                                        let mut packed8rgb: [u16; 6] = [0, 0, 0, 0, 0, 0];

                                        let texcoord = Blocks::get_tex_coords(block, cubeside);
                                        for (ind, v) in side.chunks(4).enumerate() {
                                            static AMB_CHANGES: [u8; 4] = [0, 3, 6, 10];

                                            let amb_spots: &[vec::IVec3; 3] =
                                                Cube::get_amb_occul_spots(cubeside, ind as u8);

                                            let amb_change = amb_spots
                                                .iter()
                                                .map(|vec| self.blockatmemo(*vec + spot, &mut memo))
                                                .filter(|&result| result != 0)
                                                .count();

                                            let base_light: i32 =
                                                v[3] as i32 - AMB_CHANGES[amb_change] as i32; // Perform calculations as i32
                                            let adjusted_light: i32 = if hit_block {
                                                base_light - 3
                                            } else {
                                                base_light
                                            };
                                            let clamped_light: u8 =
                                                adjusted_light.clamp(0, 15) as u8; // Clamp in i32 context, then cast to u8

                                            let pack = PackedVertex::pack(
                                                i as u8 + v[0],
                                                j as u8 + v[1],
                                                k as u8 + v[2],
                                                ind as u8,
                                                clamped_light,
                                                0u8, //TEMPORARY UNUSED
                                                texcoord.0,
                                                texcoord.1,
                                            );
                                            let packedcolor = PackedVertex::pack_rgb(
                                                blocklighthere.x,
                                                blocklighthere.y,
                                                blocklighthere.z,
                                            );

                                            packed32[ind] = pack.0;
                                            packed8[ind] = pack.1;
                                            packed8rgb[ind] = packedcolor;
                                        }

                                        data32.extend_from_slice(packed32.as_slice());
                                        data8.extend_from_slice(packed8.as_slice());
                                        data8rgb.extend_from_slice(packed8rgb.as_slice());

                                        if Blocks::is_semi_transparent(neigh_block) {
                                            tops.insert(
                                                vec::IVec2 {
                                                    x: i + neigh.x,
                                                    y: k + neigh.z,
                                                },
                                                j + neigh.y,
                                            );
                                        }
                                    } else {
                                        tops.insert(
                                            vec::IVec2 {
                                                x: i + neigh.x,
                                                y: k + neigh.z,
                                            },
                                            j + neigh.y,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let rm = ReadyMesh::new(
            index,
            &chunklock.pos,
            data32.len() as i32,
            tdata32.len() as i32,
            vdata.len() as i32,
        );
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

    pub fn stamp_here(
        &self,
        spot: &vec::IVec3,
        model: &JVoxModel,
        implicated: Option<&mut HashSet<IVec2>>,
    ) {
        let mut local_implicated_chunks; // Declare a mutable local HashSet for when None is provided
        let implicated_chunks; // This will be the reference used throughout the function
        let mut implicated_provided = false;

        // Determine which HashSet to use
        match implicated {
            Some(hs) => {
                implicated_chunks = hs;
                implicated_provided = true;
            }
            None => {
                local_implicated_chunks = HashSet::new(); // Create a new HashSet when None is provided
                implicated_chunks = &mut local_implicated_chunks; // Use the local HashSet
            }
        };

        for i in &model.model.models {
            let size = i.size;
            for v in &i.voxels {
                let rearr_point = IVec3::new(
                    v.point.x as i32 - (size.x / 2) as i32,
                    v.point.z as i32,
                    v.point.y as i32 - (size.y / 2) as i32,
                );

                let c_pos = ChunkSystem::spot_to_chunk_pos(&(*spot + rearr_point));
                implicated_chunks.insert(c_pos);
                self.set_block_no_sound(
                    IVec3::new(
                        spot.x + rearr_point.x,
                        spot.y + rearr_point.y,
                        spot.z + rearr_point.z,
                    ),
                    (v.color_index.0).clamp(0, Blocks::get_texs_length() as u8) as u32,
                    false,
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
            (cpos.x % 255) as u8,
            (cpos.y % 255) as u8,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]; // This needs to be a fixed-size array of bytes (u8).

        // Create a new RNG instance with the seed.
        let mut rng = StdRng::from_seed(seed);

        // // Generate some random numbers
        // let rand_number1: u32 = rng.gen();
        // let rand_number2: u32 = rng.gen();
        let mut implicated: HashSet<vec::IVec2> = HashSet::new();

        let should_break = false;

        let dim_floors = Planets::get_floor_blocks(self.planet_type as u32);

        let dim_range = Planets::get_voxel_model_index_range(self.planet_type as u32);

        //Two rng per chunk!
        //let spot: u32 = rng.gen_range(0..(CW as u32 * CW as u32)*(CH-40) as u32);
        //let item: u32 = rng.gen_range(dim_range.0 as u32..dim_range.1 as u32);

        //let mut index = 0;

        for x in 0..CW {
            for z in 0..CW {
                for y in (0..CH - 40).rev() {
                    let coord = IVec3::new(cpos.x * CW + x, y, cpos.y * CW + z);
                    //if index == spot {
                    if dim_floors.contains(&self.natural_blockat(coord)) {
                        let featnoise = self.feature_noise(IVec2 {
                            x: coord.x * 20,
                            y: coord.z * 20,
                        }) * 20.0;
                        if featnoise > 0.0 {
                            let item: u32 = (featnoise as u32 - dim_range.0 as u32) as u32;
                            let item2: u32 = rng.gen_range(0..128);

                            if item <= dim_range.1 as u32
                                && item >= dim_range.0 as u32
                                && item2 >= 127 as u32
                            {
                                self.stamp_here(
                                    &coord,
                                    &self.voxel_models.as_ref().unwrap()[item as usize],
                                    Some(&mut implicated),
                                );
                            }
                        }

                        // should_break = true;
                        // break;
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

    pub fn biome_noise(&self, spot: vec::IVec2) -> f64 {
        const XZDIVISOR1: f64 = 100.35 * 4.0;

        let y = 20;

        let noise1 = f64::max(
            0.0,
            self.perlin.get([
                spot.x as f64 / XZDIVISOR1,
                y as f64,
                spot.y as f64 / XZDIVISOR1,
            ]),
        );

        noise1
    }

    pub fn ore_noise(&self, spot: vec::IVec3) -> f64 {
        const XYZDIVISOR: f64 = 15.53;

        let noise1 = f64::max(
            0.0,
            self.perlin.get([
                spot.x as f64 / XYZDIVISOR,
                spot.y as f64 / XYZDIVISOR,
                spot.z as f64 / XYZDIVISOR,
            ]),
        );

        noise1 * ((60.0 - spot.y as f64).max(0.0) / 7.0)
    }

    pub fn feature_noise(&self, spot: vec::IVec2) -> f64 {
        const XZDIVISOR1: f64 = 45.35 * 4.0;

        let y = 20;

        let noise1 = f64::max(
            0.0,
            self.perlin.get([
                (spot.x as f64 + 200.0) / XZDIVISOR1,
                y as f64,
                spot.y as f64 / XZDIVISOR1,
            ]),
        );

        noise1
    }

    pub fn cave_noise(&self, spot: vec::IVec3) -> f64 {
        const XZDIVISOR1: f64 = 25.35;

        let noise1 = f64::max(
            0.0,
            self.perlin.get([
                (spot.x as f64) / XZDIVISOR1,
                (spot.y as f64) / XZDIVISOR1,
                spot.z as f64 / XZDIVISOR1,
            ]),
        );

        noise1
    }

    pub fn noise_func(&self, spot: vec::IVec3) -> f64 {
        let xzdivisor1 = 600.35 * 4.0;
        let _xzdivisor2 = 1000.35 * 4.0;

        let mut y = spot.y - 20;

        let noise1 = f64::max(
            0.0,
            20.0 + self.perlin.get([
                spot.x as f64 / xzdivisor1,
                y as f64 / 200.35,
                spot.z as f64 / xzdivisor1,
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
                spot.x as f64 / 100.35,
                y as f64 / 50.35,
                spot.z as f64 / 100.35,
            ]) * 10.0
                + self.perlin.get([
                    spot.x as f64 / 300.35,
                    y as f64 / 100.35,
                    spot.z as f64 / 300.35,
                ]) * 10.0
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
        let noisemix = ChunkSystem::mix(noise1, noise2, p);

        let noise3 = f64::max(
            0.0,
            50.0 + self.perlin.get([
                spot.x as f64 / 25.35,
                y as f64 / 25.35,
                spot.z as f64 / 25.35,
            ]) * 10.0
                + self.perlin.get([
                    spot.x as f64 / 60.35,
                    y as f64 / 50.35,
                    spot.z as f64 / 60.35,
                ]) * 10.0
                - f64::max(y as f64 / 3.0, 0.0),
        );

        let mut p2 = self.perlin.get([
            (spot.x as f64 + 4500.0) / 250.0,
            (spot.y as f64 + 5000.0) / 250.0,
            (spot.z as f64 - 5000.0) / 250.0,
        ]) * 10.0;

        p2 = f64::max(p2, 0.0);
        p2 = f64::min(p2, 1.0);

        ChunkSystem::mix(noisemix, noise3, p2)
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
                    spot.x as f64 / 25.35,
                    y as f64 / 65.35,
                    spot.z as f64 / 25.35,
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
            Some(b) => *b,
            None => {
                let b = self.blockat(spot);
                memo.insert(spot, b);
                b
            }
        };

        // if memo.contains_key(&spot) {
        //     return *memo.get(&spot).unwrap();
        // } else {
        //     let b = self.blockat(spot);
        //     memo.insert(spot, b);
        //     return b;
        // }
    }

    pub fn blockat(&self, spot: vec::IVec3) -> u32 {
        // if self.headless {
        //     if self.generated_chunks.contains_key(&ChunkSystem::spot_to_chunk_pos(&spot)) {

        //     } else {
        //         self.generate_chunk(&ChunkSystem::spot_to_chunk_pos(&spot))
        //     }
        // }

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
            None => return self.natural_blockat(spot),
        }
    }

    pub fn natural_blockat(&self, spot: vec::IVec3) -> u32 {
        if spot.y == 0 {
            return 15;
        }

        if self.cave_noise(spot) > 0.5 {
            return 0;
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
                static WL: f32 = 60.0;

                let biomenum = self.biome_noise(IVec2 {
                    x: spot.x,
                    y: spot.z,
                });
                let biomenum2 = self.biome_noise(IVec2 {
                    x: spot.x * 20 + 5000,
                    y: spot.z * 20 + 5000,
                });

                let mut underdirt = 5;
                let mut surface = 3;
                let mut undersurface = 4;
                let mut liquid = 2;
                let mut beach = 1;

                if biomenum > 0.0 {
                    underdirt = 1;
                    surface = 1;
                    undersurface = 1;
                    liquid = 2;
                    beach = 1;
                } else {
                    if biomenum2 > 0.0 {
                        surface = 34;
                    }
                }

                if self.noise_func(spot) > 10.0 {
                    if self.noise_func(spot + vec::IVec3 { x: 0, y: 10, z: 0 }) > 10.0 {
                        if self.ore_noise(spot) > 1.0 {
                            return 35;
                        } else {
                            return underdirt;
                        }
                    }
                    if spot.y > (WL + 2.0) as i32
                        || self.noise_func(spot + vec::IVec3 { x: 0, y: 5, z: 0 }) > 10.0
                    {
                        if self.noise_func(spot + vec::IVec3 { x: 0, y: 1, z: 0 }) < 10.0 {
                            return surface;
                        }
                        return undersurface;
                    } else {
                        return beach;
                    }
                } else {
                    if spot.y < WL as i32 {
                        return liquid;
                    } else {
                        return 0;
                    }
                }
            }
        }
    }
}
