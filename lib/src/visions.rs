use glfw::ffi::glfwGetTime;

use crate::game::*;


impl Game {




    pub fn draw_current_vision(&self, overlayfade: f32) {

        match self.current_vision {
            Some(VisionType::Model(model_index)) => {

                
                unsafe {

                    gl::Clear(gl::DEPTH_BUFFER_BIT);

                    //gl::Disable(gl::DEPTH_TEST);
                    gl::Disable(gl::CULL_FACE);
                    gl::UseProgram(self.modelshader.shader_id);
            let mvp_loc = gl::GetUniformLocation(self.modelshader.shader_id, b"mvp\0".as_ptr() as *const i8);


            gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, self.visions_camera.mvp.to_cols_array().as_ptr());
            gl::Uniform1i(
                gl::GetUniformLocation(
                    self.modelshader.shader_id,
                    b"ourTexture\0".as_ptr() as *const i8,
                ),
                1,
            );



                    let index = model_index;
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
                                1.0,
                            );

                                    gl::Uniform3f(
                                        gl::GetUniformLocation(
                                            self.modelshader.shader_id,
                                            b"pos\0".as_ptr() as *const i8,
                                        ),
                                        0.0,0.0,0.0
                                    );



                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"interp_time\0".as_ptr() as *const i8,
                                ),
                                1.0
                            );

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"lastpos\0".as_ptr() as *const i8,
                                ),
                                0.0, 0.0,0.0
                            );
                            

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"xrot\0".as_ptr() as *const i8,
                                ),
                                0.0,
                            );
                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"yrot\0".as_ptr() as *const i8,
                                ),
                                3.14159,
                            );

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"opacity\0".as_ptr() as *const i8,
                                ),
                                overlayfade
                            );

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"zrot\0".as_ptr() as *const i8,
                                ),
                                0.0,
                            );

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"ambientBrightMult\0".as_ptr() as *const i8,
                                ),
                                1.0,
                            );

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"camPos\0".as_ptr() as *const i8,
                                ),
                                self.visions_camera.position.x,
                                self.visions_camera.position.y,
                                self.visions_camera.position.z
                            );

                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"lastrot\0".as_ptr() as *const i8,
                                ),
                                0.0,0.0,0.0
                            );


                            gl::Uniform3f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"camDir\0".as_ptr() as *const i8,
                                ),
                                self.visions_camera.direction.x,
                                self.visions_camera.direction.y,
                                self.visions_camera.direction.z
                            );

                            gl::Uniform1f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"viewDistance\0".as_ptr() as *const i8,
                                ),
                                8.0
                            );

                            gl::Uniform4f(
                                gl::GetUniformLocation(
                                    self.modelshader.shader_id,
                                    b"fogCol\0".as_ptr() as *const i8,
                                ),
                                1.0, 0.0 , 0.0, 1.0
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
                //gl::Enable(gl::DEPTH_TEST);
                    gl::Enable(gl::CULL_FACE);
                }
            }
            Some(VisionType::Vox(vox_index)) => {

            }
            None => {

            }
        }



    }
}