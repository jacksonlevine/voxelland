use core::time;
use gl::types::{GLenum, GLuint};
use glam::{Vec3, Vec4};
use glfw::ffi::glfwGetTime;
use glfw::{Action, Key, MouseButton};
use gltf::Gltf;
use walkdir::WalkDir;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, current};

use crate::chunk::{ChunkFacade, ChunkSystem};

use crate::camera::Camera;
use crate::collisioncage::*;
use crate::fader::Fader;
use crate::planetinfo::Planets;
use crate::raycast::*;
use crate::shader::Shader;
use crate::texture::Texture;
use crate::vec::{self, IVec2, IVec3};
use crate::voxmodel::JVoxModel;
use crate::worldgeometry::WorldGeometry;
use std::sync::RwLock;

pub struct ControlsState {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub back: bool,
    pub up: bool,
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
    hostile_world_sky_bottom: Vec4
}

pub struct Game {
    chunksys: Arc<ChunkSystem>,
    shader0: Shader,
    skyshader: Shader,
    pub modelshader: Shader,
    pub camera: Arc<Mutex<Camera>>,
    run_chunk_thread: Arc<AtomicBool>,
    chunk_thread: Option<thread::JoinHandle<()>>,
    vars: GameVariables,
    controls: ControlsState,
    faders: Arc<RwLock<Vec<Fader>>>,
    prev_time: f32,
    delta_time: f32,
    user_bound_box: BoundBox,
    coll_cage: CollCage,
    grounded: bool,
    jumping_up: bool,
    time_falling_scalar: f32,
    current_jump_y: f32,
    allowable_jump_height: f32,
    pub initial_timer: f32,
    pub voxel_models: Arc<Vec<JVoxModel>>,
    pub gltf_models: Vec<(gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>)>,
    pub gltf_vbos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_vaos: Vec<Vec<Vec<GLuint>>>,
    pub gltf_counts: Vec<Vec<Vec<usize>>>,
    pub gltf_drawmodes:Vec<Vec<Vec<GLenum>>>,
    pub gltf_ebos: Vec<Vec<Vec<GLuint>>>
}

enum FaderNames {
    FovFader = 0,
}

impl Game {
    pub fn new() -> Game {
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
        ];

        


        let mut csys =ChunkSystem::new(10, 1, 0);

        let vmarc = Arc::new(voxel_models);
        let vmarc2 = vmarc.clone();

        csys.voxel_models = Some(vmarc);

