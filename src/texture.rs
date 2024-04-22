use image::{self, GenericImageView};

pub struct Texture {
    id: gl::types::GLuint,
}

impl Texture {
    pub fn new(texpath: &'static str) -> Result<Texture, String> {
        let mut tex = Texture { id: 0 };
        let img = match image::open(texpath) {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to load texture {}", e)),
        };
        let (width, height) = img.dimensions();
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut tex.id);
            gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TextureParameteri(tex.id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(tex.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TextureStorage2D(tex.id, 1, gl::RGBA, width as i32, height as i32); // Optionally create storage first
            gl::TextureSubImage2D(
                tex.id,
                0,
                0,
                0,
                width as i32,
                height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.to_rgba8().as_flat_samples().as_slice().as_ptr() as *const gl::types::GLvoid,
            );
        }
        Ok(tex)
    }
    pub fn add_to_unit(&self, unit: u32) {
        unsafe {
            gl::BindTextureUnit(unit as u32, self.id)
        }
    }
}
