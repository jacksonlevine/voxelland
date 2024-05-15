use gl;
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;

#[derive(Clone)]
pub struct Shader {
    pub shader_id: gl::types::GLuint,
    pub vao: gl::types::GLuint,
}

impl Shader {
    pub fn new(vertpath: &str, fragpath: &str) -> Shader {
        let vertex_shader = Self::compile_shader(vertpath, gl::VERTEX_SHADER);
        let fragment_shader = Self::compile_shader(fragpath, gl::FRAGMENT_SHADER);
        let shader_prog = Self::link_shader_program(vertex_shader, fragment_shader);

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        let mut vao: u32 = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL Error after creating vertex array: {}", error);
            }
        }

        Shader {
            shader_id: shader_prog,
            vao,
        }
    }

    fn compile_shader(path: &str, shader_type: gl::types::GLenum) -> gl::types::GLuint {
        let mut file = File::open(path).unwrap();
        let mut shader_source = String::new();
        file.read_to_string(&mut shader_source).unwrap();
        let shader_source_c_str = std::ffi::CString::new(shader_source.as_bytes()).unwrap();

        let shader = unsafe { gl::CreateShader(shader_type) };

        unsafe {
            gl::ShaderSource(shader, 1, &shader_source_c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut success = gl::FALSE as gl::types::GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as gl::types::GLint {
                let mut log = vec![0; 512];
                gl::GetShaderInfoLog(
                    shader,
                    512,
                    ptr::null_mut(),
                    log.as_mut_ptr() as *mut gl::types::GLchar,
                );
                println!(
                    "ERROR::SHADER::COMPILATION_FAILED\n{}",
                    str::from_utf8(&log).unwrap()
                );
            }
        }

        shader
    }

    fn link_shader_program(
        vertex_shader: gl::types::GLuint,
        fragment_shader: gl::types::GLuint,
    ) -> gl::types::GLuint {
        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            let mut success = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as gl::types::GLint {
                let mut log = vec![0; 512];
                gl::GetProgramInfoLog(
                    program,
                    512,
                    ptr::null_mut(),
                    log.as_mut_ptr() as *mut gl::types::GLchar,
                );
                println!(
                    "ERROR::PROGRAM::LINKING_FAILED\n{}",
                    str::from_utf8(&log).unwrap()
                );
            }
        }

        program
    }
}
