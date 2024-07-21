use once_cell::sync::Lazy;

use crate::specialblocks::vertexutils::rotate_coordinates_around_y_negative_90;
use crate::textureface::TextureFace;







pub struct ChestInfo {
    
}


impl ChestInfo {



    pub fn chest_model_from_index(index: usize) -> &'static Vec<f32> {
        static models: Lazy<Vec<Vec<f32>>> = Lazy::new(|| {
            vec![
                ChestInfo::base_chest_model().to_vec(),
                rotate_coordinates_around_y_negative_90(ChestInfo::base_chest_model(), 1),
                rotate_coordinates_around_y_negative_90(ChestInfo::base_chest_model(), 2),
                rotate_coordinates_around_y_negative_90(ChestInfo::base_chest_model(), 3)
            ]
        });
        &(*models)[index]
    }

    pub fn get_chest_uvs() -> Vec<f32> {
        let straps = TextureFace::new(13,0);
        let blank = TextureFace::new(14,0);
        let front = TextureFace::new(14,2);

        let uvs = vec![
            blank.blx, blank.bly,blank.brx, blank.bry,
            blank.brx, blank.bry,blank.brx, blank.bry,
            blank.trx, blank.tr_y,blank.brx, blank.bry,

            blank.trx, blank.tr_y,blank.tlx, blank.tly,
            blank.tlx, blank.tly,blank.tlx, blank.tly,
            blank.blx, blank.bly,blank.tlx, blank.tly,

            blank.blx, blank.bly,blank.brx, blank.bry,
            blank.brx, blank.bry,blank.brx, blank.bry,
            blank.trx, blank.tr_y,blank.brx, blank.bry,

            blank.trx, blank.tr_y,blank.tlx, blank.tly,
            blank.tlx, blank.tly,blank.tlx, blank.tly,
            blank.blx, blank.bly,blank.tlx, blank.tly,

            straps.blx, straps.bly,straps.brx, straps.bry,
            straps.brx, straps.bry,straps.brx, straps.bry,
            straps.trx, straps.tr_y,straps.brx, straps.bry,

            straps.trx, straps.tr_y,straps.tlx, straps.tly,
            straps.tlx, straps.tly,straps.tlx, straps.tly,
            straps.blx, straps.bly, straps.tlx, straps.tly,

            front.blx, front.bly, front.brx, front.bry,
            front.brx, front.bry, front.brx, front.bry,
            front.trx, front.tr_y,  front.brx, front.bry,

            front.trx, front.tr_y,  front.tlx, front.tly,
            front.tlx, front.tly,  front.tlx, front.tly,
            front.blx, front.bly,  front.tlx, front.tly,

            straps.blx, straps.bly,  straps.brx, straps.bry,
            straps.brx, straps.bry,  straps.brx, straps.bry,
            straps.trx, straps.tr_y,  straps.brx, straps.bry,

            straps.trx, straps.tr_y,  straps.trx, straps.tr_y,
            straps.tlx, straps.tly,  straps.trx, straps.tr_y,
            straps.blx, straps.bly,  straps.trx, straps.tr_y,

            blank.blx, blank.bly,  blank.brx, blank.bry,
            blank.brx, blank.bry,  blank.brx, blank.bry,
            blank.trx, blank.tr_y,  blank.brx, blank.bry,

            blank.trx, blank.tr_y,  blank.brx, blank.bry,
            blank.tlx, blank.tly,  blank.brx, blank.bry,
            blank.blx, blank.bly,   blank.brx, blank.bry,
        ];
        uvs
    }

    pub fn base_chest_model() -> &'static [f32] {
        static player_is_minus_z: [f32; 180] = [
            0.0, 0.0, 1.0, 0.0, 10.0,
            0.0, 0.0, 0.0, 0.0, 10.0,
            0.0, 1.0, 0.0, 0.0, 10.0,
    
            0.0, 1.0, 0.0, 0.0, 10.0,
            0.0, 1.0, 1.0, 0.0, 10.0,
            0.0, 0.0, 1.0, 0.0, 10.0,
    
            1.0, 0.0, 0.0, 0.0, 10.0,
            1.0, 0.0, 1.0, 0.0, 10.0,
            1.0, 1.0, 1.0, 0.0, 10.0,
    
            1.0, 1.0, 1.0, 0.0, 10.0,
            1.0, 1.0, 0.0, 0.0, 10.0,
            1.0, 0.0, 0.0, 0.0, 10.0,
    
            1.0, 0.0, 1.0, 0.0, 14.0,
            0.0, 0.0, 1.0, 0.0, 14.0,
            0.0, 1.0, 1.0, 0.0, 14.0,
    
            0.0, 1.0, 1.0, 0.0, 14.0,
            1.0, 1.0, 1.0, 0.0, 14.0,
            1.0, 0.0, 1.0, 0.0, 14.0,
    
            0.0, 0.0, 0.0, 0.0, 14.0,
            1.0, 0.0, 0.0, 0.0, 14.0,
            1.0, 1.0, 0.0, 0.0, 14.0,
    
            1.0, 1.0, 0.0, 0.0, 14.0,
            0.0, 1.0, 0.0, 0.0, 14.0,
            0.0, 0.0, 0.0, 0.0, 14.0,
    
            0.0, 1.0, 0.0, 0.0, 16.0,
            1.0, 1.0, 0.0, 0.0, 16.0,
            1.0, 1.0, 1.0, 0.0, 16.0,
    
            1.0, 1.0, 1.0, 0.0, 16.0,
            0.0, 1.0, 1.0, 0.0, 16.0,
            0.0, 1.0, 0.0, 0.0, 16.0,
    
            1.0, 0.0, 0.0, 0.0, 7.0,
            0.0, 0.0, 0.0, 0.0, 7.0,
            0.0, 0.0, 1.0, 0.0, 7.0,
    
            0.0, 0.0, 1.0, 0.0, 7.0,
            1.0, 0.0, 1.0, 0.0, 7.0,
            1.0, 0.0, 0.0, 0.0, 7.0
        ];
        &player_is_minus_z
    }
    
}