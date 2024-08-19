use gl::types::{GLsizeiptr, GLuint, GLvoid};
use glam::{Mat4, Vec3};

use crate::shader::Shader;

pub struct BlockOverlay {
    shader: Shader,
    vbo: GLuint
}

impl BlockOverlay {
    pub fn new(texture: GLuint) -> BlockOverlay {
        let shader = Shader::new(path!("assets/bovert.glsl"), path!("assets/bofrag.glsl"));
        #[cfg(feature = "glfw")]
        unsafe {
            gl::BindVertexArray(shader.vao);
            gl::BindTextureUnit(0, texture);
        }
        let mut vbo: GLuint = 0;

        let faces: [f32; 144] = [
        -0.55, -0.5, 0.5, 0.0,
        -0.55, -0.5, -0.5, 1.0,
        -0.55, 0.5, -0.5, 2.0,

        -0.55, 0.5, -0.5, 3.0,
        -0.55, 0.5, 0.5, 4.0,
        -0.55, -0.5, 0.5, 5.0,

                0.55, -0.5, -0.5, 0.0,
                0.55, -0.5, 0.5, 1.0,
                0.55, 0.5, 0.5,2.0,

                0.55, 0.5, 0.5, 3.0,
                0.55, 0.5, -0.5, 4.0,
                0.55, -0.5, -0.5,5.0,

        0.5, -0.5, 0.55, 0.0,
        -0.5, -0.5, 0.55,1.0,
        -0.5, 0.5, 0.55, 2.0,

        -0.5, 0.5, 0.55, 3.0,
        0.5, 0.5, 0.55,  4.0,
        0.5, -0.5, 0.55, 5.0,

                -0.5, -0.5, -0.55, 0.0,
                0.5, -0.5, -0.55,1.0,
                0.5, 0.5, -0.55,2.0,

                0.5, 0.5, -0.55, 3.0,
                -0.5, 0.5, -0.55, 4.0,
                -0.5, -0.5, -0.55,5.0,

        -0.5, 0.55, -0.5, 0.0,
        0.5, 0.55, -0.5,1.0,
        0.5, 0.55, 0.5,2.0,

        0.5, 0.55, 0.5, 3.0,
        -0.5, 0.55, 0.5, 4.0,
        -0.5, 0.55, -0.5,5.0,

                0.5, -0.55, -0.5, 0.0,
                -0.5, -0.55, -0.5,1.0,
                -0.5, -0.55, 0.5,2.0,

                -0.5, -0.55, 0.5, 3.0,
                0.5, -0.55, 0.5, 4.0,
                0.5, -0.55, -0.5,5.0,
    
        ];
        #[cfg(feature = "glfw")]
        unsafe {
            gl::CreateBuffers(1, &mut vbo);
            gl::NamedBufferData(vbo, (faces.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            faces.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW);

            gl::VertexArrayVertexBuffer(shader.vao, 0, vbo, 0, (4 * std::mem::size_of::<f32>()) as i32);
            gl::EnableVertexArrayAttrib(shader.vao, 0);
            gl::VertexArrayAttribFormat(shader.vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(shader.vao, 0, 0);

            gl::EnableVertexArrayAttrib(shader.vao, 1);
            gl::VertexArrayAttribFormat(shader.vao, 1, 1, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as u32);
            gl::VertexArrayAttribBinding(shader.vao, 1, 0);

            
            
        }
        BlockOverlay {
            shader,
            vbo
        }
    }

    pub fn draw_at(&self, pos: Vec3, breakprogress: i8, mvp: &Mat4, walkbob: f32) {
        #[cfg(feature = "glfw")]
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::BindVertexArray(self.shader.vao);
            gl::UseProgram(self.shader.shader_id);

            let mvp_loc = gl::GetUniformLocation(self.shader.shader_id, b"mvp\0".as_ptr() as *const i8);

            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, mvp.to_cols_array().as_ptr());

            let bp_loc = gl::GetUniformLocation(self.shader.shader_id, b"breakPhase\0".as_ptr() as *const i8);

            gl::Uniform1f(
                bp_loc,
                breakprogress as f32
            );

            let bp_loc = gl::GetUniformLocation(self.shader.shader_id, b"blockPosition\0".as_ptr() as *const i8);

            gl::Uniform3f(
                bp_loc,
                pos.x,
                pos.y,
                pos.z
            );

            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.shader.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                0,
            );

            gl::Uniform1f(
                gl::GetUniformLocation(
                    self.shader.shader_id,
                    b"walkbob\0".as_ptr() as *const i8,
                ),
                walkbob,
            );


            // info!("Break phase: {breakprogress} Pos {} {} {}", pos.x, pos.y, pos.z);

            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::Enable(gl::CULL_FACE);

        }
    }
}
