



use parking_lot::{Mutex, RwLock};
use glam::{Vec3};

use crate::chunk::ChunkSystem;
use crate::vec::IVec3;


pub fn raycast_voxel(origin: Vec3, direction: Vec3, csys: &RwLock<ChunkSystem>, max_distance: f32) -> Option<(Vec3, IVec3)> {
    let step_size = 0.1; // Smaller step sizes increase accuracy but decrease performance
    let direction = direction.normalize(); // Ensure the direction vector is normalized
    let mut current_pos = origin;

    for _ in 0..(max_distance / step_size) as i32 {
        let grid_pos = IVec3 {
            x: current_pos.x.floor() as i32,
            y: current_pos.y.floor() as i32,
            z: current_pos.z.floor() as i32,
        };

        if csys.read().collision_predicate(grid_pos) {
            // Hit a block, return the current position and the grid position
            return Some((current_pos, grid_pos));
        }

        // Move the ray forward by step_size
        current_pos += direction * step_size;
    }

    None
}

pub fn raycast_voxel_with_bob(origin: Vec3, direction: Vec3, csys: &RwLock<ChunkSystem>, max_distance: f32, walkbob: f32) -> Option<(Vec3, IVec3)> {
    
    let bob = Vec3::new(0.0, walkbob.sin() /20.0, 0.0) + Vec3::new(0.0, 0.3, 0.0);

    //info!("Raycasting with a {}, {}, {} origin shift for bob", bob.x, bob.y, bob.z);
    raycast_voxel(origin + bob, direction, csys, max_distance)
}
