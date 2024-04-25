use core::time;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Arc};
use std::thread;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, MouseButton, PWindow, Window, WindowEvent};
use glam::Vec3;

use crate::chunk::{ChunkFacade, ChunkGeo, ChunkSystem};
use crate::packedvertex::PackedVertex;
use crate::texture::Texture;
use crate::vec::IVec2;
use crate::worldgeometry::WorldGeometry; 
use crate::shader::Shader;
use crate::camera::Camera;

pub struct ControlsState {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub back: bool
}


pub struct GameVariables {
    first_mouse: bool,
    mouse_focused: bool,
    sensitivity: f32
}

pub struct Game {
    chunksys: Arc<ChunkSystem>,
    shader0: Shader,
    camera: Arc<Mutex<Camera>>,
    worldtexture: Texture,
    run_chunk_thread: Arc<AtomicBool>,
    chunk_thread: Option<thread::JoinHandle<()>>,
    vars: GameVariables,
    controls: ControlsState
}


impl Game {

    pub fn new() -> Game {
        let shader0 = Shader::new("assets/vert.glsl", "assets/frag.glsl");
        unsafe {
            gl::BindVertexArray(shader0.vao);
            let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after binding vertex array: {}", error);
                            }
        }
        let worldtexture = Texture::new("assets/world.png").unwrap();
        worldtexture.add_to_unit(0);
        Game {
            chunksys: Arc::new(ChunkSystem::new(8)),
            shader0,
            camera: Arc::new(Mutex::new(Camera::new())),
            worldtexture,
            run_chunk_thread: Arc::new(AtomicBool::new(true)),
            chunk_thread: None,
            vars: GameVariables {
                first_mouse: true,
                mouse_focused: false,
                sensitivity: 0.25
            },
            controls: ControlsState {
                left: false,
                right: false,
                forward: false,
                back: false,
            }
        }
    }

    pub fn update(&mut self) {
        
        self.draw();
        self.camera.lock().unwrap().respond_to_controls(&self.controls);
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

        // static mut TEMP_COUNT: i32 = 0;
        // unsafe {
            
        //     if TEMP_COUNT == 0 {

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
                            //TEMP_COUNT += 1;
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
                    //Game::print_matrix(&cam_lock.mvp);
                    //while gl::GetError() != gl::NO_ERROR {}


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

                // static mut geo: ChunkGeo = ChunkGeo{
                //     data32: Vec::new(),
                //     data8: Vec::new(),
                //     pos: IVec2{x:0, y:0},
                //     vbo32: 0,
                //     vbo8: 0
                // };
                // unsafe {
                //     if geo.vbo32 == 0 {
                //         unsafe {
                //             gl::CreateBuffers(1, &mut geo.vbo32);
                //             gl::CreateBuffers(1, &mut geo.vbo8);
                //         }
                //         geo.data32 = vec![
                //             PackedVertex::pack(0, 0, 0, 0, 15, 0, 0, 0).0,
                //             PackedVertex::pack(10, 0, 0, 1, 15, 0, 0, 0).0,
                //             PackedVertex::pack(10, 10, 0, 2, 15, 0, 0, 0).0,

                //             PackedVertex::pack(10, 10, 0, 3, 15, 0, 0, 0).0,
                //             PackedVertex::pack(0, 10, 0, 4, 15, 0, 0, 0).0,
                //             PackedVertex::pack(0, 0, 0, 5, 15, 0, 0, 0).0,
                //         ];

                //         geo.data8 = vec![
                //             PackedVertex::pack(0, 0, 0, 0, 15, 0, 0, 0).1,
                //             PackedVertex::pack(10, 0, 0, 1, 15, 0, 0, 0).1,
                //             PackedVertex::pack(10, 10, 0, 2, 15, 0, 0, 0).1,

                //             PackedVertex::pack(10, 10, 0, 3, 15, 0, 0, 0).1,
                //             PackedVertex::pack(0, 10, 0, 4, 15, 0, 0, 0).1,
                //             PackedVertex::pack(0, 0, 0, 5, 15, 0, 0, 0).1,
                //         ];
                //     }
                // }
                

                

                // unsafe {
                //     gl::Uniform2f(C_POS_LOC, 0 as f32, 0 as f32);
                //     WorldGeometry::bind_geometry(geo.vbo32, geo.vbo8, true, &self.shader0, &geo);
                //     gl::DrawArrays(gl::TRIANGLES, 0, geo.data32.len() as i32);
                // }


                
        //     }
        // }
            
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
    
                camlock.right = camlock.direction.cross(Vec3{x: 0.0, y: 1.0, z: 0.0}).normalize();
                camlock.up = camlock.direction.cross(camlock.right);
    
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
                self.controls.forward = (action == Action::Press || action == Action::Repeat);
            },
            Key::A => {
                self.controls.left = (action == Action::Press || action == Action::Repeat);
            },
            Key::S => {
                self.controls.back = (action == Action::Press || action == Action::Repeat);
            },
            Key::D => {
                self.controls.right = (action == Action::Press || action == Action::Repeat);
            },
            _ => {

            }
        }
    }
}