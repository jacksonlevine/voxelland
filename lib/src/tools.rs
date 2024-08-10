use std::collections::HashMap;

#[derive(PartialEq)]
pub enum Material {
    Stone,
    Wood,
    Dirt,
    NoneOrNonTool
}


pub fn get_block_material(block: u32) -> Material {
    match block {
        5 | 9 | 12 | 13 | 35 => {
            Material::Stone
        }
        4 | 3 | 34 | 1 | 48 => {
            Material::Dirt
        }
        6 | 10 | 19 | 20 | 21 | 22 | 31 | 43 | 44 => {
            Material::Wood
        }
        _ => {
            Material::Stone
        }
    }
}

pub fn get_tools_target_material(tool: u32) -> Material {
    match tool {
        37 => {
            Material::Stone
        }
        38 => {
            Material::Dirt
        }
        39 => {
            Material::Wood
        }
        _ => {
            Material::NoneOrNonTool
        }
    }
}