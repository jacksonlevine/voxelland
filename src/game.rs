use core::time;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Arc};
use std::thread::{self, sleep};
use glfw::ffi::glfwGetTime;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, MouseButton, PWindow, Window, WindowEvent};
use glam::Vec3;

use crate::chunk::{ChunkFacade, ChunkGeo, ChunkSystem};
use crate::packedvertex::PackedVertex;
use crate::texture::Texture;
use crate::vec::{self, IVec2};
use crate::worldgeometry::WorldGeometry; 
use crate::shader::Shader;
use crate::camera::Camera;
use crate::fader::Fader;
use std::sync::RwLock;
use crate::collisioncage::*;

pub struct ControlsState {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub back: bool
}


pub struct GameVariables {
    first_mouse: bool,
    mouse_focused: bool,
    sensitivity: f32,
    movement_fov_cooldown: f32
}

pub struct Game {
    chunksys: Arc<ChunkSystem>,
    shader0: Shader,
    camera: Arc<Mutex<Camera>>,
    worldtexture: Texture,
    run_chunk_thread: Arc<AtomicBool>,
    chunk_thread: Option<thread::JoinHandle<()>>,
    vars: GameVariables,
    controls: ControlsState,
    faders: Arc<RwLock<Vec<Fader>>>,
    prev_time: f32,
    delta_time: f32,
    user_bound_box: BoundBox,
    coll_cage: CollCage 
}

enum FaderNames {
    FovFader = 0
}


impl Game {

    pub fn new() -> Game {
        let shader0 = Shader::new("assets/vert.glsl", "assets/frag.glsl");
        let faders: RwLock<Vec<Fader>> = RwLock::new(Vec::new());
        let cam = Arc::new(Mutex::new(Camera::new()));

        faders.write().unwrap().extend(vec![
            Fader::new(85.0, 80.0, 30.0, false)
        ]);

        unsafe {
            gl::BindVertexArray(shader0.vao);
            let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after binding vertex array: {}", error);
                            }
        }
        let worldtexture = Texture::new("assets/world.png").unwrap();
        worldtexture.add_to_unit(0);

