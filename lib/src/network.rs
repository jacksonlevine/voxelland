use std::net::TcpStream;
use std::io::{self, Write, Read};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use bincode;

use crate::server_types::Message;

pub struct NetworkConnector {
    pub stream: Option<Arc<RwLock<TcpStream>>>,
    pub activethread: Option<JoinHandle<()>>,
    pub shouldrun: Arc<AtomicBool>
}


impl NetworkConnector {
    pub fn new() -> NetworkConnector {
        let n = NetworkConnector {
            stream: None,
            activethread: None,
            shouldrun: Arc::new(AtomicBool::new(false))
        };
        n
    }

    pub fn send (&self, message: &Message) {

        let stream = self.stream.as_ref().unwrap().clone();

        let serialized_message = bincode::serialize(&message).unwrap();
        stream.write().unwrap().write_all(&serialized_message).unwrap();
    }

    pub fn connect(&mut self, address: String) {
        self.shouldrun.store(true, std::sync::atomic::Ordering::Relaxed);
        const PACKET_SIZE: usize = 90000;
        self.stream = Some(Arc::new(RwLock::new(TcpStream::connect(address).unwrap())));

        let sr = self.shouldrun.clone();
        let stream = self.stream.as_ref().unwrap().clone();
        self.activethread = Some(thread::spawn(move || {
            let mut buffer = [0; PACKET_SIZE];
            let sr = sr.clone();
            let stream = stream.clone();

            while sr.load(std::sync::atomic::Ordering::Relaxed) {
                let mut peekbuf: [u8; 4000] = [0; 4000];
                let streamread = stream.read().unwrap();
                match streamread.peek(&mut peekbuf) {
                    Ok(t) => {
                        drop(streamread);
                        let mut streamwrite = stream.write().unwrap();
                        match streamwrite.read(&mut buffer) {
                            Ok(size) if size > 0 => {
                                let received_message: Message = bincode::deserialize(&buffer[..size]).unwrap();
                                println!("Received message from server: {:?}", received_message);
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
                    Err(e) => {

                    }
                }
            }

        }));
    }


    
}