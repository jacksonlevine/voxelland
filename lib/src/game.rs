use std::cmp::max;
use std::collections::HashSet;
use std::f32::consts::{self};
use std::io::Write;

use atomic_float::AtomicF32;
use noise::Perlin;
use once_cell::sync::Lazy;
use tracing::info;

use dashmap::DashMap;
use gl::types::{GLenum, GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::{Mat4, Vec2, Vec3, Vec4};
use glfw::ffi::glfwGetTime;
use glfw::{Action, Key, MouseButton, PWindow};
use std::time::{Duration, Instant};

use lockfree::queue::Queue;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::{params, Connection};
use uuid::Uuid;

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI8, AtomicU32, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

pub const CHUNKFADEINTIME: f32 = 0.6;
pub const CHUNKFADEIN_TIMEMULTIPLIER_TOGET1_WHENITSFULL: f32 = 1.0 / CHUNKFADEINTIME;


pub static mut CHUNKDRAWINGHERE: Lazy<DashMap<IVec2, Instant>> = Lazy::new(|| DashMap::new() );


pub static mut SELECTCUBESPOT: IVec3 = IVec3 { x: 0, y: 0, z: 0 };
use std::thread::{self, JoinHandle};
pub static mut MOUSE_ON_CUBE: bool = false;
#[cfg(feature = "audio")]
use crate::audio::{spawn_audio_thread, AudioPlayer};

pub const PLAYERSCALE: f32 = 1.0;

use crate::blockinfo::Blocks;
use crate::blockoverlay::BlockOverlay;
use crate::chunk::{ChunkFacade, ChunkSystem, AUTOMATA_QUEUED_CHANGES};

pub static mut LIST_OF_PREVIEWED_SPOTS: Vec<(IVec3, u32)> = Vec::new();

pub static mut VOXEL_SELECT_DISTANCE: f32 = 10.0;



pub static mut BUILD_PREVIEW_MODE: bool  = false;

pub static mut SELECTED_BUILD: usize = 0;

pub static mut BUILD_VOXEL_MODELS: Vec<JVoxModel> = Vec::new();

pub static mut BUILD_MODEL_OFFSET: IVec2 = IVec2::new(0,0);


use crate::camera::Camera;
use crate::collisioncage::*;
use crate::cube::Cube;
use crate::drops::Drops;
use crate::fader::Fader;
use crate::glyphface::GlyphFace;
use crate::guisystem::GuiSystem;
use crate::hud::{Hud, HudElement, SlotIndexType};
use crate::inventory::*;

use crate::modelentity::ModelEntity;
use crate::network::NetworkConnector;
use crate::planetinfo::Planets;
use crate::playerposition::PlayerPosition;
use crate::raycast::*;
use crate::recipes::{Recipe, RecipeEntry, RECIPES};
use crate::selectcube::SelectCube;
use crate::server_types::{Message, MessageType};
use crate::shader::Shader;
use crate::specialblocks::door::{self, DoorInfo};
use crate::statics::{MISCSETTINGS, MY_MULTIPLAYER_UUID, SAVE_MISC};
use crate::texture::Texture;
use crate::textureface::TextureFace;
use crate::tools::{get_block_material, get_tools_target_material, Material};
use crate::vec::{self, IVec2, IVec3};
use crate::voxmodel::JVoxModel;
use crate::windowandkey::uncapkb;
use crate::worldgeometry::WorldGeometry;

static mut CONVEYOR_SOUND_TIMER: f32 = 0.0;

pub static TRAMPOLINE_VELOCITY_FIGURE: f32 = 10.0;

pub static mut MOUSEX: f64 = 0.0;
pub static mut MOUSEY: f64 = 0.0;

pub static mut SHOWTOOLTIP: bool = false;
pub static mut TOOLTIPNAME: &'static str = "";

pub static mut SPRINTING: bool = false;
pub static mut WASFREEFALLING: bool = false;
pub static mut FREEFALLING: bool = false;

pub static mut STAMINA: i32 = 0;
pub static mut UPDATE_THE_BLOCK_OVERLAY: bool = false;

pub static mut WINDED: bool = false;
pub static mut WINDEDTIMER: f32 = 0.0;

pub static WINDEDLENGTH: f32 = 2.0;

pub static mut SINGLEPLAYER: bool = false;

pub static mut DECIDEDSPORMP: bool = false;

pub static mut MOVING: bool = false;

pub static mut SHOULDRUN: bool = false;

pub static mut WEATHERTYPE: f32 = 0.0;
pub static mut WEATHERTIMER: f32 = 0.0;
pub const WEATHERINTERVAL: f32 = 120.0;

pub static mut TRAMPOLINE: bool = false;
pub static mut TRAMP_TIMER: f32 = 0.0;

pub static QUEUE_THESE: Lazy<Queue<IVec2>> = Lazy::new(|| Queue::new());

pub static mut ON_CONVEYORS: bool = false;
pub static mut TIME_ON_CONVEYORS: f32 = 0.0;

pub static mut ROOFOVERHEAD: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

pub static mut HEADLESS: bool = false;

pub const SPRINTFOV: f32 = 83.0;
pub const FALLFOV: f32 = 93.0;

pub static mut CURRSEED: Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(0));

#[cfg(feature = "audio")]
pub static mut AUDIOPLAYER: Lazy<AudioPlayer> = Lazy::new(|| AudioPlayer::new().unwrap());

pub fn wait_for_decide_singleplayer() {
    unsafe {
        while !DECIDEDSPORMP {
            thread::sleep(Duration::from_millis(250));
        }
    }
}

pub static STARTINGITEMS: [(u32, u32); ROWLENGTH as usize] = [
    (31, 1),
    (49, 10),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
];
//USER BUILD PREVIEW

pub static mut UBP_VDATA: Lazy<Vec<f32>> = Lazy::new(|| Vec::new());
pub static mut UBP_UVDATA: Lazy<Vec<f32>> = Lazy::new(|| Vec::new());

pub static mut UBP_VBO: GLuint = 0;
pub static mut UBP_UVBO: GLuint = 0;
pub static mut UBP_VAO: GLuint = 0;

pub static mut SPAWNPOINT: Vec3 = Vec3::ZERO;

pub static mut MOUSED_SLOT: SlotIndexType = SlotIndexType::None;

pub static mut CROUCHING: bool = false;

pub static mut SONGS: [&'static str; 11] = [
    path!("assets/music/bee.mp3"),
    path!("assets/music/qv2.mp3"),
    path!("assets/music/song.mp3"),
    path!("assets/music/Farfromhome.mp3"),
    path!("assets/music/ifol.mp3"),
    path!("assets/music/NoFuture.mp3"),
    path!("assets/music/Respite.mp3"),
    path!("assets/music/SereneFacade.mp3"),
    path!("assets/music/Unease.mp3"),
    path!("assets/music/UnknownDanger.mp3"),
    path!("assets/music/empythree.mp3"),
];

pub static mut SONGTIMER: f32 = 0.0;
pub static mut SONGINTERVAL: f32 = 300.0;
pub static mut SONGINDEX: usize = 0;

pub static mut AMBIENTBRIGHTNESS: f32 = 0.0;

pub static mut CURRENT_AVAIL_RECIPES: Mutex<Vec<RecipeEntry>> = Mutex::new(Vec::new());

pub static mut ATSMALLTABLE: bool = false;

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
    pub shift: bool,
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
    pub in_climbable: bool,
    pub walkbobtimer: f32,

    pub time_tfs_at_3: f32,
}

pub enum VisionType {
    Model(usize),
    Vox(usize),
}

pub static mut GLCHUNKS: bool = true;

pub static mut PLAYERCHUNKPOS: Lazy<(AtomicI32, AtomicI32)> =
    Lazy::new(|| (AtomicI32::new(0), AtomicI32::new(0)));

pub struct PlayerCam {
    pos: (AtomicF32, AtomicF32, AtomicF32),
    dir: (AtomicF32, AtomicF32, AtomicF32),
    yaw: AtomicF32,
    pitch: AtomicF32,
}

pub struct PlayerCamSnapshot {
    pub pos: (f32, f32, f32),
    pub dir: (f32, f32, f32),
    pub yaw: f32,
    pub pitch: f32,
}

impl PlayerCam {
    pub fn snapshot(&self) -> PlayerCamSnapshot {
        let pos = (
            self.pos.0.load(Ordering::Relaxed),
            self.pos.1.load(Ordering::Relaxed),
            self.pos.2.load(Ordering::Relaxed),
        );
        let dir = (
            self.dir.0.load(Ordering::Relaxed),
            self.dir.1.load(Ordering::Relaxed),
            self.dir.2.load(Ordering::Relaxed),
        );
        let yaw = self.yaw.load(Ordering::Relaxed);
        let pitch = self.pitch.load(Ordering::Relaxed);

        PlayerCamSnapshot {
            pos,
            dir,
            yaw,
            pitch,
        }
    }
}

pub static mut PLAYERPOS: Lazy<PlayerCam> = Lazy::new(|| PlayerCam {
    pos: (
        AtomicF32::new(0.0),
        AtomicF32::new(0.0),
        AtomicF32::new(0.0),
    ),
    dir: (
        AtomicF32::new(0.0),
        AtomicF32::new(0.0),
        AtomicF32::new(0.0),
    ),
    yaw: AtomicF32::new(0.0),
    pitch: AtomicF32::new(0.0),
});

pub struct Game {
    pub chunksys: Arc<RwLock<ChunkSystem>>,
    pub shader0: Shader,
    pub oldshader: Shader,
    pub skyshader: Shader,
    pub modelshader: Shader,
    pub cloudshader: Shader,
    pub starshader: Shader,
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
    pub gltf_models: Vec<(
        gltf::Document,
        Vec<gltf::buffer::Data>,
        Vec<gltf::image::Data>,
    )>,
    pub gltf_vbos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_vaos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_counts: Vec<Vec<Vec<usize>>>,
    pub gltf_drawmodes: Vec<Vec<Vec<GLenum>>>,
    pub gltf_ebos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_textures: Vec<Vec<Vec<GLuint>>>,
    pub gltf_paths: Vec<String>,
    pub static_model_entities: Vec<ModelEntity>,
    pub non_static_model_entities: Arc<DashMap<u32, ModelEntity>>,
    pub select_cube: SelectCube,
    #[cfg(feature = "glfw")]
    pub block_overlay: BlockOverlay,
    pub ship_pos: Vec3,
    pub planet_y_offset: f32,
    #[cfg(feature = "glfw")]
    pub window: Arc<RwLock<PWindow>>,
    #[cfg(feature = "glfw")]
    pub guisys: GuiSystem,
    #[cfg(feature = "glfw")]
    pub hud: Hud,
    #[cfg(feature = "glfw")]
    pub drops: Drops,
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
    #[cfg(feature = "glfw")]
    pub tex: Texture,
    pub inwater: bool,
    pub headinwater: bool,

    pub currentbuttons: Vec<(String, String)>,
    pub loadedworld: AtomicBool,
    pub addressentered: Arc<AtomicBool>,
    pub address: Arc<Mutex<Option<String>>>,
    pub player_model_entities: Arc<DashMap<Uuid, ModelEntity>>,

    pub mouse_slot: (u32, u32),
    pub needtosend: Arc<Queue<Message>>,

    pub health: Arc<AtomicI8>,
    pub crafting_open: bool,
    pub stamina: Arc<AtomicI32>,
    pub weathertype: f32,
    pub chest_registry: Arc<DashMap<vec::IVec3, ChestInventory>>,
}

pub const ROWLENGTH: i32 = 8;

enum FaderNames {
    FovFader = 0,
    VisionsFader = 1,
}

impl Game {
    #[cfg(feature = "glfw")]
    pub fn new(
        window: &Arc<RwLock<PWindow>>,
        connectonstart: bool,
        headless: bool,
        addressentered: &Arc<AtomicBool>,
        address: &Arc<Mutex<Option<String>>>,
    ) -> JoinHandle<Game> {
        Self::newold(
            &Some(window.clone()),
            connectonstart,
            headless,
            addressentered,
            address,
        )
    }

    #[cfg(not(feature = "glfw"))]
    pub fn new(
        connectonstart: bool,
        headless: bool,
        addressentered: &Arc<AtomicBool>,
        address: &Arc<Mutex<Option<String>>>,
    ) -> JoinHandle<Game> {
        Self::newold(&None, connectonstart, headless, addressentered, address)
    }

    pub fn newold(
        window: &Option<Arc<RwLock<PWindow>>>,
        connectonstart: bool,
        headless: bool,
        addressentered: &Arc<AtomicBool>,
        address: &Arc<Mutex<Option<String>>>,
    ) -> JoinHandle<Game> {
        unsafe {
            if headless {
                HEADLESS = true;
            }
        }
        unsafe {
            SHOULDRUN = true;
        }
        let mut connectonstart = connectonstart;
        //wait_for_decide_singleplayer();

        unsafe {
            if SINGLEPLAYER {
                connectonstart = false;
            }
        }

        let oldshader = Shader::new(path!("assets/oldvert.glsl"), path!("assets/oldfrag.glsl"));
        let shader0 = Shader::new(path!("assets/vert.glsl"), path!("assets/frag.glsl"));
        let skyshader = Shader::new(path!("assets/skyvert.glsl"), path!("assets/skyfrag.glsl"));
        let faders: RwLock<Vec<Fader>> = RwLock::new(Vec::new());
        let cam = Arc::new(Mutex::new(Camera::new()));

        let stamina = Arc::new(AtomicI32::new(100));

        if !headless {
            #[cfg(feature = "audio")]
            spawn_audio_thread();
        }

        faders.write().extend(vec![
            Fader::new(83.0, 80.0, 30.0, false), //FOV fader for moving
            Fader::new(1.0, 0.0, 5.0, false),    //"Visions" fader for overlay
        ]);

        #[cfg(feature = "glfw")]
        unsafe {
            gl::BindVertexArray(shader0.vao);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after binding vertex array: {}", error);
            }
        }

        #[cfg(feature = "glfw")]
        let tex = Texture::new(path!("assets/world.png")).unwrap();
        #[cfg(feature = "glfw")]
        let weathertex = Texture::new(path!("assets/weather.png")).unwrap();

        #[cfg(feature = "glfw")]
        {
            tex.add_to_unit(0);

            weathertex.add_to_unit(2);
        }
        let randseed = if !headless {
            let mut rng = StdRng::from_entropy();

            let randseed: u32 = rng.gen_range(0..72731273);

            println!("Rand seed: {}", randseed);
            randseed
        } else {
            println!("Headless, giving seed generation duty to servero.");
            0
        };

        let mut csys = ChunkSystem::new(10, randseed, 0, headless);

        unsafe { BUILD_VOXEL_MODELS = vec![
            JVoxModel::new(path!("assets/voxelmodels/build1.vox")),
            JVoxModel::new(path!("assets/voxelmodels/build2.vox")),
            JVoxModel::new(path!("assets/voxelmodels/build3.vox")),
            JVoxModel::new(path!("assets/voxelmodels/build4.vox")),
            JVoxModel::new(path!("assets/voxelmodels/rock2.vox")),
            JVoxModel::new(path!("assets/voxelmodels/rock1.vox")),
            JVoxModel::new(path!("assets/voxelmodels/bush.vox")),
            JVoxModel::new(path!("assets/voxelmodels/rubbertree.vox")),
        ] };
        let voxel_models = vec![
            JVoxModel::new(path!("assets/voxelmodels/bush.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tree1.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tree2.vox")),
            JVoxModel::new(path!("assets/voxelmodels/rock1.vox")),
            JVoxModel::new(path!("assets/voxelmodels/rock2.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tree3.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tree4.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tree5.vox")),
            JVoxModel::new(path!("assets/voxelmodels/bamboo1.vox")),
            JVoxModel::new(path!("assets/voxelmodels/bamboo2.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tallgrass1.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tallgrass2.vox")),
            JVoxModel::new(path!("assets/voxelmodels/tallgrass3.vox")),
            JVoxModel::new(path!("assets/voxelmodels/rubbertree.vox")),
            //0-13
            JVoxModel::new(path!("assets/voxelmodels/ptree.vox")),
            JVoxModel::new(path!("assets/voxelmodels/redrock.vox")),
            JVoxModel::new(path!("assets/voxelmodels/crystal1.vox")), //14 - 16
        ];

        //csys.load_world_from_file(String::from("saves/world1"));

        //self.vars.hostile_world = false;
        //let seed = *csys.currentseed.read();
        //self.start_chunks_with_radius(10, seed, 0);
        //self.camera.lock().position = Vec3::new(0.0, 100.0, 0.0);

        let vmarc = Arc::new(voxel_models);
        let vmarc2 = vmarc.clone();

        csys.voxel_models = Some(vmarc);

        let chunksys = Arc::new(RwLock::new(csys));

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool + Send + Sync> = {
            let csys_arc = Arc::clone(&chunksys);
            Box::new(move |v: vec::IVec3| {
                let csys = csys_arc.read();
                let bitshere = csys.blockat(v.clone());

                let isntopendoor = DoorInfo::get_door_open_bit(bitshere) != 1;
                let isntladder = (bitshere & Blocks::block_id_bits()) != 20;
                let isntbamboo = (bitshere & Blocks::block_id_bits()) != 22;
                let isnttallgrass = (bitshere & Blocks::block_id_bits()) != 23;
                let isnttorch = (bitshere & Blocks::block_id_bits()) != 49;
                return isntopendoor
                    && isntladder
                    && isnttorch
                    && isntbamboo
                    && isnttallgrass
                    && csys_arc.read().collision_predicate(v);
            })
        };

        let health = Arc::new(AtomicI8::new(20));

        let cam_clone = cam.clone();
        let csysclone = chunksys.clone();
        if !headless {
            thread::spawn(move || {
                while unsafe { SHOULDRUN } {
                    let mut pos = Vec3::ZERO;
                    let mut hitblock = false;

                    match cam_clone.try_lock() {
                        Some(camlock) => {
                            pos = camlock.position.clone();
                        }
                        None => {
                            //println!("Failed to lock camera: {:?}", e);
                        }
                    }

                    if pos != Vec3::ZERO {
                        let r = csysclone.read();
                        while !hitblock && pos.y < 128.0 {
                            let ppos = vec::IVec3::new(
                                pos.x.floor() as i32,
                                pos.y.round() as i32,
                                pos.z.floor() as i32,
                            );
                            if r.blockat(ppos) != 0 {
                                hitblock = true;
                                break;
                            }
                            pos.y += 1.0;
                        }
                        unsafe {
                            if hitblock {
                                ROOFOVERHEAD.store(true, Ordering::Relaxed)
                            } else {
                                ROOFOVERHEAD.store(false, Ordering::Relaxed)
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(250));
                }
            });

            //  let csysclone = chunksys.clone();
            // let cam_clone = cam.clone();

            // thread::spawn(move || {
            //     while unsafe { SHOULDRUN } {

            //         match csysclone.try_read() {
            //             Ok(csys) => {
            //                 match QUEUE_THESE.pop() {
            //                     Some(spot) => {
            //                         csys.queue_rerender_with_key(spot, true, true);
            //                     }
            //                     None => {

            //                     }
            //                 }
            //             }
            //             Err(e) => {

            //             }
            //         }

            //         thread::sleep(Duration::from_millis(250));
            //     }
            // });
        }

        #[cfg(feature = "glfw")]
        let mut hud = Hud::new(
            &window.as_ref().unwrap().clone(),
            tex.id,
            health.clone(),
            stamina.clone(),
        );
        //IMPORTANT: Push the inv row slots first
        fn add_inventory_rows(
            elements: &mut Vec<HudElement>,
            yoffset: f32,
            rows: i32,
            start_slot: SlotIndexType,
            rowlength: i32,
        ) {
            let tf = TextureFace::new(0, 14);

            let rh: f32 = 0.2;

            for y in 0..rows {
                for i in 0..rowlength {
                    let newslot = match start_slot {
                        SlotIndexType::ChestSlot(ind) => {
                            SlotIndexType::ChestSlot(ind + i + (y * rowlength))
                        }
                        SlotIndexType::InvSlot(ind) => {
                            SlotIndexType::InvSlot(ind + i + (y * rowlength))
                        }
                        SlotIndexType::None => SlotIndexType::None,
                    };
                    let invrowel = HudElement::new(
                        Vec2::new(-(0.10 * 3.5) + i as f32 * 0.10, yoffset - y as f32 * rh),
                        Vec2::new(0.15, 0.15),
                        [
                            tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y,
                            tf.tlx, tf.tly, tf.blx, tf.bly,
                        ],
                        newslot,
                    );

                    elements.push(invrowel);
                }
            }

            let tf = TextureFace::new(0, 0);
            //The item textures on top of them
            for y in 0..rows {
                for i in 0..rowlength {
                    let newslot = match start_slot {
                        SlotIndexType::ChestSlot(ind) => {
                            SlotIndexType::ChestSlot(ind + i + (y * rowlength))
                        }
                        SlotIndexType::InvSlot(ind) => {
                            SlotIndexType::InvSlot(ind + i + (y * rowlength))
                        }
                        SlotIndexType::None => SlotIndexType::None,
                    };
                    let invrowel = HudElement::new(
                        Vec2::new(-(0.10 * 3.5) + i as f32 * 0.10, yoffset - y as f32 * rh),
                        Vec2::new(0.10, 0.10),
                        [
                            tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y,
                            tf.tlx, tf.tly, tf.blx, tf.bly,
                        ],
                        newslot,
                    );

                    elements.push(invrowel);
                }
            }

            let tf = TextureFace::new(0, 0);
            //The number textures on top of them
            for y in 0..rows {
                for i in 0..rowlength {
                    let newslot = match start_slot {
                        SlotIndexType::ChestSlot(ind) => {
                            SlotIndexType::ChestSlot(ind + i + (y * rowlength))
                        }
                        SlotIndexType::InvSlot(ind) => {
                            SlotIndexType::InvSlot(ind + i + (y * rowlength))
                        }
                        SlotIndexType::None => SlotIndexType::None,
                    };

                    // let invrowel = HudElement::new(
                    //     Vec2::new(
                    //         -(0.10 * 3.5) - 0.01 + i as f32 * 0.10,
                    //         yoffset - y as f32 * rh - 0.03,
                    //     ),
                    //     Vec2::new(0.05, 0.05),
                    //     [
                    //         tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y,
                    //         tf.tlx, tf.tly, tf.blx, tf.bly,
                    //     ],
                    //     newslot.clone(),
                    // );
                    // elements.push(invrowel);

                    // let invrowel = HudElement::new(
                    //     Vec2::new(
                    //         -(0.10 * 3.5) + 0.00 + i as f32 * 0.10,
                    //         yoffset - y as f32 * rh - 0.03,
                    //     ),
                    //     Vec2::new(0.05, 0.05),
                    //     [
                    //         tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y,
                    //         tf.tlx, tf.tly, tf.blx, tf.bly,
                    //     ],
                    //     newslot.clone(),
                    // );
                    // elements.push(invrowel);

                    let invrowel = HudElement::new(
                        Vec2::new(
                            -(0.10 * 3.5) + 0.01 + i as f32 * 0.10,
                            yoffset - y as f32 * rh - 0.03,
                        ),
                        Vec2::new(0.05, 0.05),
                        [
                            tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y,
                            tf.tlx, tf.tly, tf.blx, tf.bly,
                        ],
                        newslot.clone(),
                    );
                    elements.push(invrowel);

                    let invrowel = HudElement::new(
                        Vec2::new(
                            -(0.10 * 3.5) + 0.02 + i as f32 * 0.10,
                            yoffset - y as f32 * rh - 0.03,
                        ),
                        Vec2::new(0.05, 0.05),
                        [
                            tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y,
                            tf.tlx, tf.tly, tf.blx, tf.bly,
                        ],
                        newslot,
                    );

                    elements.push(invrowel);
                }
            }
        }

        //Bottom inv
        #[cfg(feature = "glfw")]
        add_inventory_rows(
            &mut hud.elements,
            -0.9,
            1,
            SlotIndexType::InvSlot(0),
            ROWLENGTH,
        );

        //Chest rows
        #[cfg(feature = "glfw")]
        add_inventory_rows(
            &mut hud.chestelements,
            0.4,
            4,
            SlotIndexType::ChestSlot(0),
            ROWLENGTH,
        );

        //Crosshair
        let tf = TextureFace::new(0, 13);

        #[cfg(feature = "glfw")]
        hud.elements.push(HudElement::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.08, 0.08),
            [
                tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
                tf.blx, tf.bly,
            ],
            SlotIndexType::None,
        ));

