

use std::{fs, path::Path, sync::Arc};
use tracing::info;
use dashmap::DashMap;
use gl::types::{GLsizeiptr, GLuint, GLvoid};
use glam::{Mat4, Vec3, Vec4};
use glfw::ffi::glfwGetTime;
use gltf::{accessor::{Dimensions}, image::Source, mesh::util::ReadIndices};
use rand::{rngs::StdRng, Rng, SeedableRng};
use uuid::Uuid;
use crate::{planetinfo::Planets};
use gltf::{animation::util::ReadOutputs};
use crate::{collisioncage::{CollCage, Side}, game::*, modelentity::{AggroTarget, ModelEntity}, vec};
use percent_encoding::percent_decode_str;



fn num_components(dimensions: Dimensions) -> i32 {
    match dimensions {
        Dimensions::Scalar => 1,
        Dimensions::Vec2 => 2,
        Dimensions::Vec3 => 3,
        Dimensions::Vec4 => 4,
        Dimensions::Mat2 => 4, 
        Dimensions::Mat3 => 9, 
        Dimensions::Mat4 => 16,
    }
}

fn load_document_textures(document: &gltf::Document, buffers: &[gltf::buffer::Data], base_path: &str) -> Vec<GLuint> {
    document.images().map(|image| {
        let data = match image.source() {
            Source::Uri { uri, mime_type: _ } => {
                // External image: Load from a file
                let decoded_uri = percent_decode_str(uri).decode_utf8_lossy(); // Decode the URI
                let path = format!("{}/{}", base_path, decoded_uri); // Use the decoded URI to form the path
                info!("Loading external image: {}", decoded_uri);

                match fs::read(&path) {
                    Ok(data) => {
                        // Proceed with using the image data
                        info!("Image loaded successfully.");
                        data
                    },
                    Err(e) => {
                        // Handle errors, e.g., file not found
                        panic!("Failed to read image file: {:?}", e);
                        Vec::new()
                    }
                }
            },
            Source::View { view, mime_type: _ } => {
                // Embedded image: Get data from buffer
                let buffer_index = view.buffer().index();
                let start = view.offset();
                let end = start + view.length();
                info!("Loading embedded image from buffer index: {}", buffer_index); 
                buffers[buffer_index][start..end].to_vec()
            },
        };

        let img = image::load_from_memory(&data)
            .expect("Failed to decode image")
            .to_rgba8();
        let dimensions = img.dimensions();

        let mut texture: GLuint = 0;
        #[cfg(feature = "glfw")]
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
            gl::TextureStorage2D(texture, 1, gl::RGBA8, dimensions.0 as i32, dimensions.1 as i32);
            gl::TextureSubImage2D(
                texture,
                0,
                0,
                0,
                dimensions.0 as i32,
                dimensions.1 as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_raw().as_ptr() as *const GLvoid
            );

            gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        texture
    }).collect()
}

fn load_textures(images: &[gltf::image::Data]) -> Vec<GLuint> {
    images.iter().map(|image_data| {
        let img = image::load_from_memory(&image_data.pixels).expect("Failed to decode image").to_rgba8();
        let dimensions = img.dimensions();

        let mut texture: GLuint = 0;
        #[cfg(feature = "glfw")]
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
            gl::TextureStorage2D(texture, 1, gl::RGBA8, dimensions.0 as i32, dimensions.1 as i32);
            gl::TextureSubImage2D(
                texture,
                0,
                0,
                0,
                dimensions.0 as i32,
                dimensions.1 as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_raw().as_ptr() as *const GLvoid
            );

            gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        }

        texture
    }).collect()
}

