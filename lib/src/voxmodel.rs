





use vox_format::data::*;
use vox_format::types::*;

pub struct JVoxModel {
    pub model: VoxModels<Model>,
}

impl JVoxModel {
    pub fn new(path: &'static str) -> JVoxModel {
        JVoxModel {
            model: vox_format::from_file(path).unwrap(),
        }
    }

}
