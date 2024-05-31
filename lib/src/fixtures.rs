use glfw::PWindow;
use image::GenericImageView;
use crate::glyphface::GlyphFace;
use crate::textureface::TextureFace;
use crate::MousePos;

type FixtureFunc = Box<dyn Fn()>;

pub struct Fixture {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub tooltip: String,
    pub texx: i8,
    pub texy: i8,
    pub func: FixtureFunc
}

pub struct Fixtures {
    pub fixtures: Vec<Fixture>,
    pub dirty: bool,
    pub vbo: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub data: Vec<f32>,
    pub texture: gl::types::GLuint,
    pub moused_over_id: f32,
    pub clicked_on_id: f32,
    pub guitexpixels: Vec<u8>
}

impl Fixtures {
    pub fn change_tex_coords(&mut self, tooltip: String, newtexx: i8, newtexy: i8) {
        let found_fix = self.fixtures.iter_mut().find(|item| item.tooltip == tooltip);
        match found_fix {
            Some(fix) => {
                fix.texx = newtexx;
                fix.texy = newtexy;
                self.dirty = true;
            },
            None => ()
        };
    }
    pub fn get_moused_over(&mut self, mouse: &MousePos, windowwidth: i32, windowheight: i32) {
        for (index, fix) in self.fixtures.iter().enumerate() {
            let spx: f32 = (fix.x + 1.0) * 0.5;
            let spy: f32 = (1.0 - (fix.y+fix.height)) * 0.5;
            let sw: f32 = fix.width/2.0;
            let sh: f32 = fix.height/2.0;
            let id: f32 = (index + 1) as f32;

            if  (mouse.x as f32) > spx * windowwidth as f32 &&
                (mouse.x as f32) < (spx + sw) * windowwidth as f32 &&
                (mouse.y as f32) > spy * windowheight as f32 &&
                (mouse.y as f32) < (spy + sh) * windowheight as f32
            {
                self.moused_over_id = id;
                return ();
            }
        }
        self.moused_over_id = 0.0;
    }
    pub fn new() -> Result<Fixtures, String> {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        let mut texture: gl::types::GLuint = 0;
        let img = match image::open("assets/gui.png") {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to load texture: {}", e))
        };
        let (width, height) = img.dimensions();
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
    
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.to_rgba8().as_flat_samples().as_slice().as_ptr() as *const gl::types::GLvoid,
            );
    
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let mut guipix: Vec<u8> = Vec::new();
        guipix.extend_from_slice(img.to_rgba8().as_flat_samples().as_slice());

        guipix = guipix
            .chunks(4)
            .map(|chunk| chunk[0])
            .collect();

