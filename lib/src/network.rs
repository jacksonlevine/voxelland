use std::fs::{self, File};
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{self, Read, Write};
use tracing::info;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use bincode;
use dashmap::DashMap;
use glam::Vec3;
use glfw::ffi::glfwGetTime;
use lockfree::queue::Queue;

use uuid::Uuid;

use crate::camera::Camera;
use crate::chunk::ChunkSystem;
use crate::game::{Game, CURRSEED, PLAYERPOS, UPDATE_THE_BLOCK_OVERLAY};
use crate::inventory::ChestInventory;
use crate::modelentity::{direction_to_euler, ModelEntity};
use crate::server_types::{self, Message, MessageType, MOB_BATCH_SIZE};
use crate::statics::MY_MULTIPLAYER_UUID;
use crate::vec;



pub struct NetworkConnector {
    pub stream: Option<Arc<Mutex<TcpStream>>>,
    pub recvthread: Option<JoinHandle<()>>,
    pub sendthread: Option<JoinHandle<()>>,
    pub shouldrun: Arc<AtomicBool>,
    pub csys: Arc<RwLock<ChunkSystem>>,
    pub received_world: Arc<AtomicBool>,
    pub commqueue: Arc<Queue<Message>>,
    pub highprioritycommqueue: Arc<Queue<Message>>,
    pub received_id: Arc<AtomicBool>,
    pub gknowncams: Arc<DashMap<Uuid, Vec3>>,
    pub my_uuid: Arc<RwLock<Option<Uuid>>>,
    pub nsme: Arc<DashMap<u32, ModelEntity>>,
    pub mycam: Arc<Mutex<Camera>>,
    pub shouldsend: Arc<AtomicBool>,
    pub pme: Arc<DashMap<Uuid, ModelEntity>>,
    pub sendqueue: Arc<Queue<Message>>,
    pub chest_registry: Arc<DashMap<vec::IVec3, ChestInventory>>,
}

impl NetworkConnector {
    pub fn new(csys: &Arc<RwLock<ChunkSystem>>, commqueue: &Arc<Queue<Message>>, commqueue2: &Arc<Queue<Message>>, gkc: &Arc<DashMap<Uuid, Vec3>>,
                my_uuid: &Arc<RwLock<Option<Uuid>>>, nsme: &Arc<DashMap<u32, ModelEntity>>, mycam: &Arc<Mutex<Camera>>, pme: &Arc<DashMap<Uuid, ModelEntity>>,
                chest_reg: &Arc<DashMap<vec::IVec3, ChestInventory>>, sendqueue: &Arc<Queue<Message>>) -> NetworkConnector {
        NetworkConnector {
            stream: None,
            recvthread: None,
            sendthread: None,
            shouldrun: Arc::new(AtomicBool::new(false)),
            csys: csys.clone(),
            received_world: Arc::new(AtomicBool::new(false)),
            commqueue: commqueue.clone(),
            highprioritycommqueue: commqueue2.clone(),
            received_id: Arc::new(AtomicBool::new(false)),
            gknowncams: gkc.clone(),
            my_uuid: my_uuid.clone(),
            nsme: nsme.clone(),
            mycam: mycam.clone(),
            shouldsend: Arc::new(AtomicBool::new(false)),
            pme: pme.clone(),
            sendqueue: sendqueue.clone(),
            chest_registry: chest_reg.clone()
        }
    }

    pub fn send(&self, message: &Message) {
        //info!("Sending a {}", message.message_type);

        if let Some(stream) = &self.stream {
            let serialized_message = bincode::serialize(message).unwrap();
            let mut stream_lock = stream.lock().unwrap();
            stream_lock.write_all(&serialized_message).unwrap();
        }
    }

