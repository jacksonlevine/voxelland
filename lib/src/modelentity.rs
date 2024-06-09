

use std::{ops::Bound, sync::{Arc, Mutex, RwLock}};

use dashmap::DashMap;
use glam::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use uuid::Uuid;

pub enum AggroTarget {
    ThisCamera,
    ModelEntityID(u32),
    NoAggro,
    UUID(Uuid)
}

/// Converts a direction vector to Euler angles (yaw, pitch, roll).
pub fn direction_to_euler(direction: Vec3) -> Vec3 {
    // Normalize the input direction
    let direction = direction.normalize();

    // Calculate yaw (rotation around Y axis)
    let yaw = direction.x.atan2(direction.z);

    // Calculate pitch (rotation around X axis)
    let pitch = (-direction.y).asin();

    // Roll is typically zero when converting a direction to Euler angles (depends on the use case)
    let roll = 0.0;

    Vec3::new(pitch, yaw, roll)
}

/// Converts a Vec3 containing Euler angles (in radians) to a normalized direction vector.
pub fn euler_to_direction(euler_angles: Vec3) -> Vec3 {
    // Create a quaternion from the Euler angles
    let quat = Quat::from_euler(
        EulerRot::XYZ,  // Rotation order: first pitch (X), then yaw (Y), then roll (Z)
        euler_angles.x, // Yaw around the Y-axis
        euler_angles.y, // Pitch around the X-axis
        euler_angles.z  // Roll around the Z-axis
    );

    // Assume forward direction is along positive z-axis
    let forward = Vec3::new(0.0, 0.0, 1.0);

    // Rotate the forward vector by the quaternion
    let direction = quat * forward;

    // Normalize the direction vector to handle any numerical inaccuracies
    direction.normalize()
}


use crate::{camera::Camera, chunk::ChunkSystem, collisioncage::{BoundBox, CollCage}, game::{Animation, ControlsState, Node}, raycast::{self, raycast_voxel}, vec};

static mut CURRENT_ID: u32 = 0;

pub struct ModelEntity {
    pub model_index: usize,
    pub position: Vec3,
    pub lastpos: Vec3,
    pub id: u32,
    pub scale: f32,
    pub rot: Vec3,
    pub coll_cage: CollCage,
    pub velocity: Vec3,
    pub grounded: bool,
    pub time_falling_scalar: f32,
    pub jumping_up: bool,
    pub allowable_jump_height: f32,
    pub current_jump_y: f32,
    pub bound_box: BoundBox,
    pub controls: ControlsState,
    pub direction: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub behavior_timer: f32,
    pub rng: StdRng,
    pub csys: Arc<RwLock<ChunkSystem>>,
    pub cam: Arc<Mutex<Camera>>,
    pub target: AggroTarget,
    pub speedfactor: f32,
    pub soundtimer: f32,
    pub was_grounded: bool,
    pub current_animation: Option<usize>,
    pub animation_time: f32,
    pub animations: Vec<Animation>,
    pub nodes: Vec<Node>,
    pub time_stamp: f64,
    pub hostile: bool,
    pub lastrot: Vec3
}

impl ModelEntity {
    
    pub fn new_with_jump_height(model_index: usize, pos: Vec3, scale: f32, rot: Vec3, csys: &Arc<RwLock<ChunkSystem>>, cam: &Arc<Mutex<Camera>>, jump_height: f32) -> ModelEntity {
        let mut modent = ModelEntity::new(model_index, pos, scale, rot, csys, cam);
        modent.allowable_jump_height = jump_height;
        modent
    }

    pub fn new(model_index: usize, pos: Vec3, scale: f32, rot: Vec3, csys: &Arc<RwLock<ChunkSystem>>, cam: &Arc<Mutex<Camera>>) -> ModelEntity {

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> = {
            //let csys_arc = Arc::clone(&chunksys);
            Box::new(move |_: vec::IVec3| {
                return false;
            })
        };


        unsafe {
            CURRENT_ID += 1;
            ModelEntity {
                model_index,
                position: pos,
                lastpos: pos,
                id: CURRENT_ID + 1,
                scale,
                rot,
                coll_cage: CollCage::new(solid_pred),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                grounded: false,
                time_falling_scalar: 1.0,
                jumping_up: false,
                allowable_jump_height: 7.0,
                current_jump_y: 0.0,
                bound_box: BoundBox::new(Vec3::new(0.0,0.0,0.0)),
                controls: ControlsState::new(),
                direction: Vec3::new(0.0, 0.0, 0.0),
                right: Vec3::new(0.0, 0.0, 0.0),
                up: Vec3::new(0.0, 1.0, 0.0),
                behavior_timer: 0.0,
                rng: StdRng::from_entropy(),
                csys: csys.clone(),
                cam: cam.clone(),
                target: AggroTarget::NoAggro,
                speedfactor: 1.0,
                soundtimer: 0.0,
                was_grounded: false,
                current_animation: None,
                animation_time: 0.0,
                animations: Vec::new(),
                nodes: Vec::new(),
                time_stamp: 0.0,
                hostile: false,
                lastrot: Vec3::ZERO
            }
        }
        
    }



