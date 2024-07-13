use std::fs::{self, File};
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::id;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use bincode;
use dashmap::DashMap;
use glam::Vec3;
use glfw::ffi::glfwGetTime;
use lockfree::queue::Queue;
use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::Connection;
use uuid::Uuid;

use crate::camera::Camera;
use crate::chunk::ChunkSystem;
use crate::modelentity::{direction_to_euler, ModelEntity};
use crate::server_types::{self, Entry, Message, MessageType, MobUpdateBatch};
use crate::statics::MY_MULTIPLAYER_UUID;
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

pub static bullshit_queue: Lazy<Queue<Message>> = Lazy::new(|| Queue::new());
pub static tellnow_queue: Lazy<Queue<Message>> = Lazy::new(|| Queue::new());

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

    pub fn send(&self, message: &Message, trysend: bool) {
        //println!("Sending a {}", message.message_type);

        if trysend {
            bullshit_queue.push(message.clone());

        } else {
            tellnow_queue.push(message.clone());
        }

        // if let Some(stream) = &self.stream {
            
            // let serialized_message = bincode::serialize(message).unwrap();
            // let mut written = false;
            // let mut retries = 0;
            // let mut fail = false;
            // while !written && !fail {
            //     match stream.try_lock() {
            //         Ok(mut streamlock) => {
            //             streamlock.write_all(&serialized_message).unwrap();
            //             written = true;
                        
            //         }
            //         Err(e) => {
            //             println!("Couldn't get stream lock in send for {}, trying again", message.message_type);
            //             retries += 1;
            //             thread::sleep(Duration::from_millis(50));
            //             if retries > 5 && trysend {
            //                 fail = true;
            //                 break;
            //             }
            //         }
            //     }
                 
        //     // }
        // }
    }

    pub fn sendto(message: &Message, stream: &Arc<Mutex<TcpStream>>, print: bool, trysend: bool ) {
        

        if trysend {
            bullshit_queue.push(message.clone());

        } else {
            tellnow_queue.push(message.clone());
        }


       // println!("Sending a {}", message.message_type);
        // let serialized_message = bincode::serialize(message).unwrap();
        // let mut written = false;
        // let mut retries = 0;
        // let mut fail = false;
        // while !written & !fail {
        //     match stream.try_lock() {
        //         Ok(mut streamlock) => {
        //             streamlock.write_all(&serialized_message).unwrap();
        //             written = true;
    
        //         }
        //         Err(e) => {
        //             if print {
        //                 println!("Couldn't get stream lock for {} in sendto, trying again", message.message_type);
        //             }
        //             thread::sleep(Duration::from_millis(50));
        //             // let mut rng = StdRng::from_entropy();
        //             // let rand = rng.gen_range(50..150);
        //             // thread::sleep(Duration::from_millis(rand));
        //         }
        //     }
        //     retries += 1;
        //     if trysend && retries > 5 {
        //         fail = true;
        //         break;
        //     }
        // }
        
        
    }

    pub fn sendtoold(message: &Message, stream: &Arc<Mutex<TcpStream>>, print: bool, trysend: bool ) {
        

        // if trysend {
        //     bullshit_queue.push(message.clone());

        // } else {
        //     tellnow_queue.push(message.clone());
        // }

        
       //println!("Sending a {}", message.message_type);
        let serialized_message = bincode::serialize(message).unwrap();

        
        let mut streamlock = stream.lock().unwrap();
        streamlock.write_all(&serialized_message).unwrap();
    

        
        
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

        let mut idgreeting = Message::new(MessageType::TellYouMyID, Vec3::ZERO, 0.0, 0);
        idgreeting.goose = unsafe { (*MY_MULTIPLAYER_UUID).as_u64_pair() };

        Self::sendtoold(&idgreeting, &stream2, false, false);

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
                if true {

                    match tellnow_queue.pop() {
                        Some(message) => {

                        }
                        None => {
                            match sendqueue.pop() {
                                Some(t) => {
                                    NetworkConnector::sendtoold(&t, &stream, true, false);
                                    thread::sleep(Duration::from_millis(10));
                                }
                                None => {
                                    

                                    match bullshit_queue.pop() {
                                        Some(t) => {
                                            NetworkConnector::sendtoold(&t, &stream, true, false);
                                            thread::sleep(Duration::from_millis(10));
                                        }
                                        None => {
                                            
                                            if shouldsend.load(std::sync::atomic::Ordering::Relaxed) {
                                                static mut timer: f64 = 0.0;
                                                static mut last_time: f64 = 0.0;
                                                let current_time = unsafe {
                                                    glfwGetTime()
                                                };
                                                let delta_time = current_time - unsafe { last_time };
                                                unsafe {
                                                    last_time = current_time;
                                                }
                                                
                                                if unsafe{timer} > 0.25 {

                                                    let mut message;
                                                    let mut gotcamlock = false;
                                                    match cam.try_lock() {
                                                        Ok(c) => {
                                                            let dir = direction_to_euler(c.direction.clone());
                                                            message = Message::new(MessageType::PlayerUpdate, c.position.clone(), dir.y, 0);
                                                            message.infof = c.pitch.clone();
                                                            message.info2 = c.yaw.clone() as u32;
                
                                                            gotcamlock  = true;
                
                                                            
                                                        },
                                                        Err(e) => {
                                                            message = Message::new(MessageType::None, Vec3::ZERO, 0.0, 0);
                                                            println!("Couldn't get camera lock");
                                                        },
                                                    };
                
                                                    if gotcamlock {
                                                        NetworkConnector::sendtoold(&message, &stream, true, true);
                                                    }
                                                    
                                                    unsafe{timer = 0.0;}; 
                                                } else {
                                                    unsafe {
                                                        timer += delta_time;
                                                    }
                                                }
                                            }
                                        }
                                    }







                                    
                              
                                    


                                }
                            }
                        }

                    }
                    
                









                    
                }
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
            
            NetworkConnector::sendtoold(&requdm, &stream, true, false);

            while sr.load(std::sync::atomic::Ordering::Relaxed) {
                let mut temp_buffer = vec![0; PACKET_SIZE];

                let data_available = {
                    match stream.try_lock() {
                        Ok(stream_lock) => {
                            stream_lock.peek(&mut temp_buffer).is_ok()
                        }
                        Err(e) => {
                            false
                        }
                    }
                    
                };

                if data_available {
                    let mut stream_lock = stream.lock().unwrap();
                    stream_lock.set_nonblocking(true);



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
                                MessageType::TellMeYourDamnID => {
                                    
                                }
                                MessageType::Disconnect => {
                                    pme.remove(&Uuid::from_u64_pair(comm.goose.0, comm.goose.1));
                                }
                                MessageType::ChestReg => {
                                    
                                    println!("Receiving ChestReg:");

                                    if comm.info > 0 {






                                        let mut payload_buffer = vec![0u8; comm.info as usize];
                                        let mut total_read = 0;

                                        let mut numtimes = 0;
            
                                        while total_read < comm.info as usize && numtimes < 100 {
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
                                            numtimes += 1;
                                            if numtimes > 99 {
                                                println!("Lost this chestreg. ");
                                                break;
                                            }
                                        }
            
                                        if total_read == comm.info as usize {

                                            println!("Got the expected bytes for chestreg");
                                            let mut file = File::create("chestdb").unwrap();
                                            file.write_all(&payload_buffer).unwrap();

                                            drop(file);

                                            let mut written = false;
                                            let mut retries = 0;

                                            while !written {
                                                match csys.try_write() {
                                                    Ok(csys) => {
                                                        csys.load_chests_from_file();
                                                        written = true;
                                                    }
                                                    Err(e) => {
                                                        thread::sleep(Duration::from_millis(100));
                                                        retries += 1;
                                                        println!("Retrying. {}th retry", retries);

                                                    }
                                                }
                                                println!("ENd of this loop");
                                            }
                                            
                                            println!("Got past loop");

                                            //csys.write().unwrap().load_my_inv_from_file();
                                            hpcommqueue.push(comm.clone());
                                            recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                            shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                            
                                        } else {


                                            println!("Error receiving chestreg, trying again...");
                                            NetworkConnector::sendtolocked(&reqchest, &mut stream_lock);
                                        }






                                       
                                        
                                    } else {
                                        recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                        shouldsend.store(true, std::sync::atomic::Ordering::Relaxed);
                                        println!("No chest reg, we're good");
                                    }

                                    
                                    println!("Finished this");

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
                                    let modind = comm.info2;
                                    let rot = comm.rot;
                                    let scale = 0.3;

                                    let pme: Arc<DashMap<Uuid, ModelEntity>> = pme.clone();


                                    let uuid = Uuid::from_u64_pair(comm.goose.0, comm.goose.1);

                                    //println!("Player update: {uuid}");
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

                                    if recv_world_bool.load(std::sync::atomic::Ordering::Relaxed) {
                                        println!("Saying no thanks to this second UDM");
                                        continue;
                                    }
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


                                    let recv_s = format!("{}", comm.info);

                                    println!("Received seed: {}", recv_s);

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
                                        println!("File content: {}", content);


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
                                    //*(my_uuid.write().unwrap()) = Some(uuid);


                                    
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
                                        match bincode::deserialize::<Vec<MobUpdateBatch>>(&payload_buffer) {
                                            Ok(vec) => {

                                                for recv_s in vec {
                                                    if recv_s.count > server_types::MOB_BATCH_SIZE as u8 {
                                                        println!("Ignoring invalid packet with count > {} of {}", server_types::MOB_BATCH_SIZE, recv_s.count);
                                                    } else {
                                                        for i in 0..recv_s.count.min(8) {
                                                            let msg = recv_s.msgs[i as usize].clone();
                                                            commqueue.push(msg);
                                                        }
                                                    }
                                                }
                                                
                                            }
                                            Err(e) => {
                                                println!("Failed to deserialize MobUpdateBatch Vec: {}", e);
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
