use gl;
use crate::chunk::ChunkGeo;
use crate::shader::Shader;

pub struct WorldGeometry {

}

impl WorldGeometry {
    pub fn bind_geometry(vbo32: gl::types::GLuint, vbo8: gl::types::GLuint, upload: bool, shader: &Shader, data: &ChunkGeo) {
        
        unsafe {
            if upload {
                gl::NamedBufferData(
                    vbo32,
                    (data.data32.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr, 
                    data.data32.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after named buffering of vbo32 with upload true: {}", error);
                            }
            }
            
            gl::VertexArrayVertexBuffer(shader.vao, 0, vbo32, 0, std::mem::size_of::<u32>() as i32);
            if upload {
                let u32one_attrib = gl::GetAttribLocation(
                    shader.shader_id, 
                    b"u32\0".as_ptr() as *const i8) as gl::types::GLuint;
    
                gl::VertexArrayAttribFormat(
                    shader.vao,
                    u32one_attrib, 
                    1,
                    gl::UNSIGNED_INT,
                    gl::FALSE,
                    0);
                gl::VertexArrayAttribBinding(shader.vao, u32one_attrib, 0);
                gl::EnableVertexArrayAttrib(
                    shader.vao,
                    u32one_attrib);
                gl::NamedBufferData(
                    vbo8,
                    (data.data8.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr, 
                    data.data8.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW);

                    let error = unsafe { gl::GetError() };
                            if error != gl::NO_ERROR {
                                println!("OpenGL Error after named buffering of vbo8 with upload true: {}", error);
                            }
            }
            gl::VertexArrayVertexBuffer(shader.vao, 1, vbo8, 0, std::mem::size_of::<u8>() as i32);
            if upload {
                let u8_attrib = gl::GetAttribLocation(
                    shader.shader_id, 
                    b"u8\0".as_ptr() as *const i8) as gl::types::GLuint;
                
                gl::VertexArrayAttribFormat(shader.vao,
                    u8_attrib,
                    1, 
                    gl::UNSIGNED_BYTE, 
                    gl::FALSE, 
                    0);
                
                gl::VertexArrayAttribBinding(shader.vao, u8_attrib, 1);
                gl::EnableVertexArrayAttrib(
                    shader.vao,
                    u8_attrib);
            }
            

        }
    }
}