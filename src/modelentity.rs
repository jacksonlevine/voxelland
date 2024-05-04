

use glam::*;

static mut CURRENT_ID: u32 = 0;

pub struct ModelEntity {
    pub model_index: usize,
    pub pos: Vec3,
    pub lastpos: Vec3,
    pub id: u32,
    pub scale: f32,
    pub rot: Vec3
}

impl ModelEntity {


    pub fn new(model_index: usize, pos: Vec3, scale: f32, rot: Vec3) -> ModelEntity {
        unsafe {
            CURRENT_ID += 1;
            ModelEntity {
                model_index,
                pos,
                lastpos: pos,
                id: CURRENT_ID + 1,
                scale,
                rot
            }
        }
        
    }



    pub fn set_pos(&mut self, newpos: Vec3) {
        self.lastpos = self.pos;
        self.pos = newpos;
    }
}