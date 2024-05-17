use serde::{Serialize, Deserialize};
use tokio::fs;
use voxelland::chunk::ChunkSystem;
use voxelland::vec::IVec3;
use std::collections::HashMap;
use std::sync::{Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{self, Duration};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use glam::Vec3;

use voxelland::server_types::*;


static mut PACKET_SIZE: usize = 0;

pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
    errorstrikes: i8,
}

async fn handle_client(client_id: Uuid, clients: Arc<Mutex<HashMap<Uuid, Client>>>, csys: &Arc<RwLock<ChunkSystem>>) {
    let mut buffer;
    unsafe {
        buffer = vec![0; PACKET_SIZE];
    }

    loop {
        let mut should_break = false;

        {
            let stream = {
                let clients = clients.lock().await;
                clients[&client_id].stream.clone()
            };

            let mut mystream = time::timeout(Duration::from_secs(1), stream.lock()).await.unwrap();
            match mystream.read(&mut buffer).await {
                Ok(numbytes) => {
                    if numbytes > 0 {
                        let message: Message = bincode::deserialize(&buffer[..numbytes]).unwrap();
                        match message.message_type {
                            MessageType::RequestUdm => {
                                println!("Recvd req world");
                                let world = fs::read_to_string("world/udm").await.unwrap();

                                let udmmsg = Message::new(MessageType::Udm, Vec3::ZERO, 0.0, bincode::serialized_size(&world).unwrap() as u32);
                                mystream.write_all(&bincode::serialize(&udmmsg).unwrap()).await.unwrap();

                                mystream.write_all(&bincode::serialize(&world).unwrap()).await.unwrap();
                            }
                            MessageType::RequestSeed => {
                                println!("Recvd req seed");
                                let seed = fs::read_to_string("world/seed").await.unwrap();

                                let seedmsg = Message::new(MessageType::Seed, Vec3::ZERO, 0.0, bincode::serialized_size(&seed).unwrap() as u32);
                                mystream.write_all(&bincode::serialize(&seedmsg).unwrap()).await.unwrap();

                                mystream.write_all(&bincode::serialize(&seed).unwrap()).await.unwrap();
                            }
                            MessageType::PlayerUpdate => {
                                println!("Recvd player update");
                            }
                            MessageType::BlockSet => {
                                println!("Recvd block set");
                                let spot = IVec3::new(message.x as i32, message.y as i32, message.z as i32);
                                let block = message.info;
                            
                                let csys = csys.write().await;
                                //TODO: MAKE THIS CSYS NOT QUEUE ANYTHING SO THEY DONT BUILD UP FOR NOTHING
                                csys.set_block(spot, block, true);

                                //TODO: MAKE THIS JUST WRITE A NEW LINE TO THE FILE INSTEAD OF REWRITING THE WHOLE THING
                                //(IT WILL "COMPRESS" WHEN THE SERVER RELOADS)
                                csys.save_current_world_to_file(String::from("world"));
                                
                            }
                            _ => {

                            }
                        }

                        // Redistribute the message to all clients
                        let clients = clients.lock().await;
                        //drop(stream);
                        for (id, client) in clients.iter() {
                            if *id != client_id {
                                let mut stream = client.stream.lock().await;
                                let _ = stream.write_all(&buffer[..numbytes]).await;
                            } else {
                                let _ = mystream.write_all(&buffer[..numbytes]).await;
                            }
                        }
                    } else {
                        should_break = true;
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        should_break = true;
                    } else {
                        let mut clients = time::timeout(Duration::from_secs(1), clients.lock()).await.unwrap();
                        clients.get_mut(&client_id).unwrap().errorstrikes += 1;

                        if clients.get_mut(&client_id).unwrap().errorstrikes > 4 {
                            should_break = true;
                        }
                    }
                }
            }
        }

        if should_break {
            let mut locked_clients = clients.lock().await;
            locked_clients.remove(&client_id);
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:6969").await.unwrap();
    let clients = Arc::new(Mutex::new(HashMap::new()));
    unsafe {
        PACKET_SIZE = bincode::serialized_size(&Message::new(MessageType::RequestSeed, Vec3::new(0.0,0.0,0.0), 0.0, 0)).unwrap() as usize;
    }


    let width = 10;
    let height = 10;
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let (mut window, events) = glfw
        .create_window(width, height, "VoxellandServer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.set_should_close(true);

    let csys = ChunkSystem::new(10, 0, 0);
    
    csys.load_world_from_file(String::from("world"));

    let csysarc = Arc::new(RwLock::new(csys));


    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let client_id = Uuid::new_v4();
                let stream = Arc::new(Mutex::new(stream));
                let mut locked_clients = clients.lock().await;
                locked_clients.insert(
                    client_id,
                    Client {
                        stream,
                        errorstrikes: 0,
                    },
                );
                drop(locked_clients);

                let clients_ref_clone = Arc::clone(&clients);
                let csysarc_clone = Arc::clone(&csysarc);
                tokio::spawn(async move {
                    handle_client(client_id, clients_ref_clone, &csysarc_clone).await;
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
