


use std::collections::HashMap;
use glam::{Vec3};
use vox_format::chunk::Chunk;
use crate::chunk::ChunkSystem;
use crate::vec::IVec3;

pub fn raycast_dda(origin: Vec3, direction: Vec3, csys: &ChunkSystem, max_distance: f32) -> Option<(Vec3, IVec3)> {
    let mut current_pos = origin;
    let step = IVec3 {
        x: if direction.x > 0.0 { 1 } else { -1 },
        y: if direction.y > 0.0 { 1 } else { -1 },
        z: if direction.z > 0.0 { 1 } else { -1 },
    };

    // Adjust t_max to start from the nearest cell boundary
    let mut t_max = Vec3 {
        x: if direction.x != 0.0 { ((current_pos.x + 0.5 * step.x as f32).round() - current_pos.x) / direction.x } else { f32::MAX },
        y: if direction.y != 0.0 { ((current_pos.y + 0.5 * step.y as f32).round() - current_pos.y) / direction.y } else { f32::MAX },
        z: if direction.z != 0.0 { ((current_pos.z + 0.5 * step.z as f32).round() - current_pos.z) / direction.z } else { f32::MAX },
    };

    let t_delta = Vec3 {
        x: if direction.x != 0.0 { 1.0 / direction.x.abs() } else { f32::MAX },
        y: if direction.y != 0.0 { 1.0 / direction.y.abs() } else { f32::MAX },
        z: if direction.z != 0.0 { 1.0 / direction.z.abs() } else { f32::MAX },
    };

    let mut grid_pos = IVec3 {
        x: current_pos.x.round() as i32,
        y: current_pos.y.round() as i32,
        z: current_pos.z.round() as i32,
    };

    for _ in 0..max_distance as i32 {
        if csys.blockat(grid_pos) > 0 {
            return Some((current_pos, grid_pos));
        }

        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                current_pos.x += step.x as f32 * t_max.x;
                t_max.x += t_delta.x;
            } else {
                current_pos.z += step.z as f32 * t_max.z;
                t_max.z += t_delta.z;
            }
        } else {
            if t_max.y < t_max.z {
                current_pos.y += step.y as f32 * t_max.y;
                t_max.y += t_delta.y;
            } else {
                current_pos.z += step.z as f32 * t_max.z;
                t_max.z += t_delta.z;
            }
        }

        grid_pos = IVec3 {
            x: current_pos.x.round() as i32,
            y: current_pos.y.round() as i32,
            z: current_pos.z.round() as i32,
        };
    }

    None
}