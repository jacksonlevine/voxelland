use crate::model;



pub struct Planets {}


impl Planets {
    pub fn get_voxel_model_index_range(dim_id: u32) -> (usize, usize) {
        //inclusive range of what voxel model indexes go in each dimension
        return match dim_id {
            0 => {
                (0, 9)
            }
            1 => {
                (10, 12)
            }
            _ => {
                (0, 0)
            }
        }
    }
    pub fn get_floor_block(dim_id: u32) -> u32 {
        return match dim_id {
            0 => 3,
            1 => 1,
            _ => 3
        }
    }
    pub fn get_fog_col(dim_id: u32) -> (f32, f32, f32, f32) {
        return match dim_id {
            0 => {
                (0.7, 0.8, 1.0, 1.0)
            }
            1 => {
                (0.5, 0.0, 0.0, 1.0)
            }
            _ => {
                (0.7, 0.8, 1.0, 1.0)
            }
        }
    }



    pub fn get_mob_sound(modelindex: usize) -> Option<&'static str> {
        match modelindex {
            6 => {
                Some("assets/sfx/cricket1.mp3")
            }
            _ => {
                None
            }
        }
    }
}