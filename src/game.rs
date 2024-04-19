use std::sync::{Mutex, Arc};

use crate::chunk::ChunkSystem;

pub struct Game {
    chunksys: ChunkSystem,
}




impl Game {

    pub fn new() -> Game {
        Game {
            chunksys: ChunkSystem::new(8),
        }
    }

    pub fn update(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }
    }
}