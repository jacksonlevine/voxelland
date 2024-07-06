use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerVec {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Serialize, Deserialize)]
pub struct PlayerPosition {
    pub pos: PlayerVec,
    pub pitch: f32,
    pub yaw: f32
}