        //HELD MOUSE ITEM SLOT

        let tf = TextureFace::new(0, 0);
        //The item texture of it

        let invrowel = HudElement::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.10, 0.10),
            [
                tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
                tf.blx, tf.bly,
            ],
            SlotIndexType::InvSlot(221),
        );
        #[cfg(feature = "glfw")]
        hud.chestelements.push(invrowel);

        let tf = TextureFace::new(0, 0);
        //The number textures on top of it

        let invrowel = HudElement::new(
            Vec2::new(0.0 + 0.01, 0.0),
            Vec2::new(0.05, 0.05),
            [
                tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
                tf.blx, tf.bly,
            ],
            SlotIndexType::InvSlot(221),
        );
        #[cfg(feature = "glfw")]
        hud.chestelements.push(invrowel);

        let invrowel = HudElement::new(
            Vec2::new(0.0 + 0.02, 0.0),
            Vec2::new(0.05, 0.05),
            [
                tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
                tf.blx, tf.bly,
            ],
            SlotIndexType::InvSlot(221),
        );
        #[cfg(feature = "glfw")]
        hud.chestelements.push(invrowel);

        let inv = Arc::new(RwLock::new(Inventory {
            dirty: true,
            inv: STARTINGITEMS,
        }));

        //let mut rng = StdRng::from_entropy();
        //let seed = rng.gen_range(0..229232);

        let server_command_queue = Arc::new(Queue::<Message>::new());
        let server_command_hp_queue = Arc::new(Queue::<Message>::new());

        let kc = Arc::new(DashMap::new());

        let my_uuid: Arc<RwLock<Option<Uuid>>> =
            Arc::new(RwLock::new(Some(unsafe { *MY_MULTIPLAYER_UUID })));

        let nsme = Arc::new(DashMap::new());

        let mut visions_camera = Camera::new();
        visions_camera.position = Vec3::new(0.0, 3.0, -5.0);
        visions_camera.recalculate();
        info!(
            "Visions camera direction: {} {} {}",
            visions_camera.direction.x, visions_camera.direction.y, visions_camera.direction.z
        );

        let pme = Arc::new(DashMap::new());

        let needtosend = Arc::new(Queue::new());

        unsafe {
            let mut rng = StdRng::from_entropy();
            SONGINDEX = (SONGINDEX + rng.gen_range(1..SONGS.len())) % SONGS.len();
        }

        #[cfg(feature = "glfw")]
        let window = &window.as_ref().unwrap().clone();

        let chest_registry = Arc::new(DashMap::new());

        let mut g = Game {
            chunksys: chunksys.clone(),
            shader0,
            oldshader,
            skyshader,
            modelshader: Shader::new(path!("assets/mvert.glsl"), path!("assets/mfrag.glsl")),
            cloudshader: Shader::new(
                path!("assets/cloudsvert.glsl"),
                path!("assets/cloudsfrag.glsl"),
            ),
            starshader: Shader::new(
                path!("assets/starsvert.glsl"),
                path!("assets/starsfrag.glsl"),
            ),
            camera: cam.clone(),
            run_chunk_thread: Arc::new(AtomicBool::new(true)),
            chunk_thread: None,
            vars: GameVariables {
                first_mouse: true,
                mouse_focused: false,

                sky_color: Vec4::new(0.3, 0.65, 1.0, 1.0),
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
                in_climbable: false,
                walkbobtimer: 0.0,

                time_tfs_at_3: 0.0,
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
            allowable_jump_height: 1.6,
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
            #[cfg(feature = "glfw")]
            block_overlay: BlockOverlay::new(tex.id),
            ship_pos: Vec3::new(0.0, 0.0, 0.0),
            planet_y_offset: 0.0,
            #[cfg(feature = "glfw")]
            window: window.clone(),
            #[cfg(feature = "glfw")]
            guisys: GuiSystem::new(&window.clone(), &tex),
            #[cfg(feature = "glfw")]
            hud,
            #[cfg(feature = "glfw")]
            drops: Drops::new(
                tex.id,
                &cam,
                &chunksys,
                &inv,
                connectonstart,
                &needtosend.clone(),
            ),
            inventory: inv,
            animations: Vec::new(),
            skins: Vec::new(),
            nodes: Vec::new(),
            current_time: 0.0,
            netconn: NetworkConnector::new(
                &chunksys,
                &server_command_queue,
                &server_command_hp_queue,
                &kc,
                &my_uuid.clone(),
                &nsme,
                &cam.clone(),
                &pme.clone(),
                &chest_registry,
                &needtosend,
            ),
            server_command_queue: server_command_queue.clone(),
            hp_server_command_queue: server_command_hp_queue.clone(),
            headless,
            known_cameras: kc,
            my_uuid,
            ambient_bright_mult: 1.0,
            daylength: 900.0,
            timeofday: Arc::new(Mutex::new(250.0)),
            sunrise_factor: 0.0,
            sunset_factor: 0.0,
            visions_timer: 0.0,
            visions_camera,
            current_vision: Some(VisionType::Model(0)),
            #[cfg(feature = "glfw")]
            tex,
            inwater: false,
            headinwater: false,
            currentbuttons: vec![
                ("Test".to_string(), "Yoo".to_string()),
                ("Test22".to_string(), "22".to_string()),
            ],
            loadedworld: AtomicBool::new(false),
            addressentered: addressentered.clone(),
            address: address.clone(),
            player_model_entities: pme,
            mouse_slot: (0, 0),
            needtosend,
            health,
            crafting_open: false,
            stamina,
            weathertype: 0.0,
            chest_registry,
        };
        #[cfg(feature = "glfw")]
        if !headless {
            g.load_model(path!("assets/models/player.glb"));
            // g.load_model(path!("assets/models/car/scene.gltf"));
            // //g.load_model(path!("assets/models/ship/scene.gltf"));
            // g.load_model(path!("assets/models/monster1/scene.gltf"));
            // g.load_model(path!("assets/models/monster2/scene.gltf"));
            // g.load_model(path!("assets/models/cow/scene.glb"));
            // g.load_model(path!("assets/models/mountain/scene.gltf"));

            // g.load_model(path!("assets/models/cricket/scene.gltf"));

            // info!("gltf model count: {}", g.gltf_models.len());

            g.create_model_vbos();
        }

        let _aeclone = g.addressentered.clone();
        let _aclone = g.address.clone();

        unsafe {
            for string in SONGS {
                #[cfg(feature = "audio")]
                AUDIOPLAYER.preload(string, string);
            }
        }

        #[cfg(feature = "audio")]
        {
            unsafe {
                AUDIOPLAYER.preload_series(
                    "grassstepseries",
                    vec![
                        path!("assets/sfx/grassstep1.mp3"),
                        path!("assets/sfx/grassstep2.mp3"),
                        path!("assets/sfx/grassstep3.mp3"),
                        path!("assets/sfx/grassstep4.mp3"),
                        path!("assets/sfx/grassstep5.mp3"),
                        path!("assets/sfx/grassstep6.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "woodstepseries",
                    vec![
                        path!("assets/sfx/woodstep1.mp3"),
                        path!("assets/sfx/woodstep2.mp3"),
                        path!("assets/sfx/woodstep3.mp3"),
                        path!("assets/sfx/woodstep4.mp3"),
                        path!("assets/sfx/woodstep5.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "mulchstepseries",
                    vec![
                        path!("assets/sfx/mulchstep1.mp3"),
                        path!("assets/sfx/mulchstep2.mp3"),
                        path!("assets/sfx/mulchstep3.mp3"),
                        path!("assets/sfx/mulchstep4.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "dirtstepseries",
                    vec![
                        path!("assets/sfx/dirtstep1.mp3"),
                        path!("assets/sfx/dirtstep2.mp3"),
                        path!("assets/sfx/dirtstep3.mp3"),
                        path!("assets/sfx/dirtstep4.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "sandstepseries",
                    vec![
                        path!("assets/sfx/sandstep1.mp3"),
                        path!("assets/sfx/sandstep2.mp3"),
                        path!("assets/sfx/sandstep3.mp3"),
                        path!("assets/sfx/sandstep4.mp3"),
                        path!("assets/sfx/sandstep5.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "doorseries",
                    vec![
                        path!("assets/sfx/door.mp3"),
                        path!("assets/sfx/door1.mp3"),
                        path!("assets/sfx/door2.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "waterstepseries",
                    vec![
                        path!("assets/sfx/water1.mp3"),
                        path!("assets/sfx/water2.mp3"),
                        path!("assets/sfx/water3.mp3"),
                        path!("assets/sfx/water4.mp3"),
                        path!("assets/sfx/water5.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "clickseries",
                    vec![
                        path!("assets/sfx/click1.mp3"),
                        path!("assets/sfx/click2.mp3"),
                        path!("assets/sfx/click3.mp3"),
                        path!("assets/sfx/click4.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "stonestepseries",
                    vec![
                        path!("assets/sfx/stonestep1.mp3"),
                        path!("assets/sfx/stonestep2.mp3"),
                        path!("assets/sfx/stonestep3.mp3"),
                        path!("assets/sfx/stonestep4.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "stoneplaceseries",
                    vec![
                        path!("assets/sfx/stoneplace1.mp3"),
                        path!("assets/sfx/stoneplace2.mp3"),
                        path!("assets/sfx/stoneplace3.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "plantplaceseries",
                    vec![
                        path!("assets/sfx/plantplace1.mp3"),
                        path!("assets/sfx/plantplace2.mp3"),
                        path!("assets/sfx/plantplace3.mp3"),
                    ],
                );

                AUDIOPLAYER.preload_series(
                    "glassplaceseries",
                    vec![
                        path!("assets/sfx/glassplace1.mp3"),
                        path!("assets/sfx/glassplace2.mp3"),
                        path!("assets/sfx/glassplace3.mp3"),
                        path!("assets/sfx/glassplace4.mp3"),
                    ],
                );

                AUDIOPLAYER.preload(
                    path!("assets/sfx/cricket1.mp3"),
                    path!("assets/sfx/cricket1.mp3"),
                );
            }
        }
        thread::spawn(move || {
            if !headless {
                // g.setup_vertex_attributes();

                //start coming down from the sky in ship
                //g.vars.ship_going_down = true;
                //g.vars.ship_going_up = false;

                g.wait_for_new_address();

                // g.initialize_being_in_world();

                // g.add_ship_colliders();
            }
            g
        })
    }

    pub fn update_avail_recipes(inv: &Arc<RwLock<Inventory>>) {
        unsafe {
            {
                CURRENT_AVAIL_RECIPES.lock().clear();
            }

            let inv = inv.write();

            for rec in RECIPES.iter() {
                let requirements = rec.0.clone();

                let mut able = true;

                for req in requirements {
                    let mut amt = 0;

                    for slot in inv.inv {
                        if slot.0 == req.0 {
                            amt += slot.1;
                        }
                    }

                    if req.1 > amt {
                        able = false;
                    }
                }

                if ATSMALLTABLE && !rec.2 {
                    able = false;
                }

                if able {
                    CURRENT_AVAIL_RECIPES
                        .lock()
                        .push(RecipeEntry::from_recipe(rec.clone()));
                }
                //let result = rec.1;
            }
        }
    }

    pub fn save_one_chest_to_file(&self, key: IVec3) {
        let seed = {
            let c = self.chunksys.read();
            let s = unsafe { CURRSEED.load(std::sync::atomic::Ordering::Relaxed) };
            s.clone()
        };

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
        let seed = {
            let c = self.chunksys.read();
            let s = unsafe { CURRSEED.load(std::sync::atomic::Ordering::Relaxed) };
            s.clone()
        };

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
        let seed = {
            let c = self.chunksys.read();
            let s = unsafe { CURRSEED.load(std::sync::atomic::Ordering::Relaxed) };
            s.clone()
        };

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
                let inv: [(u32, u32); ROWLENGTH as usize * 4] =
                    bincode::deserialize(&inventory).unwrap();
                Ok((IVec3 { x, y, z }, ChestInventory { dirty, inv }))
            })
            .unwrap();

        for chest in chest_iter {
            let (coords, chest_inventory) = chest.unwrap();
            self.chest_registry.insert(coords, chest_inventory);
        }
    }

    pub fn static_load_chests_from_file(
        seed: u32,
        chest_registry: &Arc<DashMap<IVec3, ChestInventory>>,
    ) {
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
                let inv: [(u32, u32); ROWLENGTH as usize * 4] =
                    bincode::deserialize(&inventory).unwrap();
                Ok((IVec3 { x, y, z }, ChestInventory { dirty, inv }))
            })
            .unwrap();

        for chest in chest_iter {
            let (coords, chest_inventory) = chest.unwrap();
            chest_registry.insert(coords, chest_inventory);
        }
    }

    pub fn wait_for_new_address(&mut self) {
        if self.vars.in_multiplayer {
            //print!("Enter server address (e.g., 127.0.0.1:4848): ");
            //io::stdout().flush().unwrap(); // Ensure the prompt is printed before reading input

            //let mut address = String::new();
            //io::stdin().read_line(&mut address).expect("Failed to read line");

            while !self.addressentered.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }

            let address = self.address.lock().as_ref().unwrap().trim().to_string(); // Remove any trailing newline characters

            self.netconn.connect(address); // Connect to the provided address
            info!("Connected to the server!");
        }
    }
    pub fn button_command(&mut self, str: String) {
        match str.as_str() {
            "quittomainmenu" => {
                //self.exit();
                if self.vars.in_multiplayer {
                    self.netconn
                        .send(&Message::new(MessageType::Disconnect, Vec3::ZERO, 0.0, 0))
                }
                #[cfg(feature = "glfw")]
                self.window.write().set_should_close(true);
            }
            "closemenu" => {
                self.vars.menu_open = false;
                #[cfg(feature = "glfw")]
                self.window
                    .write()
                    .set_cursor_mode(glfw::CursorMode::Disabled);
                self.set_mouse_focused(true);
            }
            "escapemenu" => {
                self.currentbuttons = vec![
                    ("Close Menu".to_string(), "closemenu".to_string()),
                    ("Recipe Book".to_string(), "recipemenu".to_string()),
                    ("Settings".to_string(), "settingsmenu".to_string()),
                    ("Quit Game".to_string(), "quittomainmenu".to_string()),
                ];
                self.vars.menu_open = true;
            }
            "settingsmenu" => {
                self.currentbuttons = vec![
                    (
                        "Back to Previous Menu".to_string(),
                        "escapemenu".to_string(),
                    ),
                    ("Key Bindings".to_string(), "bindingsmenu".to_string()),
                    ("SliderMouse Sensitivity".to_string(), "test".to_string()),
                    ("SliderMusic Volume".to_string(), "music".to_string()),
                    ("SliderSounds Volume".to_string(), "sounds".to_string()),
                ];
                self.vars.menu_open = true;
            }

            "bindingsmenu" => {
                self.currentbuttons = vec![
                    ("bindings".to_string(), "".to_string()),
                    (
                        "Back to Previous Menu".to_string(),
                        "settingsmenu".to_string(),
                    ),
                ];

                unsafe {
                    for (key, action) in MISCSETTINGS.keybinds.iter_mut() {
                        self.currentbuttons
                            .push((action.clone(), format!("{:?}", key)));
                    }

                    for (key, action) in MISCSETTINGS.mousebinds.iter_mut() {
                        self.currentbuttons
                            .push((action.clone(), format!("{:?}", key)));
                    }
                }
                self.vars.menu_open = true;
            }
            "recipemenu" => {
                self.currentbuttons = vec![(
                    "Back to Previous Menu".to_string(),
                    "escapemenu".to_string(),
                )];
                for recipe in RECIPES.iter() {
                    let mut recipestring = String::new();
                    let mut tableneeded = String::new();

                    if recipe.2 {
                        tableneeded += " ";
                    } else {
                        tableneeded += "T";
                    }

                    recipestring += "From ";

                    for (index, requirement) in recipe.0.iter().enumerate() {
                        let name = Blocks::get_name(requirement.0);
                        let count = requirement.1;
                        recipestring += format!("{} {}", count, name).as_str();

                        if count > 1 {
                            recipestring += "s";
                        }

                        if index < recipe.0.len() - 2 || index == recipe.0.len() - 1 {
                            recipestring += ", ";
                        } else {
                            recipestring += ", and ";
                        }
                    }

                    recipestring += "create ";
                    recipestring +=
                        format!("{} {}", recipe.1 .1, Blocks::get_name(recipe.1 .0)).as_str();

                    if recipe.1 .1 > 1 {
                        recipestring += "s.";
                    } else {
                        recipestring += ".";
                    }
                    self.currentbuttons.push((recipestring, tableneeded))
                }
                self.currentbuttons.push((
                    "Back to Previous Menu".to_string(),
                    "escapemenu".to_string(),
                ));

                self.vars.menu_open = true;
            }
            _ => {
                info!("Unknown button command given");
            }
        }
    }
    #[cfg(feature = "audio")]
    pub fn play_weather_sound(&mut self) {
        static mut TIMER: f32 = 0.0;
        static mut OUTSIDE_RAIN_PLAYING: bool = false;
        static mut INSIDE_RAIN_PLAYING: bool = false;
        static mut OUTSIDE_SNOW_PLAYING: bool = false;
        static mut INSIDE_SNOW_PLAYING: bool = false;
        unsafe {
            TIMER += self.delta_time;
            //println!("TIMER: {}, DELTA_TIME: {}", TYMER, self.delta_time);

            if TIMER >= 14.0 {
                OUTSIDE_RAIN_PLAYING = false;
                INSIDE_RAIN_PLAYING = false;
                OUTSIDE_SNOW_PLAYING = false;
                INSIDE_SNOW_PLAYING = false;
                TIMER = 0.0;
            }

            match WEATHERTYPE {
                2.0 => {
                    if ROOFOVERHEAD.load(Ordering::Relaxed) {
                        if !INSIDE_RAIN_PLAYING {
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/rainoutside.mp3"));
                            //w.stop_sound("assets/sfx/raininside.mp3");
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/snowoutside.mp3"));
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/snowinside.mp3"));
                            AUDIOPLAYER.play_in_head(path!("assets/sfx/raininside.mp3"));
                            TIMER = 0.0;
                            INSIDE_RAIN_PLAYING = true;
                            OUTSIDE_RAIN_PLAYING = false;
                            OUTSIDE_SNOW_PLAYING = false;
                            INSIDE_SNOW_PLAYING = false;
                        }
                    } else {
                        if !OUTSIDE_RAIN_PLAYING {
                            //w.stop_sound("assets/sfx/rainoutside.mp3");
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/raininside.mp3"));
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/snowoutside.mp3"));
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/snowinside.mp3"));
                            AUDIOPLAYER.play_in_head(path!("assets/sfx/rainoutside.mp3"));
                            TIMER = 0.0;
                            OUTSIDE_RAIN_PLAYING = true;
                            INSIDE_RAIN_PLAYING = false;
                            OUTSIDE_SNOW_PLAYING = false;
                            INSIDE_SNOW_PLAYING = false;
                            //println!("playing outside rain");
                        }
                    }
                }
                1.0 => {
                    if ROOFOVERHEAD.load(Ordering::Relaxed) {
                        if !INSIDE_SNOW_PLAYING {
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/rainoutside.mp3"));
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/raininside.mp3"));
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/snowoutside.mp3"));
                            // w.stop_sound("assets/sfx/snowinside.mp3");
                            AUDIOPLAYER.play_in_head(path!("assets/sfx/snowinside.mp3"));
                            TIMER = 0.0;
                            INSIDE_SNOW_PLAYING = true;
                            OUTSIDE_SNOW_PLAYING = false;
                            OUTSIDE_RAIN_PLAYING = false;
                            INSIDE_RAIN_PLAYING = false;
                        }
                    } else {
                        if !OUTSIDE_SNOW_PLAYING {
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/rainoutside.mp3"));
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/raininside.mp3"));
                            //w.stop_sound("assets/sfx/snowoutside.mp3");
                            AUDIOPLAYER.stop_sound(path!("assets/sfx/snowinside.mp3"));

                            AUDIOPLAYER.play_in_head(path!("assets/sfx/snowoutside.mp3"));
                            TIMER = 0.0;
                            OUTSIDE_SNOW_PLAYING = true;
                            INSIDE_SNOW_PLAYING = false;
                            OUTSIDE_RAIN_PLAYING = false;
                            INSIDE_RAIN_PLAYING = false;
                        }
                    }
                }
                _ => {
                    AUDIOPLAYER.stop_sound(path!("assets/sfx/rainoutside.mp3"));
                    AUDIOPLAYER.stop_sound(path!("assets/sfx/raininside.mp3"));
                    AUDIOPLAYER.stop_sound(path!("assets/sfx/snowoutside.mp3"));
                    AUDIOPLAYER.stop_sound(path!("assets/sfx/snowinside.mp3"));
                    OUTSIDE_RAIN_PLAYING = false;
                    INSIDE_RAIN_PLAYING = false;
                    OUTSIDE_SNOW_PLAYING = false;
                    INSIDE_SNOW_PLAYING = false;
                    //println!("Stopping");
                }
            }
        }
    }

    pub fn initialize_being_in_world(&mut self) -> JoinHandle<()> {
        if self.vars.in_multiplayer {
            //ChunkSystem::initial_rebuild_on_main_thread(&self.chunksys.clone(), &self.shader0, &self.camera.lock().position);
            while !self.netconn.received_world.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }
        }

        self.vars.hostile_world = (self.chunksys.read().planet_type % 2) != 0;

        //self.audiop.play("assets/music/Farfromhome.mp3", &ship_float_pos, &Vec3::new(0.0,0.0,0.0));
        //self.audiop.play("assets/sfx/shipland28sec.mp3", &ship_float_pos, &Vec3::new(0.0,0.0,0.0));

        let ship_float_pos = self.auto_set_spawn_point();

        self.ship_pos = ship_float_pos;
        //self.static_model_entities.push(ModelEntity::new(1, ship_float_pos, 0.07, Vec3::new(PI/2.0, 0.0, 0.0), &self.chunksys, &self.camera));
        // self.static_model_entities.push(ModelEntity::new(4, ship_float_pos, 1.5, Vec3::new(0.0, 0.0, 0.0), &self.chunksys, &self.camera));

        unsafe {
            SPAWNPOINT = ship_float_pos + Vec3::new(0.0, 4.0, 0.0);
            self.camera.lock().position = SPAWNPOINT;
        }

        //self.static_model_entities.push(ModelEntity::new(5, Vec3::new(0.0, 25.0, 200.0), 140.0, Vec3::new(0.0, 0.0, 0.0), &self.chunksys, &self.camera));
        //self.update_model_collisions(0);

        self.currentbuttons = vec![("Loading...".to_string(), "loading".to_string())];
        self.vars.menu_open = true;

        let handle = self.rebuild_whole_world_while_showing_loading_screen();

        handle
    }

    fn auto_set_spawn_point(&mut self) -> Vec3 {
        let mut ship_pos = vec::IVec3::new(20, 200, 0);

        // Function to decrement y until a block is found
        fn find_ground_y(position: &mut vec::IVec3, game: &Game) {
            while game.chunksys.read().blockat(*position) == 0 {
                position.y -= 1;
            }
        }

        // Find the ground positions
        find_ground_y(&mut ship_pos, &self);

        // Determine the highest y position found
        let decided_pos_y = ship_pos.y;

        // Update the ship's position
        ship_pos.y = decided_pos_y;

        let ship_float_pos = Vec3::new(ship_pos.x as f32, ship_pos.y as f32, ship_pos.z as f32);
        ship_float_pos
    }

    #[cfg(feature = "glfw")]
    pub fn draw_clouds(&self) {
        static mut HASUPLOADED: bool = false;
        static mut VBO: GLuint = 0;

        let vdata: [f32; 30] = [
            -100.0,
            100.5 - 10.0,
            -100.0,
            0.0,
            1.0,
            -100.0,
            100.5 - 10.0,
            100.0,
            0.0,
            0.0,
            100.0,
            100.5 - 10.0,
            100.0,
            1.0,
            0.0,
            100.0,
            100.5 - 10.0,
            100.0,
            1.0,
            0.0,
            100.0,
            100.5 - 10.0,
            -100.0,
            1.0,
            1.0,
            -100.0,
            100.5 - 10.0,
            -100.0,
            0.0,
            1.0,
        ];

        unsafe {
            gl::BindVertexArray(self.cloudshader.vao);
            gl::UseProgram(self.cloudshader.shader_id);

            if !HASUPLOADED {
                gl::CreateBuffers(1, &mut VBO);
                gl::NamedBufferData(
                    VBO,
                    (vdata.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                    vdata.as_ptr() as *const GLvoid,
                    gl::STATIC_DRAW,
                );

                // Bind vertex buffer to the vertex array object
                gl::VertexArrayVertexBuffer(
                    self.cloudshader.vao,
                    0,
                    VBO,
                    0,
                    (5 * std::mem::size_of::<f32>()) as GLsizei,
                );

                // Position attribute
                let pos_attrib = gl::GetAttribLocation(
                    self.cloudshader.shader_id,
                    b"aPos\0".as_ptr() as *const i8,
                );
                gl::EnableVertexArrayAttrib(self.cloudshader.vao, pos_attrib as GLuint);
                gl::VertexArrayAttribFormat(
                    self.cloudshader.vao,
                    pos_attrib as GLuint,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                );
                gl::VertexArrayAttribBinding(self.cloudshader.vao, pos_attrib as GLuint, 0);

                // UV attribute
                let uv_attrib = gl::GetAttribLocation(
                    self.cloudshader.shader_id,
                    b"uv\0".as_ptr() as *const i8,
                );
                gl::EnableVertexArrayAttrib(self.cloudshader.vao, uv_attrib as GLuint);
                gl::VertexArrayAttribFormat(
                    self.cloudshader.vao,
                    uv_attrib as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    (3 * std::mem::size_of::<f32>()) as GLuint,
                );
                gl::VertexArrayAttribBinding(self.cloudshader.vao, uv_attrib as GLuint, 0);

                HASUPLOADED = true;
            }

            // Set uniforms
            let cam_clone = {
                let cam_lock = self.camera.lock();
                cam_lock.clone()
            };

            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.cloudshader.shader_id, b"mvp\0".as_ptr() as *const i8),
                1,
                gl::FALSE,
                cam_clone.mvp.to_cols_array().as_ptr(),
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"opacity\0".as_ptr() as *const i8,
                ),
                1.0,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(self.cloudshader.shader_id, b"time\0".as_ptr() as *const i8),
                glfwGetTime() as f32,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"scale\0".as_ptr() as *const i8,
                ),
                1.0,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"ambientBrightMult\0".as_ptr() as *const i8,
                ),
                self.ambient_bright_mult,
            );

            gl::Uniform3f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"camDir\0".as_ptr() as *const i8,
                ),
                cam_clone.direction.x,
                cam_clone.direction.y,
                cam_clone.direction.z,
            );

            gl::Uniform3f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"camPos\0".as_ptr() as *const i8,
                ),
                cam_clone.position.x,
                cam_clone.position.y,
                cam_clone.position.z,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"viewDistance\0".as_ptr() as *const i8,
                ),
                8.0,
            );

            let fogcol = Planets::get_fog_col(self.chunksys.read().planet_type as u32);
            gl::Uniform4f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"fogCol\0".as_ptr() as *const i8,
                ),
                fogcol.0,
                fogcol.1,
                fogcol.2,
                fogcol.3,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"walkbob\0".as_ptr() as *const i8,
                ),
                self.vars.walkbobtimer,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"sunset\0".as_ptr() as *const i8,
                ),
                self.sunset_factor,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.cloudshader.shader_id,
                    b"sunrise\0".as_ptr() as *const i8,
                ),
                self.sunrise_factor,
            );

            // Draw the clouds
            gl::Disable(gl::CULL_FACE);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::Enable(gl::CULL_FACE);
        }
    }
    #[cfg(feature = "glfw")]
    pub fn draw_stars(&self) {
        static mut HASUPLOADED: bool = false;
        static mut VBO: GLuint = 0;
        static mut hasmulted: bool = false;

        let mut vdata: [f32; 120] = [
            200.0, -2.9, 0.0, 1.0, 0.5, 0.0, -10.6, 200.0, 0.5, 0.0, 0.0, 200.0, 0.0, 0.5, 0.5,
            -200.0, -5.9, 0.0, 0.0, 0.5, 0.0, -10.6, 200.0, 0.5, 0.0, -200.0, -14.9, 200.0, 0.0,
            0.0, 0.0, -4.8, -200.0, 0.5, 1.0, -200.0, -5.9, 0.0, 0.0, 0.5, -200.0, -0.8, -200.0,
            0.0, 1.0, 0.0, -4.8, -200.0, 0.5, 1.0, 200.0, -2.9, 0.0, 1.0, 0.5, 0.0, 200.0, 0.0,
            0.5, 0.5, 200.0, -2.9, 0.0, 1.0, 0.5, 200.0, -10.7, 200.0, 1.0, 0.0, 0.0, -10.6, 200.0,
            0.5, 0.0, -200.0, -5.9, 0.0, 0.0, 0.5, 0.0, 200.0, 0.0, 0.5, 0.5, 0.0, -10.6, 200.0,
            0.5, 0.0, 0.0, -4.8, -200.0, 0.5, 1.0, 0.0, 200.0, 0.0, 0.5, 0.5, -200.0, -5.9, 0.0,
            0.0, 0.5, 0.0, -4.8, -200.0, 0.5, 1.0, 200.0, -7.2, -200.0, 1.0, 1.0, 200.0, -2.9, 0.0,
            1.0, 0.5,
        ];
        unsafe {
            if !hasmulted {
                let mult = 2.0;
                for chunk in vdata.chunks_mut(5) {
                    chunk[0] = chunk[0] * mult;
                    chunk[2] = chunk[2] * mult;
                    chunk[1] = chunk[1] * mult;
                }
                hasmulted = true;
            }
        }

        unsafe {
            gl::BindVertexArray(self.starshader.vao);
            gl::UseProgram(self.starshader.shader_id);

            if !HASUPLOADED {
                gl::CreateBuffers(1, &mut VBO);
                gl::NamedBufferData(
                    VBO,
                    (vdata.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                    vdata.as_ptr() as *const GLvoid,
                    gl::STATIC_DRAW,
                );

                // Bind vertex buffer to the vertex array object
                gl::VertexArrayVertexBuffer(
                    self.starshader.vao,
                    0,
                    VBO,
                    0,
                    (5 * std::mem::size_of::<f32>()) as GLsizei,
                );

                // Position attribute
                let pos_attrib = gl::GetAttribLocation(
                    self.starshader.shader_id,
                    b"aPos\0".as_ptr() as *const i8,
                );
                gl::EnableVertexArrayAttrib(self.starshader.vao, pos_attrib as GLuint);
                gl::VertexArrayAttribFormat(
                    self.starshader.vao,
                    pos_attrib as GLuint,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                );
                gl::VertexArrayAttribBinding(self.starshader.vao, pos_attrib as GLuint, 0);

                // UV attribute
                let uv_attrib =
                    gl::GetAttribLocation(self.starshader.shader_id, b"uv\0".as_ptr() as *const i8);
                gl::EnableVertexArrayAttrib(self.starshader.vao, uv_attrib as GLuint);
                gl::VertexArrayAttribFormat(
                    self.starshader.vao,
                    uv_attrib as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    (3 * std::mem::size_of::<f32>()) as GLuint,
                );
                gl::VertexArrayAttribBinding(self.starshader.vao, uv_attrib as GLuint, 0);

                HASUPLOADED = true;
            }

            // Set uniforms
            let cam_clone = {
                let cam_lock = self.camera.lock();
                cam_lock.clone()
            };

            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.starshader.shader_id, b"mvp\0".as_ptr() as *const i8),
                1,
                gl::FALSE,
                cam_clone.mvp.to_cols_array().as_ptr(),
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"opacity\0".as_ptr() as *const i8,
                ),
                1.0,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(self.starshader.shader_id, b"time\0".as_ptr() as *const i8),
                glfwGetTime() as f32,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(self.starshader.shader_id, b"scale\0".as_ptr() as *const i8),
                1.0,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"ambientBrightMult\0".as_ptr() as *const i8,
                ),
                self.ambient_bright_mult,
            );

            gl::Uniform3f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"camDir\0".as_ptr() as *const i8,
                ),
                cam_clone.direction.x,
                cam_clone.direction.y,
                cam_clone.direction.z,
            );

            gl::Uniform3f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"camPos\0".as_ptr() as *const i8,
                ),
                cam_clone.position.x,
                cam_clone.position.y,
                cam_clone.position.z,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"viewDistance\0".as_ptr() as *const i8,
                ),
                8.0,
            );

            let fogcol = Planets::get_fog_col(self.chunksys.read().planet_type as u32);
            gl::Uniform4f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"fogCol\0".as_ptr() as *const i8,
                ),
                fogcol.0,
                fogcol.1,
                fogcol.2,
                fogcol.3,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"sunset\0".as_ptr() as *const i8,
                ),
                self.sunset_factor,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.starshader.shader_id,
                    b"sunrise\0".as_ptr() as *const i8,
                ),
                self.sunrise_factor,
            );

            // Draw the stars
            gl::Disable(gl::CULL_FACE);
            gl::DrawArrays(gl::TRIANGLES, 0, 24);
            gl::Enable(gl::CULL_FACE);
        }
    }
    #[cfg(feature = "glfw")]
    pub fn update_inventory(&mut self) {
        for i in ROWLENGTH * 4..ROWLENGTH * 8 {
            let realslotind = i - ROWLENGTH * 4;
            let slot = self
                .chest_registry
                .entry(self.hud.current_chest)
                .or_insert(ChestInventory {
                    dirty: true,
                    inv: [(0, 0); (ROWLENGTH * 4) as usize],
                })
                .inv[realslotind as usize];

            let idinslot = slot.0;
            let texcoords = Blocks::get_tex_coords(idinslot, crate::cube::CubeSide::LEFT);
            let tf = TextureFace::new(texcoords.0 as i8, texcoords.1 as i8);
            let bf = TextureFace::new(0, 0);
            self.hud.chestelements[i as usize].uvs = [
                tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
                tf.blx, tf.bly,
            ];

            if slot.1 > 0 {
                let count = slot.1.to_string();
                if count.len() == 2 {
                    let g1 = GlyphFace::new(count.as_bytes()[0]);
                    let g2 = GlyphFace::new(count.as_bytes()[1]);

                    self.hud.chestelements[(ROWLENGTH * 8 + realslotind * 2) as usize].uvs = [
                        g1.blx, g1.bly, g1.brx, g1.bry, g1.trx, g1.tr_y, g1.trx, g1.tr_y, g1.tlx,
                        g1.tly, g1.blx, g1.bly,
                    ];
                    self.hud.chestelements[(ROWLENGTH * 8 + realslotind * 2 + 1) as usize].uvs = [
                        g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                        g2.tly, g2.blx, g2.bly,
                    ];
                }

                if count.len() == 1 {
                    let g2 = GlyphFace::new(count.as_bytes()[0]);
                    self.hud.chestelements[(ROWLENGTH * 8 + realslotind * 2) as usize].uvs = [
                        bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                        bf.tly, bf.blx, bf.bly,
                    ];
                    self.hud.chestelements[(ROWLENGTH * 8 + realslotind * 2 + 1) as usize].uvs = [
                        g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                        g2.tly, g2.blx, g2.bly,
                    ];
                }
            } else {
                self.hud.chestelements[(ROWLENGTH * 8 + realslotind * 2) as usize].uvs = [
                    bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                    bf.tly, bf.blx, bf.bly,
                ];
                self.hud.chestelements[(ROWLENGTH * 8 + realslotind * 2 + 1) as usize].uvs = [
                    bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                    bf.tly, bf.blx, bf.bly,
                ];
            }
        }

        for i in ROWLENGTH..(ROWLENGTH * 2) {
            let realslotind = i - ROWLENGTH;
            let slot = self.inventory.read().inv[realslotind as usize];
            let idinslot = slot.0;
            let texcoords = Blocks::get_tex_coords(idinslot, crate::cube::CubeSide::LEFT);
            let tf = TextureFace::new(texcoords.0 as i8, texcoords.1 as i8);
            let bf = TextureFace::new(0, 0);
            self.hud.elements[i as usize].uvs = [
                tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
                tf.blx, tf.bly,
            ];

            if slot.1 > 0 {
                let count = slot.1.to_string();
                if count.len() == 2 {
                    let g1 = GlyphFace::new(count.as_bytes()[0]);
                    let g2 = GlyphFace::new(count.as_bytes()[1]);

                    self.hud.elements[(ROWLENGTH * 2 + realslotind * 2) as usize].uvs = [
                        g1.blx, g1.bly, g1.brx, g1.bry, g1.trx, g1.tr_y, g1.trx, g1.tr_y, g1.tlx,
                        g1.tly, g1.blx, g1.bly,
                    ];
                    self.hud.elements[(ROWLENGTH * 2 + realslotind * 2 + 1) as usize].uvs = [
                        g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                        g2.tly, g2.blx, g2.bly,
                    ];
                }

                if count.len() == 1 {
                    let g2 = GlyphFace::new(count.as_bytes()[0]);
                    self.hud.elements[(ROWLENGTH * 2 + realslotind * 2) as usize].uvs = [
                        bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                        bf.tly, bf.blx, bf.bly,
                    ];
                    self.hud.elements[(ROWLENGTH * 2 + realslotind * 2 + 1) as usize].uvs = [
                        g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                        g2.tly, g2.blx, g2.bly,
                    ];
                }
            } else {
                self.hud.elements[(ROWLENGTH * 2 + realslotind * 2) as usize].uvs = [
                    bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                    bf.tly, bf.blx, bf.bly,
                ];
                self.hud.elements[(ROWLENGTH * 2 + realslotind * 2 + 1) as usize].uvs = [
                    bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                    bf.tly, bf.blx, bf.bly,
                ];
            }
        }

        let slot = self.mouse_slot;
        let idinslot = slot.0;
        let texcoords = Blocks::get_tex_coords(idinslot, crate::cube::CubeSide::LEFT);
        let tf = TextureFace::new(texcoords.0 as i8, texcoords.1 as i8);
        let bf = TextureFace::new(0, 0);
        self.hud.chestelements[(ROWLENGTH * 16) as usize].uvs = [
            tf.blx, tf.bly, tf.brx, tf.bry, tf.trx, tf.tr_y, tf.trx, tf.tr_y, tf.tlx, tf.tly,
            tf.blx, tf.bly,
        ];

        if slot.1 > 0 {
            let count = slot.1.to_string();
            if count.len() == 2 {
                let g1 = GlyphFace::new(count.as_bytes()[0]);
                let g2 = GlyphFace::new(count.as_bytes()[1]);

                self.hud.chestelements[(ROWLENGTH * 16) as usize + 1].uvs = [
                    g1.blx, g1.bly, g1.brx, g1.bry, g1.trx, g1.tr_y, g1.trx, g1.tr_y, g1.tlx,
                    g1.tly, g1.blx, g1.bly,
                ];
                self.hud.chestelements[(ROWLENGTH * 16) as usize + 2].uvs = [
                    g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                    g2.tly, g2.blx, g2.bly,
                ];
            }

            if count.len() > 2 {
                let g1 = GlyphFace::new(43);
                let g2 = GlyphFace::new(43);

                self.hud.chestelements[(ROWLENGTH * 16) as usize + 1].uvs = [
                    g1.blx, g1.bly, g1.brx, g1.bry, g1.trx, g1.tr_y, g1.trx, g1.tr_y, g1.tlx,
                    g1.tly, g1.blx, g1.bly,
                ];
                self.hud.chestelements[(ROWLENGTH * 16) as usize + 2].uvs = [
                    g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                    g2.tly, g2.blx, g2.bly,
                ];
            }

            if count.len() == 1 {
                let g2 = GlyphFace::new(count.as_bytes()[0]);
                self.hud.chestelements[(ROWLENGTH * 16) as usize + 1].uvs = [
                    bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx,
                    bf.tly, bf.blx, bf.bly,
                ];
                self.hud.chestelements[(ROWLENGTH * 16) as usize + 2].uvs = [
                    g2.blx, g2.bly, g2.brx, g2.bry, g2.trx, g2.tr_y, g2.trx, g2.tr_y, g2.tlx,
                    g2.tly, g2.blx, g2.bly,
                ];
            }
        } else {
            self.hud.chestelements[(ROWLENGTH * 16) as usize + 1].uvs = [
                bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx, bf.tly,
                bf.blx, bf.bly,
            ];
            self.hud.chestelements[(ROWLENGTH * 16) as usize + 2].uvs = [
                bf.blx, bf.bly, bf.brx, bf.bry, bf.trx, bf.tr_y, bf.trx, bf.tr_y, bf.tlx, bf.tly,
                bf.blx, bf.bly,
            ];
        }

        self.hud.dirty = true;

        Game::update_avail_recipes(&self.inventory);
    }

    pub fn set_in_inventory(
        inv: &Arc<RwLock<Inventory>>,
        slot: usize,
        newid: u32,
        newcount: u32,
        in_m: bool,
        needtosend: &Arc<Queue<Message>>,
    ) -> Result<bool, bool> {
        let mut updaterecipes = false;
        let result;

        if in_m {
            let n = needtosend.clone();
            n.push(Message::invupdate(slot, newid, newcount));
            result = Ok(true);
        } else {
            let mut inventory = inv.write();
            // If not found, try to find an empty slot to add the new item
            let item = &mut inventory.inv[slot];

            item.0 = newid;
            item.1 = newcount;
            inventory.dirty = true;
            result = Ok(true);
            updaterecipes = true;
        }

        if updaterecipes {
            Game::update_avail_recipes(&inv);
        }

        return result;
    }

    pub fn add_to_inventory(
        inv: &Arc<RwLock<Inventory>>,
        id: u32,
        count: u32,
        in_m: bool,
        needtosend: &Arc<Queue<Message>>,
    ) -> Result<bool, bool> {
        let mut updaterecipes = false;
        let result;

        if in_m {
            let n = needtosend.clone();

            let inventory = inv.read();

            // First, try to find an item with the given `id`
            if let Some((index, item)) = inventory
                .inv
                .iter()
                .enumerate()
                .find(|(_index, item)| item.0 == id)
            {
                let mut msg = Message::new(
                    MessageType::ChestInvUpdate,
                    Vec3::ZERO,
                    id as f32,
                    index as u32,
                );
                msg.infof = item.1 as f32 + 1.0;
                msg.info2 = 1;

                n.push(msg);
                // item.1 += count;
                // inventory.dirty = true;
                updaterecipes = true;
                result = Ok(true);
            } else
            // If not found, try to find an empty slot to add the new item
            if let Some((index, _item)) = inventory
                .inv
                .iter()
                .enumerate()
                .find(|(_index, item)| item.0 == 0)
            {
                let mut msg = Message::new(
                    MessageType::ChestInvUpdate,
                    Vec3::ZERO,
                    id as f32,
                    index as u32,
                );
                msg.infof = 1.0;
                msg.info2 = 1;

                n.push(msg);
                // item.0 = id;
                // item.1 = count;
                // inventory.dirty = true;
                updaterecipes = true;
                result = Ok(true);
            } else {
                result = Err(false);
            }
        } else {
            let mut inventory = inv.write();

            // First, try to find an item with the given `id`
            if let Some(item) = inventory.inv.iter_mut().find(|item| item.0 == id) {
                item.1 += count;
                inventory.dirty = true;
                result = Ok(true);
                updaterecipes = true;
            } else
            // If not found, try to find an empty slot to add the new item
            if let Some(item) = inventory.inv.iter_mut().find(|item| item.0 == 0) {
                item.0 = id;
                item.1 = count;
                inventory.dirty = true;
                result = Ok(true);
                updaterecipes = true;
            } else {
                result = Err(false);
            }
        }

        if updaterecipes {
            Game::update_avail_recipes(&inv);
        }

        return result;
    }

    pub fn craft_recipe_index(&mut self, index: usize, all: bool) {
        unsafe {
            let recipe = {
                let r = CURRENT_AVAIL_RECIPES.lock();
                &r[index].clone().recipe
            };

            let mut hasreqs = true;
            let invlock = self.inventory.write();

            let originalinvinv = invlock.clone();
            let originalinv = invlock.inv.clone();

            drop(invlock);

            let mut morepossible = true;
            fn craft_in_local_inv(recipe: &Recipe, inven: &mut Inventory, morepossible: &mut bool) {
                let mut hasreqs = true;
                for req in &recipe.0 {
                    let mut amt = 0;

                    for i in 0..ROWLENGTH as usize {
                        let typehere = inven.inv[i].0;
                        if typehere == req.0 {
                            amt += inven.inv[i].1;
                        }
                    }

                    if amt < req.1 {
                        hasreqs = false;
                        (*morepossible) = false;
                    }
                }

                if hasreqs {
                    //Find an empty spot OR MATCHING RESULT ITEM SPOT in their imaginary inv that would exist if we were to subtract the necessary ingredients:
                    //Make an imaginary clone of their inventory:
                    let invclone = &mut inven.inv;

                    //Subtract the ingredients
                    for req in &recipe.0 {
                        let mut amt = 0;

                        for i in 0..ROWLENGTH as usize {
                            let typehere = invclone[i].0;
                            if typehere == req.0 {
                                while invclone[i].1 > 0 && amt < req.1 {
                                    amt += 1;
                                    invclone[i].1 -= 1;

                                    if invclone[i].1 == 0 {
                                        invclone[i].0 = 0;
                                    }
                                }
                                if amt >= req.1 {
                                    break;
                                }
                            }
                        }
                    }

                    //Find the predicted empty spot or matching item slot
                    let resultslot = {
                        let mut slot = None;

                        for i in 0..ROWLENGTH as usize {
                            let typehere = invclone[i].0;
                            if (typehere == 0 || typehere == recipe.1 .0)
                                && (invclone[i].1 + recipe.1 .1) <= 999
                            {
                                slot = Some(i);
                                break;
                            }
                        }

                        slot
                    };

                    match resultslot {
                        Some(slot) => {
                            invclone[slot].0 = recipe.1 .0;
                            invclone[slot].1 = invclone[slot].1 + recipe.1 .1;
                        }
                        None => {
                            (*morepossible) = false;
                        }
                    }
                }
            }

            if all {
                let mut newinv = originalinvinv.clone();

                while morepossible {
                    craft_in_local_inv(recipe, &mut newinv, &mut morepossible);
                }

                if newinv != originalinvinv {
                    for i in 0..ROWLENGTH as usize {
                        //Turning their inventory into the invclone

                        if newinv.inv[i] != originalinv[i] {
                            Game::set_in_inventory(
                                &self.inventory.clone(),
                                i,
                                newinv.inv[i].0,
                                newinv.inv[i].1,
                                self.vars.in_multiplayer,
                                &self.needtosend,
                            );
                        }
                    }
                }
            } else {
                for req in &recipe.0 {
                    let mut amt = 0;

                    for i in 0..ROWLENGTH as usize {
                        let typehere = originalinvinv.inv[i].0;
                        if typehere == req.0 {
                            amt += originalinvinv.inv[i].1;
                        }
                    }

                    if amt < req.1 {
                        hasreqs = false;
                    }
                }

                if hasreqs {
                    //Find an empty spot OR MATCHING RESULT ITEM SPOT in their imaginary inv that would exist if we were to subtract the necessary ingredients:
                    //Make an imaginary clone of their inventory:
                    let invlock = self.inventory.read();
                    let mut invclone = invlock.inv.clone();
                    drop(invlock);

                    //Subtract the ingredients
                    for req in &recipe.0 {
                        let mut amt = 0;

                        for i in 0..ROWLENGTH as usize {
                            let typehere = invclone[i].0;
                            if typehere == req.0 {
                                while invclone[i].1 > 0 && amt < req.1 {
                                    amt += 1;
                                    invclone[i].1 -= 1;

                                    if invclone[i].1 == 0 {
                                        invclone[i].0 = 0;
                                    }
                                }
                                if amt >= req.1 {
                                    break;
                                }
                            }
                        }
                    }

                    //Find the predicted empty spot or matching item slot
                    let resultslot = {
                        let mut slot = None;

                        for i in 0..ROWLENGTH as usize {
                            let typehere = invclone[i].0;
                            if (typehere == 0 || typehere == recipe.1 .0)
                                && (invclone[i].1 + recipe.1 .1) <= 999
                            {
                                slot = Some(i);
                                break;
                            }
                        }

                        slot
                    };

                    //drop(invlock);

                    //Only execute the subtraction and addition of items if they will have that result slot available
                    match resultslot {
                        None => {}
                        Some(slot) => {
                            //Take the reqs away from their real inventory by...

                            for i in 0..ROWLENGTH as usize {
                                //Turning their inventory into the invclone

                                if invclone[i] != originalinv[i] {
                                    Game::set_in_inventory(
                                        &self.inventory.clone(),
                                        i,
                                        invclone[i].0,
                                        invclone[i].1,
                                        self.vars.in_multiplayer,
                                        &self.needtosend,
                                    );
                                }
                            }

                            //Give them the resulting item
                            Game::set_in_inventory(
                                &self.inventory.clone(),
                                slot,
                                recipe.1 .0,
                                invclone[slot].1 + recipe.1 .1,
                                self.vars.in_multiplayer,
                                &self.needtosend,
                            );
                        }
                    }
                }
            }
        }
    }
    #[cfg(feature = "audio")]
    pub fn do_step_sounds(&mut self) {
        static mut TIMER: f32 = 0.0;
        static mut LAST_CAM_POS: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let cl = self.camera.lock();
        let campos = cl.position - cl.direction * 0.5;
        drop(cl);

        unsafe {
            let diff = campos.distance(LAST_CAM_POS);

            let interval = if unsafe { SPRINTING } { 0.3 } else { 0.45 };

            if diff > self.delta_time * 3.0 {
                if TIMER > interval {
                    self.do_step_sound_now(campos);
                    TIMER = 0.0;
                } else {
                    TIMER += self.delta_time;
                }

                LAST_CAM_POS = campos;
            }
        }
    }
    #[cfg(feature = "audio")]
    pub fn do_step_sound_now(&mut self, position: Vec3) {
        let campos = position;
        let camfootpos = campos - Vec3::new(0.0, 2.0, 0.0);
        let blockat = self.chunksys.read().blockat(IVec3::new(
            camfootpos.x.floor() as i32,
            camfootpos.y.floor() as i32,
            camfootpos.z.floor() as i32,
        ));
        let blockat = blockat & Blocks::block_id_bits();
        if blockat != 0 {
            unsafe {
                AUDIOPLAYER.play_next_in_series(
                    &Blocks::get_walk_series(blockat),
                    &(camfootpos),
                    &Vec3::new(0.0, 0.0, 0.0),
                    0.3,
                );
            }
        }
    }

    pub fn activate_jump_block(&mut self, position: Vec3) {
        let campos = position;
        let camfootpos = campos - Vec3::new(0.0, 2.0, 0.0);
        let spot = IVec3::new(
            camfootpos.x.floor() as i32,
            camfootpos.y.floor() as i32,
            camfootpos.z.floor() as i32,
        );
        let blockat = self.chunksys.read().blockat(spot);
        let blockat = blockat & Blocks::block_id_bits();
        // if blockat != 0 {
        //     self.audiop.write().play_next_in_series(&Blocks::get_walk_series(blockat), &(camfootpos), &Vec3::new(0.0, 0.0, 0.0), 0.5);
        // }

        match blockat {
            40 => {
                if !self.vars.in_multiplayer {
                    self.chunksys
                        .read()
                        .set_block_and_queue_rerender_no_sound(spot, 41, false, true, true);
                } else {
                    let mut message = Message::new(
                        MessageType::BlockSet,
                        Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32),
                        0.0,
                        41,
                    );
                    message.infof = 0.0;

                    self.netconn.sendqueue.push(message);
                }

                #[cfg(feature = "audio")]
                unsafe {
                    AUDIOPLAYER.play_next_in_series(
                        "clickseries",
                        &Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32),
                        &Vec3::ZERO,
                        0.5,
                    );
                }
            }
            41 => {
                if !self.vars.in_multiplayer {
                    self.chunksys
                        .read()
                        .set_block_and_queue_rerender_no_sound(spot, 40, false, true, true);
                } else {
                    let mut message = Message::new(
                        MessageType::BlockSet,
                        Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32),
                        0.0,
                        40,
                    );
                    message.infof = 0.0;

                    self.netconn.sendqueue.push(message);
                }
                #[cfg(feature = "audio")]
                unsafe {
                    AUDIOPLAYER.play_next_in_series(
                        "clickseries",
                        &Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32),
                        &Vec3::ZERO,
                        0.5,
                    );
                }
            }
            42 => {
                let d = self.camera.lock().direction.clone();

                self.camera.lock().velocity += Vec3::new(0.0, TRAMPOLINE_VELOCITY_FIGURE, 0.0) + d;
                #[cfg(feature = "audio")]
                unsafe {
                    AUDIOPLAYER.play(
                        path!("assets/sfx/boing.mp3"),
                        &(camfootpos),
                        &Vec3::new(0.0, 0.0, 0.0),
                        0.5,
                    );
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "audio")]
    pub fn update_music_volume(&mut self) {
        use crate::statics::MISCSETTINGS;

        unsafe {
            static mut PASTVOLUME: f32 = 1.0;
            if MISCSETTINGS.music_vol != PASTVOLUME {
                for songname in SONGS {
                    match AUDIOPLAYER.headsinks.get(songname) {
                        Some(s) => {
                            s.set_volume(MISCSETTINGS.music_vol);
                        }
                        None => {}
                    }
                }

                PASTVOLUME = MISCSETTINGS.music_vol;
            }
        }
    }

    pub fn takeoff_ship(&mut self) {
        if !self.vars.ship_taken_off {
            #[cfg(feature = "audio")]
            unsafe {
                AUDIOPLAYER.play(
                    path!("assets/sfx/shiptakeoff.mp3"),
                    &self.ship_pos,
                    &Vec3::ZERO,
                    1.0,
                );
            }

            self.vars.ship_going_up = true;
            self.vars.ship_going_down = false;
            self.vars.ship_taken_off = true;
            self.vars.on_new_world = false;
        }
    }

    pub fn gaussian(x: f32, peak: f32, radius: f32) -> f32 {
        let std_dev = radius / 3.0; // Controls the spread
        let variance = std_dev * std_dev;

        // Gaussian formula
        let b = f32::exp(-(x - peak).powf(2.0) / (2.0 * variance));

        // Normalize the peak to 1
        let peak_height = f32::exp(-(peak - peak).powf(2.0) / (2.0 * variance));

        return b / peak_height;
    }

    pub fn load_my_inv_from_file(&self) {
        let table_name = "invs";

        let conn = Connection::open("chestdb").unwrap();

        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                inventory BLOB
            )",
                table_name
            ),
            (),
        )
        .unwrap();

        let mut stmt = conn
            .prepare(&format!(
                "SELECT inventory FROM {} WHERE id = ?1",
                table_name
            ))
            .unwrap();

        let mut rows = stmt
            .query([self.my_uuid.read().unwrap().to_string()])
            .unwrap();

        if let Some(row) = rows.next().unwrap() {
            let inventory: Vec<u8> = row.get(0).unwrap();

            match bincode::deserialize::<[(u32, u32); ROWLENGTH as usize]>(&inventory) {
                Ok(inv) => {
                    let mut invlock = self.inventory.write();
                    invlock.inv = inv.clone();
                }
                Err(_e) => {
                    info!("Couldn't de-serialize inventory blob");
                }
            }
        } else {
        }
    }

    pub fn load_my_pos_from_file(&self) {
        let table_name = "poses";

        let conn = Connection::open("chestdb").unwrap();

        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                playerposition BLOB
            )",
                table_name
            ),
            (),
        )
        .unwrap();

        let mut stmt = conn
            .prepare(&format!(
                "SELECT playerposition FROM {} WHERE id = ?1",
                table_name
            ))
            .unwrap();

        let mut rows = stmt
            .query([self.my_uuid.read().unwrap().to_string()])
            .unwrap();

        if let Some(row) = rows.next().unwrap() {
            let pp: Vec<u8> = row.get(0).unwrap();

            match bincode::deserialize::<PlayerPosition>(&pp) {
                Ok(playpos) => {
                    let mut camlock = self.camera.lock();
                    camlock.position = Vec3::new(playpos.pos.x, playpos.pos.y, playpos.pos.z);
                    camlock.pitch = playpos.pitch;
                    camlock.yaw = playpos.yaw;
                    drop(camlock);
                }
                Err(_e) => {
                    info!("Couldn't de-serialize playerpos blob");
                }
            }
        } else {
        }
    }

    pub fn update(&mut self) {
        #[cfg(feature = "glfw")]
        {
            let current_time = unsafe { glfwGetTime() as f32 };
            self.delta_time = (current_time - self.prev_time).min(0.05);
            self.prev_time = current_time;
        }

        #[cfg(not(feature = "glfw"))]
        unsafe {
            static mut PREVTIME: Lazy<Instant> = Lazy::new(|| Instant::now());
            let delta_time = match Instant::now().checked_duration_since(*PREVTIME) {
                Some(time) => time,
                None => Duration::from_secs_f32(0.0),
            };
            self.delta_time = delta_time.as_secs_f32().min(0.05);
            (*PREVTIME) = Instant::now();
        }

        let stam = self.stamina.load(Ordering::Relaxed);

        if unsafe { MOVING } {
            self.vars.walkbobtimer = self.vars.walkbobtimer + self.delta_time * 10.0;
            self.vars.walkbobtimer %= 2.0 * consts::PI;
        }
        
        if unsafe { BUILD_PREVIEW_MODE} {
            unsafe {
                VOXEL_SELECT_DISTANCE = 30.0;

            } 
        } else {
            unsafe {
                VOXEL_SELECT_DISTANCE = 10.0;
            }
        }

        #[cfg(feature = "audio")]
        self.update_music_volume();

        unsafe {
            if TRAMPOLINE {
                TRAMP_TIMER += self.delta_time;
            }
            if TRAMP_TIMER > 0.5 {
                TRAMPOLINE = false;
                TRAMP_TIMER = 0.0;
            }
        }

        unsafe {
            if ON_CONVEYORS {
                if CONVEYOR_SOUND_TIMER <= 0.0 {
                    #[cfg(feature = "audio")]
                    AUDIOPLAYER.play_in_head(path!("assets/sfx/onconveyor.mp3"));

                    CONVEYOR_SOUND_TIMER = 2.5;
                } else {
                    CONVEYOR_SOUND_TIMER -= self.delta_time;
                }
                TIME_ON_CONVEYORS += self.delta_time;
            }
        }

        if !self.vars.in_multiplayer || self.headless {
            unsafe {
                WEATHERTIMER += self.delta_time;
                if WEATHERTIMER >= WEATHERINTERVAL {
                    let mut rand = StdRng::from_entropy();
                    let randint: usize = rand.gen_range(0..=2);
                    WEATHERTYPE = randint as f32;
                    WEATHERTIMER = 0.0;
                }
            }
        }

        if !self.headless {
            #[cfg(feature = "audio")]
            self.play_weather_sound();
        }

        unsafe {
            if WINDED {
                if WINDEDTIMER < WINDEDLENGTH {
                    WINDED = true;
                    WINDEDTIMER += self.delta_time;
                } else {
                    WINDED = false;
                    WINDEDTIMER = 0.0;
                }
            }
        }

        static mut sprintchecktimer: f32 = 0.0;
        unsafe {
            if sprintchecktimer > 0.2 {
                sprintchecktimer = 0.0;

                if self.controls.shift && !self.vars.in_climbable {
                    if stam > 0 {
                        unsafe {
                            SPRINTING = true;
                            // self.stamina.store(stam - 4, Ordering::Relaxed);
                        }
                    } else {

                        // if stam < 0 {
                        //     unsafe {
                        //         WINDED = true;
                        //     }
                        // }

                        // unsafe { SPRINTING = false }
                        // if stam < 100 {
                        //     self.stamina.store(stam + 2, Ordering::Relaxed);
                        // }
                    }
                } else {
                    unsafe {
                        SPRINTING = false;
                    }
                    // if stam < 100 {
                    //     self.stamina.store(stam + 2, Ordering::Relaxed);
                    // }
                }
            } else {
                sprintchecktimer += self.delta_time;
            }
        }

        unsafe {
            STAMINA = self.stamina.load(Ordering::Relaxed);
        }

        let mut rng = StdRng::from_entropy();
        if !self.vars.in_multiplayer {
            unsafe {
                if SONGTIMER < SONGINTERVAL {
                    SONGTIMER += self.delta_time;
                } else {
                    SONGTIMER = 0.0;

                    if !self.headless {
                        SONGINDEX = (SONGINDEX + rng.gen_range(1..SONGS.len())) % SONGS.len();

                        #[cfg(feature = "audio")]
                        AUDIOPLAYER.play_in_head(SONGS[SONGINDEX]);
                    }
                }
            }
        }

        unsafe {
            AMBIENTBRIGHTNESS = self.ambient_bright_mult;
        }

        let mut todlock = self.timeofday.lock();

        if !self.vars.menu_open || self.vars.in_multiplayer {
            *todlock = (*todlock + self.delta_time) % self.daylength;
        }

        let gaussian_value =
            Self::gaussian(*todlock, self.daylength / 2.0, self.daylength / 2.0) * 1.3;
        self.ambient_bright_mult = gaussian_value.clamp(0.08, 1.0);

        self.sunset_factor = Self::gaussian(
            *todlock,
            self.daylength * (3.0 / 4.0),
            self.daylength / 16.0,
        );
        self.sunrise_factor = Self::gaussian(*todlock, self.daylength / 6.0, self.daylength / 16.0);

        drop(todlock);

        let fadersread = self.faders.read();

        let overlayfade = fadersread[FaderNames::VisionsFader as usize].value.clone();

        drop(fadersread);

        #[cfg(feature = "glfw")]
        if !self.headless {
            let (x, y) = self.window.read().get_cursor_pos();

            unsafe {
                MOUSEX = x;
                MOUSEY = y;
            }

            self.hud.mousetrans = HudElement::xytondc(x, y);
            if self.hud.chest_open {
                let mut isoverlappingany = false;
                for i in 0..ROWLENGTH as usize {
                    let hudel = &self.hud.elements[i];

                    if hudel.overlaps(x, y) {
                        unsafe {
                            MOUSED_SLOT = SlotIndexType::InvSlot(i as i32);
                            let inv = self.inventory.read();
                            TOOLTIPNAME = Blocks::get_name(inv.inv[i].0);

                            SHOWTOOLTIP = true;
                            isoverlappingany = true;
                        }
                    }
                }

                for i in 0..ROWLENGTH as usize * 4 {
                    let hudel = &self.hud.chestelements[i];

                    if hudel.overlaps(x, y) {
                        unsafe {
                            MOUSED_SLOT = SlotIndexType::ChestSlot(i as i32);

                            match self.chunksys.try_read() {
                                Some(csys) => {
                                    match self.chest_registry.get(&self.hud.current_chest) {
                                        Some(chest) => {
                                            TOOLTIPNAME = Blocks::get_name(chest.value().inv[i].0);
                                        }
                                        None => {}
                                    }
                                }
                                None => {}
                            }

                            SHOWTOOLTIP = true;
                            isoverlappingany = true;
                        }
                    }
                }
                if !isoverlappingany {
                    unsafe {
                        SHOWTOOLTIP = false;
                        MOUSED_SLOT = SlotIndexType::None;
                    }
                }
                // unsafe {
                //     match MOUSED_SLOT {
                //         SlotIndexType::ChestSlot(e) => {
                //             info!("Moused chest slot {}", HudElement::ass_slot_to_shader_float(&MOUSED_SLOT));
                //         },
                //         SlotIndexType::InvSlot(e) => {
                //             info!("Moused inv slot {}", HudElement::ass_slot_to_shader_float(&MOUSED_SLOT));
                //         },
                //         SlotIndexType::None => {
                //             info!("Moused no invslot");
                //         },
                //     }
                // }
            }
            // match self.needtosend.pop() {
            //     Some(comm) => {
            //         self.netconn.send(&comm);
            //     }
            //     None => {}
            // }

            let mut morestuff = true;
            for _ in 0..5 {
                match self.hp_server_command_queue.pop() {
                    Some(comm) => {
                        match comm.message_type {
                            MessageType::BlockSet => {
                                if comm.infof == 1.0 {
                                    if comm.info == 0 {
                                        self.chunksys.read().set_block_and_queue_rerender(
                                            IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                            comm.info,
                                            true,
                                            true,
                                            false,
                                        );
                                    } else {
                                        self.chunksys.read().set_block_and_queue_rerender(
                                            IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                            comm.info,
                                            false,
                                            true,
                                            false,
                                        );
                                    }
                                } else {
                                    if comm.info == 0 {
                                        self.chunksys.read().set_block_and_queue_rerender_no_sound(
                                            IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                            comm.info,
                                            true,
                                            true,
                                            false,
                                        );
                                    } else {
                                        self.chunksys.read().set_block_and_queue_rerender_no_sound(
                                            IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                            comm.info,
                                            false,
                                            true,
                                            false,
                                        );
                                    }
                                }

                                unsafe {
                                    UPDATE_THE_BLOCK_OVERLAY = true;
                                }
                            }
                            MessageType::MultiBlockSet => {
                                let cread = self.chunksys.read();

                                cread.set_block_no_sound(
                                    IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                    comm.info,
                                    true,
                                );

                                cread.set_block_and_queue_rerender(
                                    comm.otherpos,
                                    comm.info2,
                                    true,
                                    true,
                                    false,
                                );
                                unsafe {
                                    UPDATE_THE_BLOCK_OVERLAY = true;
                                }
                            }
                            MessageType::ChestReg => {
                                self.load_my_inv_from_file();
                                self.load_my_pos_from_file();
                            }
                            MessageType::ChestInvUpdate => {
                                let currchest = comm.otherpos;

                                let destslot = comm.info;

                                let slotindextype = match comm.info2 {
                                    0 => SlotIndexType::ChestSlot(destslot as i32),
                                    1 => SlotIndexType::InvSlot(destslot as i32),
                                    _ => SlotIndexType::None,
                                };

                                let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);

                                let mut updateinv = false;

                                match *self.my_uuid.read() {
                                    Some(ud) => {
                                        if uuid == ud && comm.z == 1.0 {
                                            //this message is intended for my inv and it is to replace my mouse_slot
                                            self.mouse_slot.0 = comm.x as u32;
                                            self.mouse_slot.1 = comm.y as u32;
                                            updateinv = true;
                                        }
                                    }
                                    None => {}
                                }

                                match slotindextype {
                                    SlotIndexType::ChestSlot(e) => {
                                        //let csys = self.chunksys.write();
                                        let mut chestinv = self
                                            .chest_registry
                                            .entry(currchest)
                                            .or_insert(ChestInventory {
                                                dirty: false,
                                                inv: [(0, 0); ROWLENGTH as usize * 4],
                                            });

                                        let slot = &mut chestinv.inv[e as usize];

                                        // let wasthere = slot.clone();

                                        slot.0 = comm.rot as u32;
                                        slot.1 = comm.infof as u32;
                                        updateinv = true;
                                        //comm.x = wasthere.0 as f32; comm.y = wasthere.1 as f32;
                                    }
                                    SlotIndexType::InvSlot(e) => {
                                        let ud = match *self.my_uuid.read() {
                                            Some(ud) => Some(ud.clone()),
                                            None => None,
                                        };

                                        match ud {
                                            Some(ud) => {
                                                if uuid == ud {
                                                    let playerinv = &mut self.inventory.write();
                                                    let slot = &mut playerinv.inv[e as usize];

                                                    // let wasthere = slot.clone();

                                                    slot.0 = comm.rot as u32;
                                                    slot.1 = comm.infof as u32;

                                                    updateinv = true;
                                                }
                                            }
                                            None => todo!(),
                                        }

                                        //comm.x = wasthere.0 as f32; comm.y = wasthere.1 as f32;
                                    }
                                    SlotIndexType::None => {}
                                }

                                if updateinv {
                                    self.update_inventory();
                                }
                            }

                            _ => {}
                        }
                    }
                    None => {
                        morestuff = false;
                    }
                }
            }
            let morestuff = true;
            //while morestuff {

            for _ in 0..5 {
                match self.server_command_queue.pop() {
                    Some(comm) => {
                        match comm.message_type {
                            MessageType::RequestTakeoff => {
                                self.takeoff_ship();
                            }
                            MessageType::TimeUpdate => {
                                //println!("Songindex: {}", unsafe { SONGINDEX });
                                let mut todlock = self.timeofday.lock();
                                *todlock = comm.infof;
                                unsafe {
                                    WEATHERTYPE = comm.rot;
                                }
                                unsafe {
                                    let newsongindex = comm.info;

                                    if SONGINDEX as u32 != newsongindex {
                                        SONGINDEX = newsongindex as usize;

                                        #[cfg(feature = "audio")]
                                        for (name, sink) in &AUDIOPLAYER.headsinks {
                                            sink.stop();
                                        }

                                        #[cfg(feature = "audio")]
                                        AUDIOPLAYER.play_in_head(SONGS[SONGINDEX]);
                                    }
                                }
                            }
                            MessageType::BlockSet => {
                                if comm.info == 0 {
                                    self.chunksys.read().set_block_and_queue_rerender(
                                        IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                        comm.info,
                                        true,
                                        true,
                                        false,
                                    );
                                } else {
                                    self.chunksys.read().set_block_and_queue_rerender(
                                        IVec3::new(comm.x as i32, comm.y as i32, comm.z as i32),
                                        comm.info,
                                        false,
                                        true,
                                        false,
                                    );
                                }
                            }
                            MessageType::MobUpdate => {
                                //println!("Got mobupdate");
                                // println!("MobUpdate: {}", comm);
                                let newpos = Vec3::new(comm.x, comm.y, comm.z);
                                let id = comm.info;
                                let modind = comm.info2;
                                let rot = comm.rot;
                                let scale = comm.infof;
                                let sounding = comm.bo;
                                let hostile = comm.hostile;
                                // if sounding {
                                //     info!("We got a sounding message");
                                // }

                                let nsme = self.non_static_model_entities.clone();
                                //info!("Mob update. NSME Length: {}", nsme.len());
                                match nsme.get_mut(&id) {
                                    Some(mut me) => {
                                        let modent = me.value_mut();
                                        (*modent).lastpos = (*modent).position.clone();
                                        (*modent).position = newpos;
                                        (*modent).scale = scale;
                                        (*modent).lastrot = (*modent).rot.clone();
                                        (*modent).rot = Vec3::new(0.0, rot, 0.0);
                                        (*modent).sounding = sounding;
                                        (*modent).hostile = hostile;
                                        unsafe {
                                            (*modent).time_stamp = glfwGetTime();
                                        }
                                    }
                                    None => {
                                        //info!("Received an update for a mob {} that doesn't exist. Creating it...", id);
                                        self.insert_static_model_entity(
                                            id,
                                            modind as usize,
                                            newpos,
                                            scale,
                                            Vec3::new(0.0, rot, 0.0),
                                            5.0,
                                            hostile,
                                        );
                                    }
                                };
                            }
                            MessageType::PlayerUpdate => {
                                let newpos = Vec3::new(comm.x, comm.y, comm.z);
                                //let id = comm.info;
                                let modind = 0;
                                let rot = comm.rot;
                                let scale = PLAYERSCALE;
                                //let sounding  = comm.bo;

                                let pme: Arc<DashMap<Uuid, ModelEntity>> =
                                    self.player_model_entities.clone();

                                let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);
                                //info!("NSME Length: {}", nsme.len());
                                match pme.get_mut(&uuid) {
                                    Some(mut me) => {
                                        let modent = me.value_mut();
                                        (*modent).lastpos = (*modent).position.clone();
                                        (*modent).position = newpos;
                                        (*modent).scale = scale;
                                        (*modent).lastrot = (*modent).rot.clone();
                                        (*modent).rot = Vec3::new(0.0, rot, 0.0);
                                        //(*modent).sounding = sounding;
                                        unsafe {
                                            (*modent).time_stamp = glfwGetTime();
                                        }
                                    }
                                    None => {
                                        info!("Received an update for a player {} that doesn't exist. Creating it...", uuid);
                                        self.insert_player_model_entity(
                                            uuid,
                                            0, //0 for player
                                            newpos,
                                            scale,
                                            Vec3::new(0.0, rot, 0.0),
                                            5.0,
                                        );
                                    }
                                };
                            }
                            MessageType::Seed => {
                                //Means we're going to a new world
                                self.non_static_model_entities.clear();
                            }
                            _ => {}
                        }
                    }
                    None => {
                        break;
                    }
                }
            }

            //}

            for i in self.faders.write().iter_mut().enumerate() {
                if i.1.tick(self.delta_time) {
                    if i.0 == (FaderNames::FovFader as usize) {
                        self.camera.lock().update_fov(i.1.value);
                    }
                }
            }
            if ((self.controls.forward
                || self.controls.back
                || self.controls.left
                || self.controls.right)
                && unsafe { SPRINTING })
                || unsafe { FREEFALLING }
            {
                if unsafe { FREEFALLING } {
                    if !self.faders.read()[FaderNames::FovFader as usize].really {
                        self.faders.write()[FaderNames::FovFader as usize].reallyup();
                    }
                } else {
                    if !self.faders.read()[FaderNames::FovFader as usize].mode
                        || self.faders.read()[FaderNames::FovFader as usize].really
                    {
                        self.faders.write()[FaderNames::FovFader as usize].up();
                    }
                }
            } else {
                if self.faders.read()[FaderNames::FovFader as usize].mode {
                    self.faders.write()[FaderNames::FovFader as usize].down();
                }
            }
            self.draw();

            //if !self.vars.ship_taken_off {
            let camclone = self.draw_select_cube();
            //}

            
            unsafe {
                if BUILD_PREVIEW_MODE && MOUSE_ON_CUBE {
                    LIST_OF_PREVIEWED_SPOTS = Self::draw_user_build_preview(
                        unsafe { &SELECTCUBESPOT },
                        &BUILD_VOXEL_MODELS[SELECTED_BUILD],
                        &self.oldshader,
                        &camclone,
                        self.ambient_bright_mult,
                        self.vars.walkbobtimer,
                        &self.tex,
                    );
                }
                
            }
            
            
            

            self.guisys.draw_text(0);

            let mvp = camclone.mvp;

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
                        self.faders.write()[FaderNames::VisionsFader as usize].down();
                    } else {
                        self.visions_timer += self.delta_time;
                    }
                }
            }

            unsafe {
                #[cfg(feature = "audio")]
                AUDIOPLAYER.update();
            }

            //let camlock = self.camera.lock();

            let pos = camclone.position.clone();
            let dir = camclone.direction.clone();
            let right = camclone.right.clone();
            let yaw = camclone.yaw.clone();
            let pitch = camclone.pitch.clone();

            //drop(camlock);
            // unsafe {
            //     PLAYERCHUNKPOS.0.store(camchunkpos.x, Ordering::Relaxed);
            //     PLAYERCHUNKPOS.1.store(camchunkpos.y, Ordering::Relaxed);
            // }

            unsafe {
                PLAYERPOS.pos.0.store(pos.x, Ordering::Relaxed);
                PLAYERPOS.pos.1.store(pos.y, Ordering::Relaxed);
                PLAYERPOS.pos.2.store(pos.z, Ordering::Relaxed);

                PLAYERPOS.dir.0.store(dir.x, Ordering::Relaxed);
                PLAYERPOS.dir.1.store(dir.y, Ordering::Relaxed);
                PLAYERPOS.dir.2.store(dir.z, Ordering::Relaxed);

                PLAYERPOS.yaw.store(yaw, Ordering::Relaxed);
                PLAYERPOS.pitch.store(pitch, Ordering::Relaxed);
            }

            #[cfg(feature = "audio")]
            unsafe {
                AUDIOPLAYER.set_listener_attributes(pos, right);
            }
            #[cfg(feature = "audio")]
            self.do_step_sounds();

            if self.inventory.read().dirty {
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

            // let camlock = self.camera.lock();
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
                self.planet_y_offset =
                    (self.planet_y_offset + self.delta_time * planet_speed).clamp(-1000.0, 0.0);
            }
            if self.vars.ship_going_up {
                self.planet_y_offset =
                    (self.planet_y_offset - self.delta_time * planet_speed).clamp(-1000.0, 0.0);
            }
        }

        if self.initial_timer < 1.5 {
            self.initial_timer += self.delta_time;
            //println!("Initial timer: {}", self.initial_timer);
        } else {
            if self.headless {
                //println!("Headless so updating nsmes");
                self.update_non_static_model_entities();
            } else {
                if !self.vars.in_multiplayer {
                    //println!("Singleplayer so updating nsmes");
                    self.update_non_static_model_entities();
                } else {
                    //YOu are in multiplayer
                    //println!("MUltiplayer so aug updating nsmes");
                    self.update_server_received_modents();
                }
                if overlayfade <= 0.1 {
                    self.update_movement_and_physics();
                }
            }
        }

        //info!("Planet y off: {}", self.planet_y_offset);
    }

    pub fn update_movement_and_physics(&mut self) {
        static mut NUDM: Lazy<Arc<DashMap<IVec3, u32>>> = Lazy::new(|| Arc::new(DashMap::new()));
        static mut UDM: Lazy<Arc<DashMap<IVec3, u32>>> = Lazy::new(|| Arc::new(DashMap::new()));
        static mut PERL: Lazy<Arc<RwLock<Perlin>>> =
            Lazy::new(|| Arc::new(RwLock::new(Perlin::new(0))));
        static mut hasbeenset: bool = false;

        unsafe {
            let cr = self.chunksys.read();
            if !hasbeenset {
                (*NUDM) = cr.nonuserdatamap.clone();
                (*UDM) = cr.userdatamap.clone();
                (*PERL) = cr.perlin.clone();
                hasbeenset = true;
            }
        }

        static mut SPOTIFSHIFTING: Vec3 = Vec3::ZERO;
        static mut SPOTSET: bool = false;

        let camarc = self.camera.clone();
        let mut cam_clone = {
            let camlock = camarc.lock();

            let cam_clone: Camera = camlock.clone();
            cam_clone
        };

        unsafe {
            if CROUCHING {
                if !SPOTSET {
                    let y = cam_clone.position.y.round();
                    let mut hardspot = cam_clone.position.floor();
                    hardspot.y = y;
                    SPOTIFSHIFTING = hardspot + Vec3::new(0.5, 0.0, 0.5);
                    SPOTSET = true;
                }
            } else {
                SPOTSET = false;
            }
        }

        static mut wasngrounded: bool = false;

        match *self.my_uuid.read() {
            Some(uuid) => match self.known_cameras.get_mut(&uuid) {
                Some(mut pos) => {
                    *pos = cam_clone.position;
                }
                None => {}
            },
            None => {}
        }

        static mut wasinwater: bool = false;

        let vel = cam_clone.velocity.clone();

        let feetpos = cam_clone.position - Vec3::new(0.0, 1.0, 0.0);

        let underfeetpos = feetpos - Vec3::new(0.0, 1.0, 0.0);

        let feetposi = vec::IVec3::new(
            feetpos.x.floor() as i32,
            feetpos.y.floor() as i32,
            feetpos.z.floor() as i32,
        );
        let headposi = vec::IVec3::new(
            cam_clone.position.x.floor() as i32,
            cam_clone.position.y.floor() as i32,
            cam_clone.position.z.floor() as i32,
        );
        let feetposi2 = vec::IVec3::new(
            feetpos.x.floor() as i32,
            (feetpos.y - 0.25).floor() as i32,
            feetpos.z.floor() as i32,
        );

        let underfeetposi = vec::IVec3::new(
            underfeetpos.x.floor() as i32,
            underfeetpos.y.floor() as i32,
            underfeetpos.z.floor() as i32,
        );

        let blockfeetin = unsafe {
            ChunkSystem::_blockat(&NUDM, &UDM, &PERL.read(), feetposi) & Blocks::block_id_bits()
        };
        let blockfeetinlower = unsafe {
            ChunkSystem::_blockat(&NUDM, &UDM, &PERL.read(), feetposi2) & Blocks::block_id_bits()
        };
        let blockbitsunderfeet =
            unsafe { ChunkSystem::_blockat(&NUDM, &UDM, &PERL.read(), underfeetposi) };
        let blockunderfeet = blockbitsunderfeet & Blocks::block_id_bits();
        // println!("BUF: {}", blockunderfeet);

        let blockheadin = unsafe {
            ChunkSystem::_blockat(&NUDM, &UDM, &PERL.read(), headposi) & Blocks::block_id_bits()
        };

        if blockheadin == 2 {
            self.headinwater = true;
        } else {
            self.headinwater = false;
        }

        static mut wasconveyor: bool = false;

        let mut conveyor = false;

        match blockunderfeet {
            46 => unsafe {
                if !TRAMPOLINE {
                    TRAMPOLINE = true;
                    let d = cam_clone.direction;
                    cam_clone.velocity += Vec3::new(0.0, TRAMPOLINE_VELOCITY_FIGURE, 0.0) + d;
                    #[cfg(feature = "audio")]
                    AUDIOPLAYER.play(
                        path!("assets/sfx/boing.mp3"),
                        &(feetpos),
                        &Vec3::new(0.0, 0.0, 0.0),
                        0.5,
                    );
                }
            },
            45 => {
                //Conveyor

                conveyor = true;
                static DIRS: [Vec3; 4] = [
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    Vec3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    Vec3 {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ];
                let dir = Blocks::get_direction_bits(blockbitsunderfeet);
                //println!("Dir: {}", dir);

                let multiplier = 2.4;
                //println!("MUltiplier: {}", multiplier);
                {
                    let vel = (DIRS[dir as usize] * 70.0 * multiplier) * self.delta_time;
                    cam_clone.velocity.x = vel.x;
                    cam_clone.velocity.z = vel.z;
                }
            }
            _ => {}
        }

        if conveyor {
            unsafe {
                ON_CONVEYORS = true;
                if !wasconveyor {
                    CONVEYOR_SOUND_TIMER = 0.0;
                    TIME_ON_CONVEYORS = 0.0;
                }
            }
        } else {
            unsafe {
                ON_CONVEYORS = false;
            }
        }
        unsafe {
            wasconveyor = conveyor;
        }

        let feetinwater = blockfeetin == 2;
        let feetinwaterlower = blockfeetinlower == 2;

        unsafe {
            if feetinwater != wasinwater {
                #[cfg(feature = "audio")]
                if !wasinwater {
                    AUDIOPLAYER.play_next_in_series("waterstepseries", &feetpos, &vel, 0.6);
                }
                wasinwater = feetinwater;
            }
        }

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

        if !self.coll_cage.solid.contains(&Side::FLOOR) && !(unsafe { CROUCHING }) {
            self.grounded = false;
            unsafe {
                wasngrounded = true;
            }
        } else {
        }

        const GRAV: f32 = 9.8;

        if self.inwater || self.vars.in_climbable {
            unsafe {
                if WASFREEFALLING {
                    FREEFALLING = false;
                    WASFREEFALLING = false;
                    self.vars.time_tfs_at_3 = 0.0;
                    self.time_falling_scalar = 1.0;

                    #[cfg(feature = "audio")] // TODO: why alloc?
                    AUDIOPLAYER.stop_head_sound(path!("assets/sfx/freefall.mp3").to_string());
                    if self.inwater {
                        #[cfg(feature = "audio")]
                        AUDIOPLAYER.play_in_head(path!("assets/sfx/splash.mp3"));
                    }
                }
            }
            self.time_falling_scalar = 1.0;
            if !self.grounded {
                cam_clone.velocity += Vec3::new(0.0, -2.0 * self.delta_time, 0.0);
                if unsafe { CROUCHING } {
                    cam_clone.velocity += Vec3::new(0.0, -5.0 * self.delta_time, 0.0);
                }
            }

            if self.controls.up {
                let amount = unsafe {
                    if SPRINTING {
                        12.0
                    } else {
                        7.0
                    }
                };
                cam_clone.velocity += Vec3::new(0.0, amount * self.delta_time, 0.0);
            }
        } else {
            if !self.grounded && !self.jumping_up {
                self.time_falling_scalar =
                    (self.time_falling_scalar + self.delta_time * 3.0).min(3.0);

                if self.time_falling_scalar >= 3.0 {
                    unsafe {
                        FREEFALLING = true;
                    }
                    unsafe {
                        if !WASFREEFALLING {
                            WASFREEFALLING = true;
                            #[cfg(feature = "audio")]
                            AUDIOPLAYER.play_in_head(path!("assets/sfx/freefall.mp3"));
                        }
                    }

                    self.vars.time_tfs_at_3 += self.delta_time;
                } else {
                    self.vars.time_tfs_at_3 = 0.0;
                }
                //println!("Time falscal: {}", self.time_falling_scalar);
            } else {
                self.time_falling_scalar = 1.0;
                unsafe {
                    FREEFALLING = false;
                    if WASFREEFALLING {
                        WASFREEFALLING = false;
                        #[cfg(feature = "audio")]
                        AUDIOPLAYER.stop_head_sound(path!("assets/sfx/freefall.mp3").to_string());
                    }
                }
            }

            if !self.grounded && !self.jumping_up {
                cam_clone.velocity +=
                    Vec3::new(0.0, -GRAV * self.time_falling_scalar * self.delta_time, 0.0);
            }

            if self.jumping_up {
                if cam_clone.position.y < self.current_jump_y + self.allowable_jump_height {
                    let curr_cam_y = cam_clone.position.y;
                    cam_clone.velocity += Vec3::new(
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
                self.current_jump_y = cam_clone.position.y;
                self.jumping_up = true;
                self.controls.up = false;
            }
        }

        let mut proposed = {
            let mut camlock = self.camera.lock();

            (*camlock) = cam_clone;

            let proposed = unsafe {
                if CROUCHING && self.grounded {
                    camlock.respond_to_controls(&self.controls, &self.delta_time, 1.5)
                } else {
                    camlock.respond_to_controls(&self.controls, &self.delta_time, 5.5)
                }
            };

            cam_clone = (*camlock).clone();
            proposed
        };
        unsafe {
            if CROUCHING {
                fn cap_distance(proposed: Vec3, reference: Vec3, max_distance: f32) -> Vec3 {
                    let direction = proposed - reference;
                    let distance = direction.length();

                    if distance > max_distance {
                        let v3 = reference + direction.normalize() * max_distance;
                        Vec3::new(v3.x, proposed.y, v3.z)
                    } else {
                        proposed
                    }
                }

                let prop2 = Vec3::new(proposed.x, 0.0, proposed.z);
                let spotshift = Vec3::new(SPOTIFSHIFTING.x, 0.0, SPOTIFSHIFTING.z);

                if proposed.distance(SPOTIFSHIFTING) > 1.0 {
                    proposed = cap_distance(proposed, SPOTIFSHIFTING, 1.0);
                }
            }
        }

        self.user_bound_box
            .set_center(proposed + Vec3::new(0.0, -0.5, 0.0), 0.2, 0.95);
        self.coll_cage.update_colliding(&self.user_bound_box);

        let mut corr_made: Vec<Vec3> = Vec::new();

        let mut stepsoundqueued = false;
        let mut activate_jump_queued = false;
        let mut falldamage = None;

        if self.coll_cage.colliding.len() > 0 {
            for side in &self.coll_cage.colliding {
                if !corr_made.contains(&self.coll_cage.normals[*side as usize]) {
                    proposed += self.coll_cage.normals[*side as usize]
                        * self.coll_cage.penetrations[*side as usize];
                    corr_made.push(self.coll_cage.normals[*side as usize]);
                }
                if *side == Side::FLOOR {
                    self.grounded = true;
                    unsafe {
                        if wasngrounded {
                            if self.vars.time_tfs_at_3 > 0.0 {
                                falldamage = Some(self.vars.time_tfs_at_3);
                            }

                            self.vars.time_tfs_at_3 = 0.0;
                            activate_jump_queued = true;
                            stepsoundqueued = true;
                            wasngrounded = false;
                        }
                    }
                }
                if *side == Side::ROOF {
                    self.jumping_up = false;
                    self.grounded = false;
                }
            }
        }

        cam_clone.position = Vec3::new(proposed.x, proposed.y, proposed.z);

        let cc_center = cam_clone.position + Vec3::new(0.0, -1.0, 0.0);
        self.coll_cage.update_readings(cc_center);

        //let offset = self.coll_cage.get_smoothed_floor_y(camlock.position);

        //camlock.position.y = offset;

        cam_clone.recalculate();

        let pos = cam_clone.position.clone();

        {
            let mut camlock = self.camera.lock();
            *camlock = cam_clone;
        }

        #[cfg(feature = "audio")]
        if stepsoundqueued {
            self.do_step_sound_now(pos);
        }

        if activate_jump_queued {
            self.activate_jump_block(pos);
        }

        match falldamage {
            Some(fd) => {
                unsafe {
                    #[cfg(feature = "audio")]
                    AUDIOPLAYER.play_in_head(path!("assets/sfx/falldamage.mp3"));
                }
                self.take_damage((fd * 20.0) as u8);
            }
            None => {}
        }
    }

    pub fn take_damage(&mut self, amount: u8) {
        let h = self.health.load(std::sync::atomic::Ordering::Relaxed);
        let newamount = (h - amount as i8).max(0);
        self.health
            .store(newamount, std::sync::atomic::Ordering::Relaxed);
        if newamount <= 0 {
            //DEAD

            unsafe {
                #[cfg(feature = "audio")]
                AUDIOPLAYER.play_in_head(path!("assets/sfx/death.mp3"));
            }
            let mut camlock = self.camera.lock();
            let campos = camlock.position.clone();

            let mut inv = self.inventory.write();
            for i in 0..ROWLENGTH {
                let amt = inv.inv[i as usize].1;
                #[cfg(feature = "glfw")]
                self.drops.add_drop(
                    campos + Vec3::new(0.0, 2.0, 0.0),
                    inv.inv[i as usize].0,
                    amt,
                );
            }
            inv.inv = STARTINGITEMS;

            unsafe {
                camlock.position = SPAWNPOINT;
                camlock.velocity = Vec3::ZERO;
            }

            drop(camlock);
            self.health.store(20, std::sync::atomic::Ordering::Relaxed);
        }
    }

    #[cfg(feature = "glfw")]
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

            let camlock = self.camera.lock();
            let c = camlock.clone();
            drop(camlock);
            let cam_clone = c;
            gl::Uniform1f(C_P_LOC, pitch);
            gl::Uniform3f(
                C_D_LOC,
                cam_clone.direction.x,
                cam_clone.direction.y,
                cam_clone.direction.z,
            );
            drop(cam_clone);

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
    #[cfg(feature = "glfw")]
    pub fn draw_select_cube(&mut self) -> Camera {
        static mut LAST_CAM_POS: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        static mut LAST_CAM_DIR: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        static mut LAST_BLOCK_POS: IVec3 = IVec3 { x: 0, y: 0, z: 0 };

        static mut HIT_RESULT: Option<(Vec3, IVec3)> = None;

        static mut BLOCK_TYPE: u32 = 0;

        static mut BLOCK_MATERIAL: Material = Material::Dirt;

        static mut BREAK_TIME: f32 = 0.0;

        let cam_clone = {
            let c = self.camera.lock();
            c.clone()
        };

        unsafe {
            if cam_clone.position != LAST_CAM_POS
                || cam_clone.direction != LAST_CAM_DIR
                || UPDATE_THE_BLOCK_OVERLAY
            {
                UPDATE_THE_BLOCK_OVERLAY = false;

                LAST_CAM_POS = cam_clone.position;
                LAST_CAM_DIR = cam_clone.direction;

                HIT_RESULT = raycast_voxel_with_bob(
                    cam_clone.position,
                    cam_clone.direction,
                    &self.chunksys,
                    VOXEL_SELECT_DISTANCE,
                    self.vars.walkbobtimer,
                );

                BLOCK_TYPE = match HIT_RESULT {
                    Some((_head, hit)) => {
                        unsafe {
                            MOUSE_ON_CUBE = true;
                        }
                        if LAST_BLOCK_POS != hit {
                            BREAK_TIME = 0.0;
                            LAST_BLOCK_POS = hit;
                        }
                        self.chunksys.read().blockat(hit) & Blocks::block_id_bits()
                    }
                    None => {
                        unsafe {
                            MOUSE_ON_CUBE = false;
                        }
                        
                        0
                    },
                };

                BLOCK_MATERIAL = get_block_material(BLOCK_TYPE);
            }

            match HIT_RESULT {
                Some((_head, hit)) => {
                    unsafe {
                        static mut LASTSPOT: IVec3 = IVec3::new(0,0,0);

                        SELECTCUBESPOT = hit;

                        if SELECTCUBESPOT != LASTSPOT {

                            BUILD_MODEL_OFFSET = IVec2::new(0, 0);

                            LASTSPOT = SELECTCUBESPOT;
                        }
                    }
                    let hitvec3 = Vec3::new(hit.x as f32, hit.y as f32, hit.z as f32);
                    self.select_cube
                        .draw_at(hitvec3, &cam_clone.mvp, self.vars.walkbobtimer);
                    let bprog = (BREAK_TIME / Blocks::get_break_time(BLOCK_TYPE)).clamp(0.0, 1.0);

                    let slot_selected = self.hud.bumped_slot;
                    let slot = {
                        let b = self.inventory.read().inv[slot_selected];
                        b.clone()
                    };

                    let tooltype = get_tools_target_material(slot.0);

                    let tool_is_for_this_material = tooltype == BLOCK_MATERIAL;

                    let mut modifier = 1.0;
                    if tool_is_for_this_material {
                        modifier = 4.0;
                    }

                    if self.vars.mouse_clicked && !self.crafting_open && !self.vars.menu_open {
                        self.block_overlay.draw_at(
                            hitvec3,
                            (bprog * 8.0).floor() as i8,
                            &cam_clone.mvp,
                            self.vars.walkbobtimer,
                        );
                        BREAK_TIME = BREAK_TIME + self.delta_time * modifier;
                        if bprog >= 1.0 {
                            if !self.vars.ship_taken_off {
                                self.cast_break_ray();
                                //UPDATE_THE_OVERLAY = true;
                            }
                            BREAK_TIME = 0.0;
                        }
                    }
                    //println!("DISWD");
                }
                None => {}
            }
        }
        {
            let mut c = self.camera.lock();
            (*c) = cam_clone.clone();
        }
        cam_clone
    }

    #[cfg(feature = "glfw")]
    pub fn draw(&self) {
        let campitch = self.camera.lock().pitch;

        //Sky
        #[cfg(feature = "glfw")]
        match self.vars.hostile_world {
            true => {
                self.draw_sky(
                    self.vars.hostile_world_sky_color,
                    self.vars.hostile_world_sky_bottom,
                    self.ambient_bright_mult,
                    campitch,
                );
            }
            false => {
                self.draw_sky(
                    self.vars.sky_color,
                    self.vars.sky_bottom,
                    self.ambient_bright_mult,
                    campitch,
                );
            }
        }

        //Chunks
        unsafe {
            gl::BindVertexArray(self.shader0.vao);
            gl::UseProgram(self.shader0.shader_id);
        }

        if true {
            //unsafe { GLCHUNKS } {

            let ugqarc = self.chunksys.read().finished_user_geo_queue.clone();

            match ugqarc.pop() {
                Some(ready) => {
                    //info!("Some user queue");
                    // info!("Weird!");

                    let bankarc = self.chunksys.read().geobank[ready.geo_index].clone();

                    let cs = self.chunksys.read();

                    let mut cmemlock = cs.chunk_memories.lock();

                    cmemlock.memories[ready.geo_index].length = ready.newlength;
                    cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
                    cmemlock.memories[ready.geo_index].vlength = ready.newvlength;
                    cmemlock.memories[ready.geo_index].wvlength = ready.newwvlength;
                    cmemlock.memories[ready.geo_index].pos = ready.newpos;
                    cmemlock.memories[ready.geo_index].used = true;

                    //info!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
                    //info!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
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

                    let wvv = cmemlock.memories[ready.geo_index].wvvbo;
                    let wuvv = cmemlock.memories[ready.geo_index].wuvvbo;

                    let vbo8rgb = cmemlock.memories[ready.geo_index].vbo8rgb;
                    let tvbo8rgb = cmemlock.memories[ready.geo_index].tvbo8rgb;

                    WorldGeometry::bind_geometry(
                        v32,
                        v8,
                        vbo8rgb,
                        true,
                        &self.shader0,
                        bankarc.solids(),
                    );
                    WorldGeometry::bind_geometry(
                        tv32,
                        tv8,
                        tvbo8rgb,
                        true,
                        &self.shader0,
                        bankarc.transparents(),
                    );

                    WorldGeometry::bind_old_geometry(
                        vv,
                        uvv,
                        &bankarc.vdata.lock(),
                        &bankarc.uvdata.lock(),
                        &self.oldshader,
                    );
                    WorldGeometry::bind_old_geometry(
                        wvv,
                        wuvv,
                        &bankarc.wvdata.lock(),
                        &bankarc.wuvdata.lock(),
                        &self.oldshader,
                    );
                }
                None => {}
            }

            let gqarc = self.chunksys.read().finished_geo_queue.clone();

            match gqarc.pop() {
                Some(ready) => {
                    //info!("Weird!");

                    let bankarc = self.chunksys.read().geobank[ready.geo_index].clone();

                    let cs = self.chunksys.read();

                    let mut cmemlock = cs.chunk_memories.lock();

                    cmemlock.memories[ready.geo_index].length = ready.newlength;
                    cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
                    cmemlock.memories[ready.geo_index].vlength = ready.newvlength;
                    cmemlock.memories[ready.geo_index].wvlength = ready.newwvlength;
                    cmemlock.memories[ready.geo_index].pos = ready.newpos;
                    cmemlock.memories[ready.geo_index].used = true;

                    //info!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
                    //info!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
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

                    let wvv = cmemlock.memories[ready.geo_index].wvvbo;
                    let wuvv = cmemlock.memories[ready.geo_index].wuvvbo;

                    let vbo8rgb = cmemlock.memories[ready.geo_index].vbo8rgb;
                    let tvbo8rgb = cmemlock.memories[ready.geo_index].tvbo8rgb;

                    WorldGeometry::bind_geometry(
                        v32,
                        v8,
                        vbo8rgb,
                        true,
                        &self.shader0,
                        bankarc.solids(),
                    );
                    WorldGeometry::bind_geometry(
                        tv32,
                        tv8,
                        tvbo8rgb,
                        true,
                        &self.shader0,
                        bankarc.transparents(),
                    );

                    WorldGeometry::bind_old_geometry(
                        vv,
                        uvv,
                        &bankarc.vdata.lock(),
                        &bankarc.uvdata.lock(),
                        &self.oldshader,
                    );
                    WorldGeometry::bind_old_geometry(
                        wvv,
                        wuvv,
                        &bankarc.wvdata.lock(),
                        &bankarc.wuvdata.lock(),
                        &self.oldshader,
                    );

                    let mut userstuff = true;
                    while userstuff {
                        match ugqarc.pop() {
                            Some(ready) => {
                                //info!("Some user queue");
                                // info!("Weird!");

                                let bankarc = self.chunksys.read().geobank[ready.geo_index].clone();

                                //let mut cmemlock = self.chunksys.chunk_memories.lock();

                                cmemlock.memories[ready.geo_index].length = ready.newlength;
                                cmemlock.memories[ready.geo_index].tlength = ready.newtlength;
                                cmemlock.memories[ready.geo_index].vlength = ready.newvlength;
                                cmemlock.memories[ready.geo_index].wvlength = ready.newwvlength;
                                cmemlock.memories[ready.geo_index].pos = ready.newpos;
                                cmemlock.memories[ready.geo_index].used = true;

                                //info!("Received update to {} {} {} {}", ready.newlength, ready.newtlength, ready.newpos.x, ready.newpos.y);
                                //info!("New cmemlock values: {} {} {} {} {}", cmemlock.memories[ready.geo_index].length, cmemlock.memories[ready.geo_index].tlength, cmemlock.memories[ready.geo_index].pos.x, cmemlock.memories[ready.geo_index].pos.y, cmemlock.memories[ready.geo_index].used);
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

                                let wvv = cmemlock.memories[ready.geo_index].wvvbo;
                                let wuvv = cmemlock.memories[ready.geo_index].wuvvbo;

                                let vbo8rgb = cmemlock.memories[ready.geo_index].vbo8rgb;
                                let tvbo8rgb = cmemlock.memories[ready.geo_index].tvbo8rgb;

                                WorldGeometry::bind_geometry(
                                    v32,
                                    v8,
                                    vbo8rgb,
                                    true,
                                    &self.shader0,
                                    bankarc.solids(),
                                );
                                WorldGeometry::bind_geometry(
                                    tv32,
                                    tv8,
                                    tvbo8rgb,
                                    true,
                                    &self.shader0,
                                    bankarc.transparents(),
                                );

                                WorldGeometry::bind_old_geometry(
                                    vv,
                                    uvv,
                                    &bankarc.vdata.lock(),
                                    &bankarc.uvdata.lock(),
                                    &self.oldshader,
                                );
                                WorldGeometry::bind_old_geometry(
                                    wvv,
                                    wuvv,
                                    &bankarc.wvdata.lock(),
                                    &bankarc.wuvdata.lock(),
                                    &self.oldshader,
                                );
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

        let cam_clone = {
            let cam_lock = self.camera.lock();
            cam_lock.clone()
        };

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
        static mut WALKBOB_LOC: i32 = 0;
        unsafe {
            if C_POS_LOC == -1 {
                C_POS_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"chunkpos\0".as_ptr() as *const i8,
                );

                WALKBOB_LOC = gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"walkbob\0".as_ptr() as *const i8,
                );
                MVP_LOC =
                    gl::GetUniformLocation(self.shader0.shader_id, b"mvp\0".as_ptr() as *const i8);
                //info!("MVP LOC: {}", MVP_LOC);
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

            gl::UniformMatrix4fv(
                MVP_LOC,
                1,
                gl::FALSE,
                cam_clone.mvp.to_cols_array().as_ptr(),
            );
            gl::Uniform3f(
                CAM_POS_LOC,
                cam_clone.position.x,
                cam_clone.position.y,
                cam_clone.position.z,
            );
            gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, self.ambient_bright_mult);
            gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
            gl::Uniform1f(UNDERWATER_LOC, if self.headinwater { 1.0 } else { 0.0 });
            gl::Uniform1f(WALKBOB_LOC, self.vars.walkbobtimer);
            gl::Uniform3f(
                CAM_DIR_LOC,
                cam_clone.direction.x,
                cam_clone.direction.y,
                cam_clone.direction.z,
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
            let fc = Planets::get_fog_col(self.chunksys.read().planet_type as u32);
            gl::Uniform4f(FOGCOL_LOC, fc.0, fc.1, fc.2, fc.3);
        }

        let cs = self.chunksys.read();
        let cmem = cs.chunk_memories.lock();
        for (_index, cfl) in cmem.memories.iter().enumerate() {
            if cfl.used {
                let dd1: Mutex<Vec<u32>> = Mutex::new(Vec::new());
                let dd2: Mutex<Vec<u8>> = Mutex::new(Vec::new());
                let dd3: Mutex<Vec<u16>> = Mutex::new(Vec::new());
                let dd: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>, &Mutex<Vec<u16>>) = (&dd1, &dd2, &dd3);

                WorldGeometry::bind_geometry(
                    cfl.vbo32,
                    cfl.vbo8,
                    cfl.vbo8rgb,
                    false,
                    &self.shader0,
                    dd,
                );
                unsafe {
                    gl::Uniform2f(C_POS_LOC, cfl.pos.x as f32, cfl.pos.y as f32);

                    unsafe {
                        if !CHUNKDRAWINGHERE.contains_key(&cfl.pos) {
                            CHUNKDRAWINGHERE.insert(cfl.pos, Instant::now());
                        }

                        match CHUNKDRAWINGHERE.get(&cfl.pos) {
                            Some(instant) => {
                                let instant = instant.value();
                                let timeelapsed = instant.elapsed();
                                gl::Uniform1f(gl::GetUniformLocation(
                                    self.shader0.shader_id,
                                    b"elapsedFade\0".as_ptr() as *const i8,
                                ), timeelapsed.as_secs_f32().min(CHUNKFADEINTIME) * CHUNKFADEIN_TIMEMULTIPLIER_TOGET1_WHENITSFULL);
                            },
                            None => {
                                gl::Uniform1f(gl::GetUniformLocation(
                                    self.shader0.shader_id,
                                    b"elapsedFade\0".as_ptr() as *const i8,
                                ), 0.0);
                            },
                        }
                    }
                    
                    

                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        info!("OpenGL Error after uniforming the chunk pos: {}", error);
                    }
                    //info!("Rendering {} in chunk at {}, {}", banklock.data32.len(), banklock.pos.x, banklock.pos.y);
                    gl::DrawArrays(gl::TRIANGLES, 0, cfl.length as i32);
                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        info!("OpenGL Error after drawing arrays: {}", error);
                    }
                    // info!("Chunk rending!");
                }
            }
        }
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
        self.draw_models();

        for (_index, cfl) in cmem.memories.iter().enumerate() {
            if cfl.used {
                let dd1: Mutex<Vec<u32>> = Mutex::new(Vec::new());
                let dd2: Mutex<Vec<u8>> = Mutex::new(Vec::new());
                let dd3: Mutex<Vec<u16>> = Mutex::new(Vec::new());
                let dd: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>, &Mutex<Vec<u16>>) = (&dd1, &dd2, &dd3);

                unsafe {
                    gl::BindVertexArray(self.shader0.vao);
                    gl::UseProgram(self.shader0.shader_id);
                }

                WorldGeometry::bind_geometry(
                    cfl.tvbo32,
                    cfl.tvbo8,
                    cfl.tvbo8rgb,
                    false,
                    &self.shader0,
                    dd,
                );
                unsafe {
                    gl::Uniform2f(C_POS_LOC, cfl.pos.x as f32, cfl.pos.y as f32);

                    unsafe {
                        match CHUNKDRAWINGHERE.get(&cfl.pos) {
                            Some(instant) => {
                                let instant = instant.value();
                                let timeelapsed = instant.elapsed();
                                gl::Uniform1f(gl::GetUniformLocation(
                                    self.shader0.shader_id,
                                    b"elapsedFade\0".as_ptr() as *const i8,
                                ), timeelapsed.as_secs_f32().min(CHUNKFADEINTIME) * CHUNKFADEIN_TIMEMULTIPLIER_TOGET1_WHENITSFULL);
                            },
                            None => {
                                gl::Uniform1f(gl::GetUniformLocation(
                                    self.shader0.shader_id,
                                    b"elapsedFade\0".as_ptr() as *const i8,
                                ), 0.0);
                            },
                        }
                    }

                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        info!("OpenGL Error after uniforming the chunk pos: {}", error);
                    }
                    //info!("Rendering {} in chunk at {}, {}", banklock.data32.len(), banklock.pos.x, banklock.pos.y);
                    gl::DrawArrays(gl::TRIANGLES, 0, cfl.tlength as i32);
                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        info!("OpenGL Error after drawing arrays: {}", error);
                    }
                    // info!("Chunk rending!");
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
                static mut WALKBOB_LOC: i32 = 0;
                unsafe {
                    if MVP_LOC == -1 {
                        MVP_LOC = gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"mvp\0".as_ptr() as *const i8,
                        );
                        //info!("MVP LOC: {}", MVP_LOC);

                        WALKBOB_LOC = gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"walkbob\0".as_ptr() as *const i8,
                        );

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

                    gl::UniformMatrix4fv(
                        MVP_LOC,
                        1,
                        gl::FALSE,
                        cam_clone.mvp.to_cols_array().as_ptr(),
                    );
                    gl::Uniform3f(
                        CAM_POS_LOC,
                        cam_clone.position.x,
                        cam_clone.position.y,
                        cam_clone.position.z,
                    );
                    gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, self.ambient_bright_mult);
                    gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
                    gl::Uniform1f(UNDERWATER_LOC, 0.0);
                    gl::Uniform3f(
                        CAM_DIR_LOC,
                        cam_clone.direction.x,
                        cam_clone.direction.y,
                        cam_clone.direction.z,
                    );

                    gl::Uniform1f(
                        gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"time\0".as_ptr() as *const i8,
                        ),
                        glfwGetTime() as f32,
                    );
                    gl::Uniform1f(
                        gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"weathertype\0".as_ptr() as *const i8,
                        ),
                        WEATHERTYPE,
                    );

                    gl::Uniform1f(SUNSET_LOC, self.sunset_factor);
                    gl::Uniform1f(WALKBOB_LOC, self.vars.walkbobtimer);
                    gl::Uniform1f(SUNRISE_LOC, self.sunrise_factor);
                    gl::Uniform1i(
                        gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"ourTexture\0".as_ptr() as *const i8,
                        ),
                        0,
                    );
                    gl::Uniform1i(
                        gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"weatherTexture\0".as_ptr() as *const i8,
                        ),
                        2,
                    );
                    // let fc = Planets::get_fog_col(self.chunksys.read().planet_type as u32);
                    // gl::Uniform4f(
                    //     FOGCOL_LOC,
                    //     fc.0,
                    //     fc.1,
                    //     fc.2,
                    //     fc.3
                    // );
                }

                unsafe {
                    gl::Uniform1f(
                        gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"renderingweather\0".as_ptr() as *const i8,
                        ),
                        0.0,
                    );

                    gl::Uniform3f(
                        gl::GetUniformLocation(
                            self.oldshader.shader_id,
                            b"transformpos\0".as_ptr() as *const i8,
                        ),
                        0.0,
                        0.0,
                        0.0,
                    );
                }

                WorldGeometry::bind_old_geometry_no_upload(cfl.vvbo, cfl.uvvbo, &self.oldshader);

                unsafe {
                    //gl::Disable(gl::CULL_FACE);
                    gl::DrawArrays(gl::TRIANGLES, 0, cfl.vlength as i32 / 5);
                    let error = gl::GetError();
                    if error != gl::NO_ERROR {
                        info!("OpenGL Error after drawing arrays: {}", error);
                    }
                    //gl::Enable(gl::CULL_FACE);
                    // info!("Chunk rending!");
                }

                if unsafe { WEATHERTYPE } != 0.0 {
                    WorldGeometry::bind_old_geometry_no_upload(
                        cfl.wvvbo,
                        cfl.wuvvbo,
                        &self.oldshader,
                    );
                    unsafe {
                        gl::Uniform3f(
                            gl::GetUniformLocation(
                                self.oldshader.shader_id,
                                b"transformpos\0".as_ptr() as *const i8,
                            ),
                            0.0,
                            0.0,
                            0.0,
                        );
                        gl::Uniform1f(
                            gl::GetUniformLocation(
                                self.oldshader.shader_id,
                                b"renderingweather\0".as_ptr() as *const i8,
                            ),
                            1.0,
                        );
                    }

                    unsafe {
                        //gl::Disable(gl::CULL_FACE);
                        gl::DrawArrays(gl::TRIANGLES, 0, cfl.wvlength as i32 / 5);
                        let error = gl::GetError();
                        if error != gl::NO_ERROR {
                            info!("OpenGL Error after drawing arrays: {}", error);
                        }
                        //gl::Enable(gl::CULL_FACE);
                        // info!("Chunk rending!");
                    }
                }
            }
        }
        #[cfg(feature = "glfw")]
        self.draw_stars();
        // #[cfg(feature = "glfw")]
        // self.draw_clouds();
    }

    pub fn start_world(&mut self) {
        (*self.run_chunk_thread).store(true, Ordering::Relaxed);
        self.initial_timer = 0.0;

        let rctarc = self.run_chunk_thread.clone();
        let carc = self.camera.clone();
        let csysarc = self.chunksys.clone();

        //Uncomment to do automata (just snow updating grass simulation for now)
        //csysarc.write().do_automata(&carc);

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
            info!("Thread joined successfully!");
        } else {
            info!("No thread to join or already joined.");
        }
        #[cfg(feature = "glfw")]
        self.drops.drops.clear();

        self.non_static_model_entities.clear();
        self.chunksys.write().exit();
    }

    pub fn start_chunks_with_radius(&mut self, newradius: u8, seed: u32, nt: usize) {
        (*self.run_chunk_thread).store(false, Ordering::Relaxed);

        if let Some(handle) = self.chunk_thread.take() {
            handle.join().unwrap();
            info!("Thread joined successfully!");
        } else {
            info!("No thread to join or already joined.");
        }

        #[cfg(feature = "glfw")]
        self.drops.drops.clear();
        self.non_static_model_entities.clear();

        self.chunksys.write().reset(newradius, seed, nt);

        self.chunksys.write().voxel_models = Some(self.voxel_models.clone());

        //self.drops.csys = self.chunksys.clone();

        let mut rng = StdRng::from_entropy();

        if !self.vars.in_multiplayer {
            if nt == 1 {
                self.create_non_static_model_entity(
                    0,
                    Vec3::new(-100.0, 100.0, 350.0),
                    5.0,
                    Vec3::new(0.0, 0.0, 0.0),
                    7.0,
                    false,
                );

                for _i in 0..4 {
                    if rng.gen_range(0..3) <= 2 {
                        self.create_non_static_model_entity(
                            2,
                            Vec3::new(
                                rng.gen_range(-200.0..200.0),
                                80.0,
                                rng.gen_range(-200.0..200.0),
                            ),
                            5.0,
                            Vec3::new(0.0, 0.0, 0.0),
                            7.0,
                            false,
                        );
                        self.create_non_static_model_entity(
                            2,
                            Vec3::new(
                                rng.gen_range(-200.0..200.0),
                                80.0,
                                rng.gen_range(-200.0..200.0),
                            ),
                            5.0,
                            Vec3::new(0.0, 0.0, 0.0),
                            7.0,
                            false,
                        );

                        self.create_non_static_model_entity(
                            3,
                            Vec3::new(
                                rng.gen_range(-200.0..200.0),
                                80.0,
                                rng.gen_range(-200.0..200.0),
                            ),
                            5.0,
                            Vec3::new(0.0, 0.0, 0.0),
                            3.0,
                            false,
                        );
                        self.create_non_static_model_entity(
                            3,
                            Vec3::new(
                                rng.gen_range(-200.0..200.0),
                                80.0,
                                rng.gen_range(-200.0..200.0),
                            ),
                            5.0,
                            Vec3::new(0.0, 0.0, 0.0),
                            3.0,
                            false,
                        );
                    }
                }
            }
        }

        self.initialize_being_in_world();

        self.start_world();
    }

    pub fn rebuild_whole_world_while_showing_loading_screen(
        &mut self,
    ) -> std::thread::JoinHandle<()> {
        // let _csys = self.chunksys.clone();
        // let _campos = self.camera.lock().position.clone();
        // let _shader = self.shader0.clone();

        let threadhandle = thread::spawn(move || {
            //ChunkSystem::initial_rebuild_on_main_thread(&csys, &shader, &campos)
        });

        threadhandle

        // while !threadhandle.is_finished() {

        //     //self.draw();
        //     self.window.read()
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

    pub fn chunk_thread_inner_function(
        cam_arc: &Arc<Mutex<Camera>>,
        csys_arc: &Arc<RwLock<ChunkSystem>>,
        last_user_c_pos: &mut vec::IVec2,
    ) {
        //info!("Starting over the CTIF");
        let _rng = StdRng::from_entropy();

        // let mut lightcheckstuff = true;

        // while lightcheckstuff {
        //     match check_for_intercepting.pop() {
        //         Some(spot) => {
        //             let mut implicated = HashSet::new();

        //             let mut lightraylistlist = Vec::new();

        //             match csys_arc.read() {
        //                 Ok(csys) => match csys.lightmap.lock() {
        //                     Ok(lightmap) => {
        //                         match lightmap.get(&spot) {
        //                             Some(lightseg) => {
        //                                 lightraylistlist.push(lightseg.rays.clone());
        //                             }
        //                             None => {}
        //                         };
        //                     }
        //                     Err(_) => {}
        //                 },
        //                 Err(_) => {}
        //             }

        //             for raylist in lightraylistlist {
        //                 for ray in &raylist {
        //                     let chunkofthisraysorigin = ChunkSystem::spot_to_chunk_pos(&ray.origin);
        //                     // match self.takencare.get(&chunkofthisraysorigin) {
        //                     //     Some(chunk) => {
        //                     //         implicated.insert(chunk.geo_index);
        //                     //     }
        //                     //     None => {

        //                     //     }
        //                     // }
        //                     implicated.insert(chunkofthisraysorigin);
        //                 }
        //             }

        //             //TEMPORARILY DISABLED UNTIL WE CAN DO A LIGHT UPDATE WITHOUT STUTTERING THE MAIN FUCKING THREAD DUMBASS
        //             //let c = csys_arc.read();
        //             // for i in implicated {
        //             //     QUEUE_THESE.push(i);
        //             //     //c.queue_rerender_with_key(i, true, true);
        //             // }
        //         }
        //         None => {
        //             lightcheckstuff = false;
        //         }
        //     }
        // }

        let mut lightstuff = true;
        while lightstuff {
            let csys_arc = csys_arc.read();

            match csys_arc.light_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, true, true);
                    //info!("Popping stuff LIGHT {}", rng.gen_range(0..255));
                }
                None => {
                    lightstuff = false;
                }
            }
        }

        let mut userstuff = true;
        while userstuff {
            let csys_arc = csys_arc.read();

            match csys_arc.user_rebuild_requests.pop() {
                Some(index) => {
                    //info!("Popping stuff USER {}", rng.gen_range(0..255));
                    csys_arc.rebuild_index(index, true, false);

                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            //info!("Popping stuff LIGHT {}", rng.gen_range(0..255));
                            csys_arc.rebuild_index(index, true, true);
                        }
                        None => {}
                    }
                }
                None => {
                    userstuff = false;
                }
            }
        }
        let mut genstuff = true;
        while genstuff {
            let csys_arc = csys_arc.read();

            match csys_arc.gen_rebuild_requests.pop() {
                Some(index) => {
                    csys_arc.rebuild_index(index, true, false);
                    // info!("Popping stuff GEN {}", rng.gen_range(0..255));
                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            // info!("Popping stuff USER {}", rng.gen_range(0..255));
                            csys_arc.rebuild_index(index, true, false);
                        }
                        None => {}
                    }
                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            // info!("Popping stuff LIGHT {}", rng.gen_range(0..255));
                            csys_arc.rebuild_index(index, true, true);
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
            unsafe {
                match AUTOMATA_QUEUED_CHANGES.pop() {
                    Some(comm) => {
                        println!("Poppin one");
                        let csys_arc = csys_arc.read();

                        if (csys_arc.blockat(comm.spot) & Blocks::block_id_bits())
                            == comm.expectedhere
                        {
                            println!("Settin");
                            csys_arc.set_block(comm.spot, comm.changeto, false);
                            csys_arc.queue_rerender_with_key(
                                ChunkSystem::spot_to_chunk_pos(&comm.spot),
                                false,
                                false,
                            );
                            //csys_arc.rebuild_index(comm.geo_index, false, false);
                        } else {
                            println!(
                                "Expected {} here but its {} for this change",
                                comm.expectedhere,
                                (csys_arc.blockat(comm.spot) & Blocks::block_id_bits())
                            );
                        }
                    }
                    None => {}
                }
            }
            let csys_arc = csys_arc.read();

            match csys_arc.background_rebuild_requests.pop() {
                Some(index) => {
                    // info!("Popping stuff BACKGROUND {}", rng.gen_range(0..255));
                    csys_arc.rebuild_index(index, false, false);

                    match csys_arc.user_rebuild_requests.pop() {
                        Some(index) => {
                            // info!("Popping stuff USER {}", rng.gen_range(0..255));
                            csys_arc.rebuild_index(index, true, false);
                            let _userstuff = true;
                        }
                        None => {}
                    }

                    match csys_arc.light_rebuild_requests.pop() {
                        Some(index) => {
                            // info!("Popping stuff LIGHT {}", rng.gen_range(0..255));
                            csys_arc.rebuild_index(index, true, true);
                        }
                        None => {}
                    }

                    match csys_arc.gen_rebuild_requests.pop() {
                        Some(index) => {
                            // info!("Popping stuff GEN {}", rng.gen_range(0..255));
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

        let camlock = cam_arc.lock();
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

            if
            /*user_c_pos != *last_user_c_pos &&*/
            time_since_last_check >= 5.0 {
                *last_user_c_pos = user_c_pos;

                time_since_last_check = 0.0;

                let mut neededspots: Vec<IVec2> = Vec::new();

                let cam_lock = cam_arc.lock();
                let user_cpos = IVec2 {
                    x: (cam_lock.position.x / 15.0).floor() as i32,
                    y: (cam_lock.position.z / 15.0).floor() as i32,
                };
                drop(cam_lock);

                let radius = {
                    let x = csys_arc.read().radius;
                    x.clone()
                };

                for i in -(radius as i32)..(radius as i32) {
                    for k in -(radius as i32)..(radius as i32) {
                        let csys_arc = csys_arc.read();

                        let tcarc = csys_arc.takencare.clone();
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
                {
                    let csyschunks = csys_arc.read().chunks.clone();

                    for carc in &csyschunks {
                        match carc.try_lock() {
                            Some(cf) => {
                                sorted_chunk_facades.push(*cf);
                            }
                            None => {}
                        }
                    }
                }

                let (unused_or_distant, used_and_close): (Vec<ChunkFacade>, Vec<ChunkFacade>) =
                    sorted_chunk_facades.drain(..).partition(|chunk| {
                        if !chunk.used {
                            true
                        } else {
                            let dist = (chunk.pos.x - user_cpos.x).abs()
                                + (chunk.pos.y - user_cpos.y).abs();
                            dist >= radius as i32 * 2
                        }
                    });

                sorted_chunk_facades.extend(unused_or_distant);
                sorted_chunk_facades.extend(used_and_close);
                //info!("Neededspots size: {}", neededspots.len());

                neededspots.sort_by(|a, b| {
                    let dist_a = (a.x - user_c_pos.x).pow(2) + (a.y - user_c_pos.y).pow(2);
                    let dist_b = (b.x - user_c_pos.x).pow(2) + (b.y - user_c_pos.y).pow(2);
                    dist_a.cmp(&dist_b)
                });

                for (index, ns) in neededspots.iter().enumerate() {
                    let csys_arc = csys_arc.read();
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
                        None => {}
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
            // let deadlocks = deadlock::check_deadlock();

            // if !deadlocks.is_empty() {
            //     println!("{} deadlocks detected", deadlocks.len());
            //     for (i, threads) in deadlocks.iter().enumerate() {
            //         println!("Deadlock #{}", i);
            //         for t in threads {
            //             println!("Thread Id {:#?}", t.thread_id());
            //             println!("{:#?}", t.backtrace());
            //         }
            //     }
            // }

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
                let x_offset = (xpos - LASTX) * MISCSETTINGS.mouse_sense as f64;
                let y_offset = (LASTY - ypos) * MISCSETTINGS.mouse_sense as f64;

                LASTY = ypos;
                LASTX = xpos;

                static mut LASTCAM: Lazy<Camera> = Lazy::new(|| Camera::default());

                let mut cam_clone = {
                    let c = self.camera.lock();
                    c.clone()
                };

                cam_clone.yaw += x_offset as f32;
                cam_clone.pitch += y_offset as f32;

                cam_clone.pitch = cam_clone.pitch.clamp(-89.0, 89.0);

                cam_clone.direction.x = cam_clone.yaw.to_radians().cos() as f32
                    * cam_clone.pitch.to_radians().cos() as f32;
                cam_clone.direction.y = cam_clone.pitch.to_radians().sin();
                cam_clone.direction.z =
                    cam_clone.yaw.to_radians().sin() * cam_clone.pitch.to_radians().cos();
                cam_clone.direction = cam_clone.direction.normalize();

                cam_clone.right = Vec3::new(0.0, 1.0, 0.0)
                    .cross(cam_clone.direction)
                    .normalize();
                cam_clone.up = cam_clone.direction.cross(cam_clone.right).normalize();

                cam_clone.recalculate();

                {
                    let mut c = self.camera.lock();
                    (*c) = cam_clone;
                }

                #[cfg(feature = "show_cam_pos")]
                info!(
                    "Cam dir: {}, {}, {}",
                    cam_clone.direction.x, cam_clone.direction.y, cam_clone.direction.z
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
        SAVE_MISC();
    }
    pub fn delete_block_recursively(
        chunksys: &Arc<RwLock<ChunkSystem>>,
        id: u32,
        at: IVec3,
        set: &mut HashSet<IVec2>,
    ) {
        let mut stack = vec![at]; // Initialize stack with initial position

        while let Some(current) = stack.pop() {
            // Check if the block at the current position is already deleted

            let chunksys = chunksys.read();

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
        let cl = {
            let cl = self.camera.lock();
            cl.clone()
        };
        match raycast_voxel_with_bob(
            cl.position,
            cl.direction,
            &self.chunksys,
            unsafe { VOXEL_SELECT_DISTANCE },
            self.vars.walkbobtimer,
        ) {
            Some((tip, block_hit)) => {
                let blockbits = self.chunksys.read().blockat(block_hit);
                let blockat = blockbits & Blocks::block_id_bits();
                if blockat == 16 {
                    let mut set: HashSet<IVec2> = HashSet::new();
                    Game::delete_block_recursively(&self.chunksys, 16, block_hit, &mut set);
                    for key in set {
                        self.chunksys
                            .read()
                            .queue_rerender_with_key(key, true, false);
                    }
                    #[cfg(feature = "glfw")]
                    self.drops.add_drop(tip, 17, 1);
                } else if blockat == 19 {
                    //Door stuff
                    let top = DoorInfo::get_door_top_bit(blockbits);
                    let other_half;

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
                            0,
                        );
                        message.info2 = 0;
                        message.otherpos = other_half;

                        self.netconn.send(&message);
                    } else {
                        self.chunksys.read().set_block(block_hit, 0, true);
                        self.chunksys
                            .read()
                            .set_block_and_queue_rerender(other_half, 0, true, true, false);
                    }
                } else {
                    if blockat != 0 {
                        #[cfg(feature = "glfw")]
                        self.drops.add_drop(tip, blockat, 1);
                    }

                    //TODO: PROBLEM HERE THAT WILL ALLOW USERS TO KEEP DUPING A BLOCK AS LONG AS THE SERVER DOESNT RESPOND
                    if self.vars.in_multiplayer {
                        let message = Message::new(
                            MessageType::BlockSet,
                            Vec3::new(block_hit.x as f32, block_hit.y as f32, block_hit.z as f32),
                            0.0,
                            0,
                        );
                        self.netconn.send(&message);
                    } else {
                        self.chunksys
                            .read()
                            .set_block_and_queue_rerender(block_hit, 0, true, true, false);
                    }
                }
            }
            None => {}
        }
    }
    #[cfg(feature = "glfw")]
    pub fn scroll(&mut self, y: f64) {


        let mut invrowchange = 0;
        if y > 0.0 {
            invrowchange += 1;
        }
        if y < 0.0 {
            invrowchange -= 1;
        }

        if unsafe {BUILD_PREVIEW_MODE} {
            let mut proposednewbuild = unsafe { SELECTED_BUILD } as i8 + invrowchange;
            if proposednewbuild < 0 {
                proposednewbuild = unsafe { BUILD_VOXEL_MODELS.len() } as i8 - 1;
            }
            unsafe {
                SELECTED_BUILD = proposednewbuild as usize % BUILD_VOXEL_MODELS.len();
            }
        } else {
            let mut proposednewslot = self.hud.bumped_slot as i8 + invrowchange;
            if proposednewslot < 0 {
                proposednewslot = ROWLENGTH as i8 - 1;
            }
            self.hud.bumped_slot = proposednewslot as usize % ROWLENGTH as usize;
            self.hud.dirty = true;
            self.hud.update();
        }
        
        
    }

    #[cfg(feature = "glfw")]
    pub fn draw_user_build_preview(
        spot: &IVec3,
        vox: &JVoxModel,
        oldshader: &Shader,
        cam_clone: &Camera,
        amb_bm: f32,
        walkbobt: f32,
        texture: &Texture,
    ) -> Vec<(IVec3, u32)> {
        use crate::{
            chunk::ChW,
            specialblocks::{
                chest::*, conveyor::*, crafttable::*, door::*, ladder::*, tallgrass::*, torch::*,
            },
            textureface::ONE_OVER_16,
        };

        static mut vec: Vec<(IVec3, u32)> = Vec::new();

        unsafe {
            use glam::{I16Vec3, U16Vec3};

            use crate::{
                cube::CubeSide, packedvertex::PackedVertex,
                specialblocks::crafttable::CraftTableInfo,
            };

            if UBP_VAO == 0 {
                gl::GenVertexArrays(1, &mut UBP_VAO);

                gl::BindVertexArray(UBP_VAO);
                texture.add_to_unit(0);

                gl::CreateBuffers(1, &mut UBP_VBO);
                gl::CreateBuffers(1, &mut UBP_UVBO);
            }

            gl::BindVertexArray(UBP_VAO);

            gl::UseProgram(oldshader.shader_id);

            static mut LAST_MOD_IND: i32 = -99;

            if LAST_MOD_IND != vox.idnumber {

                vec.clear();
                //println!("Lastmodind {LAST_MOD_IND} is not equal to {}", vox.idnumber);

                UBP_UVDATA.clear();
                UBP_VDATA.clear();

                for i in &vox.model.models {
                    let size = i.size;
                    for v in &i.voxels {
                        let rearr_point = IVec3::new(
                            v.point.x as i32 - (size.x / 2) as i32,
                            v.point.z as i32,
                            v.point.y as i32 - (size.y / 2) as i32,
                        );
                        //println!("{:?}", rearr_point);
                        


                        let doorbottomuvs = DoorInfo::get_door_uvs(TextureFace::new(11, 0));
                        let doortopuvs = DoorInfo::get_door_uvs(TextureFace::new(11, 1));

                     
                        let combined = v.color_index.0 as u32; 

                        vec.push((rearr_point , combined));
                        let block = combined & Blocks::block_id_bits();
                        let flags = combined & Blocks::block_flag_bits();

                        if block != 0 {

                            {
       
                                let texcoord = Blocks::get_tex_coords(block, CubeSide::LEFT);

                                let blocklighthere = U16Vec3::new(5, 5, 5);

                                let packedrgb = PackedVertex::pack_rgb(
                                    blocklighthere.x,
                                    blocklighthere.y,
                                    blocklighthere.z,
                                );

                                let prgb: u32 =
                                    0b0000_0000_0000_0000_0000_0000_0000_0000 | (packedrgb) as u32;

                                for vert in [
                                    0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 0.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 1.0 +rearr_point.z as f32, 0.0, 14.0,
0.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,
1.0 +rearr_point.x as f32, 1.0 +rearr_point.y as f32, 0.0 +rearr_point.z as f32, 0.0, 14.0,

                                ].chunks(5)
                                {
                                    UBP_VDATA.extend_from_slice(&[
                                        vert[0] as f32,
                                        vert[1] as f32,
                                        vert[2] as f32,
                                        f32::from_bits(prgb),
                                        vert[4],
                                    ])
                                }
                                let tc = Vec2::new(
                                    texcoord.0 as f32 * ONE_OVER_16,
                                    -(texcoord.1 as f32 * ONE_OVER_16),
                                );
                                UBP_UVDATA.extend_from_slice(&[
                                    0.032546114176511765+tc.x, 0.999256021110341+tc.y, 0.0, 0.0,
                            -0.00038831273559480906+tc.x, 0.966321587562561+tc.y, 0.0, 0.0,
                            -0.00038831273559480906+tc.x, 0.999256021110341+tc.y, 0.0, 0.0,
                            0.032546114176511765+tc.x, 0.9995140247046947+tc.y, 0.0, 0.0,
                            -0.00038831273559480906+tc.x, 0.9665796048939228+tc.y, 0.0, 0.0,
                            -0.00038831273559480906+tc.x, 0.9995140247046947+tc.y, 0.0, 0.0,
                            0.03291086480021477+tc.x, 0.9997720420360565+tc.y, 0.0, 0.0,
                            -2.3560685804113746e-05+tc.x, 0.9668375849723816+tc.y, 0.0, 0.0,
                            -2.3560685804113746e-05+tc.x, 0.9997720420360565+tc.y, 0.0, 0.0,
                            0.03291086480021477+tc.x, 1.000030018389225+tc.y, 0.0, 0.0,
                            -2.3560685804113746e-05+tc.x, 0.9670955687761307+tc.y, 0.0, 0.0,
                            -2.3560685804113746e-05+tc.x, 1.000030018389225+tc.y, 0.0, 0.0,
                            0.03280412033200264+tc.x, 0.9994072914123535+tc.y, 0.0, 0.0,
                            -0.0001303069293498993+tc.x, 0.9664728343486786+tc.y, 0.0, 0.0,
                            -0.0001303069293498993+tc.x, 0.9994072914123535+tc.y, 0.0, 0.0,
                            0.033017635345458984+tc.x, 0.9997720406099688+tc.y, 0.0, 0.0,
                            8.318200707435608e-05+tc.x, 0.9668375849723816+tc.y, 0.0, 0.0,
                            8.318200707435608e-05+tc.x, 0.9997720406099688+tc.y, 0.0, 0.0,
                            0.032546114176511765+tc.x, 0.999256021110341+tc.y, 0.0, 0.0,
                            0.032546114176511765+tc.x, 0.966321587562561+tc.y, 0.0, 0.0,
                            -0.00038831273559480906+tc.x, 0.966321587562561+tc.y, 0.0, 0.0,
                            0.032546114176511765+tc.x, 0.9995140247046947+tc.y, 0.0, 0.0,
                            0.032546114176511765+tc.x, 0.9665796048939228+tc.y, 0.0, 0.0,
                            -0.00038831273559480906+tc.x, 0.9665796048939228+tc.y, 0.0, 0.0,
                            0.03291086480021477+tc.x, 0.9997720420360565+tc.y, 0.0, 0.0,
                            0.03291086480021477+tc.x, 0.9668375849723816+tc.y, 0.0, 0.0,
                            -2.3560685804113746e-05+tc.x, 0.9668375849723816+tc.y, 0.0, 0.0,
                            0.03291086480021477+tc.x, 1.000030018389225+tc.y, 0.0, 0.0,
                            0.03291086480021477+tc.x, 0.9670955687761307+tc.y, 0.0, 0.0,
                            -2.3560685804113746e-05+tc.x, 0.9670955687761307+tc.y, 0.0, 0.0,
                            0.03280412033200264+tc.x, 0.9994072914123535+tc.y, 0.0, 0.0,
                            0.03280412033200264+tc.x, 0.9664728343486786+tc.y, 0.0, 0.0,
                            -0.0001303069293498993+tc.x, 0.9664728343486786+tc.y, 0.0, 0.0,
                            0.033017635345458984+tc.x, 0.9997720406099688+tc.y, 0.0, 0.0,
                            0.033017635345458984+tc.x, 0.9668375849723816+tc.y, 0.0, 0.0,
                            8.318200707435608e-05+tc.x, 0.9668375849723816+tc.y, 0.0, 0.0,

                                ]);
                            }
                        }
                    }
                }

                LAST_MOD_IND = vox.idnumber;

                WorldGeometry::bind_old_geometry_diff_vao(
                    UBP_VBO,
                    UBP_UVBO,
                    &UBP_VDATA,
                    &UBP_UVDATA,
                    &oldshader,
                    UBP_VAO
                );
            } else {
                //WorldGeometry::bind_old_geometry_no_upload(UBP_VBO, UBP_UVBO, &oldshader);
                //WorldGeometry::bind_old_geometry(UBP_VBO, UBP_UVBO, &UBP_VDATA, &UBP_UVDATA, &oldshader);
            }

            static mut MVP_LOC: i32 = -1;
            static mut CAM_POS_LOC: i32 = 0;
            static mut AMBIENT_BRIGHT_MULT_LOC: i32 = 0;
            static mut VIEW_DISTANCE_LOC: i32 = 0;
            static mut UNDERWATER_LOC: i32 = 0;
            static mut CAM_DIR_LOC: i32 = 0;
            static mut SUNSET_LOC: i32 = 0;
            static mut SUNRISE_LOC: i32 = 0;
            static mut WALKBOB_LOC: i32 = 0;
            unsafe {
                if MVP_LOC == -1 {
                    MVP_LOC =
                        gl::GetUniformLocation(oldshader.shader_id, b"mvp\0".as_ptr() as *const i8);
                    //info!("MVP LOC: {}", MVP_LOC);

                    WALKBOB_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"walkbob\0".as_ptr() as *const i8,
                    );

                    CAM_POS_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"camPos\0".as_ptr() as *const i8,
                    );
                    AMBIENT_BRIGHT_MULT_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"ambientBrightMult\0".as_ptr() as *const i8,
                    );
                    VIEW_DISTANCE_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"viewDistance\0".as_ptr() as *const i8,
                    );
                    UNDERWATER_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"underWater\0".as_ptr() as *const i8,
                    );
                    CAM_DIR_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"camDir\0".as_ptr() as *const i8,
                    );
                    SUNSET_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"sunset\0".as_ptr() as *const i8,
                    );
                    SUNRISE_LOC = gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"sunrise\0".as_ptr() as *const i8,
                    );
                }

                gl::UniformMatrix4fv(
                    MVP_LOC,
                    1,
                    gl::FALSE,
                    cam_clone.mvp.to_cols_array().as_ptr(),
                );
                gl::Uniform3f(
                    CAM_POS_LOC,
                    cam_clone.position.x,
                    cam_clone.position.y,
                    cam_clone.position.z,
                );
                gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, amb_bm);
                gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
                gl::Uniform1f(UNDERWATER_LOC, 0.0);
                gl::Uniform3f(
                    CAM_DIR_LOC,
                    cam_clone.direction.x,
                    cam_clone.direction.y,
                    cam_clone.direction.z,
                );

                gl::Uniform1f(
                    gl::GetUniformLocation(oldshader.shader_id, b"time\0".as_ptr() as *const i8),
                    glfwGetTime() as f32,
                );
                gl::Uniform1f(
                    gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"weathertype\0".as_ptr() as *const i8,
                    ),
                    WEATHERTYPE,
                );

                gl::Uniform1f(SUNSET_LOC, 0.0);
                gl::Uniform1f(WALKBOB_LOC, walkbobt);
                gl::Uniform1f(SUNRISE_LOC, 0.0);
                gl::Uniform1i(
                    gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"ourTexture\0".as_ptr() as *const i8,
                    ),
                    0,
                );
                gl::Uniform1i(
                    gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"weatherTexture\0".as_ptr() as *const i8,
                    ),
                    2,
                );

                // let fc = Planets::get_fog_col(self.chunksys.read().planet_type as u32);
                // gl::Uniform4f(
                //     FOGCOL_LOC,
                //     fc.0,
                //     fc.1,
                //     fc.2,
                //     fc.3
                // );
            }

            unsafe {
                gl::Uniform1f(
                    gl::GetUniformLocation(
                        oldshader.shader_id,
                        b"renderingweather\0".as_ptr() as *const i8,
                    ),
                    0.0,
                );
            }

            gl::Uniform3f(
                gl::GetUniformLocation(
                    oldshader.shader_id,
                    b"transformpos\0".as_ptr() as *const i8,
                ),
                spot.x as f32 + BUILD_MODEL_OFFSET.x as f32,
                spot.y as f32,
                spot.z as f32 + BUILD_MODEL_OFFSET.y as f32,
            );

            //println!("UVDATALEN: {}", UBP_VDATA.len() as f32 / 5.0);

            gl::DrawArrays(gl::TRIANGLES, 0, (*UBP_VDATA).len() as i32 / 5);

        }
        unsafe { vec.clone() }
    }

    #[cfg(feature = "glfw")]
    pub fn cast_place_ray(&mut self) {
        let slot_selected = self.hud.bumped_slot;
        let slot = self.inventory.read().inv[slot_selected];

        let mut updateinv = false;
        let mut openedcraft = false;

        if true {
            let cl = {
                let c = self.camera.lock();
                c.clone()
            };

            match raycast_voxel_with_bob(
                cl.position,
                cl.direction,
                &self.chunksys,
                unsafe { VOXEL_SELECT_DISTANCE },
                self.vars.walkbobtimer,
            ) {
                Some((tip, block_hit)) => {
                    let mut blockbitshere = self.chunksys.read().blockat(block_hit);
                    let blockidhere = blockbitshere & Blocks::block_id_bits();

                    if blockidhere == 19 {
                        let top = DoorInfo::get_door_top_bit(blockbitshere);
                        let otherhalf;

                        if top == 1 {
                            otherhalf = block_hit + IVec3::new(0, -1, 0);
                        } else {
                            otherhalf = block_hit + IVec3::new(0, 1, 0);
                        }
                        let mut otherhalfbits = self.chunksys.read().blockat(otherhalf);

                        DoorInfo::toggle_door_open_bit(&mut blockbitshere);
                        DoorInfo::toggle_door_open_bit(&mut otherhalfbits);

                        if self.vars.in_multiplayer {
                            let mut message = Message::new(
                                MessageType::MultiBlockSet,
                                Vec3::new(
                                    block_hit.x as f32,
                                    block_hit.y as f32,
                                    block_hit.z as f32,
                                ),
                                0.0,
                                blockbitshere,
                            );
                            message.info2 = otherhalfbits;
                            message.otherpos = otherhalf;
                            self.netconn.send(&message);
                        } else {
                            self.chunksys
                                .write()
                                .set_block(otherhalf, otherhalfbits, true);
                            self.chunksys.write().set_block_and_queue_rerender(
                                block_hit,
                                blockbitshere,
                                true,
                                true,
                                true,
                            );
                        }
                    } else if blockidhere == 21 {
                        //RIGHT CLICKED A CHEST

                        //let _csys = self.chunksys.write();

                        self.hud.current_chest = block_hit;
                        updateinv = true;
                    } else if blockidhere == 31 {
                        unsafe {
                            ATSMALLTABLE = false;
                        }
                        self.crafting_open = true;

                        self.window
                            .write()
                            .set_cursor_mode(glfw::CursorMode::Normal);
                        openedcraft = true;
                    } else if slot.0 != 0 && slot.1 > 0 {
                        let id = slot.0;
                        let diff = (tip + Vec3::new(-0.5, -0.5, -0.5))
                            - (Vec3::new(
                                block_hit.x as f32,
                                block_hit.y as f32,
                                block_hit.z as f32, //TODO ADD -0.5 to these?????
                            ));

                        let hit_normal;

                        // Determine the primary axis of intersection
                        if (diff.x).abs() > (diff.y).abs() && (diff.x).abs() > (diff.z).abs() {
                            // The hit was primarily along the X-axis
                            hit_normal = vec::IVec3::new(if diff.x > 0.0 { 1 } else { -1 }, 0, 0);
                        } else if (diff.y).abs() > (diff.x).abs() && (diff.y).abs() > (diff.z).abs()
                        {
                            // The hit was primarily along the Y-axis
                            hit_normal = vec::IVec3::new(0, if diff.y > 0.0 { 1 } else { -1 }, 0);
                        } else {
                            // The hit was primarily along the Z-axis
                            hit_normal = vec::IVec3::new(0, 0, if diff.z > 0.0 { 1 } else { -1 });
                        }

                        info!(
                            "Hit normal is {} {} {}",
                            hit_normal.x, hit_normal.y, hit_normal.z
                        );

                        let place_point = block_hit + hit_normal;
                        info!(
                            "Attempting to place {} at {} {} {}",
                            id, place_point.x, place_point.y, place_point.z
                        );

                        //Don't allow placing blocks where solid blocks or the player are
                        let blockbitsatplacepoint = self.chunksys.read().blockat(place_point);
                        let blockidatplacepoint = blockbitsatplacepoint & Blocks::block_id_bits();

                        if !Blocks::is_overwritable(blockidatplacepoint) {
                            return ();
                        }

                        let camblockspot = IVec3::new(
                            cl.position.x.floor() as i32,
                            cl.position.y.round() as i32,
                            cl.position.z.floor() as i32,
                        );

                        if place_point == camblockspot
                            || place_point == camblockspot - IVec3::new(0, 1, 0)
                        {
                            return ();
                        }

                        if id == 19 {
                            //Door shit

                            let neighbor_axes = vec![
                                IVec3::new(1, 0, 0),
                                IVec3::new(0, 0, 1),
                                IVec3::new(1, 0, 0),
                                IVec3::new(0, 0, 1),
                            ];

                            let place_above = place_point + IVec3::new(0, 1, 0);
                            let place_below = place_point + IVec3::new(0, -1, 0);

                            let csysread = self.chunksys.read();

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

                                let mut left: IVec3 = IVec3::new(0, 0, 0);
                                let mut right: IVec3 = IVec3::new(0, 0, 0);

                                if direction == 0 || direction == 1 {
                                    left = place_point - neighbor_axes[direction as usize];
                                    right = place_point + neighbor_axes[direction as usize];
                                } else {
                                    left = place_point + neighbor_axes[direction as usize];
                                    right = place_point - neighbor_axes[direction as usize];
                                }

                                let csysread = self.chunksys.read();

                                let mut blockbitsright = csysread.blockat(right);
                                let mut blockbitsleft = csysread.blockat(left);

                                drop(csysread);

                                if (blockbitsright & Blocks::block_id_bits()) == 19 {
                                    let neighdir = Blocks::get_direction_bits(blockbitsright);
                                    if neighdir == direction
                                        && DoorInfo::get_door_top_bit(blockbitsright) == 0
                                    {
                                        let csysread = self.chunksys.read();

                                        let rightup = right + IVec3::new(0, 1, 0);
                                        let mut neightopbits = csysread.blockat(rightup);

                                        DoorInfo::set_opposite_door_bits(&mut top_id, 1);
                                        DoorInfo::set_opposite_door_bits(&mut bottom_id, 1);

                                        DoorInfo::set_opposite_door_bits(&mut blockbitsright, 0);
                                        DoorInfo::set_opposite_door_bits(&mut neightopbits, 0);

                                        let _chunktoreb = ChunkSystem::spot_to_chunk_pos(&right);

                                        if self.vars.in_multiplayer {
                                            let mut message = Message::new(
                                                MessageType::MultiBlockSet,
                                                Vec3::new(
                                                    right.x as f32,
                                                    right.y as f32,
                                                    right.z as f32,
                                                ),
                                                0.0,
                                                blockbitsright,
                                            );

                                            message.info2 = neightopbits;
                                            message.otherpos = rightup;

                                            self.netconn.send(&message);
                                        } else {
                                            self.chunksys.read().set_block_and_queue_rerender(
                                                right,
                                                blockbitsright,
                                                false,
                                                true,
                                                true,
                                            );
                                            self.chunksys.read().set_block_and_queue_rerender(
                                                rightup,
                                                neightopbits,
                                                false,
                                                true,
                                                true,
                                            );
                                        }
                                    }
                                }

                                if (blockbitsleft & Blocks::block_id_bits()) == 19 {
                                    let neighdir = Blocks::get_direction_bits(blockbitsleft);
                                    if neighdir == direction
                                        && DoorInfo::get_door_top_bit(blockbitsleft) == 0
                                    {
                                        let leftup = left + IVec3::new(0, 1, 0);

                                        let csysread = self.chunksys.read();

                                        let mut neightopbits = csysread.blockat(leftup);

                                        DoorInfo::set_opposite_door_bits(&mut top_id, 1);
                                        DoorInfo::set_opposite_door_bits(&mut bottom_id, 1);

                                        DoorInfo::set_opposite_door_bits(&mut blockbitsleft, 0);
                                        DoorInfo::set_opposite_door_bits(&mut neightopbits, 0);

                                        let _chunktoreb = ChunkSystem::spot_to_chunk_pos(&left);

                                        if self.vars.in_multiplayer {
                                            let mut message = Message::new(
                                                MessageType::MultiBlockSet,
                                                Vec3::new(
                                                    left.x as f32,
                                                    left.y as f32,
                                                    left.z as f32,
                                                ),
                                                0.0,
                                                blockbitsleft,
                                            );

                                            message.info2 = neightopbits;
                                            message.otherpos = leftup;

                                            self.netconn.send(&message);
                                        } else {
                                            self.chunksys.read().set_block_and_queue_rerender(
                                                left,
                                                blockbitsleft,
                                                false,
                                                true,
                                                true,
                                            );
                                            self.chunksys.read().set_block_and_queue_rerender(
                                                leftup,
                                                neightopbits,
                                                false,
                                                true,
                                                true,
                                            );
                                        }
                                    }
                                }

                                if self.vars.in_multiplayer {
                                    let mut message = Message::new(
                                        MessageType::MultiBlockSet,
                                        Vec3::new(
                                            place_point.x as f32,
                                            place_point.y as f32,
                                            place_point.z as f32,
                                        ),
                                        0.0,
                                        bottom_id,
                                    );

                                    message.info2 = top_id;
                                    message.otherpos = place_above;

                                    self.netconn.send(&message);
                                } else {
                                    self.chunksys.read().set_block_and_queue_rerender(
                                        place_point,
                                        bottom_id,
                                        false,
                                        true,
                                        true,
                                    );
                                    self.chunksys.read().set_block_and_queue_rerender(
                                        place_above,
                                        top_id,
                                        false,
                                        true,
                                        true,
                                    );
                                }
                            }
                        } else if id == 45 {
                            //conveyor shit

                            let mut conveyor_id = id;

                            let diffx = cl.position.x - place_point.x as f32;
                            let diffz = cl.position.z - place_point.z as f32;

                            let mut direction = 0;

                            if diffx.abs() > diffz.abs() {
                                direction = if diffx > 0.0 { 1 } else { 3 };
                            } else {
                                direction = if diffz > 0.0 { 2 } else { 0 };
                            }

                            Blocks::set_direction_bits(&mut conveyor_id, direction);

                            if self.vars.in_multiplayer {
                                let message = Message::new(
                                    MessageType::BlockSet,
                                    Vec3::new(
                                        place_point.x as f32,
                                        place_point.y as f32,
                                        place_point.z as f32,
                                    ),
                                    0.0,
                                    conveyor_id,
                                );

                                self.netconn.send(&message);
                            } else {
                                self.chunksys.read().set_block_and_queue_rerender(
                                    place_point,
                                    conveyor_id,
                                    false,
                                    true,
                                    false,
                                );
                            }
                        } else if id == 20 {
                            //ladder shit

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
                                        place_point.z as f32,
                                    ),
                                    0.0,
                                    ladder_id,
                                );

                                self.netconn.send(&message);
                            } else {
                                self.chunksys.read().set_block_and_queue_rerender(
                                    place_point,
                                    ladder_id,
                                    false,
                                    true,
                                    false,
                                );
                            }
                        } else if id == 21 {
                            //Chest shit

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
                                        place_point.z as f32,
                                    ),
                                    0.0,
                                    chest_id,
                                );

                                self.netconn.send(&message);
                            } else {
                                self.chunksys.read().set_block_and_queue_rerender(
                                    place_point,
                                    chest_id,
                                    false,
                                    true,
                                    false,
                                );
                            }
                        } else {
                            if !Blocks::is_non_placeable(slot.0) {
                                if self.vars.in_multiplayer {
                                    let message = Message::new(
                                        MessageType::BlockSet,
                                        Vec3::new(
                                            place_point.x as f32,
                                            place_point.y as f32,
                                            place_point.z as f32,
                                        ),
                                        0.0,
                                        id,
                                    );
                                    self.netconn.send(&message);
                                } else {
                                    self.chunksys.read().set_block_and_queue_rerender(
                                        place_point,
                                        id,
                                        false,
                                        true,
                                        false,
                                    );
                                }
                            }
                        }
                        if !Blocks::is_non_placeable(slot.0) {
                            if self.vars.in_multiplayer {
                                if slot.1 == 1 {
                                    let mutslot = &mut self.inventory.write().inv[slot_selected];
                                    mutslot.1 = 0;
                                    mutslot.0 = 0;

                                    let mut msg = Message::new(
                                        MessageType::ChestInvUpdate,
                                        Vec3::ZERO,
                                        0.0,
                                        slot_selected as u32,
                                    );
                                    msg.infof = 0.0;
                                    msg.info2 = 1;

                                    self.netconn.send(&msg);
                                } else {
                                    let slot = &self.inventory.read().inv[slot_selected];

                                    let mut msg = Message::new(
                                        MessageType::ChestInvUpdate,
                                        Vec3::ZERO,
                                        slot.0 as f32,
                                        slot_selected as u32,
                                    );
                                    msg.infof = slot.1 as f32 - 1.0;
                                    msg.info2 = 1;

                                    self.netconn.send(&msg);
                                }
                            } else {
                                if slot.1 == 1 {
                                    let mutslot = &mut self.inventory.write().inv[slot_selected];
                                    mutslot.1 = 0;
                                    mutslot.0 = 0;
                                } else {
                                    let mutslot = &mut self.inventory.write().inv[slot_selected];
                                    mutslot.1 -= 1;
                                }
                            }
                        }
                    }
                }

                None => {}
            }

            if Blocks::is_food(slot.0) {
                //GET THIS FOODS HEALTH STATS

                let foodstats = Blocks::get_food_stats(slot.0);

                //HEAL SOME HEALTH AND STAMINA:

                let h = self.health.load(Ordering::Relaxed);

                //let s = self.stamina.load(Ordering::Relaxed);

                self.health
                    .store((h + foodstats.0 as i8).min(20), Ordering::Relaxed);
                // self.stamina
                //     .store((s + foodstats.1).min(100), Ordering::Relaxed);

                //REDUCE THE INV ITEM:
                if self.vars.in_multiplayer {
                    if slot.1 == 1 {
                        let mutslot = &mut self.inventory.write().inv[slot_selected];
                        mutslot.1 = 0;
                        mutslot.0 = 0;

                        let mut msg = Message::new(
                            MessageType::ChestInvUpdate,
                            Vec3::ZERO,
                            0.0,
                            slot_selected as u32,
                        );
                        msg.infof = 0.0;
                        msg.info2 = 1;

                        self.netconn.send(&msg);
                    } else {
                        let slot = &self.inventory.read().inv[slot_selected];

                        let mut msg = Message::new(
                            MessageType::ChestInvUpdate,
                            Vec3::ZERO,
                            slot.0 as f32,
                            slot_selected as u32,
                        );
                        msg.infof = slot.1 as f32 - 1.0;
                        msg.info2 = 1;

                        self.netconn.send(&msg);
                    }
                } else {
                    if slot.1 == 1 {
                        let mutslot = &mut self.inventory.write().inv[slot_selected];
                        mutslot.1 = 0;
                        mutslot.0 = 0;
                    } else {
                        let mutslot = &mut self.inventory.write().inv[slot_selected];
                        mutslot.1 -= 1;
                    }
                }
            }
        } else {
        }

        if updateinv {
            self.update_inventory();
            self.hud.chest_open = true;

            self.window
                .write()
                .set_cursor_mode(glfw::CursorMode::Normal);
            self.set_mouse_focused(false);
        }

        if openedcraft {
            self.set_mouse_focused(false);
        }
    }
    #[cfg(feature = "glfw")]
    pub fn mouse_button(&mut self, mb: MouseButton, a: Action) {
        if self.hud.chest_open {
            match unsafe {
                MISCSETTINGS
                    .mousebinds
                    .get(&format!("{:?}", mb))
                    .unwrap_or(&"_".to_string())
                    .as_str()
            } {
                "Break/Attack" => {
                    //self.vars.mouse_clicked = a == Action::Press;

                    if a == Action::Press {
                        let mut updateinv = false;
                        {
                            //let csys = self.chunksys.write();
                            unsafe {
                                match MOUSED_SLOT {
                                    SlotIndexType::ChestSlot(e) => {
                                        match self.chest_registry.get_mut(&self.hud.current_chest) {
                                            Some(mut ch) => {
                                                let slot = &mut ch.value_mut().inv[e as usize];

                                                //IF This slot has an item id the same as our mouse slot
                                                if slot.0 == self.mouse_slot.0 {
                                                    if self.vars.in_multiplayer {
                                                        /*OTHERPOS: CURRENT CHEST */
                                                        /*INFO: DEST SLOT INDEX */
                                                        /*INFO2: SLOT INDEX TYPE */
                                                        /*ROT: ID */
                                                        /*INFOF: COUNT */
                                                        /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
                                                        /*Z: IF MOUSE_SLOT IS REPLACED */
                                                        /*BO: IF WE WANT SERVER-SIDE CHEST-TO-MOUSE DISPLACEMENT (NO if this is adding to a stack, it will put the previous stack in our hand) */
                                                        let mut msg = Message::new(
                                                            MessageType::ChestInvUpdate,
                                                            Vec3::new(0 as f32, 0 as f32, 1.0),
                                                            slot.0 as f32,
                                                            e as u32,
                                                        );
                                                        msg.otherpos = self.hud.current_chest;
                                                        msg.info2 = /*0 = CHEST, 1 = INV, 2 = NONE */0;
                                                        msg.infof =
                                                            (slot.1 + self.mouse_slot.1) as f32;
                                                        msg.bo = false;
                                                        self.netconn.send(&msg);
                                                    } else {
                                                        slot.1 = slot.1 + self.mouse_slot.1;

                                                        self.mouse_slot = (0, 0);
                                                    }
                                                } else
                                                //SWAP YOUR mouse_slot AND slot
                                                {
                                                    let buff = slot.clone();
                                                    if self.vars.in_multiplayer {
                                                        /*OTHERPOS: CURRENT CHEST */
                                                        /*INFO: DEST SLOT INDEX */
                                                        /*INFO2: SLOT INDEX TYPE */
                                                        /*ROT: ID */
                                                        /*INFOF: COUNT */
                                                        /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
                                                        /*Z: IF MOUSE_SLOT IS REPLACED */
                                                        /*BO: IF WE WANT SERVER-SIDE CHEST-TO-MOUSE DISPLACEMENT (NO if this is adding to a stack, it will put the previous stack in our hand) */
                                                        let mut msg = Message::new(
                                                            MessageType::ChestInvUpdate,
                                                            Vec3::new(
                                                                buff.0 as f32,
                                                                buff.1 as f32,
                                                                1.0,
                                                            ),
                                                            self.mouse_slot.0 as f32,
                                                            e as u32,
                                                        );
                                                        msg.otherpos = self.hud.current_chest;
                                                        msg.info2 = /*0 = CHEST, 1 = INV, 2 = NONE */0;
                                                        msg.infof = self.mouse_slot.1 as f32;
                                                        msg.bo = true;
                                                        self.netconn.send(&msg);
                                                    } else {
                                                        slot.0 = self.mouse_slot.0;
                                                        slot.1 = self.mouse_slot.1;

                                                        self.mouse_slot.0 = buff.0;
                                                        self.mouse_slot.1 = buff.1;
                                                        updateinv = true;
                                                    }
                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                    SlotIndexType::InvSlot(e) => {
                                        let slot = &mut self.inventory.write().inv[e as usize];

                                        //IF This slot has an item id the same as our mouse slot
                                        if slot.0 == self.mouse_slot.0 {
                                            if self.vars.in_multiplayer {
                                                /*OTHERPOS: CURRENT CHEST */
                                                /*INFO: DEST SLOT INDEX */
                                                /*INFO2: SLOT INDEX TYPE */
                                                /*ROT: ID */
                                                /*INFOF: COUNT */
                                                /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
                                                /*Z: IF MOUSE_SLOT IS REPLACED */
                                                /*BO: IF WE WANT SERVER-SIDE CHEST-TO-MOUSE DISPLACEMENT (NO if this is adding to a stack, it will put the previous stack in our hand) */
                                                let mut msg = Message::new(
                                                    MessageType::ChestInvUpdate,
                                                    Vec3::new(0 as f32, 0 as f32, 1.0),
                                                    slot.0 as f32,
                                                    e as u32,
                                                );
                                                msg.otherpos = self.hud.current_chest;
                                                msg.info2 = /*0 = CHEST, 1 = INV, 2 = NONE */1;
                                                msg.infof = (slot.1 + self.mouse_slot.1) as f32;
                                                msg.bo = false;
                                                self.netconn.send(&msg);
                                            } else {
                                                slot.1 = slot.1 + self.mouse_slot.1;

                                                self.mouse_slot = (0, 0);
                                            }
                                        } else
                                        //SWAP YOUR mouse_slot AND slot
                                        {
                                            let buff = slot.clone();

                                            if self.vars.in_multiplayer {
                                                /*OTHERPOS: CURRENT CHEST */
                                                /*INFO: DEST SLOT INDEX */
                                                /*INFO2: SLOT INDEX TYPE */
                                                /*ROT: ID */
                                                /*INFOF: COUNT */
                                                /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
                                                /*Z: IF MOUSE_SLOT IS REPLACED */
                                                /*BO: IF WE WANT SERVER-SIDE CHEST-TO-MOUSE DISPLACEMENT (NO if this is adding to a stack, it will put the previous stack in our hand) */
                                                let mut msg = Message::new(
                                                    MessageType::ChestInvUpdate,
                                                    Vec3::new(buff.0 as f32, buff.1 as f32, 1.0),
                                                    self.mouse_slot.0 as f32,
                                                    e as u32,
                                                );
                                                msg.otherpos = self.hud.current_chest;
                                                msg.info2 = /*0 = CHEST, 1 = INV, 2 = NONE */ 1;
                                                msg.infof = self.mouse_slot.1 as f32;
                                                msg.bo = true;
                                                self.netconn.send(&msg);
                                            } else {
                                                slot.0 = self.mouse_slot.0;
                                                slot.1 = self.mouse_slot.1;

                                                self.mouse_slot.0 = buff.0;
                                                self.mouse_slot.1 = buff.1;
                                                updateinv = true;
                                            }
                                        }
                                    }
                                    SlotIndexType::None => {}
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
                "Place/Use" => {
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
            match unsafe {
                MISCSETTINGS
                    .mousebinds
                    .get(&format!("{:?}", mb))
                    .unwrap_or(&"_".to_string())
                    .as_str()
            } {
                "Break/Attack" => {
                    self.vars.mouse_clicked = a == Action::Press;
                    // if self.vars.mouse_clicked {
                    //     self.cast_break_ray();
                    // }
                }
                "Place/Use" => {
                    
                    self.vars.right_mouse_clicked = a == Action::Press;
                    if !self.vars.ship_taken_off {
                        
                        if self.vars.right_mouse_clicked {
                            //println!("RMC");
                            if unsafe { BUILD_PREVIEW_MODE } {
                                
                                unsafe {
                                    
                                    if MOUSE_ON_CUBE {
                                        let mut implic = HashSet::new();
                                        //println!("LOPS len: {}", LIST_OF_PREVIEWED_SPOTS.len());
                                        for (spot, block) in LIST_OF_PREVIEWED_SPOTS.clone() {
                                            let spot = spot + SELECTCUBESPOT + IVec3::new(BUILD_MODEL_OFFSET.x, 0, BUILD_MODEL_OFFSET.y);
                                            let chunkspot = ChunkSystem::spot_to_chunk_pos(&spot);
                                            implic.insert(chunkspot);
                                            //println!("Setting a block {} at {}", block, spot);
                                            self.chunksys.read().set_block_no_sound(spot, block, true);
                                        }
                    
                                        for imp in implic {
                                            self.chunksys.read().queue_rerender_with_key(imp, true, false);
                                        }
                                    }   
                                    
    
                                }
                            } else {
                                self.cast_place_ray();
                            }
                            
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

            let currseed = unsafe { CURRSEED.load(std::sync::atomic::Ordering::Relaxed) };
            let nt = 0;

            self.vars.hostile_world = (nt % 2) != 0;

            self.start_chunks_with_radius(10, currseed, nt as usize);
        } else {
            let mut rng = StdRng::from_entropy();

            let seed: u32 = rng.gen_range(0..2232328);
            println!("This called");

            static mut CURR_NT: usize = 0;
            self.camera.lock().position = Vec3::new(0.0, 100.0, 0.0);

            unsafe {
                self.vars.hostile_world = (CURR_NT % 2) == 0;
                CURR_NT = (CURR_NT + 1) % 2;
                unsafe { CURRSEED.store(seed, Ordering::Relaxed) };
                self.start_chunks_with_radius(10, seed, CURR_NT);

                info!("Now noise type is {}", self.chunksys.read().planet_type);
            }
        }

        // self.chunksys.load_world_from_file(String::from("saves/world1"));
        // self.vars.hostile_world = false;
        // let seed = *self.chunksys.currentseed.read();
        // self.start_chunks_with_radius(10, seed, 0);
        // self.camera.lock().position = Vec3::new(0.0, 100.0, 0.0);
    }

    #[cfg(feature = "glfw")]
    pub fn keyboard(&mut self, key: Key, action: Action) {
        use crate::keybinds::{ABOUTTOREBIND, LISTENINGFORREBIND};

        {
            if action == Action::Press && unsafe { BUILD_PREVIEW_MODE && MOUSE_ON_CUBE } {
                unsafe { match key {
                    Key::Left => {
                        BUILD_MODEL_OFFSET.x -= 1;
                    }
                    Key::Right => {
                        BUILD_MODEL_OFFSET.x += 1;
                    }
                    Key::Up => {
                        BUILD_MODEL_OFFSET.y += 1;
                    }
                    Key::Down => {
                        BUILD_MODEL_OFFSET.y -= 1;
                    }
                    _ => {
    
                    }
                } }
            }
            
            match unsafe {
                MISCSETTINGS
                    .keybinds
                    .get(&key.get_scancode().unwrap_or(0))
                    .unwrap_or(&"_".to_string())
                    .as_str()
            } {
                "Build Mode Toggle" => {
                    if action == Action::Press {
                        unsafe{BUILD_PREVIEW_MODE = !BUILD_PREVIEW_MODE};
                    }
                }
                "Exit/Menu" => {
                    if action == Action::Press {
                        if !self.vars.menu_open && !self.hud.chest_open && !self.crafting_open {
                            self.button_command("escapemenu".to_string());
                        } else {
                            self.vars.menu_open = false;
                            self.window
                                .write()
                                .set_cursor_mode(glfw::CursorMode::Disabled);
                            self.set_mouse_focused(true);
                            unsafe {
                                uncapkb.store(true, Ordering::Relaxed);
                            }
                        }

                        if self.crafting_open {
                            self.crafting_open = false;
                            self.window
                                .write()
                                .set_cursor_mode(glfw::CursorMode::Disabled);
                            self.set_mouse_focused(true);
                            unsafe {
                                uncapkb.store(true, Ordering::Relaxed);
                            }
                        }

                        if self.hud.chest_open {
                            self.hud.chest_open = false;
                            self.window
                                .write()
                                .set_cursor_mode(glfw::CursorMode::Disabled);
                            self.set_mouse_focused(true);
                            unsafe {
                                uncapkb.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
                "Forward" => {
                    if action == Action::Press || action == Action::Repeat {
                        self.controls.forward = true;
                    } else {
                        self.controls.forward = false;
                    }
                }
                "Left" => {
                    if action == Action::Press || action == Action::Repeat {
                        self.controls.left = true;
                    } else {
                        self.controls.left = false;
                    }
                }
                "Craft" => {
                    if action == Action::Press {
                        unsafe {
                            ATSMALLTABLE = true;
                        }
                        Game::update_avail_recipes(&self.inventory);
                        self.crafting_open = true;

                        self.window
                            .write()
                            .set_cursor_mode(glfw::CursorMode::Normal);
                        self.set_mouse_focused(false);
                    } else {
                    }
                }
                "Backward" => {
                    if action == Action::Press || action == Action::Repeat {
                        self.controls.back = true;
                    } else {
                        self.controls.back = false;
                    }
                }
                "Right" => {
                    if action == Action::Press || action == Action::Repeat {
                        self.controls.right = true;
                    } else {
                        self.controls.right = false;
                    }
                }
                "Jump/Swim/Climb Up" => {
                    if action == Action::Press || action == Action::Repeat {
                        self.controls.up = true;
                    } else {
                        self.controls.up = false;
                    }
                }
                "Sprint" => {
                    if action == Action::Press || action == Action::Repeat {
                        self.controls.shift = true;
                    } else {
                        self.controls.shift = false;
                    }
                }
                "Crouch" => unsafe {
                    if action == Action::Press || action == Action::Repeat {
                        CROUCHING = true;
                    } else {
                        CROUCHING = false;
                    }
                },

                // Key::H => {
                //     if action == Action::Press || action == Action::Repeat {
                //         unsafe {
                //             GLCHUNKS = !GLCHUNKS;
                //         }
                //     }
                // }
                // Key::M => {
                //     if action == Action::Press {
                //         unsafe { WEATHERTYPE = WEATHERTYPE + 1.0 };
                //         if unsafe { WEATHERTYPE } > 2.0 {
                //             unsafe { WEATHERTYPE = 0.0 };
                //         }
                //     }
                // }
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
                //         self.chunksys.read().save_current_world_to_file(String::from("saves/world1"));
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
                //         let mut camlock = self.camera.lock();
                //         camlock.position = self.ship_pos + Vec3::new(5.0, 2.0, 0.0);
                //     }
                // }
                "Fov Up" => {
                    self.faders.write()[FaderNames::FovFader as usize].up();
                    self.faders.write()[FaderNames::FovFader as usize].top += 1.0;
                    self.faders.write()[FaderNames::FovFader as usize].bottom += 1.0;
                }
                "Fov Down" => {
                    self.faders.write()[FaderNames::FovFader as usize].down();
                    self.faders.write()[FaderNames::FovFader as usize].top -= 1.0;
                    self.faders.write()[FaderNames::FovFader as usize].bottom -= 1.0;
                }

                // Key::P => { //VISION
                //     if action == Action::Press
                //         && !self.faders.read()[FaderNames::VisionsFader as usize].mode
                //     {
                //         let mut rng = StdRng::from_entropy();
                //         self.current_vision =
                //             Some(VisionType::Model(rng.gen_range(2..self.gltf_models.len())));
                //         self.visions_timer = 0.0;
                //         self.faders.write()[FaderNames::VisionsFader as usize].up();
                //         #[cfg(feature = "audio")]
                //         unsafe {
                //             AUDIOPLAYER.play_in_head("assets/sfx/dreambell.mp3");
                //         }
                //     }
                // }

                // Key::L => {
                //     if action == Action::Press {
                //         self.vars.menu_open = !self.vars.menu_open;
                //     }

                // }
                // Key::O => {
                //     //self.faders.write()[FaderNames::VisionsFader as usize].down();
                // }
                _ => {}
            }
        }
    }
}
