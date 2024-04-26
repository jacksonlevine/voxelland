use vox_format::types::*;
use vox_format::data::*;



pub struct JVoxModel {
    model: VoxModels<Model>,

}

impl JVoxModel {
    pub fn new() -> JVoxModel {
        JVoxModel {
            model: vox_format::from_file("assets/voxelmodel.vox").unwrap()
        }
    }


    
}