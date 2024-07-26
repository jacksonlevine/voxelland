use glam::IVec4;
use image::{self, GenericImageView};
use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;
use tracing::info;
pub struct Texture {
    pub id: gl::types::GLuint,
    pub data: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub size: (u32, u32)
}

impl Texture {
    pub fn new(texpath: &'static str) -> Result<Texture, String> {
        let mut id = 0;
        let img = match image::open(texpath) {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to load texture {}", e)),
        };
        let (width, height) = img.dimensions();
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after creating texture: {}", error);
            }
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TextureParameteri(id, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(id, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after texture params: {}", error);
            }
            gl::TextureStorage2D(id, 1, gl::RGBA8, width as i32, height as i32); // Optionally create storage first
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after creating texture storage: {}", error);
            }
            let data: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = img.to_rgba8().clone();
            

            gl::TextureSubImage2D(
                id,
                0,
                0,
                0,
                width as i32,
                height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_flat_samples().as_slice().as_ptr() as *const gl::types::GLvoid,
            );
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after texture subbing: {}", error);
            }
            Ok(Texture {
                id,
                data,
                size: (width, height)
            })
        }
        
    }

    pub fn update_texture(&mut self, delta_time: f32) {
        let pix = self.data.as_mut();
    
        let width = 544;
        let chans = 4;
    
        static mut TIMER: f32 = 0.0;
        static mut PERL: Lazy<Perlin> = Lazy::new(|| Perlin::new(0));
        static mut CONVEYOR_TIMER: f32 = 0.0;
    
        unsafe {
            if TIMER < 100.0 {
                TIMER += delta_time * 2.0;
            } else {
                TIMER = 0.0;
            }
    
            // Water
            let base_color = IVec4::new(0, 45, 100, 140);
            let mut coord = glam::IVec2::new(0, 0);
    
            let start_y = 256 + 270;
            let start_x = 36;
            let square_size = 18;
            for y in start_y..(start_y + square_size) {
                coord.x = 0;
                for x in start_x..(start_x + square_size) {
                    let i = (y * width + x) * chans;
                    let added_noise: f64 = (PERL.get([(coord.x as f64 / 4.0), TIMER as f64, (coord.y as f64 / 4.0)]) * 70.0).max(-10.0);
    
                    pix[i] = ((base_color.x + (added_noise) as i32).max(0)).min(254) as u8;
                    pix[i + 1] = ((base_color.y + (added_noise) as i32).max(0)).min(254) as u8;
                    pix[i + 2] = ((base_color.z + (added_noise) as i32).max(0)).min(254) as u8;
                    pix[i + 3] = ((base_color.w).max(0)).min(254) as u8;
                    coord.x += 1;
                }
                coord.y += 1;
            }
    
            // Conveyor Top
            if CONVEYOR_TIMER >= 0.02 {
                CONVEYOR_TIMER = 0.0;
    
                let start_y: usize = 418;
                let start_x: usize = 180;
                let square_size: usize = 18;
    
                // Create a temporary buffer to hold the shifted pixels
                let mut temp_buffer = vec![0u8; (square_size * square_size * chans) as usize];
    
                // Shift each row up by one and wrap the top row to the bottom
                for y in 0..square_size {
                    for x in 0..square_size {
                        let src_y = (y + 1) % square_size;
                        let src_index = ((start_y + src_y) * width + (start_x + x)) * chans;
                        let dest_index = (y * square_size + x) * chans;
    
                        temp_buffer[dest_index] = pix[src_index];
                        temp_buffer[dest_index + 1] = pix[src_index + 1];
                        temp_buffer[dest_index + 2] = pix[src_index + 2];
                        temp_buffer[dest_index + 3] = pix[src_index + 3];
                    }
                }
    
                // Copy the temporary buffer back to the texture
                for y in 0..square_size {
                    for x in 0..square_size {
                        let dest_index = ((start_y + y) * width + (start_x + x)) * chans;
                        let src_index = (y * square_size + x) * chans;
    
                        pix[dest_index] = temp_buffer[src_index];
                        pix[dest_index + 1] = temp_buffer[src_index + 1];
                        pix[dest_index + 2] = temp_buffer[src_index + 2];
                        pix[dest_index + 3] = temp_buffer[src_index + 3];
                    }
                }
            } else {
                CONVEYOR_TIMER += delta_time;
            }
        }
    
        unsafe {
            gl::TextureSubImage2D(
                self.id,
                0,
                0,
                0,
                self.size.0 as i32,
                self.size.1 as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.data.as_flat_samples().as_slice().as_ptr() as *const gl::types::GLvoid,
            );
        }
    }
    pub fn add_to_unit(&self, unit: u32) {
        unsafe {
            gl::BindTextureUnit(unit as u32, self.id);
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                info!("OpenGL Error after binding texture unit: {}", error);
            }
        }
    }
}
