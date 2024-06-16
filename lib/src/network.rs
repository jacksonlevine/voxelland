use std::fs::{self, File};
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use bincode;
use dashmap::DashMap;
use glam::Vec3;
use lockfree::queue::Queue;
use rusqlite::Connection;
use uuid::Uuid;

use crate::camera::Camera;
use crate::chunk::ChunkSystem;
use crate::modelentity::{direction_to_euler, ModelEntity};
use crate::server_types::{self, Entry, Message, MessageType, MobUpdateBatch};
use crate::vec::IVec3;


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
    pub shouldsend: Arc<AtomicBool>
}

impl NetworkConnector {
    pub fn new(csys: &Arc<RwLock<ChunkSystem>>, commqueue: &Arc<Queue<Message>>, commqueue2: &Arc<Queue<Message>>, gkc: &Arc<DashMap<Uuid, Vec3>>,
                my_uuid: &Arc<RwLock<Option<Uuid>>>, nsme: &Arc<DashMap<u32, ModelEntity>>, mycam: &Arc<Mutex<Camera>>) -> NetworkConnector {
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
            shouldsend: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn send(&self, message: &Message) {
        println!("Sending a {}", message.message_type);

        if let Some(stream) = &self.stream {
            let serialized_message = bincode::serialize(message).unwrap();
            let mut stream_lock = stream.lock().unwrap();
            stream_lock.write_all(&serialized_message).unwrap();
        }
    }

    pub fn sendto(message: &Message, stream: &Arc<Mutex<TcpStream>>) {
        println!("Sending a {}", message.message_type);
        let serialized_message = bincode::serialize(message).unwrap();
        let mut stream_lock = stream.lock().unwrap();
        stream_lock.write_all(&serialized_message).unwrap();
    }

    pub fn sendtolocked(message: &Message, stream: &mut TcpStream) {
        println!("Sending a {}", message.message_type);
        let serialized_message = bincode::serialize(message).unwrap();
        stream.write_all(&serialized_message).unwrap();
    }



    pub fn connect<A: ToSocketAddrs>(&mut self, address: A) {
        self.shouldrun.store(true, std::sync::atomic::Ordering::Relaxed);
        const PACKET_SIZE: usize = 90000;
        let tcp_stream = TcpStream::connect(address).unwrap();
        tcp_stream.set_nonblocking(true).unwrap();
        self.stream = Some(Arc::new(Mutex::new(tcp_stream)));

        let sr = self.shouldrun.clone();
        let sr2 = sr.clone();

        let stream = self.stream.as_ref().unwrap().clone();
        let stream2 = stream.clone();

        let csys = self.csys.clone();
        let recv_world_bool = self.received_world.clone();
        let commqueue = self.commqueue.clone();
        let gknowncams = self.gknowncams.clone();
        let my_uuid = self.my_uuid.clone();
        let nsmes = self.nsme.clone();


        let shouldsend = self.shouldsend.clone();
        let shouldsend2 = self.shouldsend.clone();


        let camclone = self.mycam.clone();

        let hpcommqueue = self.highprioritycommqueue.clone();

        self.sendthread = Some(thread::spawn(move || {
            let sr = sr2.clone();
            let stream = stream2.clone();
            let cam = camclone.clone();
            let shouldsend = shouldsend.clone();
            while sr.load(std::sync::atomic::Ordering::Relaxed) {
                if shouldsend.load(std::sync::atomic::Ordering::Relaxed) {
                    let c = cam.lock().unwrap();
                    let dir = direction_to_euler(c.direction);
                    let message = Message::new(MessageType::PlayerUpdate, c.position, dir.y, 0);
                    drop(c);

                    NetworkConnector::sendto(&message, &stream);
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }));

        
        self.recvthread = Some(thread::spawn(move || {
            let mut buffer = vec![0; PACKET_SIZE];
            let csys = csys.clone();

            let sumsg = Message::new(MessageType::ShutUpMobMsgs, Vec3::ZERO, 0.0, 0);
            let shouldsend = shouldsend2.clone();

            NetworkConnector::sendto(&sumsg, &stream);
            
            shouldsend.store(false, std::sync::atomic::Ordering::Relaxed);
            
            let requdm = Message::new(MessageType::RequestUdm, Vec3::ZERO, 0.0, 0);
            let reqseed = Message::new(MessageType::RequestSeed, Vec3::ZERO, 0.0, 0);
            let reqpt = Message::new(MessageType::RequestPt, Vec3::ZERO, 0.0, 0);
            NetworkConnector::sendto(&requdm, &stream);

            while sr.load(std::sync::atomic::Ordering::Relaxed) {
                let mut temp_buffer = vec![0; PACKET_SIZE];

                let data_available = {
                    let stream_lock = stream.lock().unwrap();
                    stream_lock.peek(&mut temp_buffer).is_ok()
                };

                if data_available {
                    let mut stream_lock = stream.lock().unwrap();




                    match stream_lock.read(&mut buffer) {
                        Ok(size) if size > 0 => {
                            let recv_m: Message = match bincode::deserialize(&buffer[..size]) {
                                Ok(msg) => {
                                    msg
                                }
                                Err(e) => {
                                    Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                                }
                            };

                            match recv_m.message_type {
                                MessageType::None => {
                                    
                                }
                                MessageType::RequestUdm => {

                                },
                                MessageType::RequestSeed => {
                                    
                                },
                                MessageType::PlayerUpdate => {
                                    commqueue.push(recv_m.clone());
                                },
                                MessageType::BlockSet => {
                                    // if recv_m.info == 0 {
                                    //     csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                    //     recv_m.info, true, true);
                                    // } else {
                                    //     csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                    //     recv_m.info, false, true);
                                    // }
                                    hpcommqueue.push(recv_m.clone());
                                },
                                MessageType::Udm => {
                                    shouldsend.store(false, std::sync::atomic::Ordering::Relaxed);
                                    
                                    stream_lock.set_nonblocking(false).unwrap();
                                    //println!("Receiving Udm:");



                                    let mut buff = vec![0 as u8; recv_m.info as usize];

                                    stream_lock.set_read_timeout(Some(Duration::from_secs(2)));

                                    match stream_lock.read_exact(&mut buff) {


                                        Ok(_) => {
                                            let mut file = File::create("db").unwrap();
                                            file.write_all(&buff).unwrap();

                                            // Now open the SQLite database file
                                            //let conn = Connection::open("db").unwrap();

                                            // // Perform operations on the SQLite database as needed
                                            // let mut stmt = conn.prepare("SELECT x, y, z, value FROM userdatamap").unwrap();
                                            // let userdatamap_iter = stmt.query_map([], |row| {
                                            //     Ok(Entry {
                                            //         key: IVec3::new(row.get(0)?, row.get(1)?, row.get(2)?),
                                            //         value: row.get(3)?,
                                            //     })
                                            // }).unwrap();

                                            // let mut entries: Vec<Entry> = Vec::new();
                                            // for entry in userdatamap_iter {
                                            //     entries.push(entry.unwrap());
                                            // }

                                            // for entry in entries {
                                            //     csys.write().unwrap().set_block(entry.key, entry.value, true);
                                            // }

                                            // csys.write().unwrap().save_current_world_to_file(String::from("mp"));
                                            NetworkConnector::sendtolocked(&reqseed, &mut stream_lock);
                                        }
                                        Err(e) => {
                                            println!("Error receiving, trying again...");
                                            NetworkConnector::sendtolocked(&requdm, &mut stream_lock);
                                        }

                                    }

                                    

                                    //println!("{}", recv_s);
        
                                            // fs::create_dir_all("mp").unwrap();
                                            // let mut file = File::create("mp/udm").unwrap(); 
                                            // file.write_all(recv_s.as_bytes()).unwrap();
                                },
                                MessageType::Seed => {
                                    //println!("Receiving Seed:");
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.read_exact(&mut buff).unwrap();
                                    fs::create_dir_all("mp").unwrap();
                                    let mut file = File::create("mp/seed").unwrap(); 

                                    
                                    let recv_s: String = bincode::deserialize(&buff).unwrap();
                                    //println!("{}", recv_s);

                                    file.write_all(recv_s.as_bytes()).unwrap();



                                    commqueue.push(recv_m.clone());
                                    
                                    NetworkConnector::sendtolocked(&reqpt, &mut stream_lock);
                                },
                                MessageType::RequestTakeoff => {
                                    commqueue.push(recv_m.clone());
                                },
                                MessageType::RequestPt => {
                                    
                                },
                                MessageType::Pt => {
                                    //println!("Receiving Pt:");
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.read_exact(&mut buff).unwrap();
                                    fs::create_dir_all("mp").unwrap();
                                    let mut file = File::create("mp/pt").unwrap(); 

                                    
                                    let recv_s: String = bincode::deserialize(&buff).unwrap();
                                    //println!("{}", recv_s);

                                    file.write_all(recv_s.as_bytes()).unwrap();




                                    csys.write().unwrap().load_world_from_file(String::from("mp"));
                                    recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                    stream_lock.set_nonblocking(true).unwrap();
                                    shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                },
                                MessageType::YourId => {
                                    //println!("Receiving Your ID:");
                                    stream_lock.set_nonblocking(false).unwrap();
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.read_exact(&mut buff).unwrap();
                                    let recv_s: (u64, u64) = bincode::deserialize(&buff).unwrap();

                                    let uuid = Uuid::from_u64_pair(recv_s.0, recv_s.1);
                                    //println!("{}", uuid);


                                    gknowncams.insert(
                                        uuid.clone(), Vec3::ZERO
                                    );
                                    *(my_uuid.write().unwrap()) = Some(uuid);
                                    stream_lock.set_nonblocking(true).unwrap();
                                },
                                MessageType::MobUpdate => {
                                    
                                    commqueue.push(recv_m.clone());
                                    
                                },
                                MessageType::NewMob => {
                                    let newid = recv_m.info;

                                    let newtype = recv_m.info2;

                                    let newpos = Vec3::new(recv_m.x, recv_m.y, recv_m.z);
                                },
                                MessageType::WhatsThatMob => todo!(),
                                MessageType::ShutUpMobMsgs =>  {
                                    
                                },
                                MessageType::MobUpdateBatch =>  {
                                    //println!("Receiving a Mob Batch:");
                                
                                    stream_lock.set_nonblocking(false).unwrap();
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.set_read_timeout(Some(Duration::from_millis(50)));
                                    match stream_lock.read_exact(&mut buff) {
                                        Ok(_) => {
                                            let recv_s: MobUpdateBatch = bincode::deserialize(&buff).unwrap();

                                            if recv_s.count > server_types::MOB_BATCH_SIZE as u8 {
                                                println!("Ignoring invalid packe with count > {} of {}", server_types::MOB_BATCH_SIZE, recv_s.count);
                                            } else {
                                                for i in 0..recv_s.count.min(8) {
                                                    let msg = recv_s.msgs[i as usize].clone();
                                                    commqueue.push(msg);
                                                }
                                                
                                            }

                                            //println!("{}", recv_s);
                                        },
                                        Err(e) => {
                                            //println!("Sorry champ! Missed that one!, {}", e);
                                        },
                                    }
                                    

                                    stream_lock.set_nonblocking(true).unwrap();

                                    

                                    
                                },
                                MessageType::TimeUpdate => {
                                    commqueue.push(recv_m.clone());
                                }
                            }

                            //println!("Received message from server: {:?}", recv_m);
                        }
                        Ok(_) => {
                            println!("Connection closed by server");
                            break;
                        }
                        Err(e) => {
                            println!("Failed to receive message: {}", e);
                            break;
                        }
                    }
                }
            }
        }));
    }
}
