use core::time;
use std::borrow::BorrowMut;
use std::cmp::max;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::io::{self, Write};
use std::ops::DerefMut;
use std::slice::Chunks;
use std::time::Duration;
use dashmap::DashMap;
use gl::types::{GLenum, GLuint};
use glam::{Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::{glfwGetCursorPos, glfwGetTime, GLFWwindow};
use glfw::{Action, Key, MouseButton, PWindow};
use gltf::Gltf;
use lockfree::queue::{self, Queue};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use uuid::Uuid;
use vox_format::types::Model;
use walkdir::WalkDir;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, current, JoinHandle};

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
use crate::hud::{Hud, HudElement, SlotIndexType};
use crate::modelentity::ModelEntity;
use crate::network::NetworkConnector;
use crate::planetinfo::Planets;
use crate::raycast::*;
use crate::selectcube::SelectCube;
use crate::server_types::{Message, MessageType};
use crate::shader::Shader;
use crate::specialblocks::door::{self, DoorInfo};
use crate::texture::Texture;
use crate::textureface::TextureFace;
use crate::vec::{self, IVec2, IVec3};
use crate::voxmodel::JVoxModel;
use crate::worldgeometry::WorldGeometry;
use crate::inventory::*;
use std::sync::RwLock;




pub static STARTINGITEMS: [(u32, u32); 5] = [
    (18, 99),
    (20, 99),
    (21, 99),
    (10, 99),
    (19, 99)
];


pub static mut MOUSED_SLOT: SlotIndexType = SlotIndexType::None;


pub static mut SONGS: [&'static str; 7] = [
    "assets/music/Farfromhome.mp3",
    "assets/music/ifol.mp3",
    "assets/music/NoFuture.mp3",
    "assets/music/Respite.mp3",
    "assets/music/SereneFacade.mp3",
    "assets/music/Unease.mp3",
    "assets/music/UnknownDanger.mp3",
];

pub static mut SONGTIMER: f32 = 299.0;
pub static mut SONGINTERVAL: f32 = 150.0;
pub static mut SONGINDEX: usize = 0;




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


static REQUIRED_SHIP_FLYAWAY_HEIGHT: f32 = 0.0;


pub struct ControlsState {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub back: bool,
    pub up: bool,
    pub lookingleft: bool,
    pub lookingright: bool,
    pub shift: bool
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
            lookingright: false,
            shift: false,
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
    pub first_mouse: bool,
    pub mouse_focused: bool,
    pub sensitivity: f32,
    pub sky_color: Vec4,
    pub sky_bottom: Vec4,
    pub mouse_clicked: bool,
    pub right_mouse_clicked: bool,
    pub hostile_world: bool,
    pub hostile_world_sky_color: Vec4,
    pub hostile_world_sky_bottom: Vec4,
    pub ship_going_up: bool,
    pub ship_going_down: bool,
    pub break_time: f32,
    pub near_ship: bool,
    pub ship_taken_off: bool,
    pub on_new_world: bool,
    pub in_multiplayer: bool,
    pub menu_open: bool,
    pub main_menu: bool,
    pub in_climbable: bool
}

pub enum VisionType {
    Model(usize),
    Vox(usize)
}


pub struct Game {
    pub chunksys: Arc<RwLock<ChunkSystem>>,
    pub shader0: Shader,
    pub oldshader: Shader,
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
    pub non_static_model_entities: Arc<DashMap<u32, ModelEntity>>,
    pub select_cube: SelectCube,
    pub block_overlay: BlockOverlay,
    pub ship_pos: Vec3,
    pub planet_y_offset: f32,
    pub window: Arc<RwLock<PWindow>>,
    pub guisys: GuiSystem,
    pub hud: Hud,
    pub drops: Drops,
    pub audiop: Arc<RwLock<AudioPlayer>>,
    pub inventory: Arc<RwLock<Inventory>>,
    pub animations: Vec<Vec<Animation>>,
    pub skins: Vec<Skin>,
    pub nodes: Vec<Vec<Node>>,
    pub current_time: f32,
    pub netconn: NetworkConnector,
    pub server_command_queue: Arc<lockfree::queue::Queue<Message>>,
    pub hp_server_command_queue: Arc<lockfree::queue::Queue<Message>>,
    pub headless: bool,
    pub known_cameras: Arc<DashMap<Uuid, Vec3>>,
    pub my_uuid: Arc<RwLock<Option<Uuid>>>,
    pub ambient_bright_mult: f32,
    pub daylength: f32,
    pub timeofday: Arc<Mutex<f32>>,
    pub sunrise_factor: f32,
    pub sunset_factor: f32,
    pub visions_timer: f32,
    pub visions_camera: Camera,
    pub current_vision: Option<VisionType>,
    pub tex: Texture,
    pub inwater: bool,
    pub headinwater: bool,

    pub currentbuttons: Vec<(&'static str, &'static str)>,
    pub loadedworld: AtomicBool,
    pub addressentered: Arc<AtomicBool>,
    pub address: Arc<Mutex<Option<String>>>,
    pub player_model_entities: Arc<DashMap<Uuid, ModelEntity>>,

    pub mouse_slot: (u32, u32),
    pub needtosend: Arc<Queue<Message>>
}

enum FaderNames {
    FovFader = 0,
    VisionsFader = 1
}

impl Game {
    pub fn new(window: &Arc<RwLock<PWindow>>, connectonstart: bool, headless: bool, addressentered: &Arc<AtomicBool>, address: &Arc<Mutex<Option<String>>>) -> JoinHandle<Game> {

        let oldshader = Shader::new("assets/oldvert.glsl", "assets/oldfrag.glsl");
        let shader0 = Shader::new("assets/vert.glsl", "assets/frag.glsl");
        let skyshader = Shader::new("assets/skyvert.glsl", "assets/skyfrag.glsl");
        let faders: RwLock<Vec<Fader>> = RwLock::new(Vec::new());
        let cam = Arc::new(Mutex::new(Camera::new()));

        faders
            .write()
            .unwrap()
            .extend(vec![
                Fader::new(83.0, 80.0, 30.0, false), //FOV fader for moving
                Fader::new(1.0, 0.0, 5.0, false)    //"Visions" fader for overlay
                ]);

        unsafe {
            gl::BindVertexArray(shader0.vao);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL Error after binding vertex array: {}", error);
            }
        }
        let tex = Texture::new("assets/world.png").unwrap();
        tex.add_to_unit(0);

        let audiop = Arc::new(RwLock::new(AudioPlayer::new().unwrap()));


        let audiopoption = match headless {
            true => {
                None
            }
            false => {
                Some(audiop.clone())
            }
        };

        let mut csys = ChunkSystem::new(10, 0, 0, headless, audiopoption);
        let voxel_models = vec![
            JVoxModel::new("assets/voxelmodels/bush.vox"),
            JVoxModel::new("assets/voxelmodels/tree1.vox"),
            JVoxModel::new("assets/voxelmodels/tree2.vox"),
            JVoxModel::new("assets/voxelmodels/rock1.vox"),
            JVoxModel::new("assets/voxelmodels/rock2.vox"),

            JVoxModel::new("assets/voxelmodels/ptree.vox"),
            JVoxModel::new("assets/voxelmodels/redrock.vox"),
            JVoxModel::new("assets/voxelmodels/crystal1.vox"),
        ];

        
        

        //csys.load_world_from_file(String::from("saves/world1"));


        //self.vars.hostile_world = false;
        //let seed = *csys.currentseed.read().unwrap();
        //self.start_chunks_with_radius(10, seed, 0);
        //self.camera.lock().unwrap().position = Vec3::new(0.0, 100.0, 0.0);



        let vmarc = Arc::new(voxel_models);
        let vmarc2 = vmarc.clone();

        csys.voxel_models = Some(vmarc);

        let chunksys = Arc::new(RwLock::new(csys));

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> = {
            let csys_arc = Arc::clone(&chunksys);
            Box::new(move |v: vec::IVec3| {

                let csys = csys_arc.read().unwrap();
                let bitshere = csys.blockat(v.clone());

                
                let isntopendoor = DoorInfo::get_door_open_bit(bitshere) != 1;
                let isntladder = (bitshere & Blocks::block_id_bits()) != 20;
                return isntopendoor && isntladder && csys_arc.read().unwrap().collision_predicate(v);
            })
        };
        let mut hud = Hud::new(&window.clone(), tex.id);

