



pub struct Planets {}


impl Planets {
    pub fn get_voxel_model_index_range(dim_id: u32) -> (usize, usize) {
        //inclusive range of what voxel model indexes go in each dimension
        return match dim_id {
            0 => {
                (0, 13)
            }
            1 => {
                (14, 16)
            }
            _ => {
                (0, 0)
            }
        }
    }
    pub fn get_floor_blocks(dim_id: u32) -> Vec<u32> {
        return match dim_id {
            0 => vec![3, 34],
            1 => vec![1],
            _ => vec![3, 34]
        }
    }
    pub fn get_fog_col(dim_id: u32) -> (f32, f32, f32, f32) {
        return match dim_id {
            0 => {
                (0.4, 0.75, 1.0, 1.0)
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
            3 => {
                Some("assets/sfx/monster2.mp3")
            }
            _ => {
                None
            }
        }
    }

    pub fn get_mob_volume(modelindex: usize) -> f32 {
        match modelindex {
            6 => {
                0.02
            }
            3 => {
                0.6
            }
            _ => {
                0.5
            }
        }
    }

    pub fn get_mob_attack_interval(modelindex: usize) -> f32 {
        match modelindex {
            3 => {
                0.2
            }
            _ => {
                1.0
            }
        }
    }

    pub fn get_mob_sound_interval(modelindex: usize) -> f32 {
        match modelindex {
            3 => {
                5.0
            }
            _ => {
                10.0
            }
        }
    }
}