        let chunksys = Arc::new(csys);

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool> = {
            let csys_arc = Arc::clone(&chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.blockat(v) != 0;
            })
        };
        let mut g = Game {
            chunksys,
            shader0,
            skyshader,
            modelshader: Shader::new("assets/mvert.glsl", "assets/mfrag.glsl"),
            camera: cam,
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
            },
            controls: ControlsState {
                left: false,
                right: false,
                forward: false,
                back: false,
                up: false,
            },
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
            gltf_ebos: Vec::new()
        };
        g.load_model("assets/models/car/scene.gltf");
        g.create_model_vbos();
        // g.setup_vertex_attributes();
        g
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

        if(self.initial_timer < 1.5) {
            self.initial_timer += self.delta_time;
        } else {
            self.update_movement_and_physics();
        }

        
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
            .set_center(proposed + Vec3::new(0.0, -0.5, 0.0), 0.2, 0.85);
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
        camlock.position = proposed;
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

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
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
        let ugqarc = self.chunksys.finished_user_geo_queue.clone();

        match ugqarc.pop() {
            Some(ready) => {
                //println!("Some user queue");
               // println!("Weird!");

                let bankarc = self.chunksys.geobank[ready.geo_index].clone();

                let mut cmemlock = self.chunksys.chunk_memories.lock().unwrap();

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

        let gqarc = self.chunksys.finished_geo_queue.clone();

        match gqarc.pop() {
            Some(ready) => {

                //println!("Weird!");

                let bankarc = self.chunksys.geobank[ready.geo_index].clone();

                let mut cmemlock = self.chunksys.chunk_memories.lock().unwrap();

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
                
                                let bankarc = self.chunksys.geobank[ready.geo_index].clone();
                
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
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.shader0.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                0,
            );
            let fc = Planets::get_fog_col(self.chunksys.noise_type as u32);
            gl::Uniform4f(
                FOGCOL_LOC,
                fc.0, 
                fc.1,
                fc.2,
                fc.3
            );

            drop(cam_lock);
        }


        let cmem = self.chunksys.chunk_memories.lock().unwrap();
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

        self.draw_models();

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

    
    pub fn start_chunks_with_radius(&mut self, newradius: u8, seed: u32, nt: usize) {

        (*self.run_chunk_thread).store(false, Ordering::Relaxed);

        if let Some(handle) = self.chunk_thread.take() { // take the handle out safely
            handle.join().unwrap(); // Join the thread, handle errors appropriately
            println!("Thread joined successfully!");
        } else {
            println!("No thread to join or already joined.");
        }
        
        for i in &self.chunksys.geobank {
            unsafe {
                gl::DeleteBuffers(1, &i.vbo32);
                gl::DeleteBuffers(1, &i.tvbo32);
                gl::DeleteBuffers(1, &i.vbo8);
                gl::DeleteBuffers(1, &i.tvbo8);
            }
        }
        let mut csys = ChunkSystem::new(newradius, seed, nt);
        csys.voxel_models = Some(self.voxel_models.clone());
        self.chunksys = Arc::new(csys);

        self.coll_cage.solid_pred  = {
            let csys_arc = Arc::clone(&self.chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.blockat(v) != 0;
            })
        };

        self.start_world();
    }

    pub fn chunk_thread_inner_function(cam_arc: &Arc<Mutex<Camera>>, csys_arc: &Arc<ChunkSystem>, last_user_c_pos: &mut vec::IVec2) {




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
        let mut backgroundstuff = true;
        while backgroundstuff {
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

            
            static mut time_since_last_check: f32 = 2.0;

            let user_c_pos = ChunkSystem::spot_to_chunk_pos(&IVec3::new(
                vec3.x.floor() as i32,
                vec3.y.floor() as i32,
                vec3.z.floor() as i32,
            ));

            if user_c_pos != *last_user_c_pos && time_since_last_check >= 2.0 {
                *last_user_c_pos = user_c_pos;

                time_since_last_check = 0.0;

                let mut neededspots: Vec<IVec2> = Vec::new();

                let cam_lock = cam_arc.lock().unwrap();
                let user_cpos = IVec2 {
                    x: (cam_lock.position.x / 15.0).floor() as i32,
                    y: (cam_lock.position.z / 15.0).floor() as i32,
                };
                drop(cam_lock);

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
                }
            } else {
                time_since_last_check += delta_time;
            }
        }
    }

    pub fn chunk_thread_function(
        runcheck: &AtomicBool,
        cam_arc: Arc<Mutex<Camera>>,
        csys_arc: Arc<ChunkSystem>,
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
    pub fn mouse_button(&mut self, mb: MouseButton, a: Action) {
        match mb {
            glfw::MouseButtonLeft => {
                self.vars.mouse_clicked = a == Action::Press;
                if self.vars.mouse_clicked {
                    let cl = self.camera.lock().unwrap();
                    match raycast_dda(cl.position, cl.direction, &self.chunksys, 10.0) {
                        Some((_tip, block_hit)) => {
                            self.chunksys.set_block_and_queue_rerender(block_hit, 0, true, true);
                        }
                        None => {}
                    }
                }
            }
            glfw::MouseButtonRight => {
                self.vars.right_mouse_clicked = a == Action::Press;
                if self.vars.right_mouse_clicked {
                    let cl = self.camera.lock().unwrap();
                    match raycast_dda(cl.position, cl.direction, &self.chunksys, 10.0) {
                        
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
                            self.chunksys.set_block_and_queue_rerender(place_point, 1, false, true);
                        }

                        None => {}
                    }
                }
            }
            _ => {}
        }
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
                    static SEEDS: [u8; 12] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
                    static mut CURR_SEED: usize = 0;
                    static mut CURR_NT: usize = 0;
                    self.camera.lock().unwrap().position = Vec3::new(0.0, 100.0, 0.0);
                    unsafe {
                        self.vars.hostile_world = (CURR_SEED % 2) == 0;
                        CURR_NT = (CURR_NT + 1) % 2;
                        self.start_chunks_with_radius(10, CURR_SEED as u32, CURR_NT);
                        CURR_SEED = (CURR_SEED + 1) % 11;
                        
                        println!("Now noise type is {}", self.chunksys.noise_type);
                    }
                }
            }
            _ => {}
        }
    }
}
