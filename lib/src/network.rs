use std::fs::{self, File};
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{self, Write, Read};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use bincode;
use glam::Vec3;
use lockfree::queue::Queue;

use crate::chunk::ChunkSystem;
use crate::server_types::{Message, MessageType};
use crate::vec::IVec3;


pub struct NetworkConnector {
    pub stream: Option<Arc<Mutex<TcpStream>>>,
    pub recvthread: Option<JoinHandle<()>>,
    pub sendthread: Option<JoinHandle<()>>,
    pub shouldrun: Arc<AtomicBool>,
    pub csys: Arc<RwLock<ChunkSystem>>,
    pub received_world: Arc<AtomicBool>,
    pub commqueue: Arc<Queue<Message>>
}

impl NetworkConnector {
    pub fn new(csys: &Arc<RwLock<ChunkSystem>>, commqueue: &Arc<Queue<Message>>) -> NetworkConnector {
        NetworkConnector {
            stream: None,
            recvthread: None,
            sendthread: None,
            shouldrun: Arc::new(AtomicBool::new(false)),
            csys: csys.clone(),
            received_world: Arc::new(AtomicBool::new(false)),
            commqueue: commqueue.clone()
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

        self.sendthread = Some(thread::spawn(move || {
            let sr = sr2.clone();
            let stream = stream2.clone();
            while sr.load(std::sync::atomic::Ordering::Relaxed) {
                //let message = Message::new(MessageType::PlayerUpdate, Vec3::new(0.0, 0.0, 0.0), 0.0, 0);
                //NetworkConnector::sendto(&message, &stream);
                thread::sleep(Duration::from_secs(1));
            }
        }));

        
        self.recvthread = Some(thread::spawn(move || {
            let mut buffer = vec![0; PACKET_SIZE];
            let csys = csys.clone();


            
            let requdm = Message::new(MessageType::RequestUdm, Vec3::ZERO, 0.0, 0);
            let reqseed = Message::new(MessageType::RequestSeed, Vec3::ZERO, 0.0, 0);
            let reqpt = Message::new(MessageType::RequestPt, Vec3::ZERO, 0.0, 0);
            NetworkConnector::sendto(&requdm, &stream);

            while sr.load(std::sync::atomic::Ordering::Relaxed) {
                let mut temp_buffer = vec![0; PACKET_SIZE];

                {
                    let stream_lock = stream.lock().unwrap();
                    match stream_lock.peek(&mut temp_buffer) {
                        Ok(_) => {}
                        Err(_) => continue,
                    }
                }

                {
                    let mut stream_lock = stream.lock().unwrap();




                    match stream_lock.read(&mut buffer) {
                        Ok(size) if size > 0 => {
                            let recv_m: Message = bincode::deserialize(&buffer[..size]).unwrap();

                            match recv_m.message_type {
                                MessageType::None => {
                                    
                                }
                                MessageType::RequestUdm => {

                                },
                                MessageType::RequestSeed => {
                                    
                                },
                                MessageType::PlayerUpdate => {
                                    
                                },
                                MessageType::BlockSet => {
                                    if recv_m.info == 0 {
                                        csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                        recv_m.info, true, true);
                                    } else {
                                        csys.read().unwrap().set_block_and_queue_rerender(IVec3::new(recv_m.x as i32, recv_m.y as i32, recv_m.z as i32), 
                                        recv_m.info, false, true);
                                    }
                                },
                                MessageType::Udm => {
                                    stream_lock.set_nonblocking(false).unwrap();
                                    println!("Receiving Udm:");
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.read_exact(&mut buff).unwrap();


                                    let recv_s: String = bincode::deserialize(&buff).unwrap();

                                    println!("{}", recv_s);

                                    fs::create_dir_all("mp").unwrap();
                                    let mut file = File::create("mp/udm").unwrap(); 
                                    file.write_all(recv_s.as_bytes()).unwrap();

                                    NetworkConnector::sendtolocked(&reqseed, &mut stream_lock);
                                },
                                MessageType::Seed => {
                                    println!("Receiving Seed:");
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.read_exact(&mut buff).unwrap();
                                    fs::create_dir_all("mp").unwrap();
                                    let mut file = File::create("mp/seed").unwrap(); 

                                    
                                    let recv_s: String = bincode::deserialize(&buff).unwrap();
                                    println!("{}", recv_s);

                                    file.write_all(recv_s.as_bytes()).unwrap();
                                    
                                    NetworkConnector::sendtolocked(&reqpt, &mut stream_lock);
                                },
                                MessageType::RequestTakeoff => {
                                    commqueue.push(recv_m.clone());
                                },
                                MessageType::RequestPt => {
                                    
                                },
                                MessageType::Pt => {
                                    println!("Receiving Pt:");
                                    let mut buff = vec![0 as u8; recv_m.info as usize];
                                    stream_lock.read_exact(&mut buff).unwrap();
                                    fs::create_dir_all("mp").unwrap();
                                    let mut file = File::create("mp/pt").unwrap(); 

                                    
                                    let recv_s: String = bincode::deserialize(&buff).unwrap();
                                    println!("{}", recv_s);

                                    file.write_all(recv_s.as_bytes()).unwrap();
                                    csys.write().unwrap().load_world_from_file(String::from("mp"));
                                    recv_world_bool.store(true, std::sync::atomic::Ordering::Relaxed);
                                    stream_lock.set_nonblocking(true).unwrap();
                                },
                            }

                            println!("Received message from server: {:?}", recv_m);
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
