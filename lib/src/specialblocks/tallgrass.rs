use once_cell::sync::Lazy;

use crate::specialblocks::vertexutils::rotate_coordinates_around_y_negative_90;
use crate::textureface::TextureFace;







pub struct TallGrassInfo {
    
}


impl TallGrassInfo {



    pub fn tallgrass_model_from_index(index: usize) -> &'static Vec<f32> {
        static models: Lazy<Vec<Vec<f32>>> = Lazy::new(|| {
            vec![
                TallGrassInfo::base_tallgrass_model().to_vec(),
                TallGrassInfo::base_tallgrass_model().to_vec(),
                TallGrassInfo::base_tallgrass_model().to_vec(),
                TallGrassInfo::base_tallgrass_model().to_vec()
            ]
        });
        &(*models)[index]
    }

    pub fn get_tallgrass_uvs() -> Vec<f32> {
        let face = TextureFace::new(1,3);

        let uvs = vec![
            face.tlx, face.tly, face.blx, face.bly,
            face.blx, face.bly,face.blx, face.bly,
            face.brx, face.bry,face.blx, face.bly,

            face.brx, face.bry,face.blx, face.bly,
            face.trx, face.tr_y,face.blx, face.bly,
            face.tlx, face.tly, face.blx, face.bly,

            face.tlx, face.tly, face.blx, face.bly,
            face.blx, face.bly,face.blx, face.bly,
            face.brx, face.bry,face.blx, face.bly,

            face.brx, face.bry,face.blx, face.bly,
            face.trx, face.tr_y,face.blx, face.bly,
            face.tlx, face.tly, face.blx, face.bly,
        ];
        uvs
    }

    pub fn base_tallgrass_model() -> &'static [f32] {
        static player_is_minus_z: [f32; 60] = [
            0.0, 1.0, 0.0,     0.0, 14.0, 
            0.0, 0.0, 0.0,     0.0, 14.0, 
            1.0, 0.0, 1.0,     0.0, 14.0, 

            1.0, 0.0, 1.0,     0.0, 14.0, 
            1.0, 1.0, 1.0,     0.0, 14.0, 
            0.0, 1.0, 0.0,     0.0, 14.0,

            0.0, 1.0, 1.0,     0.0, 14.0,
            0.0, 0.0, 1.0,     0.0, 14.0,
            1.0, 0.0, 0.0,     0.0, 14.0,

            1.0, 0.0, 0.0,     0.0, 14.0,
            1.0, 1.0, 0.0,     0.0, 14.0,
            0.0, 1.0, 1.0,     0.0, 14.0,
        ];
        &player_is_minus_z
    }
}