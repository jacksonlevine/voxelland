use std::{collections::HashMap, fs, path::Path, str::FromStr};

use dashmap::DashMap;
use gl::types::{GLsizeiptr, GLuint, GLvoid};
use glam::{Mat4, Vec3, Vec4};
use gltf::{accessor::{DataType, Dimensions}, image::Source, mesh::util::ReadIndices, Semantic};

use crate::{game::Game, modelentity::ModelEntity, vec};

fn convert_to_vec<T: bytemuck::Pod>(data: &[u8]) -> Vec<u8> {
    bytemuck::cast_slice(data).to_vec()
}
fn num_components(dimensions: Dimensions) -> i32 {
    match dimensions {
        Dimensions::Scalar => 1,
        Dimensions::Vec2 => 2,
        Dimensions::Vec3 => 3,
        Dimensions::Vec4 => 4,
        Dimensions::Mat2 => 4,  // 2x2 matrix is treated as 4 components (2 per column)
        Dimensions::Mat3 => 9,  // 3x3 matrix is treated as 9 components (3 per column)
        Dimensions::Mat4 => 16, // 4x4 matrix is treated as 16 components (4 per column)
    }
}

fn load_document_textures(document: &gltf::Document, buffers: &[gltf::buffer::Data], base_path: &str) -> Vec<GLuint> {
    document.images().map(|image| {
        let data = match image.source() {
            Source::Uri { uri, mime_type } => {
                // External image: Load from a file
                let path = format!("{}/{}", base_path, uri);
                println!("Loading external image: {}", uri); // Print the file name
                fs::read(path).expect("Failed to read image file")
            },
            Source::View { view, mime_type } => {
                // Embedded image: Get data from buffer
                let buffer_index = view.buffer().index();
                let start = view.offset();
                let end = start + view.length();
                println!("Loading embedded image from buffer index: {}", buffer_index); // Print the buffer index
                buffers[buffer_index][start..end].to_vec()
            },
        };

        let img = image::load_from_memory(&data)
            .expect("Failed to decode image")
            .to_rgba8();
        let dimensions = img.dimensions();

        let mut texture: GLuint = 0;
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

            // Set texture parameters
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
    // Create rotation matrices for each axis
    let rx = Mat4::from_rotation_x(-xrot);
    let ry = Mat4::from_rotation_y(yrot);
    let rz = Mat4::from_rotation_z(zrot);

    // Multiply the rotation matrices together
    // Note: The order of multiplication here assumes a specific rotation order (Rz * Ry * Rx)
    // This order might need to be adjusted based on how you want the rotations to be applied
    rz * ry * rx
}
fn rasterize_triangle(triangle: [Vec3; 3], collision_map: &DashMap<vec::IVec3, u8>) {
    // This is a placeholder for the rasterization logic. You need to implement this based on your grid/collision map resolution.
    // Simple bounding box rasterization (for demonstration purposes):
    let min_x = triangle.iter().map(|v| v.x.floor() as i32).min().unwrap_or(0);
    let max_x = triangle.iter().map(|v| v.x.ceil() as i32).max().unwrap_or(0);
    let min_y = triangle.iter().map(|v| v.y.floor() as i32).min().unwrap_or(0);
    let max_y = triangle.iter().map(|v| v.y.ceil() as i32).max().unwrap_or(0);
    let min_z = triangle.iter().map(|v| v.z.floor() as i32).min().unwrap_or(0);
    let max_z = triangle.iter().map(|v| v.z.ceil() as i32).max().unwrap_or(0);

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                // Check if the (x, y, z) is inside the triangle; this is a simplification
                collision_map.insert(vec::IVec3::new(x, y, z), 0);
            }
        }
    }
}


impl Game {

    pub fn update_model_collisions(&self, model_entity_index: usize) {
        let entity = &self.model_entities[model_entity_index];
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
                            let final_vertex = rotated_vertex + entity.pos;
                            final_vertex
                        });
    
                        // Rasterize the triangle and update the collision map
                        rasterize_triangle(transformed_triangle, &self.chunksys.justcollisionmap);
                    }
                }
            }
        }
    }

    pub fn draw_models(&self) {


                

        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::UseProgram(self.modelshader.shader_id);
            let mvp_loc = gl::GetUniformLocation(self.modelshader.shader_id, b"mvp\0".as_ptr() as *const i8);
            let cam_lock = self.camera.lock().unwrap();

            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, cam_lock.mvp.to_cols_array().as_ptr());
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.modelshader.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                1,
            );

            


            for modelent in &self.model_entities {
                let index = modelent.model_index;
                let vaosetset = &self.gltf_vaos[index];

                //println!("Doing Vaosetset {index}");
                let texsetset = &self.gltf_textures[index];

                for (ind, vaoset) in vaosetset.iter().enumerate() {
                    //println!("Doing Vaoset {ind} of Vaosetset {index}");

                    let texset = &texsetset[ind];

                    for(ii, vao) in vaoset.iter().enumerate() {
                        //println!("Doing Vao {ii} of Vaoset {ind} of Vaosetset {index}");
                        gl::BindVertexArray(*vao);

                            
                            if let Some(texture_id) = texset.get(0) {
                                gl::BindTextureUnit(1, *texture_id); 
                            }

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"scale\0".as_ptr() as *const i8,
                                ),
                                modelent.scale,
                            );

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"pos\0".as_ptr() as *const i8,
                                ),
                                modelent.pos.x,
                                modelent.pos.y,
                                modelent.pos.z
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


                        
                        gl::DrawElements(self.gltf_drawmodes[index][ind][ii],  self.gltf_counts[index][ind][ii] as i32, gl::UNSIGNED_INT, std::ptr::null());
                    }
                    
                }

                
            }
                        

            gl::Enable(gl::CULL_FACE);
        }
        
    }
    pub fn load_model(&mut self, path: &'static str) {
        let model = gltf::import(path).expect("Failed to load model");
        self.gltf_models.push(model);
        let path = Path::new(path);
        let gp = path.parent() // This returns the parent directory as an Option<&Path>
            .map(|p| p.to_str().unwrap_or("")) // Convert the Path to a &str
            .unwrap_or("") // Handle the case where `parent()` returns None
            .to_string(); // Convert &str to String
        self.gltf_paths.push(gp);
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

    pub fn create_model_vbos(&mut self) {
        for (index, (document, buffers, images)) in self.gltf_models.iter().enumerate() {
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