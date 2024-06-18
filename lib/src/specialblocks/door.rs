use once_cell::sync::Lazy;

use crate::specialblocks::vertexutils::rotate_coordinates_around_y_negative_90;
use crate::textureface::TextureFace;
use crate::textureface::TEXTURE_WIDTH;









pub struct DoorInfo {
    
}

pub const DOOROPEN_BITS: u32 = 0b0000_0000_0000_0100_0000_0000_0000_0000;
pub const DOORTOP_BITS: u32 = 0b0000_0000_0000_1000_0000_0000_0000_0000;

pub const OPPOSITEDOOR_BITS: u32 = 0b0000_0000_0001_0000_0000_0000_0000_0000;
pub const BLOCK_DIRECTION_BITS: u32 = 0b0000_0000_0000_0011_0000_0000_0000_0000;




impl DoorInfo {

    pub fn get_direction_bits(input: u32) -> u32 {
        return (input & BLOCK_DIRECTION_BITS) >> 16;
    }

    pub fn set_direction_bits(input: &mut u32, direction: u32) {
        let bits = direction << 16;
        *input |= bits;
    }

    pub fn door_model_from_index(index: usize) -> &'static Vec<f32> {
        static models: Lazy<Vec<Vec<f32>>> = Lazy::new(|| {
            vec![
                DoorInfo::base_door_model().to_vec(),
                rotate_coordinates_around_y_negative_90(DoorInfo::base_door_model(), 1),
                rotate_coordinates_around_y_negative_90(DoorInfo::base_door_model(), 2),
                rotate_coordinates_around_y_negative_90(DoorInfo::base_door_model(), 3)
            ]
        });
        &(*models)[index]
    }

    pub fn get_opposite_door_bits(input: u32) -> u32 {
        return (input & OPPOSITEDOOR_BITS) >> 20;
    }
    pub fn set_opposite_door_bits(input: &mut u32, bit: u32) {
        let bits = bit << 20;
        *input = (*input) & !OPPOSITEDOOR_BITS;
        *input |= bits;
    }
    pub fn get_door_open_bit(input: u32) -> u32 {
        return (input & DOOROPEN_BITS) >> 18;
    }

    pub fn set_door_open_bit(input: &mut u32, bit: u32) {
        let bits = bit << 18;
        *input = (*input) & !DOOROPEN_BITS;
        *input |= bits;
    }

    pub fn toggle_door_open_bit(input: &mut u32) {
        *input ^= DOOROPEN_BITS;
    }

    pub fn get_door_top_bit(input: u32) -> u32 {
        return (input & DOORTOP_BITS) >> 19;
    }

    pub fn get_door_uvs(face: TextureFace) -> Vec<f32> {
        let side = TextureFace::new(12, 0);

        let uvs = vec![
            face.blx, face.bly,face.brx, face.bry,
            face.brx, face.bry,face.brx, face.bry,
            face.trx, face.tr_y,face.brx, face.bry,

            face.trx, face.tr_y,face.trx, face.tr_y,
            face.tlx, face.tly,face.trx, face.tr_y,
            face.blx, face.bly,face.trx, face.tr_y,//front
            
            
            side.brx, side.bry, side.trx, side.tr_y,
            side.trx, side.tr_y, side.trx, side.tr_y,
            side.brx-(TEXTURE_WIDTH/4.0), side.tr_y, side.trx, side.tr_y,
            
            side.brx-(TEXTURE_WIDTH/4.0), side.tr_y, side.trx, side.tr_y,
            side.brx-(TEXTURE_WIDTH/4.0), side.bry, side.trx, side.tr_y,
            side.brx, side.bry, side.trx, side.tr_y,//left  
            
            

            
            side.brx, side.bry, side.trx, side.tr_y,
            side.trx, side.tr_y, side.trx, side.tr_y,
            side.brx-(TEXTURE_WIDTH/4.0), side.tr_y, side.trx, side.tr_y,

            side.brx-(TEXTURE_WIDTH/4.0), side.tr_y, side.trx, side.tr_y,
            side.brx-(TEXTURE_WIDTH/4.0), side.bry, side.trx, side.tr_y,
            side.brx, side.bry, side.trx, side.tr_y,//top
            
            

            side.brx, side.bry, side.trx, side.tr_y,
            side.trx, side.tr_y, side.trx, side.tr_y,
            side.brx-(TEXTURE_WIDTH/4.0), side.tr_y, side.trx, side.tr_y,

            side.brx-(TEXTURE_WIDTH/4.0), side.tr_y, side.trx, side.tr_y,
            side.brx-(TEXTURE_WIDTH/4.0), side.bry, side.trx, side.tr_y,
            side.brx, side.bry, side.trx, side.tr_y,//right
            
            

            face.blx, face.bly, face.blx, face.bly,
            face.brx, face.bry, face.blx, face.bly,
            face.trx, face.tr_y,face.blx, face.bly,

            face.trx, face.tr_y,face.blx, face.bly,
            face.tlx, face.tly,face.blx, face.bly,
            face.blx, face.bly,face.blx, face.bly,//back
        ];
        uvs
    }

    pub fn base_door_model() -> &'static [f32] {
        static player_is_minus_z: [f32; 150] = [
            0.0, 0.0, 0.0, 0.0, 16.0, // front
            1.0, 0.0, 0.0, 0.0, 16.0,
            1.0, 1.0, 0.0, 0.0, 16.0,
    
            1.0, 1.0, 0.0, 0.0, 16.0,
            0.0, 1.0, 0.0, 0.0, 16.0,
            0.0, 0.0, 0.0, 0.0, 16.0,
    
            0.0, 0.0, 0.0, 0.0, 16.0,
            0.0, 1.0, 0.0, 0.0, 16.0,
            0.0, 1.0, 0.25, 0.0, 16.0,
    
            0.0, 1.0, 0.25, 0.0, 16.0,
            0.0, 0.0, 0.25, 0.0, 16.0,
            0.0, 0.0, 0.0, 0.0, 16.0, // left
            
            0.0, 1.0, 0.0, 0.0, 16.0,
            1.0, 1.0, 0.0, 0.0, 16.0,
            1.0, 1.0, 0.25, 0.0, 16.0,
    
            1.0, 1.0, 0.25, 0.0, 16.0,
            0.0, 1.0, 0.25, 0.0, 16.0,
            0.0, 1.0, 0.0, 0.0, 16.0, // top
            
            1.0, 1.0, 0.0, 0.0, 16.0,
            1.0, 0.0, 0.0, 0.0, 16.0,
            1.0, 0.0, 0.25, 0.0, 16.0,
    
            1.0, 0.0, 0.25, 0.0, 16.0,
            1.0, 1.0, 0.25, 0.0, 16.0,
            1.0, 1.0, 0.0, 0.0, 16.0, // right
    
            1.0, 0.0, 0.25, 0.0, 16.0,
            0.0, 0.0, 0.25, 0.0, 16.0,
            0.0, 1.0, 0.25, 0.0, 16.0,
    
            0.0, 1.0, 0.25, 0.0, 16.0,
            1.0, 1.0, 0.25, 0.0, 16.0,
            1.0, 0.0, 0.25, 0.0, 16.0, // back
        ];
        &player_is_minus_z
    }
}