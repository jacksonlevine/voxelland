use gl::types::{GLsizeiptr, GLuint, GLvoid};
use glam::{Mat4, Vec3};

use crate::shader::Shader;






pub struct SelectCube {
    shader: Shader,
    vbo: GLuint
}


impl SelectCube {
    pub fn new() -> SelectCube {
        let shader = Shader::new("assets/wfvert.glsl", "assets/wffrag.glsl");

        unsafe {
            gl::BindVertexArray(shader.vao);
        }
        let mut vbo: GLuint = 0;

        let faces: [f32; 72] = [

            -0.501, -0.501, -0.501,  0.501, -0.501, -0.501, // Bottom ace
            0.501, -0.501, -0.501,   0.501, -0.501,  0.501,
            0.501, -0.501,  0.501,  -0.501, -0.501,  0.501,
            -0.501, -0.501,  0.501, -0.501, -0.501, -0.501,
    
            -0.501,  0.501, -0.501,  0.501,  0.501, -0.501, // Top ace
            0.501,  0.501, -0.501,   0.501,  0.501,  0.501,
            0.501,  0.501,  0.501,  -0.501,  0.501,  0.501,
            -0.501,  0.501,  0.501, -0.501,  0.501, -0.501,
    
            -0.501, -0.501, -0.501, -0.501,  0.501, -0.501, // Side Edges
            0.501, -0.501, -0.501,   0.501,  0.501, -0.501,
            0.501, -0.501,  0.501,   0.501,  0.501,  0.501,
            -0.501, -0.501,  0.501, -0.501,  0.501,  0.501
    
        ];

        unsafe {
            gl::CreateBuffers(1, &mut vbo);
            gl::NamedBufferData(vbo, (faces.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            faces.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW);

            gl::VertexArrayVertexBuffer(shader.vao, 0, vbo, 0, (3 * std::mem::size_of::<f32>()) as i32);
            gl::EnableVertexArrayAttrib(shader.vao, 0);
            gl::VertexArrayAttribFormat(shader.vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(shader.vao, 0, 0);
            
        }
        SelectCube {
            shader,
            vbo
        }
    }

    pub fn draw_at(&self, pos: Vec3, mvp: &Mat4, walkbob: f32) {
        unsafe {
            gl::BindVertexArray(self.shader.vao);
            gl::UseProgram(self.shader.shader_id);
            let mvp_loc = gl::GetUniformLocation(self.shader.shader_id, b"mvp\0".as_ptr() as *const i8);

            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, mvp.to_cols_array().as_ptr());

            let t_loc = gl::GetUniformLocation(self.shader.shader_id, b"translation\0".as_ptr() as *const i8);

            gl::Uniform3f(
                t_loc,
                pos.x,
                pos.y,
                pos.z
            );

            gl::Uniform1f(
                gl::GetUniformLocation(self.shader.shader_id, b"walkbob\0".as_ptr() as *const i8),
                walkbob
            );

            gl::PolygonMode( gl::FRONT_AND_BACK, gl::LINE );

            gl::DrawArrays(gl::LINES, 0, 24);

            gl::PolygonMode( gl::FRONT_AND_BACK, gl::FILL );
        }
    }
}