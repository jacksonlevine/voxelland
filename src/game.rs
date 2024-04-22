use core::time;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Arc};
use std::thread;

use crate::chunk::{ChunkFacade, ChunkSystem};
use crate::texture::Texture;
use crate::vec::IVec2;
use crate::worldgeometry::WorldGeometry; 
use crate::shader::Shader;
use crate::camera::Camera;

pub struct Game {
    chunksys: Arc<Mutex<ChunkSystem>>,
    shader0: Shader,
    camera: Arc<Mutex<Camera>>,
    worldtexture: Texture,
    run_chunk_thread: AtomicBool,
    chunk_thread: Option<thread::JoinHandle<()>>
}


impl Game {

    pub fn new() -> Game {
        let shader0 = Shader::new("assets/vert.glsl", "assets/frag.glsl");
        unsafe {
            gl::BindVertexArray(shader0.vao);
        }
        let worldtexture = Texture::new("assets/world.png").unwrap();
        worldtexture.add_to_unit(0);
        Game {
            chunksys: Arc::new(Mutex::new(ChunkSystem::new(8))),
            shader0,
            camera: Arc::new(Mutex::new(Camera::new())),
            worldtexture,
            run_chunk_thread: AtomicBool::new(true),
            chunk_thread: None
        }
    }

    pub fn update(&mut self) {
        self.draw();
    }

    pub fn draw(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::BindVertexArray(self.shader0.vao);
        }

        let c_lock = self.chunksys.lock().unwrap();
        match c_lock.geoqueue.pop() {
            Some(index) => {
                WorldGeometry::bind_geometry(
                    c_lock.geobank[index].vbo32,
                    c_lock.geobank[index].vbo8, true,
                    &self.shader0, &c_lock.geobank[index]);
            },
            None => {

            }
        }
        drop(c_lock);

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
            gl::Uniform3f(CAM_POS_LOC, cam_lock.position.x, cam_lock.position.y, cam_lock.position.z);
            gl::Uniform1f(AMBIENT_BRIGHT_MULT_LOC, 1.0);
            gl::Uniform1f(VIEW_DISTANCE_LOC, 8.0);
            gl::Uniform1f(UNDERWATER_LOC, 0.0);
            gl::Uniform3f(CAM_DIR_LOC, cam_lock.direction.x, cam_lock.direction.y, cam_lock.direction.z);
            gl::Uniform1f(SUNSET_LOC, 0.0);
            gl::Uniform1f(SUNRISE_LOC, 0.0);
            gl::Uniform1i(gl::GetUniformLocation(self.shader0.shader_id, b"ourTexture\0".as_ptr() as *const i8), 0)
            drop(cam_lock);
        }
        let c_lock = self.chunksys.lock().unwrap();
        for cf in &c_lock.chunks {
            if cf.used {
                let cgeo = &c_lock.geobank[cf.geo_index];
                WorldGeometry::bind_geometry(cgeo.vbo32, cgeo.vbo8, false, &self.shader0, cgeo);
                unsafe {
                    gl::Uniform2f(C_POS_LOC, cgeo.pos.x as f32, cgeo.pos.y as f32);
                    gl::DrawArrays(gl::TRIANGLES, 0, cgeo.data32.len() as i32);
                }
            }
        }
    }

    pub fn start_world(&mut self) {
        let handle = thread::spawn(|| {
            self.chunk_thread_function();
        });
        self.chunk_thread = Some(handle);
    }

    pub fn chunk_thread_function(runcheck: &AtomicBool, cam_arc: Arc<Mutex<Camera>>, csys_arc: Arc<ChunkSystem>) {
        while runcheck.load(Ordering::Relaxed) {
            let mut neededspots: Vec<IVec2> = Vec::new();

            let cam_lock = cam_arc.lock().unwrap();
            let user_cpos = IVec2{x: (cam_lock.position.x / 15.0).floor() as i32, y: (cam_lock.position.y / 15.0).floor() as i32};
            drop(cam_lock);

            for i in -(csys_arc.radius as i8)..(csys_arc.radius as i8) {
                for k in -(csys_arc.radius as i8)..(csys_arc.radius as i8) {
                    let this_spot = IVec2{x: user_cpos.x + i as i32, y: user_cpos.y + k as i32};
                    if !csys_arc.takencare.contains_key(&this_spot) {
                        neededspots.push(this_spot);
                    }
                }
            }

            let mut sorted_chunk_facades: Vec<ChunkFacade> = Vec::new();
            sorted_chunk_facades.extend_from_slice(&csys_arc.chunks);


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
                csys_arc.move_and_rebuild(sorted_chunk_facades[index].geo_index, *ns) 
            }

            thread::sleep(time::Duration::from_secs(2));
        }
    }
}