        let chunksys = Arc::new(ChunkSystem::new(8));

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool> = {
            let csys_arc = Arc::clone(&chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.blockat(v) != 0;
            })
        };
        Game {
            chunksys,
            shader0,
            camera: cam,
            worldtexture,
            run_chunk_thread: Arc::new(AtomicBool::new(true)),
            chunk_thread: None,
            vars: GameVariables {
                first_mouse: true,
                mouse_focused: false,
                sensitivity: 0.25,
                movement_fov_cooldown: 0.1
            },
            controls: ControlsState {
                left: false,
                right: false,
                forward: false,
                back: false,
            },
            faders: Arc::new(faders),
            prev_time: 0.0,
            delta_time: 0.0,
            user_bound_box: BoundBox::new(Vec3::new(0.0,0.0,0.0), Vec3::new(0.0,0.0,0.0)),
            coll_cage: CollCage::new(solid_pred)
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
        if self.controls.forward || self.controls.back || self.controls.left || self.controls.right {
            if(!self.faders.read().unwrap()[FaderNames::FovFader as usize].mode) {
                self.faders.write().unwrap()[FaderNames::FovFader as usize].up();
            }
        } else {
            if(self.faders.read().unwrap()[FaderNames::FovFader as usize].mode) {
                self.faders.write().unwrap()[FaderNames::FovFader as usize].down();
            }
                
        }
            
                

        
        self.draw();

        let mut camlock = self.camera.lock().unwrap();

        let cc_center = camlock.position + Vec3::new(0.0, -1.0, 0.0);
        self.coll_cage.update_readings(cc_center);



        let mut proposed = camlock.respond_to_controls(&self.controls, &self.delta_time, 10.0);
        self.user_bound_box.set_center(proposed + Vec3::new(0.0, -0.5, 0.0), 0.85, 0.2);
        self.coll_cage.update_colliding(&self.user_bound_box);
        let mut corr_made: Vec<Vec3> = Vec::new();
        if self.coll_cage.colliding.len() > 0 {
            for side in &self.coll_cage.colliding {
                if !corr_made.contains(&self.coll_cage.normals[*side as usize]) {
                    proposed += self.coll_cage.normals[*side as usize] * self.coll_cage.penetrations[*side as usize];
                    corr_made.push(self.coll_cage.normals[*side as usize]);
                }
            }
        }
        camlock.position = proposed;
        camlock.recalculate();
    }

    fn print_matrix(mvp: &glam::Mat4) {
        println!("MVP Matrix:");
        for i in 0..4 {
            println!(
                "[{:>10.3}, {:>10.3}, {:>10.3}, {:>10.3}]",
                mvp.col(i).x, mvp.col(i).y, mvp.col(i).z, mvp.col(i).w
            );
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after clear color bit: {}", error);
                            }
            gl::ClearColor(0.5, 0.7, 1.0, 1.0);
            let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after clear color: {}", error);
                            }
            gl::BindVertexArray(self.shader0.vao);
            let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after binding v array: {}", error);
                            }
            gl::UseProgram(self.shader0.shader_id);
            let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after saying to use shader rogram: {}", error);
                            }
        }

        let gqarc = self.chunksys.geoqueue.clone();


                match gqarc.pop() {
                    Some(index) => {
                        let bankarc = self.chunksys.geobank[index].clone();
                        let banklock = bankarc.lock().unwrap();
                        WorldGeometry::bind_geometry(
                            banklock.vbo32,
                            banklock.vbo8, true,
                            &self.shader0, &banklock);
                            #[cfg(feature = "yap_about_chunks")]
                            println!("Chunk popped!");
                    },
                    None => {

                    }
                } 


                static mut C_POS_LOC: i32 = 0;
                static mut MVP_LOC: i32 = 0;
                static mut CAM_POS_LOC: i32 = 0;
                static mut AMBIENT_BRIGHT_MULT_LOC: i32 = 0;
                static mut VIEW_DISTANCE_LOC: i32 = 0;
                static mut UNDERWATER_LOC: i32 = 0;
                static mut CAM_DIR_LOC: i32 = 0;
                static mut SUNSET_LOC: i32 = 0;
                static mut SUNRISE_LOC: i32 = 0;
                unsafe {
                    if C_POS_LOC == 0 {
                        C_POS_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"chunkpos\0".as_ptr() as *const i8);
                        MVP_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"mvp\0".as_ptr() as *const i8);
                        //println!("MVP LOC: {}", MVP_LOC);
                        CAM_POS_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"camPos\0".as_ptr() as *const i8);
                        AMBIENT_BRIGHT_MULT_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"ambientBrightMult\0".as_ptr() as *const i8);
                        VIEW_DISTANCE_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"viewDistance\0".as_ptr() as *const i8);
                        UNDERWATER_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"underWater\0".as_ptr() as *const i8);
                        CAM_DIR_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"camDir\0".as_ptr() as *const i8);
                        SUNSET_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"sunset\0".as_ptr() as *const i8);
                        SUNRISE_LOC = gl::GetUniformLocation(self.shader0.shader_id, b"sunrise\0".as_ptr() as *const i8);
                    }
                    let cam_lock = self.camera.lock().unwrap();


                    gl::UniformMatrix4fv(MVP_LOC, 1, gl::FALSE, cam_lock.mvp.to_cols_array().as_ptr());

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming mvp: {}", error);
                            }

                    gl::Uniform3f(CAM_POS_LOC, cam_lock.position.x, cam_lock.position.y, cam_lock.position.z);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming campos: {}", error);
                            }


                    gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, 1.0);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming amb bright: {}", error);
                            }


                    gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming view dist: {}", error);
                            }


                    gl::Uniform1f(UNDERWATER_LOC, 0.0);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming uw: {}", error);
                            }


                    gl::Uniform3f(CAM_DIR_LOC, cam_lock.direction.x, cam_lock.direction.y, cam_lock.direction.z);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming cam dir: {}", error);
                            }


                    gl::Uniform1f(SUNSET_LOC, 0.0);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming sunset: {}", error);
                            }


                    gl::Uniform1f(SUNRISE_LOC, 0.0);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming sunrise: {}", error);
                            }


                    gl::Uniform1i(gl::GetUniformLocation(self.shader0.shader_id, b"ourTexture\0".as_ptr() as *const i8), 0);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after uniforming ourTexture unit number: {}", error);
                            }


                    drop(cam_lock);
                }
                for cfarc in &self.chunksys.chunks {
                    match cfarc.try_lock() {
                        Ok(cfl) => {
                            if cfl.used {
                                let bankarc = self.chunksys.geobank[cfl.geo_index].clone();
                                let banklock = bankarc.lock().unwrap();

                                WorldGeometry::bind_geometry(banklock.vbo32, banklock.vbo8, true, &self.shader0, &banklock);
                                unsafe {
                                    gl::Uniform2f(C_POS_LOC, banklock.pos.x as f32, banklock.pos.y as f32);
                                    let error = unsafe { gl::GetError() };
                                    if error != gl::NO_ERROR {
                                        println!("OpenGL Error after uniforming the chunk pos: {}", error);
                                    }
                                    //println!("Rendering {} in chunk at {}, {}", banklock.data32.len(), banklock.pos.x, banklock.pos.y);
                                    gl::DrawArrays(gl::TRIANGLES, 0, banklock.data32.len() as i32);
                                    let error = unsafe { gl::GetError() };
                                    if error != gl::NO_ERROR {
                                        println!("OpenGL Error after drawing arrays: {}", error);
                                    }
                                   // println!("Chunk rending!");
                                
                                }
                            }
                        },
                        Err(e) => {

                        }
                    }
                    
                }


            
    }

    pub fn start_world(&mut self) {

        (*self.run_chunk_thread).store(true, Ordering::Relaxed);

        let rctarc = self.run_chunk_thread.clone();
        let carc = self.camera.clone();
        let csysarc = self.chunksys.clone();

        let handle = thread::spawn(move || {
            Game::chunk_thread_function(&rctarc, carc, csysarc);
        });
        
        self.chunk_thread = Some(handle);


    }

    pub fn chunk_thread_function(runcheck: &AtomicBool, cam_arc: Arc<Mutex<Camera>>, csys_arc: Arc<ChunkSystem>) {

        //static mut TEMP_COUNT: i32 = 0;

        while runcheck.load(Ordering::Relaxed) {
            
            let mut neededspots: Vec<IVec2> = Vec::new();

            let cam_lock = cam_arc.lock().unwrap();
            let user_cpos = IVec2{x: (cam_lock.position.x / 15.0).floor() as i32, y: (cam_lock.position.z / 15.0).floor() as i32};
            drop(cam_lock);

            let tcarc = csys_arc.takencare.clone();
            let tclock = tcarc.lock().unwrap();
            for i in -(csys_arc.radius as i32)..(csys_arc.radius as i32) {
                for k in -(csys_arc.radius as i32)..(csys_arc.radius as i32) {
                    let this_spot = IVec2{x: user_cpos.x + i as i32, y: user_cpos.y + k as i32};
                    if !tclock.contains_key(&this_spot) {
                        neededspots.push(this_spot);
                    }
                }
            }
            drop(tclock);

            let mut sorted_chunk_facades: Vec<ChunkFacade> = Vec::new();

            for carc in &csys_arc.chunks {
                match carc.try_lock() {
                    Ok(cf) => {

                        sorted_chunk_facades.push(*cf);
                    },
                    Err(_) => {

                    }
                }
            }

            let (unused_or_distant, used_and_close): (Vec<ChunkFacade>, Vec<ChunkFacade>) = sorted_chunk_facades.drain(..)
                .partition(|chunk| {
                    if !chunk.used {
                        true
                    } else {
                        let dist = (chunk.pos.x - user_cpos.x).abs() + 
                                (chunk.pos.y - user_cpos.y).abs();
                        dist >= csys_arc.radius as i32 * 2
                    }
                });

            sorted_chunk_facades.extend(unused_or_distant);
            sorted_chunk_facades.extend(used_and_close);


            for (index, ns) in neededspots.iter().enumerate() {
                //unsafe {
                 //   if TEMP_COUNT == 0 {
                        csys_arc.move_and_rebuild(sorted_chunk_facades[index].geo_index, *ns);
                   //     TEMP_COUNT += 1;
                  //  }
               // }
                
            }
            thread::sleep(time::Duration::from_secs(2));
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

                camlock.direction.x = camlock.yaw.to_radians().cos() as f32 * camlock.pitch.to_radians().cos() as f32;
                camlock.direction.y = camlock.pitch.to_radians().sin();
                camlock.direction.z = camlock.yaw.to_radians().sin() * camlock.pitch.to_radians().cos();
                camlock.direction = camlock.direction.normalize();
    
                camlock.right = Vec3::new(0.0, 1.0, 0.0).cross(camlock.direction).normalize();
                camlock.up = camlock.direction.cross(camlock.right).normalize();
    
                camlock.recalculate();
                #[cfg(feature = "show_cam_pos")]
                println!("Cam dir: {}, {}, {}", camlock.direction.x, camlock.direction.y, camlock.direction.z);
            }
            
        }
    }
    pub fn set_mouse_focused(&mut self, tf: bool) {
        if tf {
            self.vars.mouse_focused = true;
        } else {
            self.vars.mouse_focused = false;
        }
    }
    pub fn mouse_button(&mut self, mb: MouseButton, a: Action) {

    }
    pub fn keyboard(&mut self, key: Key, action: Action) {
        match key {
            Key::W => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.forward = true;
                } else {
                    self.controls.forward = false;
                }
            },
            Key::A => {
                if action == Action::Press || action == Action::Repeat {
                    self.controls.left = true;
                } else {
                    self.controls.left = false;
                }
            },
            Key::S => {
                
                if action == Action::Press || action == Action::Repeat {
                    self.controls.back= true;
                } else {
                    self.controls.back = false;
                }
            },
            Key::D => {
                
                if action == Action::Press || action == Action::Repeat {
                    self.controls.right = true;
                } else {
                    self.controls.right = false;
                }

            },
            Key::Num0 => {
                let mut camlock = self.camera.lock().unwrap();
                let f = camlock.fov;
                camlock.update_fov(f + 1.0);
            },
            Key::Num9 => {
                let mut camlock = self.camera.lock().unwrap();
                let f = camlock.fov;
                camlock.update_fov(f - 1.0);
            },
            _ => {

            }
        }
    }
}