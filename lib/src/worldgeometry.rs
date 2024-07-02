use std::sync::Mutex;

use crate::chunk::ChunkGeo;
use crate::shader::Shader;
use gl;
use gl::types::{GLsizei, GLsizeiptr, GLuint, GLvoid};

pub struct WorldGeometry {}

impl WorldGeometry {

    pub fn bind_old_geometry_no_upload(
        vbov: GLuint,
        vbouv: GLuint,
        shader: &Shader,
    ) {
        unsafe {


            // Bind vertex buffer to the vertex array object
            gl::VertexArrayVertexBuffer(shader.vao, 0, vbov, 0, (5 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                eprintln!("OpenGL Error after associating vbov with vao: {}", error);
            }

            // Position attribute
            let pos_attrib = gl::GetAttribLocation(shader.shader_id, b"position\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, pos_attrib as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                pos_attrib as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(shader.vao, pos_attrib as GLuint, 0);

            // Block brightness attribute
            let brightness_attrib = gl::GetAttribLocation(shader.shader_id, b"blockBright\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, brightness_attrib as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                brightness_attrib as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as GLuint,
            );
            gl::VertexArrayAttribBinding(shader.vao, brightness_attrib as GLuint, 0);

            // Ambient brightness attribute
            let amb_brightness = gl::GetAttribLocation(shader.shader_id, b"ambientBright\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, amb_brightness as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                amb_brightness as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as GLuint,
            );
            gl::VertexArrayAttribBinding(shader.vao, amb_brightness as GLuint, 0);


            // Bind UV buffer to the vertex array object
            gl::VertexArrayVertexBuffer(shader.vao, 1, vbouv, 0, (4 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                eprintln!("OpenGL Error after associating vbouv with vao: {}", error);
            }

            // UV attribute
            let uv_attrib = gl::GetAttribLocation(shader.shader_id, b"uv\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, uv_attrib as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                uv_attrib as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(shader.vao, uv_attrib as GLuint, 1);

            // UV base attribute
            let uv_attrib2 = gl::GetAttribLocation(shader.shader_id, b"uvbase\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, uv_attrib2 as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                uv_attrib2 as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * std::mem::size_of::<f32>()) as GLuint,
            );
            gl::VertexArrayAttribBinding(shader.vao, uv_attrib2 as GLuint, 1);
        }
    }
    
    pub fn bind_old_geometry(
        vbov: GLuint,
        vbouv: GLuint,
        vdata: &[f32],
        uvdata: &[f32],
        shader: &Shader,
    ) {
        unsafe {
            // Upload vertex data to named buffer
            gl::NamedBufferData(
                vbov,
                (vdata.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vdata.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                eprintln!("Bind world geom err (vbov): {}", error);
            }

            // Bind vertex buffer to the vertex array object
            gl::VertexArrayVertexBuffer(shader.vao, 0, vbov, 0, (5 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                eprintln!("OpenGL Error after associating vbov with vao: {}", error);
            }

            // Position attribute
            let pos_attrib = gl::GetAttribLocation(shader.shader_id, b"position\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, pos_attrib as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                pos_attrib as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(shader.vao, pos_attrib as GLuint, 0);

            // Block brightness attribute
            let brightness_attrib = gl::GetAttribLocation(shader.shader_id, b"blockBright\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, brightness_attrib as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                brightness_attrib as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as GLuint,
            );
            gl::VertexArrayAttribBinding(shader.vao, brightness_attrib as GLuint, 0);

            // Ambient brightness attribute
            let amb_brightness = gl::GetAttribLocation(shader.shader_id, b"ambientBright\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, amb_brightness as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                amb_brightness as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as GLuint,
            );
            gl::VertexArrayAttribBinding(shader.vao, amb_brightness as GLuint, 0);

            // Upload UV data to named buffer
            gl::NamedBufferData(
                vbouv,
                (uvdata.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                uvdata.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Bind UV buffer to the vertex array object
            gl::VertexArrayVertexBuffer(shader.vao, 1, vbouv, 0, (4 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                eprintln!("OpenGL Error after associating vbouv with vao: {}", error);
            }

            // UV attribute
            let uv_attrib = gl::GetAttribLocation(shader.shader_id, b"uv\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, uv_attrib as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                uv_attrib as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(shader.vao, uv_attrib as GLuint, 1);

            // UV base attribute
            let uv_attrib2 = gl::GetAttribLocation(shader.shader_id, b"uvbase\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, uv_attrib2 as GLuint);
            gl::VertexArrayAttribFormat(
                shader.vao,
                uv_attrib2 as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * std::mem::size_of::<f32>()) as GLuint,
            );
            gl::VertexArrayAttribBinding(shader.vao, uv_attrib2 as GLuint, 1);
        }
    }

    pub fn bind_geometry(
        vbo32: gl::types::GLuint,
        vbo8: gl::types::GLuint,
        vbo8rgb: GLuint,
        upload: bool,
        shader: &Shader,
        data: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>, &Mutex<Vec<u8>>),
    ) {
        unsafe {
            if upload {
                let datalock = data.0.lock().unwrap();
                gl::NamedBufferData(
                    vbo32,
                    (datalock.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                    datalock.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW,
                );

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!(
                        "OpenGL Error after named buffering of vbo32 with upload true: {}",
                        error
                    );
                }
            }

            gl::VertexArrayVertexBuffer(
                shader.vao,
                0,
                vbo32,
                0,
                std::mem::size_of::<u32>() as i32,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL Error after associating vbo32 with vao: {}", error);
            }
            if upload {
                let u32one_attrib =
                    gl::GetAttribLocation(shader.shader_id, b"u32\0".as_ptr() as *const i8)
                        as gl::types::GLuint;

                gl::EnableVertexArrayAttrib(shader.vao, u32one_attrib);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u32 array attrib: {}", error);
                }

                gl::VertexArrayAttribIFormat(shader.vao, u32one_attrib, 1, gl::UNSIGNED_INT, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u32 attrib format: {}", error);
                }
                gl::VertexArrayAttribBinding(shader.vao, u32one_attrib, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u32 attrib binding: {}", error);
                }
                let data1lock = data.1.lock().unwrap();
                gl::NamedBufferData(
                    vbo8,
                    (data1lock.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr,
                    data1lock.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW,
                );

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!(
                        "OpenGL Error after named buffering of vbo8 with upload true: {}",
                        error
                    );
                }

                let data2lock = data.2.lock().unwrap();
                gl::NamedBufferData(
                    vbo8rgb,
                    (data2lock.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr,
                    data2lock.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW,
                );

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!(
                        "OpenGL Error after named buffering of vbo8rgb with upload true: {}",
                        error
                    );
                }
            }
            gl::VertexArrayVertexBuffer(
                shader.vao,
                1,
                vbo8,
                0,
                std::mem::size_of::<u8>() as i32,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL Error after associating vbo8 with vao: {}", error);
            }
            if upload {
                let u8_attrib =
                    gl::GetAttribLocation(shader.shader_id, b"eightbit\0".as_ptr() as *const i8)
                        as gl::types::GLuint;
                //println!("U8 attrib location: {}", u8_attrib);
                gl::EnableVertexArrayAttrib(shader.vao, u8_attrib);

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u8 array attrib: {}", error);
                }

                gl::VertexArrayAttribIFormat(shader.vao, u8_attrib, 1, gl::UNSIGNED_BYTE, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u8 array attrib format: {}", error);
                }

                gl::VertexArrayAttribBinding(shader.vao, u8_attrib, 1);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u8 array attrib binding: {}", error);
                }
            }

            gl::VertexArrayVertexBuffer(
                shader.vao,
                1,
                vbo8rgb,
                0,
                std::mem::size_of::<u8>() as i32,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL Error after associating vbo8 with vao: {}", error);
            }
            if upload {
                let u8_attrib =
                    gl::GetAttribLocation(shader.shader_id, b"eightbitrgb\0".as_ptr() as *const i8)
                        as gl::types::GLuint;
                //println!("U8 attrib location: {}", u8_attrib);
                gl::EnableVertexArrayAttrib(shader.vao, u8_attrib);

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u8 array attrib: {}", error);
                }

                gl::VertexArrayAttribIFormat(shader.vao, u8_attrib, 1, gl::UNSIGNED_BYTE, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u8 array attrib format: {}", error);
                }

                gl::VertexArrayAttribBinding(shader.vao, u8_attrib, 1);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL Error after u8 array attrib binding: {}", error);
                }
            }
        }
    }
}
