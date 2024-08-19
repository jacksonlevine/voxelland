use std::sync::{Arc};
use parking_lot::{Mutex, RwLock};

use glam::Vec2;
use glfw::PWindow;

use crate::{shader::Shader, text::Text, texture::Texture};

pub struct GuiSystem {
    pub texts: Vec<Text>,
    pub window: Arc<RwLock<PWindow>>,
    pub menu_shader: Shader
}

impl GuiSystem {
    pub fn new(window: &Arc<RwLock<PWindow>>, texture: &Texture) -> GuiSystem {
        let menu_shader = Shader::new(
            path!("assets/menuvert.glsl"), 
            path!("assets/menufrag.glsl")
        );
        let mut texts = Vec::new();

        texts.push(Text::new("", &window.read(), menu_shader.shader_id, Vec2::new(100.0,100.0), texture.id));

        texts.push(Text::new("Press B to board the ship.", &window.read(), menu_shader.shader_id, Vec2::new(800.0,100.0), texture.id));

        GuiSystem {
            texts,
            window: window.clone(),
            menu_shader
        }
    }

    pub fn draw_texts(&self) {
        for text in &self.texts {
            text.draw();
        }
    }

    pub fn draw_text(&self, ind: usize) {
        self.texts[ind].draw();
    }
}
