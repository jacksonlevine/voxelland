use gl::types::{GLsizeiptr, GLuint, GLvoid};
use gltf::{accessor::{DataType, Dimensions}, mesh::util::ReadIndices, Semantic};

use crate::game::Game;

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

impl Game {

    pub fn draw_models(&self) {


                

        unsafe {
            gl::UseProgram(self.modelshader.shader_id);
            let mvp_loc = gl::GetUniformLocation(self.modelshader.shader_id, b"mvp\0".as_ptr() as *const i8);
            let cam_lock = self.camera.lock().unwrap();

            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, cam_lock.mvp.to_cols_array().as_ptr());

            for (index, vaosetset) in self.gltf_vaos.iter().enumerate() {
                for (ind, vaoset) in vaosetset.iter().enumerate() {
                    for(ii, vao) in vaoset.iter().enumerate() {
                        gl::BindVertexArray(*vao);
                        gl::DrawElements(self.gltf_drawmodes[index][ind][ii],  self.gltf_counts[index][ind][ii] as i32, gl::UNSIGNED_INT, std::ptr::null());
                    }
                    
                }
            }
        }
        
    }
    pub fn load_model(&mut self, path: &'static str) {
        let model = gltf::import(path).expect("Failed to load model");
        self.gltf_models.push(model);
    }


    fn extract_and_upload_data(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Vec<u8> {
        let view = accessor.view().unwrap(); // Get the buffer view
        let buffer = &buffers[view.buffer().index()]; // Get the buffer data
        let start = view.offset() + accessor.offset();
        let end = start + accessor.count() * accessor.size();
    
        // Extract the raw data based on the type
        match accessor.data_type() {
            DataType::U8 => buffer[start..end].to_vec(),
            DataType::U16 => convert_to_vec::<u16>(&buffer[start..end]),
            DataType::U32 => convert_to_vec::<u32>(&buffer[start..end]),
            DataType::I8 => convert_to_vec::<i8>(&buffer[start..end]),
            DataType::I16 => convert_to_vec::<i16>(&buffer[start..end]),
            DataType::F32 => convert_to_vec::<f32>(&buffer[start..end]),
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
    

    pub fn create_model_vbos(&mut self) {
        for (index, (document, buffers, _images)) in self.gltf_models.iter().enumerate() {
            self.gltf_counts.push(Vec::new());
            self.gltf_drawmodes.push(Vec::new());
            self.gltf_vaos.push(Vec::new());
            self.gltf_vbos.push(Vec::new());

            for mesh in document.meshes() {
                let mut mesh_vbos = Vec::new();
                let mut mesh_vaos = Vec::new();
                let mut mesh_counts = Vec::new();
                let mut mesh_drawmodes = Vec::new();

                for primitive in mesh.primitives() {
                    //if let Some((_, accessor)) = primitive.attributes().find(|(semantic, _)| *semantic == Semantic::Positions) {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                        let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
                        let indices = Game::collect_indices(reader.read_indices().unwrap()); 

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

                        unsafe {
                            gl::CreateBuffers(1, &mut vbo);
                            gl::NamedBufferData(
                                vbo,
                                (positions.len() * std::mem::size_of::<[f32; 3]>()) as GLsizeiptr,
                                positions.as_ptr() as *const GLvoid,
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
                            gl::VertexArrayElementBuffer(vao, ebo);
                        }
                        mesh_vaos.push(vao);
                    //}
                }
                self.gltf_vbos[index].push(mesh_vbos);
                self.gltf_vaos[index].push(mesh_vaos);
                self.gltf_counts[index].push(mesh_counts);
                self.gltf_drawmodes[index].push(mesh_drawmodes);
            }
        }
    }

    // pub fn setup_vertex_attributes(&mut self) {
    //     for (index, vbo_ids) in self.gltf_vbos.iter().enumerate() {
            
    //         self.gltf_vaos.push(Vec::new());
    //         for (i, &vbo) in vbo_ids.iter().enumerate() {
    //             unsafe {
    //                 let mut vao: GLuint = 0;
    //                 gl::CreateVertexArrays(1, &mut vao);
    //                 gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, 3 * std::mem::size_of::<f32>() as i32);
        
    //                 gl::EnableVertexArrayAttrib(vao, 0);
    //                 gl::VertexArrayAttribFormat(vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
    //                 gl::VertexArrayAttribBinding(vao, 0, 0);
    //                 self.gltf_vaos[index].push(vao);
    //             }
    //         }
    //     }
    // }
}