        Ok(
            Fixtures {
                fixtures: Vec::new(),
                dirty: true,
                vbo: 0,
                vao,
                data: Vec::new(),
                texture,
                moused_over_id: 0.0,
                clicked_on_id: 0.0,
                guitexpixels: guipix
            }
        )
    }

    pub fn set_fixtures(&mut self, fixs: Vec<Fixture>) {
        self.fixtures = fixs;
        self.dirty = true;
    }

    fn rebuild_geometry(&mut self) {
        let mut data: Vec<f32> = Vec::new();
        for (index, fix) in self.fixtures.iter().enumerate() {
            let t = TextureFace::new(fix.texx, fix.texy);
            data.extend_from_slice(&[
                fix.x,           fix.y,            t.blx, t.bly,  index as f32 + 1.0,
                fix.x,           fix.y+fix.height, t.tlx, t.tly,  index as f32 + 1.0,
                fix.x+fix.width, fix.y+fix.height, t.trx, t.tr_y, index as f32 + 1.0,

                fix.x+fix.width, fix.y+fix.height, t.trx, t.tr_y, index as f32 + 1.0,
                fix.x+fix.width, fix.y,            t.brx, t.bry,  index as f32 + 1.0,
                fix.x,           fix.y,            t.blx, t.bly,  index as f32 + 1.0,
            ]);
        }
        self.data = data;
    }

    fn bind_geometry(&self, vbo: gl::types::GLuint, upload: bool, shader: gl::types::GLuint, data: &Vec<f32>) {

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            if upload {
                gl::BufferData(
                    gl::ARRAY_BUFFER, 
                    (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, 
                    data.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW);
            }
            let pos_attrib = gl::GetAttribLocation(
                shader, 
                b"pos\0".as_ptr() as *const i8) as gl::types::GLuint;
            gl::EnableVertexAttribArray(
                pos_attrib);
            gl::VertexAttribPointer(
                pos_attrib, 
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null());
        
            let tex_attrib = gl::GetAttribLocation(
                shader, 
                b"texcoord\0".as_ptr() as *const i8) as gl::types::GLuint;
            gl::EnableVertexAttribArray(
                tex_attrib);
            gl::VertexAttribPointer(tex_attrib,
                 2, 
                 gl::FLOAT, 
                 gl::FALSE, 
                 (5 * std::mem::size_of::<f32>()) as gl::types::GLint, 
                 (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        
            let element_id_attrib = gl::GetAttribLocation(
                shader, 
                b"elementid\0".as_ptr() as *const i8) as gl::types::GLuint;

            gl::EnableVertexAttribArray(element_id_attrib);
            gl::VertexAttribPointer(
                element_id_attrib, 
                1, 
                gl::FLOAT, 
                gl::FALSE, 
                (5 * std::mem::size_of::<f32>()) as gl::types::GLint, 
                (4 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        }
    }

    pub fn draw(&mut self, shader: gl::types::GLuint) {
        unsafe {
            gl::BindVertexArray(self.vao);
            if self.dirty {
                
                    gl::DeleteBuffers(1, &self.vbo);
                    gl::GenBuffers(1, &mut self.vbo);
                    self.rebuild_geometry();
                    self.bind_geometry(self.vbo, true, shader, &self.data);
                self.dirty = false;
            } else {
                self.bind_geometry(self.vbo, false, shader, &self.data);
            }
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::UseProgram(shader);
            gl::DrawArrays(gl::TRIANGLES, 0, (self.fixtures.len() * 6) as i32);
        }
    }

    pub fn draw_tooltip(&mut self, window: &PWindow, menu_shader: gl::types::GLuint) {
        static mut VBO: gl::types::GLuint = 0;
        let (xpos, ypos) = window.get_cursor_pos();
        let (window_width, window_height) = window.get_size();

        let normalized_x = xpos / window_width as f64;
        let normalized_y = 1.0 - (ypos / window_height as f64);

        let mut mousex = (normalized_x * 2.0 - 1.0) as f32;
        let mousey = (normalized_y * 2.0 - 1.0) as f32;

        let gwidth = 32.0/window_width as f32;
        let gheight = 32.0/window_height as f32;

        if(self.moused_over_id != 0.0) {
            let fix = &(self.fixtures[(self.moused_over_id - 1.0) as usize]);
            let tooltip = &(fix.tooltip);
            let letters_count = tooltip.len();
            let letters_width = letters_count as f32 * gwidth;

            if mousex > 0.0 {
                mousex -= letters_width as f32;
            }

            let mut letters_geometry: Vec<f32> = Vec::new();
            let mut g = GlyphFace::new(0);

            for i in 0..letters_count {
                g.set_char(tooltip.as_bytes()[i]);
                
                letters_geometry.extend_from_slice(&[
                    i as f32 * gwidth + mousex,          mousey,            g.blx, g.bly,  -1.0,
                    i as f32 * gwidth + mousex,          mousey + gheight,  g.tlx, g.tly,  -1.0,
                    i as f32 * gwidth + mousex + gwidth, mousey + gheight,  g.trx, g.tr_y, -1.0,

                    i as f32 * gwidth + mousex + gwidth, mousey + gheight,  g.trx, g.tr_y, -1.0,
                    i as f32 * gwidth + mousex + gwidth, mousey,            g.brx, g.bry,  -1.0,
                    i as f32 * gwidth + mousex,          mousey,            g.blx, g.bly,  -1.0,
                ]);
            }
            unsafe {
                gl::DeleteBuffers(1, &VBO);
                gl::GenBuffers(1, &mut VBO);
                self.bind_geometry(VBO, true, menu_shader, &letters_geometry);
                gl::DrawArrays(gl::TRIANGLES, 0, (letters_count*6) as i32);
            }
        }
        
    }
}