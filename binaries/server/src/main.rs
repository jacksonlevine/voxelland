use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use glam::Vec3;
use voxelland::chunk::ChunkSystem;
use voxelland::game::Game;
use voxelland::vec::IVec3;
use voxelland::server_types::*;

static mut PACKET_SIZE: usize = 0;

pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
    errorstrikes: i8,
}

fn handle_client(
    client_id: Uuid,
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
    csys: &Arc<RwLock<ChunkSystem>>,
) {
    let mut buffer;
    unsafe {
        buffer = vec![0; PACKET_SIZE];
    }

    loop {
        let mut should_break = false;

        {
            let stream = {
                let clients = clients.lock().unwrap();
                clients[&client_id].stream.clone()
            };

            let mut mystream = stream.lock().unwrap();
            match mystream.read(&mut buffer) {
                Ok(numbytes) => {
                    if numbytes > 0 {
                        let message: Message = match bincode::deserialize(&buffer[..numbytes]) {
                            Ok(m) => m,
                            Err(_) => {
                                println!("Erroneous message received!");
                                Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                            }
                        };
                        match message.message_type {
                            MessageType::RequestUdm => {
                                let csys = csys.read().unwrap();
                                let currseed = *(csys.currentseed.read().unwrap());
                                println!("Recvd req world");
                                let world = fs::read_to_string(format!("world/{}/udm", currseed))
                                    .unwrap();

                                let udmmsg = Message::new(
                                    MessageType::Udm,
                                    Vec3::ZERO,
                                    0.0,
                                    bincode::serialized_size(&world).unwrap() as u32,
                                );
                                mystream.write_all(&bincode::serialize(&udmmsg).unwrap()).unwrap();
                                mystream.write_all(&bincode::serialize(&world).unwrap()).unwrap();
                            }
                            MessageType::RequestSeed => {
                                let csys = csys.read().unwrap();
                                let currseed = *(csys.currentseed.read().unwrap());
                                println!("Recvd req seed");
                                let seed = fs::read_to_string(format!("world/{}/seed", currseed))
                                    .unwrap();

                                let seedmsg = Message::new(
                                    MessageType::Seed,
                                    Vec3::ZERO,
                                    0.0,
                                    bincode::serialized_size(&seed).unwrap() as u32,
                                );
                                mystream.write_all(&bincode::serialize(&seedmsg).unwrap()).unwrap();
                                mystream.write_all(&bincode::serialize(&seed).unwrap()).unwrap();
                            }
                            MessageType::PlayerUpdate => {
                                println!("Recvd player update");
                            }
                            MessageType::BlockSet => {
                                println!("Recvd block set");
                                let spot = IVec3::new(message.x as i32, message.y as i32, message.z as i32);
                                let block = message.info;
                            
                                let mut csys = csys.write().unwrap();
                                let currseed = *(csys.currentseed.read().unwrap());
                                //TODO: MAKE THIS CSYS NOT QUEUE ANYTHING SO THEY DONT BUILD UP FOR NOTHING
                                csys.set_block(spot, block, true);

                                //TODO: MAKE THIS JUST WRITE A NEW LINE TO THE FILE INSTEAD OF REWRITING THE WHOLE THING
                                //(IT WILL "COMPRESS" WHEN THE SERVER RELOADS)
                                csys.save_current_world_to_file(format!("world/{}", currseed));
                            },
                            MessageType::RequestTakeoff => {
                                println!("Recvd req takeoff");
                                let mut rng = StdRng::from_entropy();
                                println!("Created rng");
                                let newseed: u32 = rng.gen();
                                println!("Newseed: {}", newseed);
                                let mut csys = csys.write().unwrap();
                                println!("Got csys lock");
                                let curr_planet_type = csys.planet_type;
                                println!("Got planet type");
                                csys.reset(0, newseed, ((curr_planet_type + 1) % 2) as usize);
                                csys.save_current_world_to_file(format!("world/{}", newseed));
                                println!("Reset csys");
                            }
                            MessageType::RequestPt => {
                                let csys = csys.read().unwrap();
                                let currseed = *(csys.currentseed.read().unwrap());
                                let currpt = csys.planet_type;
                                println!("Recvd req pt");
                                let pt = fs::read_to_string(format!("world/{}/pt", currseed)).unwrap();

                                let ptmsg: Message = Message::new(MessageType::Pt, Vec3::ZERO, 0.0, bincode::serialized_size(&pt).unwrap() as u32);
                                mystream.write_all(&bincode::serialize(&ptmsg).unwrap()).unwrap();

                                mystream.write_all(&bincode::serialize(&pt).unwrap()).unwrap();
                            }
                            _ => {}
                        }

                        // Redistribute the message to all clients
                        let clients = clients.lock().unwrap();
                        for (id, client) in clients.iter() {
                            if *id != client_id {
                                let mut stream = client.stream.lock().unwrap();
                                let _ = stream.write_all(&buffer[..numbytes]);
                            } else {
                                let _ = mystream.write_all(&buffer[..numbytes]);
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
                        let mut clients = clients.lock().unwrap();
                        clients.get_mut(&client_id).unwrap().errorstrikes += 1;

                        if clients.get_mut(&client_id).unwrap().errorstrikes > 4 {
                            should_break = true;
                        }
                    }
                }
            }
        }

        if should_break {
            let mut locked_clients = clients.lock().unwrap();
            locked_clients.remove(&client_id);
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    println!("Welcome to VoxelLand Server Version 0.1.0.");
    println!("Hosting on port 6969.");
    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
    let clients = Arc::new(Mutex::new(HashMap::new()));
    unsafe {
        PACKET_SIZE = bincode::serialized_size(&Message::new(MessageType::RequestSeed, Vec3::new(0.0, 0.0, 0.0), 0.0, 0)).unwrap() as usize;
    }

    let width = 10;
    let height = 10;
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let (mut window, events) = glfw
        .create_window(width, height, "VoxellandServer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let initialseed: u32 = 0;

    // let mut game = Game::new(&Arc::new(RwLock::new(window)));

    // game.vars.in_multiplayer = false;

    let mut csys = ChunkSystem::new(0, initialseed, 0, true);

    csys.load_world_from_file(format!("world/{}", initialseed));

    let csysarc = Arc::new(RwLock::new(csys));

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let client_id = Uuid::new_v4();
                let stream = Arc::new(Mutex::new(stream));
                let mut locked_clients = clients.lock().unwrap();
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
                thread::spawn(move || {
                    handle_client(client_id, clients_ref_clone, &csysarc_clone);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}