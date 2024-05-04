


use std::collections::HashMap;
use glam::{Vec3};
use vox_format::chunk::Chunk;
use crate::chunk::ChunkSystem;
use crate::vec::IVec3;


pub fn raycast_dda(origin: Vec3, direction: Vec3, csys: &ChunkSystem, max_distance: f32) -> Option<(Vec3, IVec3)> {
    let step_size = 0.1; // Smaller step sizes increase accuracy but decrease performance
    let direction = direction.normalize(); // Ensure the direction vector is normalized
    let mut current_pos = origin;

    for _ in 0..(max_distance / step_size) as i32 {
        let grid_pos = IVec3 {
            x: current_pos.x.floor() as i32,
            y: current_pos.y.floor() as i32,
            z: current_pos.z.floor() as i32,
        };

        if csys.collision_predicate(grid_pos) {
            // Hit a block, return the current position and the grid position
            return Some((current_pos, grid_pos));
        }

        // Move the ray forward by step_size
        current_pos += direction * step_size;
    }

    None
}