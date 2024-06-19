use once_cell::sync::Lazy;

use crate::specialblocks::vertexutils::rotate_coordinates_around_y_negative_90;
use crate::textureface::TextureFace;







pub struct LadderInfo {
    
}


impl LadderInfo {



    pub fn ladder_model_from_index(index: usize) -> &'static Vec<f32> {
        static models: Lazy<Vec<Vec<f32>>> = Lazy::new(|| {
            vec![
                LadderInfo::base_ladder_model().to_vec(),
                rotate_coordinates_around_y_negative_90(LadderInfo::base_ladder_model(), 1),
                rotate_coordinates_around_y_negative_90(LadderInfo::base_ladder_model(), 2),
                rotate_coordinates_around_y_negative_90(LadderInfo::base_ladder_model(), 3)
            ]
        });
        &(*models)[index]
    }

    pub fn get_ladder_uvs() -> Vec<f32> {
        let face = TextureFace::new(0,1);

        let uvs = vec![
            face.blx, face.bly, face.blx, face.bly,
            face.brx, face.bry,face.blx, face.bly,
            face.trx, face.tr_y,face.blx, face.bly,

            face.trx, face.tr_y,face.blx, face.bly,
            face.tlx, face.tly,face.blx, face.bly,
            face.blx, face.bly,face.blx, face.bly,

            face.blx, face.bly,face.blx, face.bly,
            face.brx, face.bry,face.blx, face.bly,
            face.trx, face.tr_y,face.blx, face.bly,

            face.trx, face.tr_y,face.blx, face.bly,
            face.tlx, face.tly,face.blx, face.bly,
            face.blx, face.bly,face.blx, face.bly,
        ];
        uvs
    }

    pub fn base_ladder_model() -> &'static [f32] {
        static player_is_minus_z: [f32; 60] = [
            1.0, 0.0, 1.0, 0.0, 14.0,
            0.0, 0.0, 1.0, 0.0, 14.0,
            0.0, 1.0, 1.0, 0.0, 14.0,

            0.0, 1.0, 1.0, 0.0, 14.0,
            1.0, 1.0, 1.0, 0.0, 14.0,
            1.0, 0.0, 1.0, 0.0, 14.0,

            0.0, 0.0, 0.9, 0.0, 14.0,
            1.0, 0.0, 0.9, 0.0, 14.0,
            1.0, 1.0, 0.9, 0.0, 14.0,

            1.0, 1.0, 0.9, 0.0, 14.0,
            0.0, 1.0, 0.9, 0.0, 14.0,
            0.0, 0.0, 0.9, 0.0, 14.0
        ];
        &player_is_minus_z
    }
}