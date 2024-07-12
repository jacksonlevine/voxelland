use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use uuid::Uuid;
use glam::Vec3;

use crate::collisioncage::CollCage;
use crate::vec;


pub const MOB_BATCH_SIZE: usize = 6;

impl Display for Message {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Message {{ type: {}, x: {}, y: {}, z: {}, rot: {}, info: {}, info2: {} }}",
            self.message_type, self.x, self.y, self.z, self.rot, self.info, self.info2
        )
    }
}

impl Display for MobUpdateBatch {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "MobUpdateBatch {{ count: {}, msgs: [", self.count)?;
        for (i, msg) in self.msgs.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", msg)?;
        }
        write!(f, "] }}")
    }
}



#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    None,
    RequestUdm,
    RequestSeed,
    RequestPt,
    Pt,
    Udm,
    Seed,
    PlayerUpdate,
    BlockSet,
    RequestTakeoff,
    YourId,
    MobUpdate,
    NewMob,
    WhatsThatMob,
    ShutUpMobMsgs,
    MobUpdateBatch,
    TimeUpdate,
    TellYouMyID,
    MultiBlockSet,
    ChestReg,
    ReqChestReg,
    ChestInvUpdate,
    Disconnect
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            MessageType::RequestUdm => {
                write!(f, "RequestUdm")
            },
            MessageType::RequestSeed => {
                write!(f, "RequestSeed")
            },
            MessageType::PlayerUpdate => {
                write!(f, "PlayerUpdate")
            },
            MessageType::BlockSet => {
                write!(f, "BlockSet")
            },
            MessageType::Udm => {
                write!(f, "Udm")
            },
            MessageType::Seed => {
                write!(f, "Seed")
            },
            MessageType::RequestTakeoff => {
                write!(f, "RequestTakeoff")
            },
            MessageType::RequestPt => {
                write!(f, "RequestPt")
            },
            MessageType::Pt => {
                write!(f, "Pt")
            },
            MessageType::None => {
                write!(f, "None")
            },
            MessageType::YourId => {
                write!(f, "YourId")
            },
            MessageType::MobUpdate => {
                write!(f, "MobUpdate")
            },
            MessageType::NewMob => {
                write!(f, "NewMob")
            },
            MessageType::WhatsThatMob => {
                write!(f, "WhatsThatMob")
            },
            MessageType::ShutUpMobMsgs => {
                write!(f, "ShutUpMobMsgs")
            },
            MessageType::MobUpdateBatch => {
                write!(f, "MobUpdateBatch")
            },
            MessageType::TimeUpdate => {
                write!(f, "TimeUpdate")
            },
            MessageType::TellYouMyID => {
                write!(f, "Tellyoumyid")
            },
            MessageType::MultiBlockSet => {
                write!(f, "MultiBlockSet")
            },
            MessageType::ChestReg => {
                write!(f, "ChestReg")
            },
            MessageType::ReqChestReg => {
                write!(f, "ReqChestReg")
            },



            /*OTHERPOS: CURRENT CHEST */
            /*INFO: DEST SLOT INDEX */
            /*INFO2: SLOT INDEX TYPE */
            /*ROT: ID */
            /*INFOF: COUNT */
            /*X, Y:   SLOT MOVED TO MOUSE OF <GOOSE> PLAYER */
            /*Z: IF MOUSE_SLOT IS REPLACED */

            /*CHEST = 0; INV = 1; NONE = 2 */
            MessageType::ChestInvUpdate => {
                write!(f, "ChestInvUpdate")
            },
            MessageType::Disconnect => {
                write!(f, "Disconnect")
            }
        }
    } 
}

impl Message {

    pub fn invupdate(slot: usize, newid: u32, newamount: u32) -> Message {

        let mut msg = Message::new(MessageType::ChestInvUpdate, Vec3::ZERO, newid as f32, slot as u32);
        msg.infof = newamount as f32;
        msg.info2 = 1;
        msg
    }

    
    pub fn new(t: MessageType, pos: Vec3, rot: f32, info: u32) -> Message {
        Message {
            message_type: t,
            x: pos.x,
            y: pos.y,
            z: pos.z,
            rot,
            info,
            info2: 0,
            infof: 1.0,
            goose: Uuid::new_v4().as_u64_pair(),
            otherpos: vec::IVec3::new(0,0,0),
            bo: false,
            hostile: false
        }
    }

    pub fn get_serialized_size() -> usize {
        let m = Message::new(MessageType::BlockSet, Vec3::new(0.0,0.0,0.0), 0.0, 0);
        bincode::serialized_size(&m).unwrap() as usize
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rot: f32,
    pub info: u32,
    pub info2: u32,
    pub infof: f32,
    pub goose: (u64, u64),
    pub otherpos: vec::IVec3,
    pub bo: bool,
    pub hostile: bool
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub key: vec::IVec3,
    pub value: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobUpdateBatch {
    pub count: u8,
    pub msgs: [Message; MOB_BATCH_SIZE]
}

impl MobUpdateBatch {
    pub fn new(count: usize, slice: &[Message]) -> MobUpdateBatch {
        if count > MOB_BATCH_SIZE {
            panic!("No MobUpdateBatch over size {}", MOB_BATCH_SIZE);
        }
        let emptymsg = Message::new(MessageType::None, Vec3::ZERO, 0.0, 0);


        let mut msgs: [Message; MOB_BATCH_SIZE] = [
            emptymsg.clone(),
            emptymsg.clone(),
            emptymsg.clone(),
            emptymsg.clone(),

            emptymsg.clone(),
            emptymsg.clone()
        ];

        for i in 0..count {
            msgs[i] = slice[i].clone();
        }

        MobUpdateBatch {
            count: count as u8,
            msgs
        }
    }
}


