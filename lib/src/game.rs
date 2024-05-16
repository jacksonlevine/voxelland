use core::time;
use std::cmp::max;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::io::Write;
use std::slice::Chunks;
use gl::types::{GLenum, GLuint};
use glam::{Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::glfwGetTime;
use glfw::{Action, Key, MouseButton, PWindow};
use gltf::Gltf;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use vox_format::types::Model;
use walkdir::WalkDir;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, current};

use crate::audio::{self, AudioPlayer};
use crate::blockinfo::Blocks;
use crate::blockoverlay::BlockOverlay;
use crate::chunk::{ChunkFacade, ChunkSystem};

use crate::camera::Camera;
use crate::collisioncage::*;
use crate::cube::Cube;
use crate::drops::Drops;
use crate::fader::Fader;
use crate::glyphface::GlyphFace;
use crate::guisystem::GuiSystem;
use crate::hud::{Hud, HudElement};
use crate::modelentity::ModelEntity;
use crate::network::NetworkConnector;
use crate::planetinfo::Planets;
use crate::raycast::*;
use crate::selectcube::SelectCube;
use crate::server_types::{Message, MessageType};
use crate::shader::Shader;
use crate::texture::Texture;
use crate::textureface::TextureFace;
use crate::vec::{self, IVec2, IVec3};
use crate::voxmodel::JVoxModel;
use crate::worldgeometry::WorldGeometry;
use std::sync::RwLock;
#[derive(Clone)]
pub struct AnimationChannel {
    pub node_index: usize,
    pub property: gltf::animation::Property,
    pub keyframes: Vec<(f32, Vec<f32>)>,
}

#[derive(Clone)]
pub struct Animation {
    pub channels: Vec<AnimationChannel>,
    pub name: String,
}
#[derive(Clone)]
pub struct Joint {
    pub node_index: usize,
    pub inverse_bind_matrix: Mat4,
}
#[derive(Clone)]
pub struct Skin {
    pub joints: Vec<Joint>,
}
#[derive(Clone)]
pub struct Node {
    pub transform: Mat4,
    pub children: Vec<usize>,
}


static REQUIRED_SHIP_FLYAWAY_HEIGHT: f32 = -560.0;

pub struct ControlsState {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub back: bool,
    pub up: bool,
    pub lookingleft: bool,
    pub lookingright: bool
}

impl ControlsState {
    pub fn new() -> ControlsState {
        ControlsState {
            left: false,
            right: false,
            forward: false,
            back: false,
            up: false,
            lookingleft: false,
            lookingright: false
        }
    }
    pub fn clear(&mut self) {
        self.left = false;
        self.right = false;
        self.forward = false;
        self.back = false;
        self.up = false;
        self.lookingleft = false;
        self.lookingright = false;
    }
}

pub struct GameVariables {
    first_mouse: bool,
    mouse_focused: bool,
    sensitivity: f32,
    sky_color: Vec4,
    sky_bottom: Vec4,
    mouse_clicked: bool,
    right_mouse_clicked: bool,
    hostile_world: bool,
    hostile_world_sky_color: Vec4,
    hostile_world_sky_bottom: Vec4,
    ship_going_up: bool,
    ship_going_down: bool,
    break_time: f32,
    near_ship: bool,
    ship_taken_off: bool,
    on_new_world: bool
}

pub struct Inventory {
    pub dirty: bool,
    pub inv: [(u32, u32); 5]
}

pub struct Game {
    pub chunksys: Arc<RwLock<ChunkSystem>>,
    pub shader0: Shader,
    pub skyshader: Shader,
    pub modelshader: Shader,
    pub camera: Arc<Mutex<Camera>>,
    pub run_chunk_thread: Arc<AtomicBool>,
    pub chunk_thread: Option<thread::JoinHandle<()>>,
    pub vars: GameVariables,
    pub controls: ControlsState,
    pub faders: Arc<RwLock<Vec<Fader>>>,
    pub prev_time: f32,
    pub delta_time: f32,
    pub user_bound_box: BoundBox,
    pub coll_cage: CollCage,
    pub grounded: bool,
    pub jumping_up: bool,
    pub time_falling_scalar: f32,
    pub current_jump_y: f32,
    pub allowable_jump_height: f32,
    pub initial_timer: f32,
    pub voxel_models: Arc<Vec<JVoxModel>>,
    pub gltf_models: Vec<(gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>)>,
    pub gltf_vbos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_vaos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_counts: Vec<Vec<Vec<usize>>>,
    pub gltf_drawmodes:Vec<Vec<Vec<GLenum>>>,
    pub gltf_ebos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_textures: Vec<Vec<Vec<GLuint>>>,
    pub gltf_paths: Vec<String>,
    pub static_model_entities: Vec<ModelEntity>,
    pub non_static_model_entities: Vec<ModelEntity>,
    pub select_cube: SelectCube,
    pub block_overlay: BlockOverlay,
    pub ship_pos: Vec3,
    pub planet_y_offset: f32,
    pub window: Arc<RwLock<PWindow>>,
    pub guisys: GuiSystem,
    pub hud: Hud,
    pub drops: Drops,
    pub audiop: AudioPlayer,
    pub inventory: Arc<RwLock<Inventory>>,
    pub animations: Vec<Vec<Animation>>,
    pub skins: Vec<Skin>,
    pub nodes: Vec<Vec<Node>>,
    pub current_time: f32,
    pub netconn: NetworkConnector
}

enum FaderNames {
    FovFader = 0,
}

impl Game {
    pub fn new(window: &Arc<RwLock<PWindow>>) -> Game {
        let shader0 = Shader::new("assets/vert.glsl", "assets/frag.glsl");
        let skyshader = Shader::new("assets/skyvert.glsl", "assets/skyfrag.glsl");
        let faders: RwLock<Vec<Fader>> = RwLock::new(Vec::new());
        let cam = Arc::new(Mutex::new(Camera::new()));

        faders
            .write()
            .unwrap()
            .extend(vec![Fader::new(83.0, 80.0, 30.0, false)]);

        unsafe {
            gl::BindVertexArray(shader0.vao);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL Error after binding vertex array: {}", error);
            }
        }
        let tex = Texture::new("assets/world.png").unwrap();
        tex.add_to_unit(0);

        let voxel_models = vec! [
            JVoxModel::new("assets/voxelmodels/bush.vox"),
            JVoxModel::new("assets/voxelmodels/tree1.vox"),
            JVoxModel::new("assets/voxelmodels/tree2.vox"),
            JVoxModel::new("assets/voxelmodels/rock1.vox"),
            JVoxModel::new("assets/voxelmodels/rock2.vox"),

            JVoxModel::new("assets/voxelmodels/ptree.vox"),
            JVoxModel::new("assets/voxelmodels/redrock.vox"),
            JVoxModel::new("assets/voxelmodels/crystal1.vox"),
        ];

        
        let mut rng = StdRng::from_entropy();
        //let seed = rng.gen_range(0..229232);


        let mut csys = ChunkSystem::new(10, 0, 0);

        csys.load_world_from_file(String::from("saves/world1"));
        //self.vars.hostile_world = false;
        let seed = *csys.currentseed.read().unwrap();
        //self.start_chunks_with_radius(10, seed, 0);
        //self.camera.lock().unwrap().position = Vec3::new(0.0, 100.0, 0.0);



        let vmarc = Arc::new(voxel_models);
        let vmarc2 = vmarc.clone();

        csys.voxel_models = Some(vmarc);