fn get_rotation_matrix(xrot: f32, yrot: f32, zrot: f32) -> Mat4 {
    let rx = Mat4::from_rotation_x(-xrot);
    let ry = Mat4::from_rotation_y(yrot);
    let rz = Mat4::from_rotation_z(zrot);

    rz * ry * rx
}
fn rasterize_triangle(triangle: [Vec3; 3], collision_map: &DashMap<vec::IVec3, u8>) {
    
    let min_x = triangle.iter().map(|v| v.x.floor() as i32).min().unwrap_or(0);
    let max_x = triangle.iter().map(|v| v.x.ceil() as i32).max().unwrap_or(0);
    let min_y = triangle.iter().map(|v| v.y.floor() as i32).min().unwrap_or(0);
    let max_y = triangle.iter().map(|v| v.y.ceil() as i32).max().unwrap_or(0);
    let min_z = triangle.iter().map(|v| v.z.floor() as i32).min().unwrap_or(0);
    let max_z = triangle.iter().map(|v| v.z.ceil() as i32).max().unwrap_or(0);

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                collision_map.insert(vec::IVec3::new(x, y, z), 0);
            }
        }
    }
}
enum ModelEntityType<'a> {
    Static(&'a ModelEntity),
    NonStatic(&'a ModelEntity),
}




impl Game {

    pub fn update_model_collisions(&self, model_entity_index: usize) {
        let entity = &self.static_model_entities[model_entity_index];
        let (document, buffers, _images) = &self.gltf_models[entity.model_index];
    
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
                let indices = reader.read_indices().unwrap().into_u32().collect::<Vec<_>>();
    
                for i in (0..indices.len()).step_by(3) {
                    if let (Some(&i1), Some(&i2), Some(&i3)) = (indices.get(i), indices.get(i + 1), indices.get(i + 2)) {
                        let triangle = [
                            Vec3::from(positions[i1 as usize]),
                            Vec3::from(positions[i2 as usize]),
                            Vec3::from(positions[i3 as usize]),
                        ];
                        
                        let transformed_triangle = triangle.map(|vertex| {
                            let scaled_vertex = vertex * entity.scale;
                            let rotated_vertex = (get_rotation_matrix(entity.rot.x, entity.rot.y, entity.rot.z) * Vec4::new(scaled_vertex.x, scaled_vertex.y, scaled_vertex.z, 1.0)).truncate();
                            let final_vertex = rotated_vertex + entity.position;
                            final_vertex
                        });
    
                        // Rasterize the triangle and update the collision map
                        rasterize_triangle(transformed_triangle, &self.chunksys.read().unwrap().justcollisionmap);
                    }
                }
            }
        }
    }


    pub fn create_non_static_model_entity(&mut self, model_index: usize, pos: Vec3, scale: f32, rot: Vec3, jump_height: f32, hostile: bool) {
        let mut modent = ModelEntity::new_with_jump_height(model_index, pos, scale, rot, &self.chunksys, &self.camera, jump_height, hostile);
        

        //let animations = self.animations[model_index].clone();
        //let nodes = self.nodes[model_index].clone();

        modent.animations = Vec::new();
        modent.nodes = Vec::new();

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> = {
            let csys_arc = Arc::clone(&self.chunksys);
            //println!("This thing thinks the seed is {}", csys_arc.read().unwrap().currentseed.read().unwrap());
            Box::new(move |v: vec::IVec3| {
                return csys_arc.read().unwrap().collision_predicate(v);
            })
        };

        modent.coll_cage = CollCage::new(solid_pred);

        self.non_static_model_entities.insert(modent.id, modent);
    }

    pub fn insert_static_model_entity(&mut self, id: u32, model_index: usize, pos: Vec3, scale: f32, rot: Vec3, jump_height: f32, hostile: bool) {
        let mut modent = ModelEntity::new_with_id(id, model_index, pos, scale, rot, &self.chunksys, &self.camera, hostile);
        modent.allowable_jump_height = jump_height;

        let animations = self.animations[model_index].clone();
        let nodes = self.nodes[model_index].clone();

        modent.animations = animations;
        modent.nodes = nodes;

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> = {
            let csys_arc = Arc::clone(&self.chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.read().unwrap().collision_predicate(v);
            })
        };

        modent.coll_cage = CollCage::new(solid_pred);

        self.non_static_model_entities.insert(modent.id, modent);
    }
    

    pub fn insert_player_model_entity(&mut self, id: Uuid, model_index: usize, pos: Vec3, scale: f32, rot: Vec3, jump_height: f32) {
        let mut modent = ModelEntity::new_with_id(0/*Does not use model entities id system, uses players id system */, model_index, pos, scale, rot, &self.chunksys, &self.camera, false);
        modent.allowable_jump_height = jump_height;

       // let animations = self.animations[model_index].clone();
        //let nodes = self.nodes[model_index].clone();

        //modent.animations = animations;
        //modent.nodes = nodes;

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> = {
            let csys_arc = Arc::clone(&self.chunksys);
            Box::new(move |v: vec::IVec3| {
                return csys_arc.read().unwrap().collision_predicate(v);
            })
        };

        modent.coll_cage = CollCage::new(solid_pred);

        self.player_model_entities.insert(id, modent);
    }

    pub fn update_server_received_modents(&mut self) {
        let mut rng: StdRng = StdRng::from_entropy();

        let mut tookdamage: bool = false;

        for mut model in self.non_static_model_entities.iter_mut() {
            let model: &mut ModelEntity = model.value_mut();

            if model.sounding {
                if model.soundtimer < model.soundinterval {
                    model.soundtimer += self.delta_time + rng.gen_range(0.0..0.001);
                } else {
                    model.soundtimer = 0.0;
                    match model.sound {
                        Some(str) => {

                            //info!("Playing at {}, while player pos is {}", model.position, self.camera.lock().unwrap().position);
                            #[cfg(feature = "audio")]
                            unsafe {
                                                           AUDIOPLAYER.play(str, &model.position, &model.velocity, Planets::get_mob_volume(model.model_index));
 
                            }
                        }
                        None => {

                        }
                    }
                    model.sounding = false;
                }
            }

            

            if model.hostile {
                if model.attacktimer < model.attackinterval {
                    model.attacktimer += self.delta_time;
                } else {
                    let camlock = self.camera.lock().unwrap();
                    let campos = camlock.position;
                    drop(camlock);

                    if model.position.distance(campos) < 1.0 {
                        tookdamage = true;
                    }
                    model.attacktimer = 0.0;
                }
                
            }

            
        }
        if tookdamage {
            self.take_damage(4);
        }
    }

    pub fn take_damage(&mut self, amount: u8) {
        let h = self.health.load(std::sync::atomic::Ordering::Relaxed);
        let newamount = (h-amount as i8).max(0);
        self.health.store(newamount, std::sync::atomic::Ordering::Relaxed);
        if newamount <= 0 { //DEAD

            let mut camlock = self.camera.lock().unwrap();
            let campos = camlock.position.clone();

            let mut inv = self.inventory.write().unwrap();
            for i in 0..5 {
                let amt = inv.inv[i].1;
                #[cfg(feature = "glfw")]
                self.drops.add_drop(campos + Vec3::new(0.0, 2.0, 0.0), inv.inv[i].0, amt);
                
                
            }
            inv.inv = STARTINGITEMS;


            
            unsafe {
                camlock.position = SPAWNPOINT;
            }
            
            drop(camlock);
            self.health.store(20, std::sync::atomic::Ordering::Relaxed);
        }
        
    }

    pub fn update_non_static_model_entities(&mut self) {

        //println!("UYdpawdaw");
        //info!("Updating NSMEs, delta time: {}", self.delta_time);
        for mut model in self.non_static_model_entities.iter_mut() {
            let model: &mut ModelEntity = model.value_mut();
            if !model.coll_cage.solid.contains(&Side::FLOOR) {
                model.grounded = false;
                model.was_grounded = false;
            }

            if self.headless {
                model.generate_chunk_on_server_if_not_generated();
            }

            const GRAV: f32 = 9.8;

            if !model.grounded && !model.jumping_up {
                model.time_falling_scalar = (model.time_falling_scalar + self.delta_time * 5.0).min(3.0);
            } else {
                model.time_falling_scalar = 1.0;
            }

            
    
            if !model.grounded && !model.jumping_up {
                model.velocity +=
                    Vec3::new(0.0, -GRAV * model.time_falling_scalar * self.delta_time, 0.0);
            }
    
            if model.jumping_up {
                if model.position.y < model.current_jump_y + model.allowable_jump_height {
                    let curr_cam_y = model.position.y;
                    model.velocity += Vec3::new(
                        0.0,
                        (((model.current_jump_y + model.allowable_jump_height + 0.3) - curr_cam_y)
                            * 15.0)
                            * self.delta_time,
                        0.0,
                    );
                } else {
                    model.jumping_up = false;
                }
            }

            if model.controls.up && model.grounded {
                model.grounded = false;
                model.was_grounded = false;
                model.current_jump_y = model.position.y;
                model.jumping_up = true;
                model.controls.up = false;
            }

            if let Some(_current_animation) = model.current_animation {
                model.animation_time += self.delta_time;
                //apply_animation(&mut model.nodes, &model.animations[current_animation], model.animation_time);
            }

            let cc_center = model.position + Vec3::new(0.0, -1.0, 0.0);
            model.coll_cage.update_readings(cc_center);
            model.respond_to_own_controls(&self.delta_time, 5.0);
            model.behavior_loop(&self.delta_time, &self.known_cameras);

            let makebelievepos = model.position + Vec3::new(0.0, self.planet_y_offset, 0.0);

            if model.sounding && !self.headless {

                //info!("Sounding cricket at {}", model.position);

                    match model.sound {
                        Some(str) => {
                            #[cfg(feature = "audio")]
                            unsafe {
                                                            AUDIOPLAYER.play(str, &makebelievepos, &model.velocity, 1.0);

                            }
                        }   
                        None => {

                        }
                    }

                


                model.sounding = false;
            }
            
            // if (makebelievepos).distance(self.camera.lock().unwrap().position) < 30.0 {
            //     model.target = AggroTarget::ThisCamera;
            //     if model.soundtimer > 0.0 {
            //         model.soundtimer += self.delta_time;
            //     } else {
            //         let sndstr = Monsters::get_aggro_sound(model.model_index);
            //         self.audiop.play(sndstr, &makebelievepos, &model.velocity);
            //         model.soundtimer = 3.0;
            //     }
            // }
            if model.hostile {
                for knowncam in self.known_cameras.iter() {

                    let kc = knowncam.value();
                    if (makebelievepos).distance(*kc) < 30.0 {
                        model.target = AggroTarget::UUID(*knowncam.key());

                    }

                }
            }
                



            let mut proposed = if model.velocity.length() > 0.0 {
                let amt_to_subtract = model.velocity * self.delta_time * 5.0;
                model.velocity -= amt_to_subtract;
    
                model.position + amt_to_subtract
            } else {
                model.position
            };

            model.bound_box
                .set_center(proposed + Vec3::new(0.0, -0.5, 0.0), 0.2, 0.85);

            model.coll_cage.update_colliding(&model.bound_box);

            let mut corr_made: Vec<Vec3> = Vec::new();
            if model.coll_cage.colliding.len() > 0 {
                for side in &model.coll_cage.colliding {
                    if !corr_made.contains(&model.coll_cage.normals[*side as usize]) {
                        proposed += model.coll_cage.normals[*side as usize]
                            * model.coll_cage.penetrations[*side as usize];
                        corr_made.push(model.coll_cage.normals[*side as usize]);
                    }
                    if *side == Side::FLOOR {
                        if !self.headless {
                            if !model.was_grounded && model.model_index == 2 {
                                #[cfg(feature = "audio")]
                                unsafe {
                                    AUDIOPLAYER.play("assets/sfx/slam.mp3", &makebelievepos, &model.velocity, 1.0);
                                }
                                
                            }
                        }
                        
                        model.grounded = true;
                        model.was_grounded = true;
                    }
                    if *side == Side::ROOF {
                        model.jumping_up = false;
                        model.grounded = false;
                    }
                }
            }
            model.set_pos(proposed);
            model.recalculate();
            //camlock.recalculate();
        }
    }

    pub fn draw_models(&self) {


        #[cfg(feature = "glfw")]
        unsafe {

            

            //gl::DepthMask(gl::FALSE);
            gl::Disable(gl::CULL_FACE);
            gl::UseProgram(self.modelshader.shader_id);
            let mvp_loc = gl::GetUniformLocation(self.modelshader.shader_id, b"mvp\0".as_ptr() as *const i8);



            
            let camclone = {
                let cam_lock = self.camera.lock().unwrap();
                cam_lock.clone()
            };


            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, camclone.mvp.to_cols_array().as_ptr());
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.modelshader.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                1,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.modelshader.shader_id,
                    b"walkbob\0".as_ptr() as *const i8,
                ),
                self.vars.walkbobtimer,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.modelshader.shader_id,
                    b"opacity\0".as_ptr() as *const i8,
                ),
                1.0
            );

            let nsme = self.non_static_model_entities.iter().map(|e| e).collect::<Vec<_>>();
            let pme = self.player_model_entities.iter().map(|e| e).collect::<Vec<_>>();

            for modelt in self.static_model_entities.iter().map(ModelEntityType::Static)
            .chain(nsme.iter().map(|arg0| ModelEntityType::NonStatic(arg0.value())))
            .chain(pme.iter().map(|arg0| ModelEntityType::NonStatic(arg0.value())))
                 {

                let modelent = match modelt {
                    ModelEntityType::Static(entity) => {
                        entity
                    },
                    ModelEntityType::NonStatic(entity) => {
                        entity
                    },
                };
                    
                let index = modelent.model_index;
                if index < self.gltf_vaos.len() && index < self.gltf_textures.len() {
                       
                let vaosetset = &self.gltf_vaos[index];

                //info!("Doing Vaosetset {index}");
                let texsetset = &self.gltf_textures[index];

                for (ind, vaoset) in vaosetset.iter().enumerate() {
                    //info!("Doing Vaoset {ind} of Vaosetset {index}");

                    let texset = &texsetset[ind];

                    for(ii, vao) in vaoset.iter().enumerate() {
                        //info!("Doing Vao {ii} of Vaoset {ind} of Vaosetset {index}");
                        gl::BindVertexArray(*vao);

                            
                            if let Some(texture_id) = texset.get(0) {
                                gl::BindTextureUnit(1, *texture_id); 
                            }

                            match modelt {
                                ModelEntityType::Static(_) => {

                                },
                                ModelEntityType::NonStatic(_) => {
                                    
                                },
                            }

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"scale\0".as_ptr() as *const i8,
                                ),
                                modelent.scale,
                            );

                            match modelt {
                                ModelEntityType::Static(entity) => {
                                    gl::Uniform3f(
                                        gl::GetUniformLocation(
                                            self.modelshader.shader_id,
                                            b"pos\0".as_ptr() as *const i8,
                                        ),
                                        entity.position.x,
                                        entity.position.y  - 1.0,
                                        entity.position.z
                                    );
                                },
                                ModelEntityType::NonStatic(entity) => {
                                    gl::Uniform3f(
                                        gl::GetUniformLocation(
                                            self.modelshader.shader_id,
                                            b"pos\0".as_ptr() as *const i8,
                                        ),
                                        entity.position.x,
                                        entity.position.y + self.planet_y_offset - 1.0,
                                        entity.position.z
                                    );
                                },
                            }

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"interp_time\0".as_ptr() as *const i8,
                                ),
                                glfwGetTime() as f32 - modelent.time_stamp as f32
                            );

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"lastpos\0".as_ptr() as *const i8,
                                ),
                                modelent.lastpos.x,
                                modelent.lastpos.y  - 1.0,
                                modelent.lastpos.z
                            );
                            

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"xrot\0".as_ptr() as *const i8,
                                ),
                                modelent.rot.x,
                            );
                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"yrot\0".as_ptr() as *const i8,
                                ),
                                modelent.rot.y,
                            );

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"zrot\0".as_ptr() as *const i8,
                                ),
                                modelent.rot.z,
                            );

                            match modelt {
                                ModelEntityType::Static(_entity) => {
                                    gl::Uniform1f(
                                        gl::GetUniformLocation(
                                            self.modelshader.shader_id,
                                            b"ambientBrightMult\0".as_ptr() as *const i8,
                                        ),
                                        self.ambient_bright_mult,
                                    );
                                },
                                ModelEntityType::NonStatic(entity) => {

                                    let mut blocklighthere = 0.0;

                                    let samplingcoord = vec::IVec3::new(
                                        entity.position.x as i32,
                                        entity.position.y as i32,
                                        entity.position.z as i32
                                    );
                                    let csyslock = self.chunksys.read().unwrap();
                                    let lmlock = csyslock.lightmap.lock().unwrap();

                                    match lmlock.get(&samplingcoord) {
                                        Some(t) => {
                                            blocklighthere = t.sum().x as f32; //TEMPORARY USING RED
                                        }
                                        None => {

                                        }
                                    }

                                    let scaledbl = blocklighthere / 16.0;
                                    

                                    gl::Uniform1f(
                                        gl::GetUniformLocation(
                                            self.modelshader.shader_id,
                                            b"ambientBrightMult\0".as_ptr() as *const i8,
                                        ),
                                        (self.ambient_bright_mult + scaledbl).clamp(0.0, 1.0),
                                    );
                                },
                            }

                            

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"camPos\0".as_ptr() as *const i8,
                                ),
                                camclone.position.x,
                                camclone.position.y,
                                camclone.position.z
                            );

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"lastrot\0".as_ptr() as *const i8,
                                ),
                                modelent.lastrot.x,
                                modelent.lastrot.y,
                                modelent.lastrot.z
                            );


                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"camDir\0".as_ptr() as *const i8,
                                ),
                                camclone.direction.x,
                                camclone.direction.y,
                                camclone.direction.z
                            );

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"viewDistance\0".as_ptr() as *const i8,
                                ),
                                8.0
                            );

                            let fogcol = Planets::get_fog_col(self.chunksys.read().unwrap().planet_type as u32);

                            gl::Uniform4f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"fogCol\0".as_ptr() as *const i8,
                                ),
                                fogcol.0,
                                fogcol.1,
                                fogcol.2,
                                fogcol.3
                            );

                            gl::Uniform1f(gl::GetUniformLocation(
                                self.modelshader.shader_id,
                                b"sunset\0".as_ptr() as *const i8,
                            ), self.sunset_factor);
                            gl::Uniform1f(gl::GetUniformLocation(
                                self.modelshader.shader_id,
                                b"sunrise\0".as_ptr() as *const i8,
                            ), self.sunrise_factor);




                        
                        gl::DrawElements(self.gltf_drawmodes[index][ind][ii],  self.gltf_counts[index][ind][ii] as i32, gl::UNSIGNED_INT, std::ptr::null());
                    }
                    
                }
                }

             

                
            }
                        

            gl::Enable(gl::CULL_FACE);
            //gl::DepthMask(gl::TRUE);
        }
        
        unsafe {

        }
    }
    #[cfg(feature = "glfw")]
    pub fn load_model(&mut self, path: &'static str) {
        let (document, buffers, images) = gltf::import(path).expect("Failed to load model");
        self.gltf_models.push((document.clone(), buffers.clone(), images.clone()));
        let path = Path::new(path);
        let gp = path.parent()
            .map(|p| p.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_string();
        self.gltf_paths.push(gp);

        let animindex = self.animations.len();
        let nodeindex = self.nodes.len();

        self.animations.push(Vec::new());
        self.nodes.push(Vec::new());

        for animation in document.animations() {
            let mut channels = Vec::new();
            for channel in animation.channels() {
                let _sampler = channel.sampler();
                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
                let inputs = reader.read_inputs().unwrap().collect::<Vec<f32>>();
                let outputs: Vec<Vec<f32>> = match reader.read_outputs().unwrap() {
                    ReadOutputs::Translations(translations) => translations.map(|v| v.to_vec()).collect(),
                    ReadOutputs::Rotations(rotations) => rotations.into_f32().map(|v| v.to_vec()).collect(),
                    ReadOutputs::Scales(scales) => scales.map(|v| v.to_vec()).collect(),
                    ReadOutputs::MorphTargetWeights(weights) => weights.into_f32().collect::<Vec<f32>>().iter().map(|w| vec![*w]).collect(),
                };

                let keyframes = inputs.into_iter().zip(outputs).collect();

                channels.push(AnimationChannel {
                    node_index: channel.target().node().index(),
                    property: channel.target().property(),
                    keyframes,
                });
            }



            self.animations[animindex].push(Animation {
                channels,
                name: animation.name().unwrap_or_default().to_string(),
            });
        }

        for skin in document.skins() {
            let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
            let inverse_bind_matrices: Vec<Mat4> = reader.read_inverse_bind_matrices().unwrap().map(|m| {
                let flat: Vec<f32> = m.iter().flatten().cloned().collect();
                Mat4::from_cols_array(&flat.try_into().expect("Slice with incorrect length"))
            }).collect();

            let joints: Vec<Joint> = skin.joints()
                .zip(inverse_bind_matrices)
                .map(|(joint, inverse_bind_matrix)| Joint {
                    node_index: joint.index(),
                    inverse_bind_matrix,
                })
                .collect();

            self.skins.push(Skin { joints });
        }

        for node in document.nodes() {
            self.nodes[nodeindex].push(Node {
                transform: Mat4::from_cols_array_2d(&node.transform().matrix()),
                children: node.children().map(|child| child.index()).collect(),
            });
        }
    }
    
    fn collect_indices(data: ReadIndices) -> Vec<u32> {
        match data {
            ReadIndices::U8(iter) => {
                iter.map(|index| index as u32).collect()
            },
            ReadIndices::U16(iter) => {
                iter.map(|index| index as u32).collect()
            },
            ReadIndices::U32(iter) => {
                iter.collect()
            },
        }
    }
    #[cfg(feature = "glfw")]
    pub fn create_model_vbos(&mut self) {
        for (index, (document, buffers, _images)) in self.gltf_models.iter().enumerate() {
            self.gltf_counts.push(Vec::new());
            self.gltf_drawmodes.push(Vec::new());
            self.gltf_vaos.push(Vec::new());
            self.gltf_vbos.push(Vec::new());
            self.gltf_textures.push(Vec::new());

            let textures = load_document_textures(&document, &buffers, self.gltf_paths[index].as_str());

            for mesh in document.meshes() {
                let mut mesh_vbos = Vec::new();
                let mut mesh_vaos = Vec::new();
                let mut mesh_counts = Vec::new();
                let mut mesh_drawmodes = Vec::new();
                let mut textures_here = Vec::new();
                
                for primitive in mesh.primitives() {

                    let material = primitive.material();
                    let pbr = material.pbr_metallic_roughness();

                    let default_texture_index = 0;

                    let base_color_texture_index = pbr.base_color_texture().map(|info| info.texture().index())
                    .or_else(|| {
                        document.textures().nth(0).map(|tex| tex.index()) // Example: Just grab the first texture if available
                    })
                    .unwrap_or(default_texture_index);

                    textures_here.push(textures[base_color_texture_index]);

                    //if let Some((_, accessor)) = primitive.attributes().find(|(semantic, _)| *semantic == Semantic::Positions) {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                        let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
                        let indices = Game::collect_indices(reader.read_indices().unwrap()); 
                        let uvs = reader.read_tex_coords(0).unwrap().into_f32().collect::<Vec<_>>();

                        let mut ebo: GLuint = 0;
                        unsafe {
                            gl::CreateBuffers(1, &mut ebo);
                            gl::NamedBufferData(
                                ebo,
                                (indices.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
                                indices.as_ptr() as *const GLvoid,
                                gl::STATIC_DRAW,  // Usage hint
                            );
                        }


                        //let vertex_count = positions.len();
                        let index_count = indices.len();
                        let mut vbo: GLuint = 0;



                        let mut uv_vbo: GLuint = 0;

                        unsafe {
                            gl::CreateBuffers(1, &mut vbo);
                            gl::NamedBufferData(
                                vbo,
                                (positions.len() * std::mem::size_of::<[f32; 3]>()) as GLsizeiptr,
                                positions.as_ptr() as *const GLvoid,
                                gl::STATIC_DRAW,
                            );


                            gl::CreateBuffers(1, &mut uv_vbo);
                            gl::NamedBufferData(
                                uv_vbo,
                                (uvs.len() * std::mem::size_of::<[f32; 2]>()) as GLsizeiptr,
                                uvs.as_ptr() as *const GLvoid,
                                gl::STATIC_DRAW,
                            );
                        }

                        mesh_vbos.push(vbo);
                        mesh_counts.push(index_count);
                        mesh_drawmodes.push(primitive.mode().as_gl_enum());
                        

                        // Create VAO
                        let mut vao: GLuint = 0;
                        unsafe {
                            gl::CreateVertexArrays(1, &mut vao);
                            gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, (3 * std::mem::size_of::<f32>()) as i32);
                            gl::EnableVertexArrayAttrib(vao, 0);
                            gl::VertexArrayAttribFormat(vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
                            gl::VertexArrayAttribBinding(vao, 0, 0);
                            

                            gl::VertexArrayVertexBuffer(vao, 1, uv_vbo, 0, (2 * std::mem::size_of::<f32>()) as i32);
                            gl::EnableVertexArrayAttrib(vao, 1);
                            gl::VertexArrayAttribFormat(vao, 1, 2, gl::FLOAT, gl::FALSE, 0);
                            gl::VertexArrayAttribBinding(vao, 1, 1);


                            gl::VertexArrayElementBuffer(vao, ebo);

                        }
                        mesh_vaos.push(vao);
                    //}
                }
                self.gltf_vbos[index].push(mesh_vbos);
                self.gltf_vaos[index].push(mesh_vaos);
                self.gltf_counts[index].push(mesh_counts);
                self.gltf_drawmodes[index].push(mesh_drawmodes);
                self.gltf_textures[index].push(textures_here);
            }
        }
    }
}