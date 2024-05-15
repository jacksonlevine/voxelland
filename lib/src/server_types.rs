use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use uuid::Uuid;
use glam::Vec3;




#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    RequestWorld,
    RequestSeed,
    PlayerUpdate,
    BlockSet
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rot: f32,
    pub info: u32,
}
impl Message {
    pub fn new(t: MessageType, pos: Vec3, rot: f32, info: u32) -> Message {
        Message {
            message_type: t,
            x: pos.x,
            y: pos.y,
            z: pos.z,
            rot,
            info
        }
    }

    pub fn get_serialized_size() -> usize {
        let m = Message::new(MessageType::BlockSet, Vec3::new(0.0,0.0,0.0), 0.0, 0);
        bincode::serialized_size(&m).unwrap() as usize
    }
}