    pub fn new_with_id(id: u32, model_index: usize, pos: Vec3, scale: f32, rot: Vec3, csys: &Arc<RwLock<ChunkSystem>>, cam: &Arc<Mutex<Camera>>) -> ModelEntity {

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> = {
            //let csys_arc = Arc::clone(&chunksys);
            Box::new(move |_: vec::IVec3| {
                return false;
            })
        };



            ModelEntity {
                model_index,
                position: pos,
                lastpos: pos,
                id,
                scale,
                rot,
                coll_cage: CollCage::new(solid_pred),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                grounded: false,
                time_falling_scalar: 1.0,
                jumping_up: false,
                allowable_jump_height: 7.0,
                current_jump_y: 0.0,
                bound_box: BoundBox::new(Vec3::new(0.0,0.0,0.0)),
                controls: ControlsState::new(),
                direction: Vec3::new(0.0, 0.0, 0.0),
                right: Vec3::new(0.0, 0.0, 0.0),
                up: Vec3::new(0.0, 1.0, 0.0),
                behavior_timer: 0.0,
                rng: StdRng::from_entropy(),
                csys: csys.clone(),
                cam: cam.clone(),
                target: AggroTarget::NoAggro,
                speedfactor: 1.0,
                soundtimer: 0.0,
                was_grounded: false,
                current_animation: None,
                animation_time: 0.0,
                animations: Vec::new(),
                nodes: Vec::new(),
                time_stamp: 0.0,
                hostile: false,
                lastrot: Vec3::ZERO
            }
     
        
    }

    pub fn recalculate(&mut self) {
        self.direction = euler_to_direction(self.rot);
        self.right = Vec3::new(0.0, 1.0, 0.0).cross(self.direction).normalize();
        self.up = self.direction.cross(self.right);
    }

    pub fn random_behavior(&mut self, delta: &f32) {
        let rand = self.rng.gen_range(0..6);

        match rand {
            0 => {
                self.controls.clear();
                let res = raycast_voxel(self.position, self.direction, &self.csys, 5.0);
                match res {
                    Some(res) => {
                        self.controls.lookingleft = true;
                    }
                    None => {
                        self.controls.up = true;
                        self.controls.forward = true;
                    }
                }
                
            },
            2 => {
                self.controls.clear();
                let res = raycast_voxel(self.position, self.direction, &self.csys, 5.0);
                match res {
                    Some(res) => {
                        self.controls.lookingright = true;
                    }
                    None => {
                        self.controls.up = true;
                        self.controls.forward = true;
                    }
                }
            },
            3 => {
                self.controls.clear();
            },
            _ => {
                
            }
        }
    }

    pub fn behavior_loop(&mut self, delta: &f32, knowncams: &Arc<DashMap<Uuid, Vec3>>) {
        
        if self.behavior_timer < 1.0 {
            self.behavior_timer += delta;
        } else {
            match self.target {
                AggroTarget::NoAggro => {
                    self.speedfactor = 1.0;
                    self.random_behavior(delta);
                }
                AggroTarget::ModelEntityID(id) => {
                    //let modent = 
                }
                AggroTarget::ThisCamera => {
                    self.speedfactor = 2.5;
                    let campos = self.cam.lock().unwrap().position;
                    let mut diff = campos - self.position;
                    diff.y = 0.0;
                    self.set_direction(diff.normalize());
                    self.controls.up = true;
                    self.controls.forward = true;
                }
                AggroTarget::UUID(targ_id) => {
                    self.speedfactor = 2.5;
                    let campos = match knowncams.get(&targ_id) {
                        Some(vec3) => {
                            *vec3.value()
                        }
                        None => {
                            self.target = AggroTarget::NoAggro;
                            Vec3::ZERO
                        }
                    };
                    let mut diff = campos - self.position;
                    diff.y = 0.0;
                    self.set_direction(diff.normalize());
                    self.controls.up = true;
                    self.controls.forward = true;
                },
            }
            
            
            self.behavior_timer = 0.0;

        }
        
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        let eul = direction_to_euler(dir);

        self.rot = eul;
    }

    //Mutate own velocity based on internal controlsstate
    pub fn respond_to_own_controls(
        &mut self,
        delta: &f32,
        speed_mult: f32,
    ) {
        
        self.recalculate();


        

        if self.controls.forward {
            self.velocity += (self.direction * Vec3::new(1.0, 0.0, 1.0)).normalize() * *delta * speed_mult * self.speedfactor;
        }
        if self.controls.lookingleft {
            self.rot.y += 1.0 * *delta;
        }
        if self.controls.lookingright {
            self.rot.y -= 1.0 * *delta;
        }
        
        if self.velocity.length() > 0.0 {
            let amt_to_subtract = self.velocity * *delta * speed_mult;
            self.velocity -= amt_to_subtract;
        }
        #[cfg(feature = "show_mob_pos")]
        println!(
            "Mob cam pos: {}, {}, {}",
            self.position.x, self.position.y, self.position.z
        );
    }



    pub fn set_pos(&mut self, newpos: Vec3) {
        self.lastpos = self.position;
        self.position = newpos;
    }
}