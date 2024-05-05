

use std::ops::Bound;

use glam::*;

use crate::{collisioncage::{BoundBox, CollCage}, vec};

static mut CURRENT_ID: u32 = 0;

pub struct ModelEntity {
    pub model_index: usize,
    pub pos: Vec3,
    pub lastpos: Vec3,
    pub id: u32,
    pub scale: f32,
    pub rot: Vec3,
    pub coll_cage: CollCage,
    pub velocity: Vec3,
    pub grounded: bool,
    pub time_falling_scalar: f32,
    pub jumping_up: bool,
    pub allowable_jump_height: f32,
    pub current_jump_y: f32,
    pub bound_box: BoundBox
}

impl ModelEntity {


    pub fn new(model_index: usize, pos: Vec3, scale: f32, rot: Vec3) -> ModelEntity {

        let solid_pred: Box<dyn Fn(vec::IVec3) -> bool> = {
            //let csys_arc = Arc::clone(&chunksys);
            Box::new(move |_: vec::IVec3| {
                return false;
            })
        };

        unsafe {
            CURRENT_ID += 1;
            ModelEntity {
                model_index,
                pos,
                lastpos: pos,
                id: CURRENT_ID + 1,
                scale,
                rot,
                coll_cage: CollCage::new(solid_pred),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                grounded: false,
                time_falling_scalar: 1.0,
                jumping_up: false,
                allowable_jump_height: 2.0,
                current_jump_y: 0.0,
                bound_box: BoundBox::new(Vec3::new(0.0,0.0,0.0))
            }
        }
        
    }



    pub fn set_pos(&mut self, newpos: Vec3) {
        self.lastpos = self.pos;
        self.pos = newpos;
    }
}