use gl;
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;
use crate::chunk::ChunkGeo;


pub struct WorldGeometry {

}

impl WorldGeometry {
    pub fn bind_geometry(vbo32: gl::types::GLuint, vbo8: gl::types::GLuint, upload: bool, shader: gl::types::GLuint, data: &ChunkGeo) {

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo32);
            if upload {
                gl::BufferData(
                    gl::ARRAY_BUFFER, 
                    (data.data32.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr, 
                    data.data32.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW);
            }
            let u32one_attrib = gl::GetAttribLocation(
                shader, 
                b"u32\0".as_ptr() as *const i8) as gl::types::GLuint;
            gl::EnableVertexAttribArray(
                u32one_attrib);
            gl::VertexAttribPointer(
                u32one_attrib, 
                1,
                gl::UNSIGNED_INT,
                gl::FALSE,
                (1 * std::mem::size_of::<u32>()) as gl::types::GLint,
                std::ptr::null());

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo8);
            if upload {
                gl::BufferData(
                    gl::ARRAY_BUFFER, 
                    (data.data8.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr, 
                    data.data8.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW);
            }
        
            let u8_attrib = gl::GetAttribLocation(
                shader, 
                b"u8\0".as_ptr() as *const i8) as gl::types::GLuint;
            gl::EnableVertexAttribArray(
                u8_attrib);
            gl::VertexAttribPointer(u8_attrib,
                 1, 
                 gl::UNSIGNED_BYTE, 
                 gl::FALSE, 
                 (1 * std::mem::size_of::<u8>()) as gl::types::GLint, 
                 std::ptr::null());

        }
    }
}