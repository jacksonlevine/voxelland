use parking_lot::{Mutex, RwLock};
use tracing::info;
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
                info!("OpenGL Error after associating vbov with vao: {}", error);
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
            let brightness_attrib = gl::GetAttribLocation(shader.shader_id, b"blockRgb\0".as_ptr() as *const i8);
            gl::EnableVertexArrayAttrib(shader.vao, brightness_attrib as GLuint);
            gl::VertexArrayAttribIFormat(
                shader.vao,
                brightness_attrib as GLuint,
                1,
                gl::UNSIGNED_INT,
                (3 * std::mem::size_of::<u32>()) as GLuint,
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
                info!("OpenGL Error after associating vbouv with vao: {}", error);
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

            // // UV base attribute
            // let uv_attrib2 = gl::GetAttribLocation(shader.shader_id, b"uvbase\0".as_ptr() as *const i8);
            // gl::EnableVertexArrayAttrib(shader.vao, uv_attrib2 as GLuint);
            // gl::VertexArrayAttribFormat(
            //     shader.vao,
            //     uv_attrib2 as GLuint,
            //     2,
            //     gl::FLOAT,
            //     gl::FALSE,
            //     (2 * std::mem::size_of::<f32>()) as GLuint,
            // );
            // gl::VertexArrayAttribBinding(shader.vao, uv_attrib2 as GLuint, 1);
        }
    }
    pub fn bind_old_geometry_diff_vao(
        vbov: GLuint,
        vbouv: GLuint,
        vdata: &[f32],
        uvdata: &[f32],
        shader: &Shader,
        vao: GLuint
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
                info!("Bind world geom err (vbov): {}", error);
            }
    
            // Bind vertex buffer to the vertex array object
            gl::VertexArrayVertexBuffer(vao, 0, vbov, 0, (5 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after associating vbov with vao: {}", error);
            }
    
            // Position attribute
            let pos_attrib = gl::GetAttribLocation(shader.shader_id, b"position\0".as_ptr() as *const i8);
            if pos_attrib == -1 {
                info!("Error: position attribute not found in shader.");
            } else {
                gl::EnableVertexArrayAttrib(vao, pos_attrib as GLuint);
                gl::VertexArrayAttribFormat(
                    vao,
                    pos_attrib as GLuint,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                );
                gl::VertexArrayAttribBinding(vao, pos_attrib as GLuint, 0);
            }
    
            // Block brightness attribute
            let brightness_attrib = gl::GetAttribLocation(shader.shader_id, b"blockRgb\0".as_ptr() as *const i8);
            if brightness_attrib == -1 {
                info!("Error: blockRgb attribute not found in shader.");
            } else {
                gl::EnableVertexArrayAttrib(vao, brightness_attrib as GLuint);
                gl::VertexArrayAttribIFormat(
                    vao,
                    brightness_attrib as GLuint,
                    1,
                    gl::UNSIGNED_INT,
                    (3 * std::mem::size_of::<f32>()) as GLuint,
                );
                gl::VertexArrayAttribBinding(vao, brightness_attrib as GLuint, 0);
            }
    
            // Ambient brightness attribute
            let amb_brightness = gl::GetAttribLocation(shader.shader_id, b"ambientBright\0".as_ptr() as *const i8);
            if amb_brightness == -1 {
                info!("Error: ambientBright attribute not found in shader.");
            } else {
                gl::EnableVertexArrayAttrib(vao, amb_brightness as GLuint);
                gl::VertexArrayAttribFormat(
                    vao,
                    amb_brightness as GLuint,
                    1,
                    gl::FLOAT,
                    gl::FALSE,
                    (4 * std::mem::size_of::<f32>()) as GLuint,
                );
                gl::VertexArrayAttribBinding(vao, amb_brightness as GLuint, 0);
            }
    
            // Upload UV data to named buffer
            gl::NamedBufferData(
                vbouv,
                (uvdata.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                uvdata.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("Bind world geom err (vbouv): {}", error);
            }
    
            // Bind UV buffer to the vertex array object
            gl::VertexArrayVertexBuffer(vao, 1, vbouv, 0, (4 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after associating vbouv with vao: {}", error);
            }
    
            // UV attribute
            let uv_attrib = gl::GetAttribLocation(shader.shader_id, b"uv\0".as_ptr() as *const i8);
            if uv_attrib == -1 {
                info!("Error: uv attribute not found in shader.");
            } else {
                gl::EnableVertexArrayAttrib(vao, uv_attrib as GLuint);
                gl::VertexArrayAttribFormat(
                    vao,
                    uv_attrib as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                );
                gl::VertexArrayAttribBinding(vao, uv_attrib as GLuint, 1);
            }
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
                info!("Bind world geom err (vbov): {}", error);
            }
    
            // Bind vertex buffer to the vertex array object
            gl::VertexArrayVertexBuffer(shader.vao, 0, vbov, 0, (5 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after associating vbov with vao: {}", error);
            }
    
            // Position attribute
            let pos_attrib = gl::GetAttribLocation(shader.shader_id, b"position\0".as_ptr() as *const i8);
            if pos_attrib == -1 {
                info!("Error: position attribute not found in shader.");
            } else {
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
            }
    
            // Block brightness attribute
            let brightness_attrib = gl::GetAttribLocation(shader.shader_id, b"blockRgb\0".as_ptr() as *const i8);
            if brightness_attrib == -1 {
                info!("Error: blockRgb attribute not found in shader.");
            } else {
                gl::EnableVertexArrayAttrib(shader.vao, brightness_attrib as GLuint);
                gl::VertexArrayAttribIFormat(
                    shader.vao,
                    brightness_attrib as GLuint,
                    1,
                    gl::UNSIGNED_INT,
                    (3 * std::mem::size_of::<f32>()) as GLuint,
                );
                gl::VertexArrayAttribBinding(shader.vao, brightness_attrib as GLuint, 0);
            }
    
            // Ambient brightness attribute
            let amb_brightness = gl::GetAttribLocation(shader.shader_id, b"ambientBright\0".as_ptr() as *const i8);
            if amb_brightness == -1 {
                info!("Error: ambientBright attribute not found in shader.");
            } else {
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
            }
    
            // Upload UV data to named buffer
            gl::NamedBufferData(
                vbouv,
                (uvdata.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                uvdata.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("Bind world geom err (vbouv): {}", error);
            }
    
            // Bind UV buffer to the vertex array object
            gl::VertexArrayVertexBuffer(shader.vao, 1, vbouv, 0, (4 * std::mem::size_of::<f32>()) as GLsizei);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after associating vbouv with vao: {}", error);
            }
    
            // UV attribute
            let uv_attrib = gl::GetAttribLocation(shader.shader_id, b"uv\0".as_ptr() as *const i8);
            if uv_attrib == -1 {
                info!("Error: uv attribute not found in shader.");
            } else {
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
            }
        }
    }
    
    pub fn bind_geometry(
        vbo32: gl::types::GLuint,
        vbo8: gl::types::GLuint,
        vbo8rgb: GLuint,
        upload: bool,
        shader: &Shader,
        data: (&Mutex<Vec<u32>>, &Mutex<Vec<u8>>, &Mutex<Vec<u16>>),
    ) {
        //info!("BInding geomery"); //Ah yes praise the lord when this is commented out it means nothing is wrong 
        unsafe {
            if upload {
                let datalock = data.0.lock();
                gl::NamedBufferData(
                    vbo32,
                    (datalock.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                    datalock.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW,
                );

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!(
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
                info!("OpenGL Error after associating vbo32 with vao: {}", error);
            }
            if upload {
                let u32one_attrib =
                    gl::GetAttribLocation(shader.shader_id, b"u32\0".as_ptr() as *const i8)
                        as gl::types::GLuint;

                gl::EnableVertexArrayAttrib(shader.vao, u32one_attrib);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u32 array attrib: {}", error);
                }

                gl::VertexArrayAttribIFormat(shader.vao, u32one_attrib, 1, gl::UNSIGNED_INT, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u32 attrib format: {}", error);
                }
                gl::VertexArrayAttribBinding(shader.vao, u32one_attrib, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u32 attrib binding: {}", error);
                }
                let data1lock = data.1.lock();
                gl::NamedBufferData(
                    vbo8,
                    (data1lock.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr,
                    data1lock.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW,
                );

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!(
                        "OpenGL Error after named buffering of vbo8 with upload true: {}",
                        error
                    );
                }

                drop(data1lock);


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
                info!("OpenGL Error after associating vbo8 with vao: {}", error);
            }
            if upload {
                let u8_attrib =
                    gl::GetAttribLocation(shader.shader_id, b"eightbit\0".as_ptr() as *const i8)
                        as gl::types::GLuint;
                //info!("U8 attrib location: {}", u8_attrib);
                gl::EnableVertexArrayAttrib(shader.vao, u8_attrib);

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u8 array attrib: {}", error);
                }

                gl::VertexArrayAttribIFormat(shader.vao, u8_attrib, 1, gl::UNSIGNED_BYTE, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u8 array attrib format: {}", error);
                }

                gl::VertexArrayAttribBinding(shader.vao, u8_attrib, 1);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u8 array attrib binding: {}", error);
                }
            }

            gl::VertexArrayVertexBuffer(
                shader.vao,
                2,
                vbo8rgb,
                0,
                std::mem::size_of::<u16>() as i32,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after associating vbo8 with vao: {}", error);
            }
            if upload {

                let data2lock = data.2.lock();
                gl::NamedBufferData(
                    vbo8rgb,
                    (data2lock.len() * std::mem::size_of::<u16>()) as gl::types::GLsizeiptr,
                    data2lock.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW,
                );

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!(
                        "OpenGL Error after named buffering of vbo8rgb with upload true: {}",
                        error
                    );
                }


                let u8rgb_attrib =
                    gl::GetAttribLocation(shader.shader_id, b"rgb\0".as_ptr() as *const i8)
                        as gl::types::GLuint;
                //info!("U8 attrib location: {}", u8_attrib);
                gl::EnableVertexArrayAttrib(shader.vao, u8rgb_attrib);

                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u8 array attrib: {}", error);
                }

                gl::VertexArrayAttribIFormat(shader.vao, u8rgb_attrib, 1, gl::UNSIGNED_SHORT, 0);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u8 array attrib format: {}", error);
                }

                gl::VertexArrayAttribBinding(shader.vao, u8rgb_attrib, 2);
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    info!("OpenGL Error after u8 array attrib binding: {}", error);
                }
            }
        }
    }
}