    pub fn sendto(message: &Message, stream: &Arc<Mutex<TcpStream>>) {
       // info!("Sending a {}", message.message_type);
        let serialized_message = bincode::serialize(message).unwrap();
        let mut stream_lock = stream.lock().unwrap();
        let mut attempts = 0;

        loop {
            match stream_lock.write_all(&serialized_message) {
                Ok(_) => return (),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Sleep for a short duration and retry
                    std::thread::sleep(Duration::from_millis(10));
                    attempts += 1;
                    if attempts > 50 {
                        return ()
                    }
                }
                Err(e) => return (),
            }
        }
    }

    pub fn sendtolocked(message: &Message, stream: &mut TcpStream) {
       // info!("Sending a {}", message.message_type);
        let serialized_message = bincode::serialize(message).unwrap();
        stream.write_all(&serialized_message).unwrap();
    }



    pub fn connect<A: ToSocketAddrs + Clone>(&mut self, address: A) {
        self.shouldrun.store(true, std::sync::atomic::Ordering::Relaxed);
        const PACKET_SIZE: usize = 90000;
        let mut conned = false;


        while !conned {
            match TcpStream::connect(address.clone()) {
                Ok(tcp_stream) => {
                    conned = true;

                    tcp_stream.set_nonblocking(true).unwrap();
                    self.stream = Some(Arc::new(Mutex::new(tcp_stream)));

                    let sr = self.shouldrun.clone();
                    let sr2 = sr.clone();

                    let stream = self.stream.as_ref().unwrap().clone();
                    let stream2 = stream.clone();

                    let mut idgreeting = Message::new(MessageType::TellYouMyID, Vec3::ZERO, 0.0, 0);
                    idgreeting.goose = unsafe { (*MY_MULTIPLAYER_UUID).as_u64_pair() };

                    self.send(&idgreeting);

                    let csys = self.csys.clone();
                    let recv_world_bool = self.received_world.clone();
                    let commqueue = self.commqueue.clone();
                    let gknowncams = self.gknowncams.clone();
                    let _my_uuid = self.my_uuid.clone();
                    let _nsmes = self.nsme.clone();
                    let pme = self.pme.clone();


                    let shouldsend = self.shouldsend.clone();
                    let shouldsend2 = self.shouldsend.clone();


                    let camclone = self.mycam.clone();

                    let hpcommqueue = self.highprioritycommqueue.clone();

                    let sendqueue = self.sendqueue.clone();

                    let chestreg = self.chest_registry.clone();

                    self.sendthread = Some(thread::spawn(move || {
                        let sr = sr2.clone();
                        let stream = stream2.clone();
                        let cam = camclone.clone();
                        let shouldsend = shouldsend.clone();
                        while sr.load(std::sync::atomic::Ordering::Relaxed) {
                            if shouldsend.load(std::sync::atomic::Ordering::Relaxed) {
                                match sendqueue.pop() {
                                    Some(t) => {
                                        NetworkConnector::sendto(&t, &stream);
                                    }
                                    None => {

                                    }
                                }

                                let c = unsafe {
                                    PLAYERPOS.snapshot()
                                };

                           
                                let dir = direction_to_euler(c.dir.into());
                                let mut message = Message::new(MessageType::PlayerUpdate, c.pos.into(), dir.y, 0);
                                message.infof = c.pitch;
                                message.info2 = c.yaw as u32;

                                NetworkConnector::sendto(&message, &stream);
                      
                                
                                
                            }
                            thread::sleep(Duration::from_millis(250));
                        }
                    }));

                    
                    self.recvthread = Some(thread::spawn(move || {
                        let mut buffer = vec![0; PACKET_SIZE];
                        let csys = csys.clone();

                        //let sumsg = Message::new(MessageType::ShutUpMobMsgs, Vec3::ZERO, 0.0, 0);
                        let shouldsend = shouldsend2.clone();

                        //NetworkConnector::sendto(&sumsg, &stream);
                        
                        shouldsend.store(false, std::sync::atomic::Ordering::Relaxed);
                        
                        let requdm = Message::new(MessageType::RequestUdm, Vec3::ZERO, 0.0, 0);
                        let reqseed = Message::new(MessageType::RequestSeed, Vec3::ZERO, 0.0, 0);
                        let reqpt = Message::new(MessageType::RequestPt, Vec3::ZERO, 0.0, 0);
                        let reqchest = Message::new(MessageType::ReqChestReg, Vec3::ZERO, 0.0, 0);
                        
                        NetworkConnector::sendto(&requdm, &stream);

                        while sr.load(std::sync::atomic::Ordering::Relaxed) {
                            let mut temp_buffer = vec![0; PACKET_SIZE];

                            let data_available = {
                                match stream.try_lock() {
                                    Ok(stream_lock) => {
                                        stream_lock.peek(&mut temp_buffer).is_ok()
                                    }
                                    Err(_e) => {
                                        false
                                    }
                                }
                                
                            };

                            if data_available {
                                let mut stream_lock = stream.lock().unwrap();




                                match stream_lock.read(&mut buffer) {
                                    Ok(size) if size > 0 => {
                                        let comm: Message = match bincode::deserialize::<Message>(&buffer[..size]) {
                                            Ok(msg) => {

                                                match msg.message_type {
                                                    MessageType::ChestInvUpdate => {
                                                        info!("CIU incoming goose {}", Uuid::from_u64_pair(msg.goose.0, msg.goose.1));
                                                    }
                                                    _ => {

                                                    }
                                                }
                                                msg
                                            }
                                            Err(_e) => {
                                                Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                                            }
                                        };

                                        match comm.message_type {
                                            MessageType::Disconnect => {
                                                pme.remove(&Uuid::from_u64_pair(comm.goose.0, comm.goose.1));
                                            }
                                            MessageType::ChestReg => {
                                                
                                                info!("Receiving ChestReg:");

                                                if comm.info > 0 {






                                                    let mut payload_buffer = vec![0u8; comm.info as usize];
                                                    let mut total_read = 0;

                                                    let mut numtimes = 0;
                        
                                                    while total_read < comm.info as usize {
                                                        match stream_lock.read(&mut payload_buffer[total_read..]) {
                                                            Ok(n) if n > 0 => total_read += n,
                                                            Ok(_) => {
                                                                // Connection closed
                                                                info!("Connection closed by server during chestreg");
                                                                break;
                                                            }
                                                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                                                // Sleep for a short period and retry
                                                                thread::sleep(Duration::from_millis(10));
                                                            }
                                                            Err(e) => {
                                                                info!("Error receiving chestreg: {}", e);
                                                                break;
                                                            }
                                                        }
                                                        numtimes += 1;
                                                        if numtimes > 100 {
                                                            numtimes = 0;
                                                            NetworkConnector::sendtolocked(&reqchest, &mut stream_lock);
                                                        }
                                                    }
                        
                                                    if total_read == comm.info as usize {

                                                        info!("Got the expected bytes for chestreg");
                                                        let mut file = File::create("chestdb").unwrap();
                                                        file.write_all(&payload_buffer).unwrap();

                                                        let seed = unsafe {CURRSEED.load(std::sync::atomic::Ordering::Relaxed)};


                                                        Game::static_load_chests_from_file(seed, &chestreg);
                                                        //csys.write().unwrap().load_my_inv_from_file();
                                                        hpcommqueue.push(comm);
                                                        recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                                        shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                                        
                                                    } else {


                                                        info!("Error receiving chestreg, trying again...");
                                                        NetworkConnector::sendtolocked(&reqchest, &mut stream_lock);
                                                    }






                                                
                                                    
                                                } else {
                                                    recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                                    shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                                }

                                                
                                                

                                            }
                                            MessageType::ReqChestReg => {

                                            }
                                            MessageType::TellYouMyID => {

                                            }
                                            MessageType::None => {
                                                
                                            }
                                            MessageType::RequestUdm => {

                                            },
                                            MessageType::RequestSeed => {
                                                
                                            },
                                            
                                            MessageType::PlayerUpdate => {

                                                

                                                let newpos = Vec3::new(comm.x, comm.y, comm.z);
                                                //let id = comm.info;
                                                let _modind = comm.info2;
                                                let rot = comm.rot;
                                                let scale = 0.3;

                                                let pme: Arc<DashMap<Uuid, ModelEntity>> = pme.clone();


                                                let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);

                                                //info!("Player update: {uuid}");
                                                //info!("NSME Length: {}", nsme.len());
                                                match pme.get_mut(&uuid) {
                                                    Some(mut me) => {
                                                        let modent = me.value_mut();
                                                        (*modent).lastpos = (*modent).position.clone();
                                                        (*modent).position = newpos;
                                                        (*modent).scale = scale;
                                                        (*modent).lastrot = (*modent).rot.clone();
                                                        (*modent).rot = Vec3::new(0.0, rot, 0.0);
                                                        unsafe {
                                                            (*modent).time_stamp = glfwGetTime();
                                                        }
                                                        
                                                        
                                                    }
                                                    None => {
                                                        commqueue.push(comm.clone());
                                                    }
                                                };

                                                
                                            },
                                            MessageType::BlockSet => {
                                                // if recv_m.info == 0 {
                                                //     csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                                //     recv_m.info, true, true);
                                                // } else {
                                                //     csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                                //     recv_m.info, false, true);
                                                // }
                                                
                                                
                                                hpcommqueue.push(comm.clone());
                                            },
                                            MessageType::MultiBlockSet => {
                                                // if recv_m.info == 0 {
                                                //     csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                                //     recv_m.info, true, true);
                                                // } else {
                                                //     csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                                //     recv_m.info, false, true);
                                                // }
                                                hpcommqueue.push(comm.clone());
                                            },
                                            MessageType::Udm => {
                                                info!("Receiving Udm:");
                                                shouldsend.store(false, std::sync::atomic::Ordering::Relaxed);
                                                
                                                stream_lock.set_nonblocking(false).unwrap();
                                                



                                                let mut buff = vec![0 as u8; comm.info as usize];

                                                stream_lock.set_read_timeout(Some(Duration::from_secs(5)));

                                                match stream_lock.read_exact(&mut buff) {


                                                    Ok(_) => {
                                                        info!("Got the expected bytes for udm");
                                                        let mut file = File::create("db").unwrap();
                                                        file.write_all(&buff).unwrap();

                                                        NetworkConnector::sendtolocked(&reqseed, &mut stream_lock);
                                                    }
                                                    Err(e) => {
                                                        info!("Error receiving, trying again... {e}");
                                                        thread::sleep(Duration::from_millis(1000));
                                                        NetworkConnector::sendtolocked(&requdm, &mut stream_lock);
                                                    }

                                                }

                                                stream_lock.set_nonblocking(true).unwrap();
                                            },
                                            MessageType::Seed => {
                                                //info!("Receiving Seed:");
                                                // let mut buff = vec![0 as u8; comm.info as usize];

                                                // stream_lock.set_nonblocking(false).unwrap();


                                                // stream_lock.read_exact(&mut buff).unwrap();


                                                let recv_s = format!("{}", comm.info);

                                                info!("Received seed: {}", recv_s);

                                                // Create directory if not exists
                                                    fs::create_dir_all("mp").unwrap();

                                                    // Create or open file for writing
                                                    let mut file = File::create("mp/seed2").unwrap();

                                                    // Write the received seed to the file
                                                    file.write_all(recv_s.as_bytes()).unwrap();
                                                    // Flush the buffer to ensure all data is written
                                                    file.flush().unwrap();

                                                    // Verify if the content is correctly written
                                                    let content = std::fs::read_to_string("mp/seed2").unwrap();
                                                    info!("File content: {}", content);


                                                        commqueue.push(comm.clone());
                                                        
                                                        thread::sleep(Duration::from_millis(200));
                                                        NetworkConnector::sendtolocked(&reqpt, &mut stream_lock);


                                                stream_lock.set_nonblocking(true).unwrap();
                                                //info!("{}", recv_s);

                                                
                                            },
                                            MessageType::RequestTakeoff => {
                                                commqueue.push(comm.clone());
                                            },
                                            MessageType::RequestPt => {
                                                
                                            },
                                            MessageType::Pt => {
                                                //info!("Receiving Pt:");
                                                // let mut buff = vec![0 as u8; comm.info as usize];

                                                // stream_lock.set_nonblocking(false).unwrap();

                                                // stream_lock.read_exact(&mut buff).unwrap();


                                                fs::create_dir_all("mp").unwrap();
                                                let mut file = File::create("mp/pt").unwrap(); 

                                                
                                                let pt = comm.info;
                                                let recv_s = format!("{pt}");
                                                file.write_all(recv_s.as_bytes()).unwrap();




                                                csys.write().unwrap().load_world_from_file(String::from("mp"));

                                                thread::sleep(Duration::from_millis(200));
                                                NetworkConnector::sendtolocked(&reqchest, &mut stream_lock);
                                                
                            
                                                //info!("{}", recv_s);

                                                
                                                
                                            },
                                            MessageType::YourId => {
                                                // //info!("Receiving Your ID:");
                                                // stream_lock.set_nonblocking(false).unwrap();
                                                // let mut buff = vec![0 as u8; comm.info as usize];
                                                // stream_lock.read_exact(&mut buff).unwrap();

                                                let recv_s = comm.goose;
                                                let uuid = Uuid::from_u64_pair(recv_s.0, recv_s.1);
                                                //info!("{}", uuid);

                                                info!("My uuid, I am being told, is {uuid}");

                                                gknowncams.insert(
                                                    uuid.clone(), Vec3::ZERO
                                                );
                                                //*(my_uuid.write().unwrap()) = Some(uuid);


                                                
                                                // stream_lock.set_nonblocking(true).unwrap();
                                            },
                                            MessageType::MobUpdate => {
                                                
                                                commqueue.push(comm.clone());
                                                
                                            },
                                            MessageType::NewMob => {
                                                let _newid = comm.info;

                                                let _newtype = comm.info2;

                                                let _newpos = Vec3::new(comm.x, comm.y, comm.z);
                                            },
                                            MessageType::WhatsThatMob => todo!(),
                                            MessageType::ShutUpMobMsgs =>  {
                                                
                                            },
                                            MessageType::MobUpdateBatch => {
                                                //info!("Got MUB, count {}", comm.count);
                                                if comm.count > server_types::MOB_BATCH_SIZE as u8 {
                                                    info!("Ignoring invalid mobbatch with count > {} of {}", server_types::MOB_BATCH_SIZE, comm.count);
                                                } else {
                                                    for i in 0..comm.count.min(MOB_BATCH_SIZE as u8) {
                                                        
                                                        let msg = Message::from_mob_message(&comm.msgs[i as usize]);
                                                        commqueue.push(msg);
                                                    }
                                                }
                                                        
            
                                            }
                                            MessageType::TimeUpdate => {
                                                commqueue.push(comm.clone());
                                            }
                                            MessageType::ChestInvUpdate => {
                                                //info!("Receiving CIU from goose {}", Uuid::from_u64_pair(comm.goose.0, comm.goose.1));
                                                hpcommqueue.push(comm.clone());
                                            },
                                        }

                                        //info!("Received message from server: {:?}", recv_m);
                                    }
                                    Ok(_) => {
                                        info!("Connection closed by server");
                                        break;
                                    }
                                    Err(e) => {
                                        info!("Failed to receive message: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }));
                }
                Err(e) => {
                    info!("Error from connect(): {e}");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
        //let tcp_stream = TcpStream::connect(address).unwrap();
        
    }
}