        let chunksys = Arc::new(RwLock::new(csys));

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool> = {
            let csys_arc = Arc::clone(&chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.read().unwrap().collision_predicate(v);
            })
        };
        let mut hud = Hud::new(&window.clone(), tex.id);

        
        let tf = TextureFace::new(0, 14);
        //IMPORTANT: Push these first, the inv row slots
        for i in 0..5 {
            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + i as f32 * 0.10, -0.9), Vec2::new(0.15, 0.15), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ]);

            hud.elements.push(invrowel);
        }

        let tf = TextureFace::new(0, 0);
        //The item textures on top of them
        for i in 0..5 {
            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + i as f32 * 0.10, -0.9), Vec2::new(0.10, 0.10), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ]);

            hud.elements.push(invrowel);
        }


        let tf = TextureFace::new(0, 0);
        //The number textures on top of them
        for i in 0..5 {
            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + 0.01 + i as f32 * 0.10, -0.93), Vec2::new(0.05, 0.05), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ]);
            hud.elements.push(invrowel);

            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + 0.02 + i as f32 * 0.10, -0.93), Vec2::new(0.05, 0.05), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ]);

            hud.elements.push(invrowel);
        }



        let tf = TextureFace::new(0, 13);

        hud.elements.push(HudElement::new(Vec2::new(0.0, 0.0), Vec2::new(0.08, 0.08), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ]));     
        

        let inv = Arc::new(RwLock::new(Inventory{
            dirty: true,
            inv: [
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0)
            ]
        }));

        let mut g = Game {
            chunksys: chunksys.clone(),
            shader0,
            skyshader,
            modelshader: Shader::new("assets/mvert.glsl", "assets/mfrag.glsl"),
            camera: cam.clone(),
            run_chunk_thread: Arc::new(AtomicBool::new(true)),
            chunk_thread: None,
            vars: GameVariables {
                first_mouse: true,
                mouse_focused: false,
                sensitivity: 0.25,
                sky_color: Vec4::new(0.5, 0.7, 1.0, 1.0),
                sky_bottom: Vec4::new(1.0, 1.0, 1.0, 1.0),
                mouse_clicked: false,
                right_mouse_clicked: false,
                hostile_world: false,
                hostile_world_sky_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                hostile_world_sky_bottom: Vec4::new(1.0, 0.0, 0.0, 1.0),
                ship_going_up: false,
                ship_going_down: false,
                break_time: 0.0,
                near_ship: false,
                ship_taken_off: true,
                on_new_world: true
            },
            controls: ControlsState::new(),
            faders: Arc::new(faders),
            prev_time: 0.0,
            delta_time: 0.0,
            user_bound_box: BoundBox::new(Vec3::new(0.0, 0.0, 0.0)),
            coll_cage: CollCage::new(solid_pred),
            grounded: false,
            jumping_up: false,
            time_falling_scalar: 1.0,
            current_jump_y: 0.0,
            allowable_jump_height: 1.1,
            initial_timer: 0.0,
            voxel_models: vmarc2,
            gltf_models: Vec::new(),
            gltf_vbos: Vec::new(),
            gltf_vaos: Vec::new(),
            gltf_counts: Vec::new(),
            gltf_drawmodes: Vec::new(),
            gltf_ebos: Vec::new(),
            gltf_textures: Vec::new(),
            gltf_paths: Vec::new(),
            static_model_entities: Vec::new(),
            non_static_model_entities: Vec::new(),
            select_cube: SelectCube::new(),
            block_overlay: BlockOverlay::new(tex.id),
            ship_pos: Vec3::new(0.0,0.0,0.0),
            planet_y_offset: REQUIRED_SHIP_FLYAWAY_HEIGHT,
            window: window.clone(),
            guisys: GuiSystem::new(&window.clone(), &tex),
            hud,
            drops: Drops::new(tex.id, &cam, &chunksys, &inv),
            audiop: AudioPlayer::new().unwrap(),
            inventory: inv,
            animations: Vec::new(),
            skins: Vec::new(),
            nodes: Vec::new(),
            current_time: 0.0,
            netconn: NetworkConnector::new()
        };


        g.load_model("assets/models/car/scene.gltf");
        g.load_model("assets/models/ship/scene.gltf");
        g.load_model("assets/models/monster1/scene.gltf");
        g.load_model("assets/models/monster2/scene.gltf");
        g.load_model("assets/models/ship2/scene.gltf");
        g.create_model_vbos();
    
        // g.setup_vertex_attributes();

        //start coming down from the sky in ship
        g.vars.ship_going_down = true;
        g.vars.ship_going_up = false;

        g.netconn.connect(String::from("127.0.0.1:6969"));
        println!("Connected to the server!");

        let message = Message {
            message_type: MessageType::RequestSeed,
            x: 1.0,
            y: 2.0,
            z: 3.0,
            rot: 4.0,
            info: 42,
        };
    
        g.netconn.send(&message);

        println!("Sent message to the server!");


        g.audiop.preload_series("grassstepseries", vec![
            "assets/sfx/grassstep1.mp3",
            "assets/sfx/grassstep2.mp3",
            "assets/sfx/grassstep3.mp3",
            "assets/sfx/grassstep4.mp3",
            "assets/sfx/grassstep5.mp3",
            "assets/sfx/grassstep6.mp3",
        ]);

        g.audiop.preload_series("stonestepseries", vec![
            "assets/sfx/stonestep1.mp3",
           "assets/sfx/stonestep2.mp3",
           "assets/sfx/stonestep3.mp3",
            "assets/sfx/stonestep4.mp3"
        ]);


        let mut ship_pos = vec::IVec3::new(20,200,0);
        let mut ship_front = vec::IVec3::new(30,200,0);
        let mut ship_back = vec::IVec3::new(10,200,0);
         // Function to decrement y until a block is found
        fn find_ground_y(position: &mut vec::IVec3, game: &Game) {
            while game.chunksys.read().unwrap().blockat(*position) == 0 {
                position.y -= 1;
            }
        }

        // Find the ground positions
        find_ground_y(&mut ship_pos, &g);
        find_ground_y(&mut ship_front, &g);
        find_ground_y(&mut ship_back, &g);



        // Determine the highest y position found
        let decided_pos_y = max(max(ship_pos.y, ship_front.y), ship_back.y) + 10;

        // Update the ship's position
        ship_pos.y = decided_pos_y;

 


        let ship_float_pos = Vec3::new(ship_pos.x as f32, ship_pos.y as f32, ship_pos.z as f32);


        //ChunkSystem::initial_rebuild_on_main_thread(&g.chunksys.clone(), &g.shader0, &g.camera.lock().unwrap().position);
        g.rebuild_whole_world_while_showing_loading_screen();

        g.audiop.play("assets/music/Farfromhome.mp3", &ship_float_pos, &Vec3::new(0.0,0.0,0.0));
        g.audiop.play("assets/sfx/shipland28sec.mp3", &ship_float_pos, &Vec3::new(0.0,0.0,0.0));



        g.ship_pos = ship_float_pos;
        //g.static_model_entities.push(ModelEntity::new(1, ship_float_pos, 0.07, Vec3::new(PI/2.0, 0.0, 0.0), &g.chunksys, &g.camera));
        g.static_model_entities.push(ModelEntity::new(4, ship_float_pos, 1.5, Vec3::new(0.0, 0.0, 0.0), &g.chunksys, &g.camera));
        g.camera.lock().unwrap().position = ship_float_pos  + Vec3::new(5.0, 2.0, 0.0);
        g.add_ship_colliders();
        
        g
    }

    pub fn update_inventory(&mut self) {
        for i in 5..10 {
            let realslotind = i - 5;
            let slot = self.inventory.read().unwrap().inv[realslotind];
            let idinslot = slot.0;
            let texcoords = Blocks::get_tex_coords(idinslot, crate::cube::CubeSide::LEFT);
            let tf = TextureFace::new(texcoords.0 as i8, texcoords.1 as i8);
            let bf = TextureFace::new(0,0);
            self.hud.elements[i as usize].uvs = [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ];

            if slot.1 > 0 {
                let count = slot.1.to_string();
                if count.len() == 2 {
                    let g1 = GlyphFace::new(count.as_bytes()[0]);
                    let g2 = GlyphFace::new(count.as_bytes()[1]);

                    self.hud.elements[10 + realslotind * 2].uvs = [
                        g1.blx, g1.bly,
                        g1.brx, g1.bry,
                        g1.trx, g1.tr_y,

                        g1.trx, g1.tr_y,
                        g1.tlx, g1.tly,
                        g1.blx, g1.bly
                    ];
                    self.hud.elements[10 + realslotind * 2 + 1].uvs = [
                        g2.blx, g2.bly,
                        g2.brx, g2.bry,
                        g2.trx, g2.tr_y,

                        g2.trx, g2.tr_y,
                        g2.tlx, g2.tly,
                        g2.blx, g2.bly
                    ];
                }

                if count.len() == 1 {
                    let g2 = GlyphFace::new(count.as_bytes()[0]);
                    self.hud.elements[10 + realslotind * 2].uvs = [
                        bf.blx, bf.bly,
                        bf.brx, bf.bry,
                        bf.trx, bf.tr_y,

                        bf.trx, bf.tr_y,
                        bf.tlx, bf.tly,
                        bf.blx, bf.bly
                    ];
                    self.hud.elements[10 + realslotind * 2 + 1].uvs = [
                        g2.blx, g2.bly,
                        g2.brx, g2.bry,
                        g2.trx, g2.tr_y,

                        g2.trx, g2.tr_y,
                        g2.tlx, g2.tly,
                        g2.blx, g2.bly
                    ];
                }
            } else {
                self.hud.elements[10 + realslotind * 2].uvs = [
                        bf.blx, bf.bly,
                        bf.brx, bf.bry,
                        bf.trx, bf.tr_y,

                        bf.trx, bf.tr_y,
                        bf.tlx, bf.tly,
                        bf.blx, bf.bly
                    ];
                self.hud.elements[10 + realslotind * 2 + 1].uvs = [
                    bf.blx, bf.bly,
                    bf.brx, bf.bry,
                    bf.trx, bf.tr_y,

                    bf.trx, bf.tr_y,
                    bf.tlx, bf.tly,
                    bf.blx, bf.bly
                ];
            }
        }
        self.hud.dirty = true;
    }

    pub fn add_to_inventory(inv: &Arc<RwLock<Inventory>>, id: u32, count: u32) -> Result<bool, bool> {
        let mut inventory = inv.write().unwrap();
        
        // First, try to find an item with the given `id`
        if let Some(item) = inventory.inv.iter_mut().find(|item| item.0 == id) {
            item.1 += count;
            inventory.dirty = true;
            return Ok(true);
        }

        // If not found, try to find an empty slot to add the new item
        if let Some(item) = inventory.inv.iter_mut().find(|item| item.0 == 0) {
            item.0 = id;
            item.1 = count;
            inventory.dirty = true;
            return Ok(true);
        }

        // If no empty slot, return an error
        Err(false)
    }


    pub fn do_step_sounds(&mut self) {
        static mut TIMER: f32 = 0.0;
        static mut LAST_CAM_POS: Vec3 = Vec3{x: 0.0, y: 0.0, z: 0.0};
        let campos = self.camera.lock().unwrap().position;

        
        unsafe {
            let diff = campos.distance(LAST_CAM_POS); 

            if self.grounded && diff > 0.01 {
                let camfootpos = campos - Vec3::new(0.0, 2.0, 0.0);
                let blockat = self.chunksys.read().unwrap().blockat(IVec3::new(camfootpos.x.floor() as i32, camfootpos.y.floor() as i32, camfootpos.z.floor() as i32));
                if TIMER > 0.4 {
                    self.audiop.play_next_in_series(&Blocks::get_walk_series(blockat), &camfootpos, &Vec3::new(0.0, 0.0, 0.0));
                    TIMER = 0.0;
                } else {
                    TIMER += self.delta_time;
                }

                LAST_CAM_POS = campos;
            }
        }
        
    }

    pub fn takeoff_ship(&mut self) {
        if !self.vars.ship_taken_off {
            self.audiop.play("assets/sfx/shiptakeoff.mp3", &self.ship_pos, &Vec3::ZERO);
            self.vars.ship_going_up = true;
            self.vars.ship_going_down = false;
            self.vars.ship_taken_off = true;
            self.vars.on_new_world = false;
        }
        
    }

    pub fn update(&mut self) {
        
        let current_time = unsafe { glfwGetTime() as f32 };
        self.delta_time = current_time - self.prev_time;

        self.prev_time = current_time;
        for i in self.faders.write().unwrap().iter_mut().enumerate() {
            if i.1.tick(self.delta_time) {
                if i.0 == (FaderNames::FovFader as usize) {
                    self.camera.lock().unwrap().update_fov(i.1.value);
                }
            }
        }
        if self.controls.forward || self.controls.back || self.controls.left || self.controls.right
        {
            if !self.faders.read().unwrap()[FaderNames::FovFader as usize].mode {
                self.faders.write().unwrap()[FaderNames::FovFader as usize].up();
            }
        } else {
            if self.faders.read().unwrap()[FaderNames::FovFader as usize].mode {
                self.faders.write().unwrap()[FaderNames::FovFader as usize].down();
            }
        }
        self.draw();
        self.draw_models();
        if !self.vars.ship_taken_off {
            self.draw_select_cube();
        }
        
        self.guisys.draw_text(0);
        let mvp = self.camera.lock().unwrap().mvp;
        self.drops.update_and_draw_drops(&self.delta_time, &mvp);
        self.hud.update();
        self.hud.draw();
        self.audiop.update();
        let camlock = self.camera.lock().unwrap();
        let pos = camlock.position;
        let forward = camlock.direction;
        let vel = camlock.velocity;
        let up = camlock.up;
        drop(camlock);
        self.audiop.set_listener_attributes(libfmod::Vector { x: pos.x, y: pos.y, z: pos.z }, libfmod::Vector { x: vel.x, y: vel.y, z: vel.z }, libfmod::Vector { x: forward.x, y: forward.y, z: forward.z }, libfmod::Vector { x: up.x, y: up.y, z: up.z });
        self.do_step_sounds();
        if self.inventory.read().unwrap().dirty {
            self.update_inventory();
        }

        if self.vars.ship_taken_off {
            if !self.vars.on_new_world {
                if self.planet_y_offset > REQUIRED_SHIP_FLYAWAY_HEIGHT {

                } else {
                    
                    self.new_world_func();
                    self.audiop.play("assets/sfx/shipland28sec.mp3", &self.ship_pos, &Vec3::ZERO);
                    
                    self.vars.on_new_world = true;
                    self.vars.ship_going_down = true;
                    self.vars.ship_going_up = false;
                }
            } else {
                if self.planet_y_offset >= 0.0 {
                    self.vars.ship_going_down = false;
                    self.vars.ship_taken_off = false;
                }
            }
        }
        
        
        let camlock = self.camera.lock().unwrap();
        let shipdist = camlock.position.distance(self.ship_pos);
        if shipdist < 30.0 && shipdist > 10.0 {
            self.vars.near_ship = true;
            self.guisys.draw_text(1);
        } else {
            self.vars.near_ship = false;
        }
        drop(camlock);

        if self.initial_timer < 1.5  {
            self.initial_timer += self.delta_time;
        } else {
            self.update_movement_and_physics();
            self.update_non_static_model_entities();
        }

        let planet_speed = -self.planet_y_offset.clamp(-100.0, -0.5);

        if self.vars.ship_going_down {
            self.planet_y_offset = (self.planet_y_offset + self.delta_time * planet_speed).clamp(-1000.0, 0.0);
        }
        if self.vars.ship_going_up {
            self.planet_y_offset = (self.planet_y_offset - self.delta_time * planet_speed).clamp(-1000.0, 0.0);
        }
        //println!("Planet y off: {}", self.planet_y_offset);
        
        
    }

    pub fn update_movement_and_physics(&mut self) { 
        let mut camlock = self.camera.lock().unwrap();

        if !self.coll_cage.solid.contains(&Side::FLOOR) {
            self.grounded = false;
        } else {
        }

        const GRAV: f32 = 9.8;

        if !self.grounded && !self.jumping_up {
            self.time_falling_scalar = (self.time_falling_scalar + self.delta_time * 5.0).min(3.0);
        } else {
            self.time_falling_scalar = 1.0;
        }

        if !self.grounded && !self.jumping_up {
            camlock.velocity +=
                Vec3::new(0.0, -GRAV * self.time_falling_scalar * self.delta_time, 0.0);
        }

        if self.jumping_up {
            if camlock.position.y < self.current_jump_y + self.allowable_jump_height {
                let curr_cam_y = camlock.position.y;
                camlock.velocity += Vec3::new(
                    0.0,
                    (((self.current_jump_y + self.allowable_jump_height + 0.3) - curr_cam_y)
                        * 15.0)
                        * self.delta_time,
                    0.0,
                );
            } else {
                self.jumping_up = false;
            }
        }

        if self.controls.up && self.grounded {
            self.grounded = false;
            self.current_jump_y = camlock.position.y;
            self.jumping_up = true;
            self.controls.up = false;
        }

        let cc_center = camlock.position + Vec3::new(0.0, -1.0, 0.0);
        self.coll_cage.update_readings(cc_center);

        

        let mut proposed = camlock.respond_to_controls(&self.controls, &self.delta_time, 5.0);
        self.user_bound_box
            .set_center(proposed + Vec3::new(0.0, -0.5  , 0.0), 0.2, 0.85);
        self.coll_cage.update_colliding(&self.user_bound_box);
        let mut corr_made: Vec<Vec3> = Vec::new();
        if self.coll_cage.colliding.len() > 0 {
            for side in &self.coll_cage.colliding {
                if !corr_made.contains(&self.coll_cage.normals[*side as usize]) {
                    proposed += self.coll_cage.normals[*side as usize]
                        * self.coll_cage.penetrations[*side as usize];
                    corr_made.push(self.coll_cage.normals[*side as usize]);
                }
                if *side == Side::FLOOR {
                    self.grounded = true;
                }
                if *side == Side::ROOF {
                    self.jumping_up = false;
                    self.grounded = false;
                }
            }
        }
        
        camlock.position = Vec3::new(proposed.x, proposed.y, proposed.z);

        //let offset = self.coll_cage.get_smoothed_floor_y(camlock.position);

        //camlock.position.y = offset;

        camlock.recalculate();
    }

    pub fn draw_sky(&self, top: Vec4, bot: Vec4) {
        //Sky
        unsafe {
            gl::BindVertexArray(self.skyshader.vao);
            gl::UseProgram(self.skyshader.shader_id);
            gl::Disable(gl::DEPTH_TEST);
        }
        static mut T_C_LOC: i32 = -1;
        static mut B_C_LOC: i32 = 0;
        static mut C_P_LOC: i32 = 0;
        static mut A_B_LOC: i32 = 0;
        static mut S_S_LOC: i32 = 0;
        static mut S_R_LOC: i32 = 0;
        static mut C_D_LOC: i32 = 0;
        static mut P_Y_LOC: i32 = 0;

        unsafe {
            if T_C_LOC == -1 {
                T_C_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"top_color\0".as_ptr() as *const i8,
                );
                B_C_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"bot_color\0".as_ptr() as *const i8,
                );
                C_P_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"cpitch\0".as_ptr() as *const i8,
                );
                A_B_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"brightMult\0".as_ptr() as *const i8,
                );
                S_S_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"sunset\0".as_ptr() as *const i8,
                );
                S_R_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"sunrise\0".as_ptr() as *const i8,
                );
                C_D_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"camDir\0".as_ptr() as *const i8,
                );
                P_Y_LOC = gl::GetUniformLocation(
                    self.skyshader.shader_id,
                    b"planety\0".as_ptr() as *const i8,
                );
            }

            let camlock = self.camera.lock().unwrap();
            gl::Uniform1f(C_P_LOC, camlock.pitch);
            gl::Uniform3f(
                C_D_LOC,
                camlock.direction.x,
                camlock.direction.y,
                camlock.direction.z,
            );
            drop(camlock);

            gl::Uniform4f(T_C_LOC, top.x, top.y, top.z, top.w);
            gl::Uniform4f(B_C_LOC, bot.x, bot.y, bot.z, bot.w);

            gl::Uniform1f(A_B_LOC, 1.0);
            gl::Uniform1f(S_S_LOC, 0.0);
            gl::Uniform1f(S_R_LOC, 0.0);

            gl::Uniform1f(P_Y_LOC, self.planet_y_offset);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    pub fn draw_select_cube(&mut self) {

        static mut LAST_CAM_POS: Vec3 = Vec3{x:0.0, y:0.0, z:0.0};
        static mut LAST_CAM_DIR: Vec3 = Vec3{x:0.0, y:0.0, z:0.0};
        static mut LAST_BLOCK_POS : IVec3 = IVec3{x:0, y:0, z:0};

        static mut HIT_RESULT: Option<(Vec3, IVec3)> = None;

        static mut BLOCK_TYPE: u32 = 0;

        static mut BREAK_TIME: f32 = 0.0;

        let camlock = self.camera.lock().unwrap();
        unsafe {
            
            if(camlock.position != LAST_CAM_POS || camlock.direction != LAST_CAM_DIR) {
                
                LAST_CAM_POS = camlock.position;
                LAST_CAM_DIR = camlock.direction;

                HIT_RESULT = raycast_voxel(camlock.position, camlock.direction, &self.chunksys, 10.0);
                
                
                
                BLOCK_TYPE = match HIT_RESULT {
                    Some((head, hit)) => {
                        if(LAST_BLOCK_POS != hit) {
                            BREAK_TIME = 0.0;
                            LAST_BLOCK_POS = hit;
                        }
                        self.chunksys.read().unwrap().blockat(hit)
                    }
                    None => {
                        0
                    }
                    
                };
            }


            match HIT_RESULT {
                Some((_head, hit)) => {
                    let hitvec3 = Vec3::new(hit.x as f32, hit.y as f32, hit.z as f32);
                    self.select_cube.draw_at(hitvec3, &camlock.mvp);
                    let bprog = (BREAK_TIME / Blocks::get_break_time(BLOCK_TYPE)).clamp(0.0, 1.0);
              
                    
                    if self.vars.mouse_clicked {
                        self.block_overlay.draw_at(hitvec3, (bprog * 8.0).floor() as i8, &camlock.mvp);
                        BREAK_TIME = BREAK_TIME + self.delta_time;
                        if bprog >= 1.0 {
                            drop(camlock);
                            if !self.vars.ship_taken_off {
                                self.cast_break_ray();
                            }
                            BREAK_TIME = 0.0;
                        }
                    }
                }
                None => {
    
                }
            }
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.5, 0.7, 1.0, 1.0);
        }

        //Sky
        match self.vars.hostile_world {
            true => {
                self.draw_sky(self.vars.hostile_world_sky_color, self.vars.hostile_world_sky_bottom);
            }
            false => {
                self.draw_sky(self.vars.sky_color, self.vars.sky_bottom);
            }
        }
        

        //Chunks
        unsafe {
            gl::BindVertexArray(self.shader0.vao);
            gl::UseProgram(self.shader0.shader_id);
        }
        let ugqarc = self.chunksys.read().unwrap().finished_user_geo_queue.clone();

        match ugqarc.pop() {
            Some(ready) => {
                //println!("Some user queue");
               // println!("Weird!");

                let bankarc = self.chunksys.read().unwrap().geobank[ready.geo_index].clone();

                let cs = self.chunksys.read().unwrap();

                let mut cmemlock = cs.chunk_memories.lock().unwrap();

                cmemlock.memories[ready.geo_index].length = ready.newlength;
                cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
                cmemlock.memories[ready.geo_index].pos = ready.newpos;
                cmemlock.memories[ready.geo_index].used = true;

                //println!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
                //println!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
                //if num == 0 { num = 1; } else { num = 0; }
                //bankarc.num.store(num, std::sync::atomic::Ordering::Release);
                // if num == 0 {
                //     bankarc.num.store(1, Ordering::Relaxed);
                //     num = 1;
                // } else {
                //     bankarc.num.store(0, Ordering::Relaxed);
                //     num = 0;
                // };

                let v32 = cmemlock.memories[ready.geo_index].vbo32;
                let v8 = cmemlock.memories[ready.geo_index].vbo8;
                let tv32 = cmemlock.memories[ready.geo_index].tvbo32;
                let tv8 = cmemlock.memories[ready.geo_index].tvbo8;

                WorldGeometry::bind_geometry(v32, v8, true, &self.shader0, bankarc.solids());
                WorldGeometry::bind_geometry(
                    tv32,
                    tv8,
                    true,
                    &self.shader0,
                    bankarc.transparents(),
                );
            }
            None => {}
        }

        let gqarc = self.chunksys.read().unwrap().finished_geo_queue.clone();

        match gqarc.pop() {
            Some(ready) => {

                //println!("Weird!");

                let bankarc = self.chunksys.read().unwrap().geobank[ready.geo_index].clone();

                let cs = self.chunksys.read().unwrap();

                let mut cmemlock = cs.chunk_memories.lock().unwrap();

                cmemlock.memories[ready.geo_index].length = ready.newlength;
                cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
                cmemlock.memories[ready.geo_index].pos = ready.newpos;
                cmemlock.memories[ready.geo_index].used = true;

                //println!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
                //println!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
                //if num == 0 { num = 1; } else { num = 0; }
                //bankarc.num.store(num, std::sync::atomic::Ordering::Release);
                // if num == 0 {
                //     bankarc.num.store(1, Ordering::Relaxed);
                //     num = 1;
                // } else {
                //     bankarc.num.store(0, Ordering::Relaxed);
                //     num = 0;
                // };

                let v32 = cmemlock.memories[ready.geo_index].vbo32;
                let v8 = cmemlock.memories[ready.geo_index].vbo8;
                let tv32 = cmemlock.memories[ready.geo_index].tvbo32;
                let tv8 = cmemlock.memories[ready.geo_index].tvbo8;

                WorldGeometry::bind_geometry(v32, v8, true, &self.shader0, bankarc.solids());
                WorldGeometry::bind_geometry(
                    tv32,
                    tv8,
                    true,
                    &self.shader0,
                    bankarc.transparents(),
                );
                let mut userstuff = true;
                while userstuff {
                    match ugqarc.pop() {
                        Some(ready) => {
                            
                                    //println!("Some user queue");
                                    // println!("Weird!");
                
                                let bankarc = self.chunksys.read().unwrap().geobank[ready.geo_index].clone();
                
                                //let mut cmemlock = self.chunksys.chunk_memories.lock().unwrap();
                
                                cmemlock.memories[ready.geo_index].length = ready.newlength;
                                cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
                                cmemlock.memories[ready.geo_index].pos = ready.newpos;
                                cmemlock.memories[ready.geo_index].used = true;
                
                                //println!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
                                //println!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
                                //if num == 0 { num = 1; } else { num = 0; }
                                //bankarc.num.store(num, std::sync::atomic::Ordering::Release);
                                // if num == 0 {
                                //     bankarc.num.store(1, Ordering::Relaxed);
                                //     num = 1;
                                // } else {
                                //     bankarc.num.store(0, Ordering::Relaxed);
                                //     num = 0;
                                // };
                
                                let v32 = cmemlock.memories[ready.geo_index].vbo32;
                                let v8 = cmemlock.memories[ready.geo_index].vbo8;
                                let tv32 = cmemlock.memories[ready.geo_index].tvbo32;
                                let tv8 = cmemlock.memories[ready.geo_index].tvbo8;
                
                                WorldGeometry::bind_geometry(v32, v8, true, &self.shader0, bankarc.solids());
                                WorldGeometry::bind_geometry(
                                    tv32,
                                    tv8,
                                    true,
                                    &self.shader0,
                                    bankarc.transparents(),
                                );
                            
                            
                        
                        }
                        None => { userstuff = false; }
                    }
                }
            }
            None => {}
        }

        static mut C_POS_LOC: i32 = -1;
        static mut MVP_LOC: i32 = 0;
        static mut CAM_POS_LOC: i32 = 0;
        static mut AMBIENT_BRIGHT_MULT_LOC: i32 = 0;
        static mut VIEW_DISTANCE_LOC: i32 = 0;
        static mut UNDERWATER_LOC: i32 = 0;
        static mut CAM_DIR_LOC: i32 = 0;
        static mut SUNSET_LOC: i32 = 0;
        static mut SUNRISE_LOC: i32 = 0;
        static mut FOGCOL_LOC: i32 = 0;
        static mut PLANET_Y_LOC: i32 = 0;
        unsafe {
            if C_POS_LOC == -1 {
                C_POS_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"chunkpos\0".as_ptr() as *const i8,
                );
                MVP_LOC =
                    gl::GetUniformLocation(self.shader0.shader_id, b"mvp\0".as_ptr() as *const i8);
                //println!("MVP LOC: {}", MVP_LOC);
                CAM_POS_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"camPos\0".as_ptr() as *const i8,
                );
                AMBIENT_BRIGHT_MULT_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"ambientBrightMult\0".as_ptr() as *const i8,
                );
                VIEW_DISTANCE_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"viewDistance\0".as_ptr() as *const i8,
                );
                UNDERWATER_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"underWater\0".as_ptr() as *const i8,
                );
                CAM_DIR_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"camDir\0".as_ptr() as *const i8,
                );
                SUNSET_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"sunset\0".as_ptr() as *const i8,
                );
                SUNRISE_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"sunrise\0".as_ptr() as *const i8,
                );
                FOGCOL_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"fogCol\0".as_ptr() as *const i8,
                );
                PLANET_Y_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"planet_y\0".as_ptr() as *const i8,
                );
            }
            let cam_lock = self.camera.lock().unwrap();

            gl::UniformMatrix4fv(MVP_LOC, 1, gl::FALSE, cam_lock.mvp.to_cols_array().as_ptr());
            gl::Uniform3f(
                CAM_POS_LOC,
                cam_lock.position.x,
                cam_lock.position.y,
                cam_lock.position.z,
            );
            gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, 1.0);
            gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
            gl::Uniform1f(UNDERWATER_LOC, 0.0);
            gl::Uniform3f(
                CAM_DIR_LOC,
                cam_lock.direction.x,
                cam_lock.direction.y,
                cam_lock.direction.z,
            );
            gl::Uniform1f(SUNSET_LOC, 0.0);
            gl::Uniform1f(SUNRISE_LOC, 0.0);
            gl::Uniform1f(PLANET_Y_LOC, self.planet_y_offset);
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                0,
            );
            let fc = Planets::get_fog_col(self.chunksys.read().unwrap().planet_type as u32);
            gl::Uniform4f(
                FOGCOL_LOC,
                fc.0, 
                fc.1,
                fc.2,
                fc.3
            );

            drop(cam_lock);
        }
        
        let cs = self.chunksys.read().unwrap();
        let cmem = cs.chunk_memories.lock().unwrap();
        for (index, cfl) in cmem.memories.iter().enumerate() {
            if cfl.used {
                let dd1: Mutex<Vec<u32>> = Mutex::new(Vec::new());
                let dd2: Mutex<Vec<u8>> = Mutex::new(Vec::new());
                let dd: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>) = (&dd1, &dd2);

                WorldGeometry::bind_geometry(
                    cfl.vbo32,
                    cfl.vbo8,
                    false,
                    &self.shader0,
                    dd,
                );
                unsafe {
                    gl::Uniform2f(C_POS_LOC, cfl.pos.x as f32, cfl.pos.y as f32);

                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        println!("OpenGL Error after uniforming the chunk pos: {}", error);
                    }
                    //println!("Rendering {} in chunk at {}, {}", banklock.data32.len(), banklock.pos.x, banklock.pos.y);
                    gl::DrawArrays(gl::TRIANGLES, 0, cfl.length as i32);
                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        println!("OpenGL Error after drawing arrays: {}", error);
                    }
                    // println!("Chunk rending!");
                }
            }
        }
        for (index, cfl) in cmem.memories.iter().enumerate() {
            if cfl.used {
                let dd1: Mutex<Vec<u32>> = Mutex::new(Vec::new());
                let dd2: Mutex<Vec<u8>> = Mutex::new(Vec::new());
                let dd: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>) = (&dd1, &dd2);

                WorldGeometry::bind_geometry(
                    cfl.tvbo32,
                    cfl.tvbo8,
                    false,
                    &self.shader0,
                    dd,
                );
                unsafe {
                    gl::Uniform2f(C_POS_LOC, cfl.pos.x as f32, cfl.pos.y as f32);

                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        println!("OpenGL Error after uniforming the chunk pos: {}", error);
                    }
                    //println!("Rendering {} in chunk at {}, {}", banklock.data32.len(), banklock.pos.x, banklock.pos.y);
                    gl::DrawArrays(gl::TRIANGLES, 0, cfl.tlength as i32);
                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        println!("OpenGL Error after drawing arrays: {}", error);
                    }
                    // println!("Chunk rending!");
                }
            }
        }

    }



    pub fn start_world(&mut self) {
        (*self.run_chunk_thread).store(true, Ordering::Relaxed);
        self.initial_timer = 0.0;

        let rctarc = self.run_chunk_thread.clone();
        let carc = self.camera.clone();
        let csysarc = self.chunksys.clone();

        let handle = thread::spawn(move || {
            Game::chunk_thread_function(&rctarc, carc, csysarc);
        });

        self.chunk_thread = Some(handle);

        //self.chunksys.voxel_models[0].stamp_here(&vec::IVec3::new(0, 40, 0), &self.chunksys, None);
    }

    pub fn add_ship_colliders(&self) {
        self.update_model_collisions(self.static_model_entities.len() - 1);
    }

    
    pub fn start_chunks_with_radius(&mut self, newradius: u8, seed: u32, nt: usize) {

        (*self.run_chunk_thread).store(false, Ordering::Relaxed);


        if let Some(handle) = self.chunk_thread.take() { // take the handle out safely
            handle.join().unwrap(); // Join the thread, handle errors appropriately
            println!("Thread joined successfully!");
        } else {
            println!("No thread to join or already joined.");
        }
        
        for i in &self.chunksys.read().unwrap().geobank {
            unsafe {
                gl::DeleteBuffers(1, &i.vbo32);
                gl::DeleteBuffers(1, &i.tvbo32);
                gl::DeleteBuffers(1, &i.vbo8);
                gl::DeleteBuffers(1, &i.tvbo8);
            }
        }
        self.drops.drops.clear();
        self.non_static_model_entities.clear();
        let mut csys = ChunkSystem::new(newradius, seed, nt);
        csys.voxel_models = Some(self.voxel_models.clone());
        self.chunksys = Arc::new(RwLock::new(csys));

        // self.chunksys.reset(0, 0, 0);

        self.drops.csys = self.chunksys.clone();

        let mut rng = StdRng::from_entropy();
        

        if nt == 1 {
            self.create_non_static_model_entity(0, Vec3::new(-100.0, 100.0, 350.0), 5.0, Vec3::new(0.0, 0.0, 0.0), 7.0);

            for i in 0..4 {
                if rng.gen_range(0..3) <= 2 {
                    self.create_non_static_model_entity(2, Vec3::new(rng.gen_range(-200.0..200.0),80.0,rng.gen_range(-200.0..200.0)), 5.0, Vec3::new(0.0, 0.0, 0.0), 7.0);
                    self.create_non_static_model_entity(2, Vec3::new(rng.gen_range(-200.0..200.0),80.0,rng.gen_range(-200.0..200.0)), 5.0, Vec3::new(0.0, 0.0, 0.0), 7.0);
                    
                    self.create_non_static_model_entity(3, Vec3::new(rng.gen_range(-200.0..200.0),80.0,rng.gen_range(-200.0..200.0)), 5.0, Vec3::new(0.0, 0.0, 0.0), 3.0);
                    self.create_non_static_model_entity(3, Vec3::new(rng.gen_range(-200.0..200.0),80.0,rng.gen_range(-200.0..200.0)), 5.0, Vec3::new(0.0, 0.0, 0.0), 3.0);
                }
            }
            
        }

        self.coll_cage.solid_pred  = {
            let csys_arc = Arc::clone(&self.chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.read().unwrap().collision_predicate(v)
            })
        };

        let mut ship_pos = vec::IVec3::new(20,200,0);
        let mut ship_front = vec::IVec3::new(30,200,0);
        let mut ship_back = vec::IVec3::new(10,200,0);
         // Function to decrement y until a block is found
        fn find_ground_y(position: &mut vec::IVec3, game: &Game) {
            while game.chunksys.read().unwrap().blockat(*position) == 0 {
                position.y -= 1;
            }
        }

        // Find the ground positions
        find_ground_y(&mut ship_pos, &self);
        find_ground_y(&mut ship_front, &self);
        find_ground_y(&mut ship_back, &self);

        // Determine the highest y position found
        let decided_pos_y = max(max(ship_pos.y, ship_front.y), ship_back.y) + 10;
        self.rebuild_whole_world_while_showing_loading_screen();
        // Update the ship's position
        ship_pos.y = decided_pos_y;
        let ship_float_pos = Vec3::new(ship_pos.x as f32, ship_pos.y as f32, ship_pos.z as f32);
        self.ship_pos = ship_float_pos;
        let ship_index = self.static_model_entities.len()-1;
        self.static_model_entities[ship_index].position = ship_float_pos;
        self.camera.lock().unwrap().position = ship_float_pos + Vec3::new(5.0, 2.0, 0.0);
        self.add_ship_colliders();

        self.start_world();
    }


    pub fn rebuild_whole_world_while_showing_loading_screen(&mut self) {

        let csys = self.chunksys.clone();
        let campos = self.camera.lock().unwrap().position.clone();
        let shader = self.shader0.clone();

        let threadhandle = thread::spawn(move|| {
            ChunkSystem::initial_rebuild_on_main_thread(&csys, &shader, &campos)
        });

        while !threadhandle.is_finished() {

            //self.draw();
            let current_time = unsafe { glfwGetTime() as f32 };
            self.delta_time = current_time - self.prev_time;
    
            self.prev_time = current_time;

        }
        
        match threadhandle.join() {
            Ok(_) => {

            }
            Err(_) => {
                tracing::info!("The whole-world-rebuild thread didn't join back I guess????");
            }
        };
    }

    pub fn chunk_thread_inner_function(cam_arc: &Arc<Mutex<Camera>>, csys_arc: &Arc<RwLock<ChunkSystem>>, last_user_c_pos: &mut vec::IVec2) {


        

        let mut userstuff = true;
        while userstuff {

            let csys_arc = csys_arc.read().unwrap();

            match csys_arc.user_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, true);
                }
                None => {
                    userstuff = false;
                }
            }
        }
        let mut genstuff = true;
        while genstuff {
            let csys_arc = csys_arc.read().unwrap();

            match csys_arc.gen_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, true);
                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true);
                            let mut userstuff = true;
                            while userstuff {
                                match csys_arc.user_rebuild_requests.pop() {
                                    Some(index) => {
                                        csys_arc.rebuild_index(index, true);
                                    }
                                    None => {
                                        userstuff = false;
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
                None => {
                    genstuff = false;
                }
            }
        }


        let mut backgroundstuff = true;
        while backgroundstuff {
            let csys_arc = csys_arc.read().unwrap();
            match csys_arc.background_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, false);
                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true);
                            let mut userstuff = true;
                            while userstuff {
                                match csys_arc.user_rebuild_requests.pop() {
                                    Some(index) => {
                                        csys_arc.rebuild_index(index, true);
                                    }
                                    None => {
                                        userstuff = false;
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                    match csys_arc.gen_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true);
                            let mut genstuff = true;
                            while genstuff {
                                match csys_arc.gen_rebuild_requests.pop() {
                                    Some(index) => {
                                        csys_arc.rebuild_index(index, true);
                                    }
                                    None => {
                                        genstuff = false;
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
                None => {
                    backgroundstuff = false;
                }
            }
        }

        let camlock = cam_arc.lock().unwrap();
        let vec3 = camlock.position;
        drop(camlock);

        static mut last_time: f32 = 0.0;

        unsafe {


            let current_time = glfwGetTime() as f32;

            let delta_time = current_time - last_time;

            
            static mut time_since_last_check: f32 = 1.0;

            let user_c_pos = ChunkSystem::spot_to_chunk_pos(&IVec3::new(
                vec3.x.floor() as i32,
                vec3.y.floor() as i32,
                vec3.z.floor() as i32,
            ));

            if user_c_pos != *last_user_c_pos && time_since_last_check >= 1.0 {
                *last_user_c_pos = user_c_pos;

                time_since_last_check = 0.0;

                let mut neededspots: Vec<IVec2> = Vec::new();

                let cam_lock = cam_arc.lock().unwrap();
                let user_cpos = IVec2 {
                    x: (cam_lock.position.x / 15.0).floor() as i32,
                    y: (cam_lock.position.z / 15.0).floor() as i32,
                };
                drop(cam_lock);

                let csys_arc = csys_arc.read().unwrap();

                let tcarc = csys_arc.takencare.clone();
                for i in -(csys_arc.radius as i32)..(csys_arc.radius as i32) {
                    for k in -(csys_arc.radius as i32)..(csys_arc.radius as i32) {
                        let this_spot = IVec2 {
                            x: user_cpos.x + i as i32,
                            y: user_cpos.y + k as i32,
                        };
                        if !tcarc.contains_key(&this_spot) {
                            neededspots.push(this_spot);
                        }
                    }
                }

                let mut sorted_chunk_facades: Vec<ChunkFacade> = Vec::new();

                for carc in &csys_arc.chunks {
                    match carc.try_lock() {
                        Ok(cf) => {
                            sorted_chunk_facades.push(*cf);
                        }
                        Err(_) => {}
                    }
                }

                let (unused_or_distant, used_and_close): (Vec<ChunkFacade>, Vec<ChunkFacade>) =
                    sorted_chunk_facades.drain(..).partition(|chunk| {
                        if !chunk.used {
                            true
                        } else {
                            let dist = (chunk.pos.x - user_cpos.x).abs()
                                + (chunk.pos.y - user_cpos.y).abs();
                            dist >= csys_arc.radius as i32 * 2
                        }
                    });

                sorted_chunk_facades.extend(unused_or_distant);
                sorted_chunk_facades.extend(used_and_close);
                //println!("Neededspots size: {}", neededspots.len());
                for (index, ns) in neededspots.iter().enumerate() {
                    
                    csys_arc.move_and_rebuild(sorted_chunk_facades[index].geo_index, *ns);
                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true);
                            break;
                        }
                        None => {}
                    }
                    match csys_arc.gen_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true);
                        }
                        None => {}
                    }
                }
            } else {
                time_since_last_check += delta_time;
            }
        }
    }

    pub fn chunk_thread_function(
        runcheck: &AtomicBool,
        cam_arc: Arc<Mutex<Camera>>,
        csys_arc: Arc<RwLock<ChunkSystem>>,
    ) {
        //static mut TEMP_COUNT: i32 = 0;

        let mut last_user_c_pos: vec::IVec2 = vec::IVec2 {
            x: -99999,
            y: -99999,
        };

        while runcheck.load(Ordering::Relaxed) {
            Game::chunk_thread_inner_function(&cam_arc, &csys_arc, &mut last_user_c_pos);
        }
    }
    pub fn cursor_pos(&mut self, xpos: f64, ypos: f64) {
        if self.vars.mouse_focused {
            static mut LASTX: f64 = 0.0;
            static mut LASTY: f64 = 0.0;

            if self.vars.first_mouse {
                unsafe {
                    LASTX = xpos;
                    LASTY = ypos;
                }
                self.vars.first_mouse = false;
            }

            unsafe {
                let x_offset = (xpos - LASTX) * self.vars.sensitivity as f64;
                let y_offset = (LASTY - ypos) * self.vars.sensitivity as f64;

                LASTY = ypos;
                LASTX = xpos;

                let mut camlock = self.camera.lock().unwrap();

                camlock.yaw += x_offset as f32;
                camlock.pitch += y_offset as f32;

                camlock.pitch = camlock.pitch.clamp(-89.0, 89.0);

                camlock.direction.x =
                    camlock.yaw.to_radians().cos() as f32 * camlock.pitch.to_radians().cos() as f32;
                camlock.direction.y = camlock.pitch.to_radians().sin();
                camlock.direction.z =
                    camlock.yaw.to_radians().sin() * camlock.pitch.to_radians().cos();
                camlock.direction = camlock.direction.normalize();

                camlock.right = Vec3::new(0.0, 1.0, 0.0)
                    .cross(camlock.direction)
                    .normalize();
                camlock.up = camlock.direction.cross(camlock.right).normalize();

                camlock.recalculate();
                #[cfg(feature = "show_cam_pos")]
                println!(
                    "Cam dir: {}, {}, {}",
                    camlock.direction.x, camlock.direction.y, camlock.direction.z
                );
            }
        }
    }
    pub fn set_mouse_focused(&mut self, tf: bool) {
        if tf {
            self.vars.mouse_focused = true;
        } else {
            self.vars.mouse_focused = false;
            self.vars.first_mouse = true;
        }
    }
    pub fn delete_block_recursively(chunksys: &Arc<RwLock<ChunkSystem>>, id: u32, at: IVec3, set: &mut HashSet<IVec2>) {
        let mut stack = vec![at]; // Initialize stack with initial position
    
        while let Some(current) = stack.pop() {
            // Check if the block at the current position is already deleted

            let chunksys = chunksys.read().unwrap();

            if chunksys.blockat(current) != 0 {
                // Set the block at the current position
                chunksys.set_block(current, 0, true);
                let key = ChunkSystem::spot_to_chunk_pos(&current);
                set.insert(key);
                // Add neighbors to the stack if they have the same id
                for neighbor in Cube::get_neighbors() {
                    let neighbor_pos = *neighbor + current;
                    if chunksys.blockat(neighbor_pos) == id {
                        stack.push(neighbor_pos);
                    }
                }
            }
        }
    }
    pub fn cast_break_ray(&mut self) {
        let cl = self.camera.lock().unwrap();
        match raycast_voxel(cl.position, cl.direction, &self.chunksys, 10.0) {
            Some((tip, block_hit)) => {
                let blockat = self.chunksys.read().unwrap().blockat(block_hit);
                if blockat == 16 {
                    let mut set: HashSet<IVec2> = HashSet::new();
                    Game::delete_block_recursively(&self.chunksys, 16,  block_hit, &mut set);
                    for key in set {
                        self.chunksys.read().unwrap().queue_rerender_with_key(key, true)
                    }
                    self.drops.add_drop(tip, 17);
                } else {
                    if blockat != 0 {
                        self.drops.add_drop(tip, blockat);
                    }
                    
                    self.chunksys.read().unwrap().set_block_and_queue_rerender(block_hit, 0, true, true);
                }
                
                
            }
            None => {}
        }
    }
    pub fn scroll(&mut self, y: f64) {
        let mut invrowchange = 0;
        if y > 0.0 {
            invrowchange += 1;
        }
        if y < 0.0 {
            invrowchange -= 1;
        }
        let mut proposednewslot = self.hud.bumped_slot as i8 + invrowchange;
        if proposednewslot < 0 {
            proposednewslot = 4;
        }
        self.hud.bumped_slot = proposednewslot as usize % 5;
        self.hud.dirty = true;
        self.hud.update();
    }

    pub fn cast_place_ray(&mut self) {

        let slot_selected = self.hud.bumped_slot;
        let slot = self.inventory.read().unwrap().inv[slot_selected];

        if slot.0 != 0 && slot.1 > 0 {
            let id = slot.0;

            let cl = self.camera.lock().unwrap();

            match raycast_voxel(cl.position, cl.direction, &self.chunksys, 10.0) {
                
                Some((tip, block_hit)) => {

                let diff = (tip+Vec3::new(-0.5, -0.5, -0.5)) - (Vec3::new(block_hit.x as f32, block_hit.y as f32, block_hit.z as f32));

                let hit_normal;

                // Determine the primary axis of intersection
                if (diff.x).abs() > (diff.y).abs() && (diff.x).abs() > (diff.z).abs() {
                    // The hit was primarily along the X-axis
                    hit_normal = vec::IVec3::new( if diff.x > 0.0 { 1 } else { -1 }, 0, 0);

                } else if (diff.y).abs() > (diff.x).abs() && (diff.y).abs() > (diff.z).abs() {
                    // The hit was primarily along the Y-axis
                    hit_normal = vec::IVec3::new(0, if diff.y > 0.0 { 1 } else { -1 }, 0);
                } else {
                    // The hit was primarily along the Z-axis
                    hit_normal = vec::IVec3::new(0, 0, if diff.z > 0.0 { 1 } else { -1 });
                }

                println!("Hit normal is {} {} {}", hit_normal.x, hit_normal.y, hit_normal.z);


                let place_point = block_hit + hit_normal;
                    println!("Placing {} at {} {} {}", 1, place_point.x, place_point.y, place_point.z);
                    self.chunksys.read().unwrap().set_block_and_queue_rerender(place_point, id, false, true);
                }

                None => {}
            }

        }

        if slot.1 == 1 {
            let mutslot = &mut self.inventory.write().unwrap().inv[slot_selected];
            mutslot.1 = 0;
            mutslot.0 = 0;
        } else {
            let mutslot = &mut self.inventory.write().unwrap().inv[slot_selected];
            mutslot.1 -= 1;
        }

        

    }
    pub fn mouse_button(&mut self, mb: MouseButton, a: Action) {
        match mb {
            glfw::MouseButtonLeft => {
                self.vars.mouse_clicked = a == Action::Press;
                // if self.vars.mouse_clicked {
                //     self.cast_break_ray();
                // }
            }
            glfw::MouseButtonRight => {
                self.vars.right_mouse_clicked = a == Action::Press;
                if !self.vars.ship_taken_off {
                    if self.vars.right_mouse_clicked {
                        self.cast_place_ray();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn new_world_func(&mut self) {
        let mut rng = StdRng::from_entropy();

        let seed: u32 = rng.gen_range(0..2232328);


        static mut CURR_NT: usize = 0;
        self.camera.lock().unwrap().position = Vec3::new(0.0, 100.0, 0.0);
        unsafe {
            self.vars.hostile_world = (CURR_NT % 2) == 0;
            CURR_NT = (CURR_NT + 1) % 2;
            *self.chunksys.read().unwrap().currentseed.write().unwrap() = seed;
            self.start_chunks_with_radius(10, seed, CURR_NT);

            println!("Now noise type is {}", self.chunksys.read().unwrap().planet_type);
        }

        // self.chunksys.load_world_from_file(String::from("saves/world1"));
        // self.vars.hostile_world = false;
        // let seed = *self.chunksys.currentseed.read().unwrap();
        // self.start_chunks_with_radius(10, seed, 0);
        // self.camera.lock().unwrap().position = Vec3::new(0.0, 100.0, 0.0);

    }


    pub fn keyboard(&mut self, key: Key, action: Action) {
        match key {
            Key::W => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.forward = true;
                } else {
                    self.controls.forward = false;
                }
            }
            Key::A => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.left = true;
                } else {
                    self.controls.left = false;
                }
            }
            Key::S => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.back = true;
                } else {
                    self.controls.back = false;
                }
            }
            Key::D => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.right = true;
                } else {
                    self.controls.right = false;
                }
            }
            Key::Space => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.up = true;
                } else {
                    self.controls.up = false;
                }
            }
            Key::M => {
                if action == Action::Press {
                    self.takeoff_ship();
                }
            }
            Key::L => {
                if action == Action::Press {
                    self.chunksys.read().unwrap().save_current_world_to_file(String::from("saves/world1"));
                }
            }
            // Key::Num8 => {
            //     self.vars.ship_going_down = false;
            //     self.vars.ship_going_up = false;
            // }
            // Key::Num0 => {
            //     self.vars.ship_going_down = true;
            //     self.vars.ship_going_up = false;
                
            // }
            // Key::Num9 => {
            //     self.vars.ship_going_down = false;
            //     self.vars.ship_going_up = true;
            // }
            Key::B => {
                if self.vars.near_ship {
                    let mut camlock = self.camera.lock().unwrap();
                    camlock.position = self.ship_pos + Vec3::new(5.0, 2.0, 0.0);
                }
            }
            _ => {}
        }
    }
}
