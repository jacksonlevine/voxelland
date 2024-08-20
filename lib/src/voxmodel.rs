





use vox_format::data::*;
use vox_format::types::*;

pub struct JVoxModel {
    pub model: VoxModels<Model>,
    pub idnumber: i32
}

impl JVoxModel {
    pub fn new(path: &'static str) -> JVoxModel {
        static mut idnum: i32 = 0;
        unsafe {
            idnum += 1;
            JVoxModel {
                model: vox_format::from_file(path).unwrap(),
                idnumber: idnum
            }
        }
    }

}
