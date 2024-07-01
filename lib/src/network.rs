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
use glfw::ffi::glfwGetTime;
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
    pub shouldsend: Arc<AtomicBool>,
    pub pme: Arc<DashMap<Uuid, ModelEntity>>,
    pub sendqueue: Arc<Queue<Message>>
}

impl NetworkConnector {
    pub fn new(csys: &Arc<RwLock<ChunkSystem>>, commqueue: &Arc<Queue<Message>>, commqueue2: &Arc<Queue<Message>>, gkc: &Arc<DashMap<Uuid, Vec3>>,
                my_uuid: &Arc<RwLock<Option<Uuid>>>, nsme: &Arc<DashMap<u32, ModelEntity>>, mycam: &Arc<Mutex<Camera>>, pme: &Arc<DashMap<Uuid, ModelEntity>>,
                ) -> NetworkConnector {
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
            sendqueue: Arc::new(Queue::new())
        }
    }

    pub fn send(&self, message: &Message) {
        //println!("Sending a {}", message.message_type);

        if let Some(stream) = &self.stream {
            let serialized_message = bincode::serialize(message).unwrap();
            let mut stream_lock = stream.lock().unwrap();
            stream_lock.write_all(&serialized_message).unwrap();
        }
    }

    pub fn sendto(message: &Message, stream: &Arc<Mutex<TcpStream>>) {
       // println!("Sending a {}", message.message_type);
        let serialized_message = bincode::serialize(message).unwrap();
        let mut stream_lock = stream.lock().unwrap();
        stream_lock.write_all(&serialized_message).unwrap();
    }

    pub fn sendtolocked(message: &Message, stream: &mut TcpStream) {
       // println!("Sending a {}", message.message_type);
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
        let pme = self.pme.clone();


        let shouldsend = self.shouldsend.clone();
        let shouldsend2 = self.shouldsend.clone();


        let camclone = self.mycam.clone();

        let hpcommqueue = self.highprioritycommqueue.clone();

        let sendqueue = self.sendqueue.clone();

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
                    let c = cam.lock().unwrap();
                    let dir = direction_to_euler(c.direction);
                    let message = Message::new(MessageType::PlayerUpdate, c.position, dir.y, 0);
                    drop(c);

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
                    let stream_lock = stream.lock().unwrap();
                    stream_lock.peek(&mut temp_buffer).is_ok()
                };

                if data_available {
                    let mut stream_lock = stream.lock().unwrap();




                    match stream_lock.read(&mut buffer) {
                        Ok(size) if size > 0 => {
                            let comm: Message = match bincode::deserialize::<Message>(&buffer[..size]) {
                                Ok(msg) => {

                                    match msg.message_type {
                                        MessageType::ChestInvUpdate => {
                                            println!("CIU incoming goose {}", Uuid::from_u64_pair(msg.goose.0, msg.goose.1));
                                        }
                                        _ => {

                                        }
                                    }
                                    msg
                                }
                                Err(e) => {
                                    Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                                }
                            };

                            match comm.message_type {
                                MessageType::ChestReg => {
                                    
                                    println!("Receiving ChestReg:");

                                    if comm.info > 0 {






                                        let mut payload_buffer = vec![0u8; comm.info as usize];
                                        let mut total_read = 0;
            
                                        while total_read < comm.info as usize {
                                            match stream_lock.read(&mut payload_buffer[total_read..]) {
                                                Ok(n) if n > 0 => total_read += n,
                                                Ok(_) => {
                                                    // Connection closed
                                                    println!("Connection closed by server during chestreg");
                                                    break;
                                                }
                                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                                    // Sleep for a short period and retry
                                                    thread::sleep(Duration::from_millis(10));
                                                }
                                                Err(e) => {
                                                    println!("Error receiving chestreg: {}", e);
                                                    break;
                                                }
                                            }
                                        }
            
                                        if total_read == comm.info as usize {

                                            println!("Got the expected bytes for chestreg");
                                            let mut file = File::create("chestdb").unwrap();
                                            file.write_all(&payload_buffer).unwrap();


                                            csys.write().unwrap().load_chests_from_file();
                                            recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                            shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                            
                                        } else {


                                            println!("Error receiving chestreg, trying again...");
                                            NetworkConnector::sendtolocked(&reqchest, &mut stream_lock);
                                        }






                                       
                                        
                                    } else {
                                        recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                        shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                    }

                                    
                                    

                                }
                                MessageType::ReqChestReg => {

                                }
                                MessageType::RequestMyID => {

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
                                    let modind = comm.info2;
                                    let rot = comm.rot;
                                    let scale = 0.3;

                                    let pme: Arc<DashMap<Uuid, ModelEntity>> = pme.clone();


                                    let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);
                                    //println!("NSME Length: {}", nsme.len());
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
                                    println!("Receiving Udm:");
                                    shouldsend.store(false, std::sync::atomic::Ordering::Relaxed);
                                    
                                    stream_lock.set_nonblocking(false).unwrap();
                                    



                                    let mut buff = vec![0 as u8; comm.info as usize];

                                    stream_lock.set_read_timeout(Some(Duration::from_secs(2)));

                                    match stream_lock.read_exact(&mut buff) {


                                        Ok(_) => {
                                            println!("Got the expected bytes for udm");
                                            let mut file = File::create("db").unwrap();
                                            file.write_all(&buff).unwrap();

                                            NetworkConnector::sendtolocked(&reqseed, &mut stream_lock);
                                        }
                                        Err(e) => {
                                            println!("Error receiving, trying again...");
                                            NetworkConnector::sendtolocked(&requdm, &mut stream_lock);
                                        }

                                    }

                                    stream_lock.set_nonblocking(true).unwrap();
                                },
                                MessageType::Seed => {
                                    //println!("Receiving Seed:");
                                    // let mut buff = vec![0 as u8; comm.info as usize];

                                    // stream_lock.set_nonblocking(false).unwrap();


                                    // stream_lock.read_exact(&mut buff).unwrap();


                                    fs::create_dir_all("mp").unwrap();
                                    let mut file = File::create("mp/seed").unwrap(); 

                                    
                                    let recv_s = format!("{}", comm.info);

                                            file.write_all(recv_s.as_bytes()).unwrap();



                                            commqueue.push(comm.clone());
                                            
                                            thread::sleep(Duration::from_millis(200));
                                            NetworkConnector::sendtolocked(&reqpt, &mut stream_lock);


                                    stream_lock.set_nonblocking(true).unwrap();
                                    //println!("{}", recv_s);

                                    
                                },
                                MessageType::RequestTakeoff => {
                                    commqueue.push(comm.clone());
                                },
                                MessageType::RequestPt => {
                                    
                                },
                                MessageType::Pt => {
                                    //println!("Receiving Pt:");
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
                                    
                 
                                    //println!("{}", recv_s);

                                    
                                    
                                },
                                MessageType::YourId => {
                                    // //println!("Receiving Your ID:");
                                    // stream_lock.set_nonblocking(false).unwrap();
                                    // let mut buff = vec![0 as u8; comm.info as usize];
                                    // stream_lock.read_exact(&mut buff).unwrap();

                                    let recv_s = comm.goose;
                                    let uuid = Uuid::from_u64_pair(recv_s.0, recv_s.1);
                                    //println!("{}", uuid);

                                    println!("My uuid, I am being told, is {uuid}");

                                    gknowncams.insert(
                                        uuid.clone(), Vec3::ZERO
                                    );
                                    *(my_uuid.write().unwrap()) = Some(uuid);


                                    
                                    // stream_lock.set_nonblocking(true).unwrap();
                                },
                                MessageType::MobUpdate => {
                                    
                                    commqueue.push(comm.clone());
                                    
                                },
                                MessageType::NewMob => {
                                    let newid = comm.info;

                                    let newtype = comm.info2;

                                    let newpos = Vec3::new(comm.x, comm.y, comm.z);
                                },
                                MessageType::WhatsThatMob => todo!(),
                                MessageType::ShutUpMobMsgs =>  {
                                    
                                },
                                MessageType::MobUpdateBatch => {
                                    //println!("Receiving a Mob Batch:");
                                    let mut payload_buffer = vec![0u8; comm.info as usize];
                                    let mut total_read = 0;
        
                                    while total_read < comm.info as usize {
                                        match stream_lock.read(&mut payload_buffer[total_read..]) {
                                            Ok(n) if n > 0 => total_read += n,
                                            Ok(_) => {
                                                // Connection closed
                                                println!("Connection closed by server");
                                                break;
                                            }
                                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                                // Sleep for a short period and retry
                                                thread::sleep(Duration::from_millis(10));
                                            }
                                            Err(e) => {
                                                println!("Error receiving MobUpdateBatch: {}", e);
                                                break;
                                            }
                                        }
                                    }
        
                                    if total_read == comm.info as usize {
                                        match bincode::deserialize::<MobUpdateBatch>(&payload_buffer) {
                                            Ok(recv_s) => {
                                                if recv_s.count > server_types::MOB_BATCH_SIZE as u8 {
                                                    println!("Ignoring invalid packet with count > {} of {}", server_types::MOB_BATCH_SIZE, recv_s.count);
                                                } else {
                                                    for i in 0..recv_s.count.min(8) {
                                                        let msg = recv_s.msgs[i as usize].clone();
                                                        commqueue.push(msg);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("Failed to deserialize MobUpdateBatch: {}", e);
                                            }
                                        }
                                    } else {
                                        println!("Failed to read the full MobUpdateBatch payload");
                                    }
                                }
                                MessageType::TimeUpdate => {
                                    commqueue.push(comm.clone());
                                }
                                MessageType::ChestInvUpdate => {
                                    //println!("Receiving CIU from goose {}", Uuid::from_u64_pair(comm.goose.0, comm.goose.1));
                                    hpcommqueue.push(comm.clone());
                                },
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