        fn add_inventory_rows(elements: &mut Vec<HudElement>, yoffset: f32, rows: i32, start_slot: SlotIndexType) {
              
        let tf = TextureFace::new(0, 14);

        let rh: f32 = 0.2;
        //IMPORTANT: Push these first, the inv row slots
        for y in 0..rows {
            for i in 0..5 {
                let newslot = match start_slot {
                    SlotIndexType::ChestSlot(ind) => {
                        SlotIndexType::ChestSlot(ind + i + (y*5))
                    },
                    SlotIndexType::InvSlot(ind) => {
                        SlotIndexType::InvSlot(ind + i + (y*5))
                    },
                    SlotIndexType::None => {
                        SlotIndexType::None
                    },
                };
                let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + i as f32 * 0.10, yoffset - y as f32 * rh), Vec2::new(0.15, 0.15), [
                    tf.blx, tf.bly,
                    tf.brx, tf.bry,
                    tf.trx, tf.tr_y,
    
                    tf.trx, tf.tr_y,
                    tf.tlx, tf.tly,
                    tf.blx, tf.bly
                ], newslot);
    
                elements.push(invrowel);
            }
        }
        

        let tf = TextureFace::new(0, 0);
        //The item textures on top of them
        for y in 0..rows {
        for i in 0..5 {
            let newslot = match start_slot {
                SlotIndexType::ChestSlot(ind) => {
                    SlotIndexType::ChestSlot(ind + i + (y*5))
                },
                SlotIndexType::InvSlot(ind) => {
                    SlotIndexType::InvSlot(ind + i + (y*5))
                },
                SlotIndexType::None => {
                    SlotIndexType::None
                },
            };
            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + i as f32 * 0.10, yoffset - y as f32 * rh), Vec2::new(0.10, 0.10), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ], newslot);

            elements.push(invrowel);
        }
    }


        let tf = TextureFace::new(0, 0);
        //The number textures on top of them
        for y in 0..rows {
        for i in 0..5 {
            let newslot = match start_slot {
                SlotIndexType::ChestSlot(ind) => {
                    SlotIndexType::ChestSlot(ind + i + (y*5))
                },
                SlotIndexType::InvSlot(ind) => {
                    SlotIndexType::InvSlot(ind + i + (y*5))
                },
                SlotIndexType::None => {
                    SlotIndexType::None
                },
            };
            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + 0.01 + i as f32 * 0.10, yoffset  - y as f32 * rh - 0.03), Vec2::new(0.05, 0.05), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ], newslot.clone());
            elements.push(invrowel);

            let invrowel = HudElement::new(Vec2::new(-(0.10*2.0) + 0.02 + i as f32 * 0.10, yoffset  - y as f32 * rh - 0.03), Vec2::new(0.05, 0.05), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ], newslot);

            elements.push(invrowel);
        }
    }
        }

        add_inventory_rows(&mut hud.elements, -0.9, 1, SlotIndexType::InvSlot(0));


        add_inventory_rows(&mut hud.chestelements, 0.4, 4, SlotIndexType::ChestSlot(0));


        //Crosshair
        let tf = TextureFace::new(0, 13);
        

        hud.elements.push(HudElement::new(Vec2::new(0.0, 0.0), Vec2::new(0.08, 0.08), [
                tf.blx, tf.bly,
                tf.brx, tf.bry,
                tf.trx, tf.tr_y,

                tf.trx, tf.tr_y,
                tf.tlx, tf.tly,
                tf.blx, tf.bly
            ], SlotIndexType::None));     








            //HELD MOUSE ITEM SLOT




            
    
            let tf = TextureFace::new(0, 0);
            //The item texture of it


                let invrowel = HudElement::new(Vec2::new(0.0, 0.0), Vec2::new(0.10, 0.10), [
                    tf.blx, tf.bly,
                    tf.brx, tf.bry,
                    tf.trx, tf.tr_y,
    
                    tf.trx, tf.tr_y,
                    tf.tlx, tf.tly,
                    tf.blx, tf.bly
                ], SlotIndexType::InvSlot(221));
    
                hud.chestelements.push(invrowel);

    
            let tf = TextureFace::new(0, 0);
            //The number textures on top of it


                let invrowel = HudElement::new(Vec2::new(0.0  + 0.01, 0.0), Vec2::new(0.05, 0.05), [
                    tf.blx, tf.bly,
                    tf.brx, tf.bry,
                    tf.trx, tf.tr_y,
    
                    tf.trx, tf.tr_y,
                    tf.tlx, tf.tly,
                    tf.blx, tf.bly
                ], SlotIndexType::InvSlot(221));
                hud.chestelements.push(invrowel);
    
                let invrowel = HudElement::new(Vec2::new(0.0  + 0.02, 0.0), Vec2::new(0.05, 0.05), [
                    tf.blx, tf.bly,
                    tf.brx, tf.bry,
                    tf.trx, tf.tr_y,
    
                    tf.trx, tf.tr_y,
                    tf.tlx, tf.tly,
                    tf.blx, tf.bly
                ], SlotIndexType::InvSlot(221));
    
                hud.chestelements.push(invrowel);
   































        

        let inv = Arc::new(RwLock::new(Inventory{
            dirty: true,
            inv: STARTINGITEMS
        }));


        

        
        

        
        //let mut rng = StdRng::from_entropy();
        //let seed = rng.gen_range(0..229232);


        let server_command_queue = Arc::new(Queue::<Message>::new());
        let server_command_hp_queue = Arc::new(Queue::<Message>::new());




        let kc = Arc::new(DashMap::new());

        let my_uuid: Arc<RwLock<Option<Uuid>>> = Arc::new(RwLock::new(None));

        let nsme = Arc::new(DashMap::new());

        let mut visions_camera = Camera::new();
        visions_camera.position = Vec3::new(0.0, 3.0, -5.0);
        visions_camera.recalculate();
        println!("Visions camera direction: {} {} {}", visions_camera.direction.x, visions_camera.direction.y, visions_camera.direction.z);

        let pme = Arc::new(DashMap::new());

        let needtosend = Arc::new(Queue::new());

        let mut g = Game {
            chunksys: chunksys.clone(),
            shader0,
            oldshader,
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
                ship_taken_off: false,
                on_new_world: true,
                in_multiplayer: connectonstart, //For now,
                menu_open: false,
                main_menu: false,
                in_climbable: false
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
            non_static_model_entities: nsme.clone(),
            select_cube: SelectCube::new(),
            block_overlay: BlockOverlay::new(tex.id),
            ship_pos: Vec3::new(0.0,0.0,0.0),
            planet_y_offset: REQUIRED_SHIP_FLYAWAY_HEIGHT,
            window: window.clone(),
            guisys: GuiSystem::new(&window.clone(), &tex),
            hud,
            drops: Drops::new(tex.id, &cam, &chunksys, &inv, connectonstart, &needtosend.clone()),
            audiop,
            inventory: inv,
            animations: Vec::new(),
            skins: Vec::new(),
            nodes: Vec::new(),
            current_time: 0.0,
            netconn: NetworkConnector::new(&chunksys, &server_command_queue, &server_command_hp_queue, &kc, &my_uuid.clone(), &nsme, &cam.clone(), &pme.clone()),
            server_command_queue: server_command_queue.clone(),
            hp_server_command_queue: server_command_hp_queue.clone(),
            headless,
            known_cameras: kc,
            my_uuid,
            ambient_bright_mult: 1.0,
            daylength: 900.0,
            timeofday: Arc::new(Mutex::new(700.0)),
            sunrise_factor: 0.0,
            sunset_factor: 0.0,
            visions_timer : 0.0,
            visions_camera,
            current_vision: Some(VisionType::Model(0)),
            tex,
            inwater: false,
            headinwater: false,
            currentbuttons: vec![
                ("Test", "Yoo"),
                ("Test22", "22"),
            ],
            loadedworld: AtomicBool::new(false),
            addressentered: addressentered.clone(),
            address: address.clone(),
            player_model_entities: pme,
            mouse_slot: (0,0),
            needtosend
        };
        if !headless {
            g.load_model("assets/models/car/scene.gltf");
                g.load_model("assets/models/car/scene.gltf");
                //g.load_model("assets/models/ship/scene.gltf");
                g.load_model("assets/models/monster1/scene.gltf");
                g.load_model("assets/models/monster2/scene.gltf");
                g.load_model("assets/models/cow/scene.glb");

                println!("gltf model count: {}", g.gltf_models.len());
    
                g.create_model_vbos();
        }
        
                


        let aeclone = g.addressentered.clone();
        let aclone = g.address.clone();

        thread::spawn(move || {

            if !headless {
                
            
                // g.setup_vertex_attributes();
    
                //start coming down from the sky in ship
                //g.vars.ship_going_down = true;
                //g.vars.ship_going_up = false;
    
                g.wait_for_new_address();
                
                let mut audiop = g.audiop.write().unwrap();
                unsafe {
                    for string in SONGS {
                        audiop.preload(string, string);
                    }
                }
                
                
    
                audiop.preload_series("grassstepseries", vec![
                    "assets/sfx/grassstep1.mp3",
                    "assets/sfx/grassstep2.mp3",
                    "assets/sfx/grassstep3.mp3",
                    "assets/sfx/grassstep4.mp3",
                    "assets/sfx/grassstep5.mp3",
                    "assets/sfx/grassstep6.mp3",
                ]);

                audiop.preload_series("sandstepseries", vec![
                    "assets/sfx/sandstep1.mp3",
                    "assets/sfx/sandstep2.mp3",
                    "assets/sfx/sandstep3.mp3",
                    "assets/sfx/sandstep4.mp3",
                    "assets/sfx/sandstep5.mp3"
                ]);

                audiop.preload_series("doorseries", vec![
                    "assets/sfx/door.mp3",
                    "assets/sfx/door1.mp3",
                    "assets/sfx/door2.mp3",
                ]);

                audiop.preload_series("waterstepseries", vec![
                    "assets/sfx/water1.mp3",
                    "assets/sfx/water2.mp3",
                    "assets/sfx/water3.mp3",
                    "assets/sfx/water4.mp3",
                    "assets/sfx/water5.mp3"
                ]);
    
                audiop.preload_series("stonestepseries", vec![
                    "assets/sfx/stonestep1.mp3",
                    "assets/sfx/stonestep2.mp3",
                    "assets/sfx/stonestep3.mp3",
                    "assets/sfx/stonestep4.mp3"
                ]);

                audiop.preload_series("stoneplaceseries", vec![
                    "assets/sfx/stoneplace1.mp3",
                    "assets/sfx/stoneplace2.mp3",
                    "assets/sfx/stoneplace3.mp3",
                ]);


                audiop.preload_series("plantplaceseries", vec![
                    "assets/sfx/plantplace1.mp3",
                    "assets/sfx/plantplace2.mp3",
                    "assets/sfx/plantplace3.mp3"
                ]);

                audiop.preload_series("glassplaceseries", vec![
                    "assets/sfx/glassplace1.mp3",
                    "assets/sfx/glassplace2.mp3",
                    "assets/sfx/glassplace3.mp3",
                    "assets/sfx/glassplace4.mp3"
                ]);
    
                // g.initialize_being_in_world();
    
                // g.add_ship_colliders();
            }
            g
        })

        
        
    }
    
    pub fn wait_for_new_address(&mut self) {
        if self.vars.in_multiplayer {
    
    
            //print!("Enter server address (e.g., 127.0.0.1:6969): ");
            //io::stdout().flush().unwrap(); // Ensure the prompt is printed before reading input

            //let mut address = String::new();
            //io::stdin().read_line(&mut address).expect("Failed to read line");

            while !self.addressentered.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }

            let address = self.address.lock().unwrap().as_ref().unwrap().trim().to_string(); // Remove any trailing newline characters

            self.netconn.connect(address); // Connect to the provided address
            println!("Connected to the server!");
            
        }
    }
    pub fn button_command(&mut self, str: &'static str) {
        match str {
            "quittomainmenu" => {
                self.exit();
                self.window.write().unwrap().set_should_close(true);
            }
            "closemenu" => {
                self.vars.menu_open = false;
            }
            _ => {
                println!("Unknown button command given");
            }
        }
    }


    pub fn initialize_being_in_world(&mut self) -> JoinHandle<()> {
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

        // Update the ship's position
        ship_pos.y = decided_pos_y;




        let ship_float_pos = Vec3::new(ship_pos.x as f32, ship_pos.y as f32, ship_pos.z as f32);

        if self.vars.in_multiplayer {
            //ChunkSystem::initial_rebuild_on_main_thread(&self.chunksys.clone(), &self.shader0, &self.camera.lock().unwrap().position);
            while !self.netconn.received_world.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }
        }

        self.vars.hostile_world = (self.chunksys.read().unwrap().planet_type % 2) != 0;



        //self.audiop.play("assets/music/Farfromhome.mp3", &ship_float_pos, &Vec3::new(0.0,0.0,0.0));
        //self.audiop.play("assets/sfx/shipland28sec.mp3", &ship_float_pos, &Vec3::new(0.0,0.0,0.0));



        self.ship_pos = ship_float_pos;
        //self.static_model_entities.push(ModelEntity::new(1, ship_float_pos, 0.07, Vec3::new(PI/2.0, 0.0, 0.0), &self.chunksys, &self.camera));
        // self.static_model_entities.push(ModelEntity::new(4, ship_float_pos, 1.5, Vec3::new(0.0, 0.0, 0.0), &self.chunksys, &self.camera));
        self.camera.lock().unwrap().position = ship_float_pos  + Vec3::new(5.0, 2.0, 0.0);

        self.currentbuttons = vec![
            ("Loading...", "loading")
        ];
        self.vars.menu_open = true;

        let handle = self.rebuild_whole_world_while_showing_loading_screen();

        handle

        
    }

    pub fn update_inventory(&mut self) {
        for i in 20..40 {
            let realslotind = i - 20;
            let slot = self.chunksys.read().unwrap().chest_registry.entry(self.hud.current_chest).or_insert(
                ChestInventory { 
                    dirty: true, 
                    inv: [(0, 0), (0,0), (0,0), (0,0), (0,0),
                        (0,0), (0,0), (0,0), (0,0), (0,0),
                        (0,0), (0,0), (0,0), (0,0), (0,0),
                        (0,0), (0,0), (0,0), (0,0), (0,0)] 
                }
            ).inv[realslotind];
            let idinslot = slot.0;
            let texcoords = Blocks::get_tex_coords(idinslot, crate::cube::CubeSide::LEFT);
            let tf = TextureFace::new(texcoords.0 as i8, texcoords.1 as i8);
            let bf = TextureFace::new(0,0);
            self.hud.chestelements[i as usize].uvs = [
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

                    self.hud.chestelements[40 + realslotind * 2].uvs = [
                        g1.blx, g1.bly,
                        g1.brx, g1.bry,
                        g1.trx, g1.tr_y,

                        g1.trx, g1.tr_y,
                        g1.tlx, g1.tly,
                        g1.blx, g1.bly
                    ];
                    self.hud.chestelements[40 + realslotind * 2 + 1].uvs = [
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
                    self.hud.chestelements[40 + realslotind * 2].uvs = [
                        bf.blx, bf.bly,
                        bf.brx, bf.bry,
                        bf.trx, bf.tr_y,

                        bf.trx, bf.tr_y,
                        bf.tlx, bf.tly,
                        bf.blx, bf.bly
                    ];
                    self.hud.chestelements[40 + realslotind * 2 + 1].uvs = [
                        g2.blx, g2.bly,
                        g2.brx, g2.bry,
                        g2.trx, g2.tr_y,

                        g2.trx, g2.tr_y,
                        g2.tlx, g2.tly,
                        g2.blx, g2.bly
                    ];
                }
            } else {
                self.hud.chestelements[40 + realslotind * 2].uvs = [
                        bf.blx, bf.bly,
                        bf.brx, bf.bry,
                        bf.trx, bf.tr_y,

                        bf.trx, bf.tr_y,
                        bf.tlx, bf.tly,
                        bf.blx, bf.bly
                    ];
                self.hud.chestelements[40 + realslotind * 2 + 1].uvs = [
                    bf.blx, bf.bly,
                    bf.brx, bf.bry,
                    bf.trx, bf.tr_y,

                    bf.trx, bf.tr_y,
                    bf.tlx, bf.tly,
                    bf.blx, bf.bly
                ];
            }
        }
        
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





        let slot = self.mouse_slot;
        let idinslot = slot.0;
        let texcoords = Blocks::get_tex_coords(idinslot, crate::cube::CubeSide::LEFT);
        let tf = TextureFace::new(texcoords.0 as i8, texcoords.1 as i8);
        let bf = TextureFace::new(0,0);
        self.hud.chestelements[80].uvs = [
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

                self.hud.chestelements[81].uvs = [
                    g1.blx, g1.bly,
                    g1.brx, g1.bry,
                    g1.trx, g1.tr_y,

                    g1.trx, g1.tr_y,
                    g1.tlx, g1.tly,
                    g1.blx, g1.bly
                ];
                self.hud.chestelements[82].uvs = [
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
                self.hud.chestelements[81].uvs = [
                    bf.blx, bf.bly,
                    bf.brx, bf.bry,
                    bf.trx, bf.tr_y,

                    bf.trx, bf.tr_y,
                    bf.tlx, bf.tly,
                    bf.blx, bf.bly
                ];
                self.hud.chestelements[82].uvs = [
                    g2.blx, g2.bly,
                    g2.brx, g2.bry,
                    g2.trx, g2.tr_y,

                    g2.trx, g2.tr_y,
                    g2.tlx, g2.tly,
                    g2.blx, g2.bly
                ];
            }
        } else {
            self.hud.chestelements[81].uvs = [
                    bf.blx, bf.bly,
                    bf.brx, bf.bry,
                    bf.trx, bf.tr_y,

                    bf.trx, bf.tr_y,
                    bf.tlx, bf.tly,
                    bf.blx, bf.bly
                ];
            self.hud.chestelements[82].uvs = [
                bf.blx, bf.bly,
                bf.brx, bf.bry,
                bf.trx, bf.tr_y,

                bf.trx, bf.tr_y,
                bf.tlx, bf.tly,
                bf.blx, bf.bly
            ];
        }









        
        self.hud.dirty = true;
    }

    pub fn add_to_inventory(inv: &Arc<RwLock<Inventory>>, id: u32, count: u32, in_m: bool, needtosend: &Arc<Queue<Message>>) -> Result<bool, bool> {

        if in_m {

            let n = needtosend.clone();

            let inventory = inv.read().unwrap();
        
            // First, try to find an item with the given `id`
            if let Some((index, item)) = inventory.inv.iter().enumerate().find(|(index, item)| item.0 == id) {
                let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::ZERO, id as f32, index as u32);
                msg.infof = item.1 as f32 + 1.0;
                msg.info2 = 1;

                n.push(msg);
                // item.1 += count;
                // inventory.dirty = true;
                return Ok(true);
            }

            // If not found, try to find an empty slot to add the new item
            if let Some((index, item)) = inventory.inv.iter().enumerate().find(|(index, item)| item.0 == 0) {
                
                let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::ZERO, id as f32, index as u32);
                msg.infof = 1.0;
                msg.info2 = 1;

                n.push(msg);
                // item.0 = id;
                // item.1 = count;
                // inventory.dirty = true;
                return Ok(true);
            }


            Err(false)
        } else {
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
        
    }


    pub fn do_step_sounds(&mut self) {
        static mut TIMER: f32 = 0.0;
        static mut LAST_CAM_POS: Vec3 = Vec3{x: 0.0, y: 0.0, z: 0.0};
        let campos = self.camera.lock().unwrap().position;

        
        unsafe {
            let diff = campos.distance(LAST_CAM_POS); 

            if diff > self.delta_time * 2.0 {
               
                if TIMER > 0.4 {
                    self.do_step_sound_now(campos);
                    TIMER = 0.0;
                } else {
                    TIMER += self.delta_time;
                }

                LAST_CAM_POS = campos;
            }
        }
        
    }

    pub fn do_step_sound_now(&mut self, position: Vec3) {
        let campos = position;
        let camfootpos = campos - Vec3::new(0.0, 2.0, 0.0);
        let blockat = self.chunksys.read().unwrap().blockat(IVec3::new(camfootpos.x.floor() as i32, camfootpos.y.floor() as i32, camfootpos.z.floor() as i32));
        let blockat = blockat & Blocks::block_id_bits();
        if blockat != 0 {
            self.audiop.write().unwrap().play_next_in_series(&Blocks::get_walk_series(blockat), &camfootpos, &Vec3::new(0.0, 0.0, 0.0), 0.5);
        }
        
    }

    pub fn takeoff_ship(&mut self) {
        if !self.vars.ship_taken_off {
            self.audiop.write().unwrap().play("assets/sfx/shiptakeoff.mp3", &self.ship_pos, &Vec3::ZERO, 1.0);
            self.vars.ship_going_up = true;
            self.vars.ship_going_down = false;
            self.vars.ship_taken_off = true;
            self.vars.on_new_world = false;
        }
        
    }

    pub fn gaussian (x: f32, peak: f32, radius: f32) -> f32 {
        let std_dev = radius / 3.0;  // Controls the spread
        let variance = std_dev * std_dev;

        // Gaussian formula
        let b = f32::exp(-(x - peak).powf(2.0) / (2.0 * variance));

        // Normalize the peak to 1
        let peak_height = f32::exp(-(peak - peak).powf(2.0) / (2.0 * variance));

        return b / peak_height;
    }

    pub fn update(&mut self) {
        
        let current_time = unsafe { glfwGetTime() as f32 };
        self.delta_time = (current_time - self.prev_time).min(0.05);

        unsafe {
            if SONGTIMER < SONGINTERVAL {
                SONGTIMER += self.delta_time;
            } else {
                SONGTIMER = 0.0;
                self.audiop.write().unwrap().play_in_head(SONGS[SONGINDEX]);
                SONGINDEX = (SONGINDEX + 1) % SONGS.len();
                
            }
        }

        self.prev_time = current_time;
        let mut todlock = self.timeofday.lock().unwrap();
        *todlock = (*todlock + self.delta_time) % self.daylength;

        let gaussian_value = Self::gaussian(*todlock, self.daylength / 2.0, self.daylength / 2.0) * 1.3;
        self.ambient_bright_mult = gaussian_value.clamp(0.08, 1.0);

        self.sunset_factor = Self::gaussian(*todlock, self.daylength*(3.0/4.0), self.daylength/16.0);
        self.sunrise_factor = Self::gaussian(*todlock, self.daylength/6.0, self.daylength/16.0);

        drop(todlock);

        let fadersread = self.faders.read().unwrap();

        let overlayfade = fadersread[FaderNames::VisionsFader as usize].value.clone();

        drop(fadersread);

        if !self.headless {

            let (x,y) = self.window.read().unwrap().get_cursor_pos();

            self.hud.mousetrans = HudElement::xytondc(x, y);
            if self.hud.chest_open {
                let mut isoverlappingany = false;
                for i in 0..5 {
                    let hudel = &self.hud.elements[i];
                    
                    if hudel.overlaps(x, y) {
                        unsafe {
                            MOUSED_SLOT = SlotIndexType::InvSlot(i as i32);
                            isoverlappingany = true;
                        }
                    }
                }

                for i in 0..20 {
                    let hudel = &self.hud.chestelements[i];
                    
                    if hudel.overlaps(x, y) {
                        unsafe {
                            MOUSED_SLOT = SlotIndexType::ChestSlot(i as i32);
                            isoverlappingany = true;
                        }
                    }
                }
                if !isoverlappingany {
                    unsafe 
                    {MOUSED_SLOT = SlotIndexType::None;}
                }
                // unsafe {
                //     match MOUSED_SLOT {
                //         SlotIndexType::ChestSlot(e) => {
                //             println!("Moused chest slot {}", HudElement::ass_slot_to_shader_float(&MOUSED_SLOT));
                //         },
                //         SlotIndexType::InvSlot(e) => {
                //             println!("Moused inv slot {}", HudElement::ass_slot_to_shader_float(&MOUSED_SLOT));
                //         },
                //         SlotIndexType::None => {
                //             println!("Moused no invslot");
                //         },
                //     }
                // }
            }
            match self.needtosend.pop() {
                Some(comm) => {
                    self.netconn.send(&comm);
                }
                None => {

                }
            }
            let mut morestuff = true;
            while morestuff {
                match self.hp_server_command_queue.pop() {
                    Some(comm) => {
                        match comm.message_type {
                            MessageType::BlockSet => {
                                if comm.info == 0 {
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32), 
                                        comm.info, true, true);
                                    } else {
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32), 
                                        comm.info, false, true);
                                    }
                            }
                            MessageType::MultiBlockSet => {


                                        self.chunksys.read().unwrap().set_block_no_sound(IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32), 
                                        comm.info, true);

                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(comm.otherpos, 
                                        comm.info2, true, true);

                            }
                            MessageType::ChestInvUpdate => {
                                let currchest = comm.otherpos;
            
                                let destslot = comm.info;
            
                                let slotindextype = match comm.info2 {
                                    0 => {
                                        SlotIndexType::ChestSlot(destslot as i32)
                                    }
                                    1 => {
                                        SlotIndexType::InvSlot(destslot as i32)
                                    }
                                    _ => {
                                        SlotIndexType::None
                                    }
                                };

                                let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);

                                let mut updateinv = false;

                                match *self.my_uuid.read().unwrap() {
                                    Some(ud) => {
                                        if uuid == ud && comm.z == 1.0 {
                                            self.mouse_slot.0 = comm.x as u32;
                                            self.mouse_slot.1 = comm.y as u32;
                                            updateinv = true;
                                        }
                                    },
                                    None => {

                                    },
                                }

                                
            
                                match slotindextype {
                                    SlotIndexType::ChestSlot(e) => {
                                        let csys = self.chunksys.write().unwrap();
                                        let mut chestinv = csys.chest_registry.entry(currchest).or_insert(ChestInventory {
                                            dirty: false, inv: [(0,0), (0,0), (0,0), (0,0), (0,0),
                                            (0,0), (0,0), (0,0), (0,0), (0,0), 
                                            (0,0), (0,0), (0,0), (0,0), (0,0), 
                                            (0,0), (0,0), (0,0), (0,0), (0,0) ]
                                        });
            
                                        let slot = &mut chestinv.inv[e as usize];
            
                                       // let wasthere = slot.clone();
            
                                        slot.0 = comm.rot as u32;
                                        slot.1 = comm.infof as u32;
                                        updateinv = true;
                                        //comm.x = wasthere.0 as f32; comm.y = wasthere.1 as f32;
                                    }
                                    SlotIndexType::InvSlot(e) => {


                                        match *self.my_uuid.read().unwrap() {
                                            Some(ud) => {
                                                if uuid == ud {
                                                    let mut playerinv = &mut self.inventory.write().unwrap();
                                                    let slot = &mut playerinv.inv[e as usize];
                
                                                   // let wasthere = slot.clone();
                
                                                    slot.0 = comm.rot as u32;
                                                    slot.1 = comm.infof as u32;
        
                                                    updateinv = true;
        
                
                                                }
                                            }
                                            None => {

                                            }
                                        }
                                        
                                        
                                        
                                        
                                        //comm.x = wasthere.0 as f32; comm.y = wasthere.1 as f32;

                                        
                                    }
                                    SlotIndexType::None => {
            
                                    }
                                }


                                if updateinv {
                                    self.update_inventory();
                                }
            
                                
                            }
                            
                            _ => {

                            }
                        }
                    }
                    None => {
                        morestuff = false;
                    }
                }
            }
            //let mut morestuff = true;
            //while morestuff {
                match self.server_command_queue.pop() {
                    Some(comm) => {
                        match comm.message_type {
                            MessageType::RequestTakeoff => {
                                self.takeoff_ship();
                            }
                            MessageType::TimeUpdate => {
                                let mut todlock = self.timeofday.lock().unwrap();
                                *todlock = comm.infof;
                            }
                            MessageType::BlockSet => {
                                if comm.info == 0 {
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32), 
                                        comm.info, true, true);
                                    } else {
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32), 
                                        comm.info, false, true);
                                    }
                            }
                            MessageType::MobUpdate => {
                                let newpos = Vec3::new(comm.x, comm.y, comm.z);
                                let id = comm.info;
                                let modind = comm.info2;
                                let rot = comm.rot;
                                let scale = comm.infof;

                                let nsme = self.non_static_model_entities.clone();
                                //println!("NSME Length: {}", nsme.len());
                                match nsme.get_mut(&id) {
                                    Some(mut me) => {
                                        let modent = me.value_mut();
                                        (*modent).lastpos = (*modent).position.clone();
                                        (*modent).position = newpos;
                                        (*modent).scale = scale;
                                        (*modent).lastrot = (*modent).rot.clone();
                                        (*modent).rot = Vec3::new(0.0, rot, 0.0);
                                        unsafe {
                                            (*modent).time_stamp = glfwGetTime();
                                        }
                                        
                                        
                                    }
                                    None => {
                                        //println!("Received an update for a mob {} that doesn't exist. Creating it...", id);
                                        self.insert_static_model_entity(id, modind as usize, newpos, scale, Vec3::new(0.0,rot,0.0), 5.0);
                                    }
                                };
                            }
                            MessageType::PlayerUpdate => {
                                let newpos = Vec3::new(comm.x, comm.y, comm.z);
                                //let id = comm.info;
                                let modind = comm.info2;
                                let rot = comm.rot;
                                let scale = 0.3;

                                let pme: Arc<DashMap<Uuid, ModelEntity>> = self.player_model_entities.clone();


                                let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);
                                //println!("NSME Length: {}", nsme.len());
                                match pme.get_mut(&uuid) {
                                    Some(mut me) => {
                                        let modent = me.value_mut();
                                        (*modent).lastpos = (*modent).position.clone();
                                        (*modent).position = newpos;
                                        (*modent).scale = scale;
                                        (*modent).lastrot = (*modent).rot.clone();
                                        (*modent).rot = Vec3::new(0.0, rot, 0.0);
                                        unsafe {
                                            (*modent).time_stamp = glfwGetTime();
                                        }
                                        
                                        
                                    }
                                    None => {
                                        println!("Received an update for a player {} that doesn't exist. Creating it...", uuid);
                                        self.insert_player_model_entity(uuid, modind as usize, newpos, scale, Vec3::new(0.0,rot,0.0), 5.0);
                                    }
                                };
                            }
                            MessageType::Seed => {
                                //Means we're going to a new world
                                self.non_static_model_entities.clear();
                            }
                            _ => {

                            }
                        }
                    }
                    None => {
                        //morestuff = false;
                    }
                }
            //}
        
            

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

            self.tex.update_texture(self.delta_time);


            let overlaycolor = Vec4::new(0.0, 0.0, 1.0, overlayfade);
            let overlaycolor2 = Vec4::new(1.0, 0.0, 0.0, overlayfade);
            if overlayfade > 0.0 {
                self.draw_sky(overlaycolor, overlaycolor2, 1.0, 0.0);
                self.draw_current_vision(overlayfade);
                unsafe {
                    if self.visions_timer > 3.0 {
                        self.faders.write().unwrap()[FaderNames::VisionsFader as usize].down();

                    } else {
                        self.visions_timer += self.delta_time;
                    }
                }
            }
            


            self.audiop.write().unwrap().update();


            let camlock = self.camera.lock().unwrap();
            let pos = camlock.position;
            let forward = camlock.direction;
            let vel = camlock.velocity;
            let up = camlock.up;
            drop(camlock);
            self.audiop.write().unwrap().set_listener_attributes(libfmod::Vector { x: pos.x, y: pos.y, z: pos.z }, libfmod::Vector { x: vel.x, y: vel.y, z: vel.z }, libfmod::Vector { x: forward.x, y: forward.y, z: forward.z }, libfmod::Vector { x: up.x, y: up.y, z: up.z });
            self.do_step_sounds();
            if self.inventory.read().unwrap().dirty {
                self.update_inventory();
            }

            if self.vars.ship_taken_off {
                if !self.vars.on_new_world {
                    if self.planet_y_offset > REQUIRED_SHIP_FLYAWAY_HEIGHT {

                    } else {
                        
                    
                        self.new_world_func();
                        
                        
                        //self.audiop.play("assets/sfx/shipland28sec.mp3", &self.ship_pos, &Vec3::ZERO);
                        
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
            
            
            // let camlock = self.camera.lock().unwrap();
            // let shipdist = camlock.position.distance(self.ship_pos);
            // if shipdist < 30.0 && shipdist > 10.0 {
            //     self.vars.near_ship = true;
            //     self.guisys.draw_text(1);
            // } else {
            //     self.vars.near_ship = false;
            // }
            // drop(camlock);

            
            let planet_speed = -self.planet_y_offset.clamp(-100.0, -0.5);

            if self.vars.ship_going_down {
                self.planet_y_offset = (self.planet_y_offset + self.delta_time * planet_speed).clamp(-1000.0, 0.0);
            }
            if self.vars.ship_going_up {
                self.planet_y_offset = (self.planet_y_offset - self.delta_time * planet_speed).clamp(-1000.0, 0.0);
            }
        }

        if self.initial_timer < 1.5  {
            self.initial_timer += self.delta_time;
        } else {
            if self.headless {
                
                self.update_non_static_model_entities();  
            } else {
                if !self.vars.in_multiplayer {
                    self.update_non_static_model_entities();  
                }
                if overlayfade <= 0.1 {
                    self.update_movement_and_physics();
                }
                
            }
            
            
        }

        //println!("Planet y off: {}", self.planet_y_offset);
        
        
    }

    pub fn update_movement_and_physics(&mut self) { 

        let camarc = self.camera.clone();
        let mut camlock = camarc.lock().unwrap();


        static mut wasngrounded: bool = false;

        match *self.my_uuid.read().unwrap() {
            Some(uuid) => {
                match self.known_cameras.get_mut(&uuid) {
                    Some(mut pos) => {
                        *pos = camlock.position;
                    }
                    None => {

                    }
                }
            }
            None => {

            }
        }


        let feetpos = camlock.position - Vec3::new(0.0, 1.0, 0.0);

        let feetposi = vec::IVec3::new(feetpos.x.floor() as i32, feetpos.y.floor() as i32, feetpos.z.floor() as i32);
        let headposi = vec::IVec3::new(camlock.position.x.floor() as i32, camlock.position.y.floor() as i32, camlock.position.z.floor() as i32);
        let feetposi2 = vec::IVec3::new(feetpos.x.floor() as i32, (feetpos.y-0.25).floor() as i32, feetpos.z.floor() as i32);

        let blockfeetin = self.chunksys.read().unwrap().blockat(feetposi) & Blocks::block_id_bits();
        let blockfeetinlower = self.chunksys.read().unwrap().blockat(feetposi2) & Blocks::block_id_bits();
        
        let blockheadin = self.chunksys.read().unwrap().blockat(headposi) & Blocks::block_id_bits();

        let feetinwater = blockfeetin == 2;
        let feetinwaterlower = blockfeetinlower == 2;

        if Blocks::is_climbable(blockfeetin) || Blocks::is_climbable(blockheadin) {
            self.vars.in_climbable = true;
        }

        if !Blocks::is_climbable(blockfeetinlower) && !Blocks::is_climbable(blockheadin) {
            self.vars.in_climbable = false;
        }

        if feetinwater {
            self.inwater = true;
        }

        if !feetinwaterlower {
            self.inwater = false;
        }


        if !self.coll_cage.solid.contains(&Side::FLOOR) {
            self.grounded = false;
            unsafe{ wasngrounded = true; }
        } else {
        }

        const GRAV: f32 = 9.8;
        if self.inwater || self.vars.in_climbable {
            self.time_falling_scalar = 1.0;
            if !self.grounded {
                camlock.velocity += Vec3::new(0.0, -2.0*self.delta_time, 0.0);
                if self.controls.shift {
                    camlock.velocity += Vec3::new(0.0, -5.0*self.delta_time, 0.0);
                }
            }

            if self.controls.up {
                camlock.velocity += Vec3::new(0.0, 5.0*self.delta_time, 0.0);
            }
        }
        else {
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
        }
           

        let cc_center = camlock.position + Vec3::new(0.0, -1.0, 0.0);
        self.coll_cage.update_readings(cc_center);

        

        let mut proposed = camlock.respond_to_controls(&self.controls, &self.delta_time, 5.0);
        self.user_bound_box
            .set_center(proposed + Vec3::new(0.0, -0.5  , 0.0), 0.2, 0.85);
        self.coll_cage.update_colliding(&self.user_bound_box);
        let mut corr_made: Vec<Vec3> = Vec::new();

        let mut stepsoundqueued = false;

        if self.coll_cage.colliding.len() > 0 {
            for side in &self.coll_cage.colliding {
                if !corr_made.contains(&self.coll_cage.normals[*side as usize]) {
                    proposed += self.coll_cage.normals[*side as usize]
                        * self.coll_cage.penetrations[*side as usize];
                    corr_made.push(self.coll_cage.normals[*side as usize]);
                }
                if *side == Side::FLOOR {
                    self.grounded = true;
                    unsafe
                    {if wasngrounded {
                        stepsoundqueued = true;
                        wasngrounded = false; 
                    }}
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

        let pos = camlock.position.clone();
        drop(camlock);

        if stepsoundqueued {
            self.do_step_sound_now(pos);
        }

    }

    pub fn draw_sky(&self, top: Vec4, bot: Vec4, amb: f32, pitch: f32) {
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
            gl::Uniform1f(C_P_LOC, pitch);
            gl::Uniform3f(
                C_D_LOC,
                camlock.direction.x,
                camlock.direction.y,
                camlock.direction.z,
            );
            drop(camlock);

            gl::Uniform4f(T_C_LOC, top.x, top.y, top.z, top.w);
            gl::Uniform4f(B_C_LOC, bot.x, bot.y, bot.z, bot.w);

            gl::Uniform1f(A_B_LOC, amb);
            gl::Uniform1f(S_S_LOC, self.sunset_factor);
            gl::Uniform1f(S_R_LOC, self.sunrise_factor);

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
                        self.chunksys.read().unwrap().blockat(hit) & Blocks::block_id_bits()
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
        

        let campitch = self.camera.lock().unwrap().pitch;

        //Sky
        match self.vars.hostile_world {
            true => {
                self.draw_sky(self.vars.hostile_world_sky_color, self.vars.hostile_world_sky_bottom, self.ambient_bright_mult, campitch);
            }
            false => {
                self.draw_sky(self.vars.sky_color, self.vars.sky_bottom, self.ambient_bright_mult, campitch);
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
                cmemlock.memories[ready.geo_index].vlength = ready.newvlength;
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
                let vv = cmemlock.memories[ready.geo_index].vvbo;
                let uvv = cmemlock.memories[ready.geo_index].uvvbo;

                WorldGeometry::bind_geometry(v32, v8, true, &self.shader0, bankarc.solids());
                WorldGeometry::bind_geometry(
                    tv32,
                    tv8,
                    true,
                    &self.shader0,
                    bankarc.transparents(),
                );

                WorldGeometry::bind_old_geometry(vv, uvv, &bankarc.vdata.lock().unwrap(), &bankarc.uvdata.lock().unwrap(), &self.oldshader);
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
                cmemlock.memories[ready.geo_index].vlength = ready.newvlength;
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

                let vv = cmemlock.memories[ready.geo_index].vvbo;
                let uvv = cmemlock.memories[ready.geo_index].uvvbo;

                WorldGeometry::bind_geometry(v32, v8, true, &self.shader0, bankarc.solids());
                WorldGeometry::bind_geometry(
                    tv32,
                    tv8,
                    true,
                    &self.shader0,
                    bankarc.transparents(),
                );

                WorldGeometry::bind_old_geometry(vv, uvv, &bankarc.vdata.lock().unwrap(), &bankarc.uvdata.lock().unwrap(), &self.oldshader);

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
                                cmemlock.memories[ready.geo_index].vlength = ready.newvlength;
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
                                let vv = cmemlock.memories[ready.geo_index].vvbo;
                                let uvv = cmemlock.memories[ready.geo_index].uvvbo;
                
                                WorldGeometry::bind_geometry(v32, v8, true, &self.shader0, bankarc.solids());
                                WorldGeometry::bind_geometry(
                                    tv32,
                                    tv8,
                                    true,
                                    &self.shader0,
                                    bankarc.transparents(),
                                );
                            
                                WorldGeometry::bind_old_geometry(vv, uvv, &bankarc.vdata.lock().unwrap(), &bankarc.uvdata.lock().unwrap(), &self.oldshader);
                        
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
            gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, self.ambient_bright_mult);
            gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
            gl::Uniform1f(UNDERWATER_LOC, 0.0);
            gl::Uniform3f(
                CAM_DIR_LOC,
                cam_lock.direction.x,
                cam_lock.direction.y,
                cam_lock.direction.z,
            );
            gl::Uniform1f(SUNSET_LOC, self.sunset_factor);
            gl::Uniform1f(SUNRISE_LOC, self.sunrise_factor);
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
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
        
        for (index, cfl) in cmem.memories.iter().enumerate() {
            if cfl.used {
                let dd1: Mutex<Vec<u32>> = Mutex::new(Vec::new());
                let dd2: Mutex<Vec<u8>> = Mutex::new(Vec::new());
                let dd: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>) = (&dd1, &dd2);

                unsafe {
                    gl::BindVertexArray(self.shader0.vao);
                    gl::UseProgram(self.shader0.shader_id);
                }

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
                //We drew the transparents, then...

                unsafe {
                    gl::Enable(gl::CULL_FACE);
                }

                unsafe {
                    gl::BindVertexArray(self.oldshader.vao);
                    gl::UseProgram(self.oldshader.shader_id);
                }


        static mut MVP_LOC: i32 = -1;
        static mut CAM_POS_LOC: i32 = 0;
        static mut AMBIENT_BRIGHT_MULT_LOC: i32 = 0;
        static mut VIEW_DISTANCE_LOC: i32 = 0;
        static mut UNDERWATER_LOC: i32 = 0;
        static mut CAM_DIR_LOC: i32 = 0;
        static mut SUNSET_LOC: i32 = 0;
        static mut SUNRISE_LOC: i32 = 0;
        unsafe {
            if MVP_LOC == -1 {

                MVP_LOC =
                    gl::GetUniformLocation(self.oldshader.shader_id, b"mvp\0".as_ptr() as *const i8);
                //println!("MVP LOC: {}", MVP_LOC);
                CAM_POS_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"camPos\0".as_ptr() as *const i8,
                );
                AMBIENT_BRIGHT_MULT_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"ambientBrightMult\0".as_ptr() as *const i8,
                );
                VIEW_DISTANCE_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"viewDistance\0".as_ptr() as *const i8,
                );
                UNDERWATER_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"underWater\0".as_ptr() as *const i8,
                );
                CAM_DIR_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"camDir\0".as_ptr() as *const i8,
                );
                SUNSET_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"sunset\0".as_ptr() as *const i8,
                );
                SUNRISE_LOC = gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"sunrise\0".as_ptr() as *const i8,
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
            gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, self.ambient_bright_mult);
            gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
            gl::Uniform1f(UNDERWATER_LOC, 0.0);
            gl::Uniform3f(
                CAM_DIR_LOC,
                cam_lock.direction.x,
                cam_lock.direction.y,
                cam_lock.direction.z,
            );
            gl::Uniform1f(SUNSET_LOC, self.sunset_factor);
            gl::Uniform1f(SUNRISE_LOC, self.sunrise_factor);
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.oldshader.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                0,
            );
            // let fc = Planets::get_fog_col(self.chunksys.read().unwrap().planet_type as u32);
            // gl::Uniform4f(
            //     FOGCOL_LOC,
            //     fc.0, 
            //     fc.1,
            //     fc.2,
            //     fc.3
            // );

            drop(cam_lock);
        }

        WorldGeometry::bind_old_geometry_no_upload(cfl.vvbo, cfl.uvvbo, &self.oldshader);


        unsafe {

            gl::DrawArrays(gl::TRIANGLES, 0, cfl.vlength as i32 / 5);
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

    pub fn exit(&mut self) {
        (*self.run_chunk_thread).store(false, Ordering::Relaxed);


        if let Some(handle) = self.chunk_thread.take() {
            handle.join().unwrap();
            println!("Thread joined successfully!");
        } else {
            println!("No thread to join or already joined.");
        }

        self.drops.drops.clear();
        self.non_static_model_entities.clear();
        self.chunksys.write().unwrap().exit();
    }

    
    pub fn start_chunks_with_radius(&mut self, newradius: u8, seed: u32, nt: usize) {

        (*self.run_chunk_thread).store(false, Ordering::Relaxed);


        if let Some(handle) = self.chunk_thread.take() {
            handle.join().unwrap();
            println!("Thread joined successfully!");
        } else {
            println!("No thread to join or already joined.");
        }
        



        self.drops.drops.clear();
        self.non_static_model_entities.clear();



        self.chunksys.write().unwrap().reset(newradius, seed, nt);

        self.chunksys.write().unwrap().voxel_models = Some(self.voxel_models.clone());

        //self.drops.csys = self.chunksys.clone();

        let mut rng = StdRng::from_entropy();
        
        if !self.vars.in_multiplayer {
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
        }

        self.initialize_being_in_world();

        self.start_world();
    }


    pub fn rebuild_whole_world_while_showing_loading_screen(&mut self) -> std::thread::JoinHandle<()> {

        let csys = self.chunksys.clone();
        let campos = self.camera.lock().unwrap().position.clone();
        let shader = self.shader0.clone();

        let threadhandle = thread::spawn(move|| {
            ChunkSystem::initial_rebuild_on_main_thread(&csys, &shader, &campos)
        });

        threadhandle

        // while !threadhandle.is_finished() {

        //     //self.draw();
        //     self.window.read().unwrap()
        //     let current_time = unsafe { glfwGetTime() as f32 };
        //     self.delta_time = current_time - self.prev_time;
    
        //     self.prev_time = current_time;

        // }
        
        // match threadhandle.join() {
        //     Ok(_) => {

        //     }
        //     Err(_) => {
        //         tracing::info!("The whole-world-rebuild thread didn't join back I guess????");
        //     }
        // };
    }

    pub fn chunk_thread_inner_function(cam_arc: &Arc<Mutex<Camera>>, csys_arc: &Arc<RwLock<ChunkSystem>>, last_user_c_pos: &mut vec::IVec2) {

        let mut lightstuff = true;
        while lightstuff {

            let csys_arc = csys_arc.read().unwrap();

            match csys_arc.light_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, true, true);
                }
                None => {
                    lightstuff = false;
                }
            }
        }
        

        let mut userstuff = true;
        while userstuff {

            let csys_arc = csys_arc.read().unwrap();

            match csys_arc.user_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, true, false);


                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, true);
                        }
                        None => {
                        }
                    }
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
                    csys_arc.rebuild_index(index, true, false);
                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, false);
                            
                        }
                        None => {}
                    }
                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, true);
                        }
                        None => {
                        }
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
                    csys_arc.rebuild_index(index, false, false);
                    
                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, false);
                            let mut userstuff = true;
                            
                        }
                        None => {}
                    }

                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, true);
                        }
                        None => {
                        }
                    }

                    match csys_arc.gen_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, false);
                            let mut genstuff = true;
                            while genstuff {
                                match csys_arc.gen_rebuild_requests.pop() {
                                    Some(index) => {
                                        csys_arc.rebuild_index(index, true, false);
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
                            csys_arc.rebuild_index(index, true, false);
                            break;
                        }
                        None => {}
                    }
                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, true);
                            break;
                        }
                        None => {
                        }
                    }
                    match csys_arc.gen_rebuild_requests.pop() {
                        Some(index) => {
                            csys_arc.rebuild_index(index, true, false);
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
                let blockbits = self.chunksys.read().unwrap().blockat(block_hit);
                let blockat = blockbits & Blocks::block_id_bits();
                if blockat == 16 {
                    let mut set: HashSet<IVec2> = HashSet::new();
                    Game::delete_block_recursively(&self.chunksys, 16,  block_hit, &mut set);
                    for key in set {
                        self.chunksys.read().unwrap().queue_rerender_with_key(key, true, false);
                    }
                    self.drops.add_drop(tip, 17);
                } else if blockat == 19 { //Door stuff
                    let top = DoorInfo::get_door_top_bit(blockbits);
                    let mut other_half;

                    if top == 1 {
                        other_half = block_hit + IVec3::new(0, -1, 0);
                    } else {
                        other_half = block_hit + IVec3::new(0, 1, 0);
                    }

                    if self.vars.in_multiplayer {
                        let mut message = Message::new(
                            MessageType::MultiBlockSet,
                            Vec3::new(block_hit.x as f32, block_hit.y as f32, block_hit.z as f32),
                            0.0,
                            0
                        );
                        message.info2 = 0;
                        message.otherpos = other_half;

                        self.netconn.send(&message);
                    } else {
                        self.chunksys.read().unwrap().set_block(block_hit, 0, true);
                        self.chunksys.read().unwrap().set_block_and_queue_rerender(other_half, 0, true, true);
                    }

                } else {
                    if blockat != 0 {
                        self.drops.add_drop(tip, blockat);
                    }
                    
                    //TODO: PROBLEM HERE THAT WILL ALLOW USERS TO KEEP DUPING A BLOCK AS LONG AS THE SERVER DOESNT RESPOND
                    if self.vars.in_multiplayer {
                        let message = Message::new(MessageType::BlockSet, Vec3::new(block_hit.x as f32, block_hit.y as f32, block_hit.z as f32), 0.0, 0);
                        self.netconn.send(&message);
                    } else {
                        self.chunksys.read().unwrap().set_block_and_queue_rerender(block_hit, 0, true, true);
                    }
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

        let mut updateinv = false;

        if true {

            let cl = self.camera.lock().unwrap();

            match raycast_voxel(cl.position, cl.direction, &self.chunksys, 10.0) {
                
                Some((tip, block_hit)) => {

                    let mut blockbitshere = self.chunksys.read().unwrap().blockat(block_hit);
                    let blockidhere = blockbitshere & Blocks::block_id_bits();

                    if blockidhere == 19 {
                        let top = DoorInfo::get_door_top_bit(blockbitshere);
                        let mut otherhalf;

                        if top == 1 {
                            otherhalf = block_hit + IVec3::new(0,-1,0);
                            
                        } else {
                            otherhalf = block_hit + IVec3::new(0, 1, 0);
                        }
                        let mut otherhalfbits = self.chunksys.read().unwrap().blockat(otherhalf);

                        DoorInfo::toggle_door_open_bit(&mut blockbitshere);
                        DoorInfo::toggle_door_open_bit(&mut otherhalfbits);

                        if self.vars.in_multiplayer {
                            let mut message = Message::new(
                                MessageType::MultiBlockSet,
                                Vec3::new(block_hit.x as f32, block_hit.y as f32, block_hit.z as f32),
                                0.0,
                                blockbitshere
                            );
                            message.info2 = otherhalfbits;
                            message.otherpos = otherhalf;
                            self.netconn.send(&message);
                        } else {
                            self.chunksys.write().unwrap().set_block(otherhalf, otherhalfbits, true);
                            self.chunksys.write().unwrap().set_block_and_queue_rerender(block_hit, blockbitshere, true, true);
                        }
                    } else if blockidhere == 21 { //RIGHT CLICKED A CHEST
                        
                        let csys = self.chunksys.write().unwrap();

                        self.hud.current_chest = block_hit;
                        updateinv = true;
                        

                        

                    } else if slot.0 != 0 && slot.1 > 0 {
                         
                        let id = slot.0;
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
                        println!("Placing {} at {} {} {}", id, place_point.x, place_point.y, place_point.z);

                        if id == 19 { //Door shit

                            let neighbor_axes = vec![
                                IVec3::new(1,0,0),
                                IVec3::new(0,0,1),
                                IVec3::new(1,0,0),
                                IVec3::new(0,0,1)
                            ];

                            let place_above = place_point + IVec3::new(0,1,0);
                            let place_below = place_point + IVec3::new(0,-1,0);

                            let csysread = self.chunksys.read().unwrap();

                            let condition1 = csysread.blockat(place_above) == 0;
                            let condition2 = csysread.blockat(place_below) != 0;

                            drop(csysread);

                            if condition1 && condition2 {

                            let mut bottom_id = id;
                            let mut top_id = id;

                            top_id |= door::DOORTOP_BITS;


                            let diffx = cl.position.x - place_point.x as f32;
                            let diffz = cl.position.z - place_point.z as f32;


                            let mut direction = 0;

                            if diffx.abs() > diffz.abs() {
                                direction = if diffx > 0.0 { 1 } else { 3 };
                            } else {
                                direction = if diffz > 0.0 { 2 } else { 0 };
                            }

                            Blocks::set_direction_bits(&mut bottom_id, direction);
                            Blocks::set_direction_bits(&mut top_id, direction);

                            let mut left: IVec3 = IVec3::new(0,0,0);
                            let mut right: IVec3 = IVec3::new(0,0,0);

                            if direction == 0 || direction == 1 {
                                left = place_point - neighbor_axes[direction as usize];
                                right = place_point + neighbor_axes[direction as usize];
                            } else {
                                left = place_point + neighbor_axes[direction as usize];
                                right = place_point - neighbor_axes[direction as usize];
                            }

                            let csysread = self.chunksys.read().unwrap();

                            let mut blockbitsright = csysread.blockat(right);
                            let mut blockbitsleft = csysread.blockat(left);

                            drop(csysread);

                            if (blockbitsright & Blocks::block_id_bits()) == 19 {

                                let neighdir = Blocks::get_direction_bits(blockbitsright);
                                if neighdir == direction && DoorInfo::get_door_top_bit(blockbitsright) == 0 {

                                    let csysread = self.chunksys.read().unwrap();

                                    let rightup = right + IVec3::new(0,1,0);
                                    let mut neightopbits = csysread.blockat(rightup);

                                    DoorInfo::set_opposite_door_bits(&mut top_id, 1);
                                    DoorInfo::set_opposite_door_bits(&mut bottom_id, 1);

                                    DoorInfo::set_opposite_door_bits(&mut blockbitsright, 0);
                                    DoorInfo::set_opposite_door_bits(&mut neightopbits, 0);

                                    let chunktoreb = ChunkSystem::spot_to_chunk_pos(&right);

                                    if self.vars.in_multiplayer {
                                        let mut message = Message::new(
                                            MessageType::MultiBlockSet, 
                                            Vec3::new(
                                                right.x as f32, 
                                                right.y as f32, 
                                                right.z as f32), 
                                            0.0, 
                                            blockbitsright);

                                            message.info2 = neightopbits;
                                            message.otherpos = rightup;


                                        self.netconn.send(&message);

                                    } else {
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(right, blockbitsright, false, true);
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(rightup, neightopbits, false, true);
                                    }

                                }
                            }

                            if (blockbitsleft & Blocks::block_id_bits()) == 19 {
                                let neighdir = Blocks::get_direction_bits(blockbitsleft);
                                if neighdir == direction && DoorInfo::get_door_top_bit(blockbitsleft) == 0 {
                                    let leftup = left + IVec3::new(0,1,0);

                                    let csysread = self.chunksys.read().unwrap();


                                    let mut neightopbits = csysread.blockat(leftup);

                                    DoorInfo::set_opposite_door_bits(&mut top_id, 1);
                                    DoorInfo::set_opposite_door_bits(&mut bottom_id, 1);

                                    DoorInfo::set_opposite_door_bits(&mut blockbitsleft, 0);
                                    DoorInfo::set_opposite_door_bits(&mut neightopbits, 0);

                                    let chunktoreb = ChunkSystem::spot_to_chunk_pos(&left);

                                    if self.vars.in_multiplayer {
                                        let mut message = Message::new(
                                            MessageType::MultiBlockSet, 
                                            Vec3::new(
                                                left.x as f32, 
                                                left.y as f32, 
                                                left.z as f32), 
                                            0.0, 
                                            blockbitsleft);


                                            message.info2 = neightopbits;
                                            message.otherpos = leftup;

                                        self.netconn.send(&message);

                                    } else {
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(left, blockbitsleft, false, true);
                                        self.chunksys.read().unwrap().set_block_and_queue_rerender(leftup, neightopbits, false, true);
                                    }

                                }
                            }

                            if self.vars.in_multiplayer {
                                let mut message = Message::new(
                                    MessageType::MultiBlockSet, 
                                    Vec3::new(
                                        place_point.x as f32, 
                                        place_point.y as f32, 
                                        place_point.z as f32), 
                                    0.0, 
                                    bottom_id);

                                    message.info2 = top_id;
                                    message.otherpos = place_above;

                                self.netconn.send(&message);

                            } else {
                                self.chunksys.read().unwrap().set_block_and_queue_rerender(place_point, bottom_id, false, true);
                                self.chunksys.read().unwrap().set_block_and_queue_rerender(place_above, top_id, false, true);
                            }

                        }

                        } else if id == 20 { //ladder shit

                            let mut ladder_id = id;

                            let diffx = cl.position.x - place_point.x as f32;
                            let diffz = cl.position.z - place_point.z as f32;

                            let mut direction = 0;

                            if diffx.abs() > diffz.abs() {
                                direction = if diffx > 0.0 { 1 } else { 3 };
                            } else {
                                direction = if diffz > 0.0 { 2 } else { 0 };
                            }

                            Blocks::set_direction_bits(&mut ladder_id, direction);

                            if self.vars.in_multiplayer {
                                let message = Message::new(
                                    MessageType::BlockSet, 
                                    Vec3::new(
                                        place_point.x as f32, 
                                        place_point.y as f32, 
                                        place_point.z as f32), 
                                    0.0, 
                                    ladder_id);

                                self.netconn.send(&message);
                            } else {
                                self.chunksys.read().unwrap().set_block_and_queue_rerender(place_point, ladder_id, false, true);
                            }

                        } else if id == 21 { //Chest shit

                            let mut chest_id = id;

                            let diffx = cl.position.x - place_point.x as f32;
                            let diffz = cl.position.z - place_point.z as f32;

                            let mut direction = 0;

                            if diffx.abs() > diffz.abs() {
                                direction = if diffx > 0.0 { 1 } else { 3 };
                            } else {
                                direction = if diffz > 0.0 { 2 } else { 0 };
                            }

                            Blocks::set_direction_bits(&mut chest_id, direction);

                            if self.vars.in_multiplayer {
                                let message = Message::new(
                                    MessageType::BlockSet, 
                                    Vec3::new(
                                        place_point.x as f32, 
                                        place_point.y as f32, 
                                        place_point.z as f32), 
                                    0.0, 
                                    chest_id);

                                self.netconn.send(&message);
                            } else {
                                self.chunksys.read().unwrap().set_block_and_queue_rerender(place_point, chest_id, false, true);
                            }

                        } else {
                            if self.vars.in_multiplayer {
                                let message = Message::new(MessageType::BlockSet, Vec3::new(place_point.x as f32, place_point.y as f32, place_point.z as f32), 0.0, id);
                                self.netconn.send(&message);
                            } else {
                                self.chunksys.read().unwrap().set_block_and_queue_rerender(place_point, id, false, true);
                            }
                        }

                        if self.vars.in_multiplayer {
                            if slot.1 == 1 {
                                let mutslot = &mut self.inventory.write().unwrap().inv[slot_selected];
                                mutslot.1 = 0;
                                mutslot.0 = 0;


                                   let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::ZERO, 0.0, slot_selected as u32);
                                    msg.infof = 0.0;
                                    msg.info2 = 1;

                                    self.netconn.send(&msg);
                     
                                
                            } else {
                                let slot = &self.inventory.read().unwrap().inv[slot_selected];
                          

                                let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::ZERO, slot.0 as f32, slot_selected as u32);
                                    msg.infof = slot.1 as f32 - 1.0;
                                    msg.info2 = 1;

                                    self.netconn.send(&msg);

                            }
                        } else {
                            if slot.1 == 1 {
                                let mutslot = &mut self.inventory.write().unwrap().inv[slot_selected];
                                mutslot.1 = 0;
                                mutslot.0 = 0;
                            } else {
                                let mutslot = &mut self.inventory.write().unwrap().inv[slot_selected];
                                mutslot.1 -= 1;
                            }
                        }

                        
                    }

                    
                    
                }

                None => {}
            }

            

        }

        if updateinv {
            self.update_inventory();
            self.hud.chest_open = true;

            self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Normal);
            self.set_mouse_focused(false);
        }

        

    }
    pub fn mouse_button(&mut self, mb: MouseButton, a: Action) {
        if self.hud.chest_open {

            match mb {
                glfw::MouseButtonLeft => {
                    //self.vars.mouse_clicked = a == Action::Press;

                    if a == Action::Press {
                        let mut updateinv = false;
                        {
                            let csys = self.chunksys.write().unwrap();
                        unsafe {
                            match MOUSED_SLOT {
                                SlotIndexType::ChestSlot(e) => {
                                    match csys.chest_registry.get_mut(&self.hud.current_chest) {
                                        Some(mut ch) =>  {
                                            let slot = &mut ch.value_mut().inv[e as usize];


                                            let buff = slot.clone();
                                            if self.vars.in_multiplayer {





                                                        /*OTHERPOS: CURRENT CHEST */
                                                        /*INFO: DEST SLOT INDEX */
                                                        /*INFO2: SLOT INDEX TYPE */
                                                        /*ROT: ID */
                                                        /*INFOF: COUNT */
                                                        /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
                                                        /*Z: IF MOUSE_SLOT IS REPLACED */
                                                        let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::new(buff.0 as f32, buff.1 as f32, 1.0), self.mouse_slot.0 as f32, e as u32);
                                                        msg.otherpos = self.hud.current_chest;
                                                        msg.info2 = /*0 = CHEST, 1 = INV, 2 = NONE */0;
                                                        msg.infof = self.mouse_slot.1 as f32;

                                                        self.netconn.send(&msg);





                                            }
                                            else
                                            {
                                                slot.0 = self.mouse_slot.0;
                                                slot.1 = self.mouse_slot.1;

                                                self.mouse_slot.0 = buff.0;
                                                self.mouse_slot.1 = buff.1;
                                                updateinv = true;
                                            }
                                            
                                        },
                                        None => {

                                        },
                                    }
                                },
                                SlotIndexType::InvSlot(e) => {
                                    let slot = &mut self.inventory.write().unwrap().inv[e as usize];

                                    let buff = slot.clone();

                                    if self.vars.in_multiplayer {


                                                /*OTHERPOS: CURRENT CHEST */
                                                /*INFO: DEST SLOT INDEX */
                                                /*INFO2: SLOT INDEX TYPE */
                                                /*ROT: ID */
                                                /*INFOF: COUNT */
                                                /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
                                                /*Z: IF MOUSE_SLOT IS REPLACED */
                                                let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::new(buff.0 as f32, buff.1 as f32, 1.0), self.mouse_slot.0 as f32, e as u32);
                                                msg.otherpos = self.hud.current_chest;
                                                msg.info2 = /*0 = CHEST, 1 = INV, 2 = NONE */ 1;
                                                msg.infof = self.mouse_slot.1 as f32;
                                                self.netconn.send(&msg);


                                    }
                                    else
                                    {
                                        slot.0 = self.mouse_slot.0;
                                        slot.1 = self.mouse_slot.1;

                                        self.mouse_slot.0 = buff.0;
                                        self.mouse_slot.1 = buff.1;
                                        updateinv = true;
                                    }

                                },
                                SlotIndexType::None => {

                                },
                            }
                        }
                    }
                        if updateinv {
                            self.update_inventory();
                        }
                    }
                    // if self.vars.mouse_clicked {
                    //     self.cast_break_ray();
                    // }
                }
                glfw::MouseButtonRight => {
                    //self.vars.right_mouse_clicked = a == Action::Press;
                    // if !self.vars.ship_taken_off {
                    //     if self.vars.right_mouse_clicked {
                    //         self.cast_place_ray();
                    //     }
                    // }
                }
                _ => {}
            }

        } else {

            
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
    }

    pub fn new_world_func(&mut self) {


        if self.vars.in_multiplayer {

            //let msg = Message::new(MessageType::ShutUpMobMsgs, Vec3::ZERO, 0.0, 0);
            //self.netconn.send(&msg);

            self.netconn.received_world.store(false, Ordering::Relaxed);
 

            let msg = Message::new(MessageType::RequestUdm, Vec3::ZERO, 0.0, 0);
            self.netconn.send(&msg);

            while !self.netconn.received_world.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }

            let csysread = self.chunksys.read().unwrap();
            let currseed = *(csysread.currentseed.read().unwrap());
            let nt = csysread.planet_type;
            drop(csysread);

            self.vars.hostile_world = (nt % 2) != 0;

            self.start_chunks_with_radius(10, currseed, nt as usize);

        } else {
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
        }


        

        // self.chunksys.load_world_from_file(String::from("saves/world1"));
        // self.vars.hostile_world = false;
        // let seed = *self.chunksys.currentseed.read().unwrap();
        // self.start_chunks_with_radius(10, seed, 0);
        // self.camera.lock().unwrap().position = Vec3::new(0.0, 100.0, 0.0);

    }


    pub fn keyboard(&mut self, key: Key, action: Action) {
        match key {
            Key::Escape => {
                if action == Action::Press {
                    if !self.vars.menu_open && !self.hud.chest_open {

                        self.currentbuttons = vec![
                            ("Quit Game", "quittomainmenu")
                        ];
                        self.vars.menu_open = true;
    
                    } else {
                        self.vars.menu_open = false;
                    }

                    if self.hud.chest_open {
                        self.hud.chest_open = false;
                        self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Disabled);
                        self.set_mouse_focused(true);
                    }
                }
                
                
            }
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
            Key::LeftShift => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.shift = true;
                } else {
                    self.controls.shift = false;
                }
            }
            // Key::M => {
            //     if action == Action::Press {
            //         if self.vars.in_multiplayer {
            //             self.netconn.send(&Message::new(MessageType::RequestTakeoff, Vec3::ZERO, 0.0, 0));
            //         } else {
            //             self.takeoff_ship();
            //         }
                    
            //     }
            // }
            // Key::L => {
            //     if action == Action::Press {
            //         self.chunksys.read().unwrap().save_current_world_to_file(String::from("saves/world1"));
            //     }
            // }
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
            // Key::B => {
            //     if self.vars.near_ship {
            //         let mut camlock = self.camera.lock().unwrap();
            //         camlock.position = self.ship_pos + Vec3::new(5.0, 2.0, 0.0);
            //     }
            // }
            Key::Num0 => {
                self.faders.write().unwrap()[FaderNames::FovFader as usize].up();
                self.faders.write().unwrap()[FaderNames::FovFader as usize].top += 1.0;
                self.faders.write().unwrap()[FaderNames::FovFader as usize].bottom += 1.0;

            }
            Key::Num9 => {
                self.faders.write().unwrap()[FaderNames::FovFader as usize].down();
                self.faders.write().unwrap()[FaderNames::FovFader as usize].top -= 1.0;
                self.faders.write().unwrap()[FaderNames::FovFader as usize].bottom -= 1.0;
                
            }
            Key::P => {
                if action == Action::Press && !self.faders.read().unwrap()[FaderNames::VisionsFader as usize].mode {
                    let mut rng = StdRng::from_entropy();
                    self.current_vision = Some(VisionType::Model(rng.gen_range(2..self.gltf_models.len())));
                    self.visions_timer = 0.0;
                    self.faders.write().unwrap()[FaderNames::VisionsFader as usize].up();
                    self.audiop.write().unwrap().play_in_head("assets/sfx/dreambell.mp3");
                }
                

            }

            Key::L => {
                if action == Action::Press {
                    self.vars.menu_open = !self.vars.menu_open;
                }
                

            }
            Key::O => {
                //self.faders.write().unwrap()[FaderNames::VisionsFader as usize].down();
            }
            _ => {}
        }
    }
}
