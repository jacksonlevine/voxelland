use std::collections::HashSet;

use crate::chunk::ChunkSystem;
use crate::vec;
use crate::vec::IVec2;
use crate::vec::IVec3;
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
