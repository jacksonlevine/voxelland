use lockfree::queue::{self, Queue};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::{params, Connection};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use glam::Vec3;
use voxelland::chunk::ChunkSystem;
use voxelland::game::Game;
use voxelland::vec::IVec3;
use voxelland::server_types::{self, *};
use dashmap::DashMap;

static mut PACKET_SIZE: usize = 0;

type Nsme = (u32, Vec3, f32, usize, f32);

pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
    errorstrikes: i8,
}

fn handle_client(
    client_id: Uuid,
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
    csys: &Arc<RwLock<ChunkSystem>>,
    knowncams: &Arc<DashMap<Uuid, Vec3>>,
    mobspawnqueued: &Arc<AtomicBool>,
    shutupmobmsgs: &Arc<AtomicBool>,
    nsmes: &Arc<Mutex<Vec<Nsme>>>,
    wl: &Arc<Mutex<u8>>,
    tod: &Arc<Mutex<f32>>,
    queued_sql: &Arc<Queue<(u32, IVec3, u32)>>
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
                            MessageType::ShutUpMobMsgs => {
                                shutupmobmsgs.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            MessageType::RequestUdm => {
                                let writelock = wl.lock().unwrap();


                                let conn = Connection::open("db").unwrap();
        
                                



                                let csys = csys.read().unwrap();

                                let seed = csys.currentseed.read().unwrap();
                                let table_name = format!("userdatamap_{}", seed);

                                println!("Recvd req world");


                                let mut stmt = conn.prepare(&format!(
                                    "SELECT x, y, z, value FROM {}",
                                    table_name
                                )).unwrap();
                                let userdatamap_iter = stmt.query_map([], |row| {
                                    Ok(Entry {
                                        key: IVec3::new(row.get(0)?, row.get(1)?, row.get(2)?),
                                        value: row.get(3)?,
                                    })
                                }).unwrap();
                        
                                let mut entries: Vec<Entry> = Vec::new();
                                for entry in userdatamap_iter {
                                    entries.push(entry.unwrap());
                                }
                        
                                let serialized_udm = bincode::serialize(&entries).unwrap();

                                let size = bincode::serialized_size(&entries).unwrap(); 


                                let udmmsg = Message::new(
                                    MessageType::Udm,
                                    Vec3::ZERO,
                                    0.0,
                                    size as u32,
                                );

                                mystream.write_all(&bincode::serialize(&udmmsg).unwrap()).unwrap();
                                mystream.write_all(&serialized_udm).unwrap();

                                // let currseed = *(csys.currentseed.read().unwrap());
                                // println!("Recvd req world");
                                // let world = fs::read_to_string(format!("world/{}/udm", currseed))
                                //     .unwrap();

                                // let udmmsg = Message::new(
                                //     MessageType::Udm,
                                //     Vec3::ZERO,
                                //     0.0,
                                //     bincode::serialized_size(&world).unwrap() as u32,
                                // );

                                // mystream.write_all(&bincode::serialize(&udmmsg).unwrap()).unwrap();
                                // mystream.write_all(&bincode::serialize(&world).unwrap()).unwrap();
                            }
                            MessageType::RequestSeed => {
                                let writelock = wl.lock().unwrap();
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
                                knowncams.insert(client_id, Vec3::new(message.x, message.y, message.z));
                                //println!("Recvd player update");

                                let mut timeupdate = Message::new(MessageType::TimeUpdate, Vec3::ZERO, 0.0, 0);
                                timeupdate.infof = *tod.lock().unwrap();

                                match mystream.write_all(&bincode::serialize(&timeupdate).unwrap()) {
                                    Ok(_) => {
                                        
                                    }
                                    Err(e) => {
                                        println!("Error sending mob update payload {}", e)
                                    }
                                }


                                let nlock = nsmes.lock().unwrap();
                                
                                //println!("Number of mobs: {}", nlock.len());

                                let mut mobmsgs: Vec<Message> = Vec::new();
                                
                                for nsme in nlock.iter() {
                                    
                                    let id = nsme.0;
                                    let pos = nsme.1;
                                    let rot = nsme.2;
                                    let modind = nsme.3;
                    
                                    let mut mobmsg = Message::new(MessageType::MobUpdate, pos, rot, id);
                                    mobmsg.info2 = modind as u32;
                                    mobmsg.infof = nsme.4;
                                    mobmsgs.push(mobmsg);

                                }

                                drop(nlock);

                                for chunk in mobmsgs.chunks(server_types::MOB_BATCH_SIZE) {
                                    let writelock = wl.lock().unwrap();
                                    //println!("THIS CHUNK HAS LEN {}", chunk.len());
                                    let mobmsgbatch = MobUpdateBatch::new(chunk.len(), chunk);

                                    let mobmsg = Message::new(MessageType::MobUpdateBatch, Vec3::ZERO, 0.0, bincode::serialized_size(&mobmsgbatch).unwrap() as u32);

                                    match mystream.write_all(&bincode::serialize(&mobmsg).unwrap()) {
                                        Ok(_) => {

                                        }
                                        Err(e) => {
                                            println!("Error sending mob update header {}", e)
                                        }
                                    }
                                    // drop(writelock);
                                     thread::sleep(Duration::from_millis(10));
                                    // let writelock = wl.lock().unwrap();
                                    match mystream.write_all(&bincode::serialize(&mobmsgbatch).unwrap()) {
                                        Ok(_) => {
                                            
                                        }
                                        Err(e) => {
                                            println!("Error sending mob update payload {}", e)
                                        }
                                    }
                                    // drop(writelock);
                                     thread::sleep(Duration::from_millis(10));
                                }

                                

                            }
                            MessageType::BlockSet => {
                                println!("Recvd block set");
                                let spot = IVec3::new(message.x as i32, message.y as i32, message.z as i32);
                                let block = message.info;
                            
                                let mut csys = csys.write().unwrap();
                                let currseed = *(csys.currentseed.read().unwrap());


                                csys.set_block(spot, block, true);

                                //TODO: MAKE THIS JUST WRITE A NEW LINE TO THE FILE INSTEAD OF REWRITING THE WHOLE THING
                                //(IT WILL "COMPRESS" WHEN THE SERVER RELOADS)
                                //csys.save_current_world_to_file(format!("world/{}", currseed));
                                queued_sql.push((currseed, spot, block));
                                //csys.write_new_udm_entry(spot, block);
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

                                drop(csys);
                                mobspawnqueued.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            MessageType::RequestPt => {

                                let writelock = wl.lock().unwrap();
                                let csys = csys.read().unwrap();
                                let currseed = *(csys.currentseed.read().unwrap());
                                let currpt = csys.planet_type;
                                println!("Recvd req pt");
                                let pt = fs::read_to_string(format!("world/{}/pt", currseed)).unwrap();

                                let ptmsg: Message = Message::new(MessageType::Pt, Vec3::ZERO, 0.0, bincode::serialized_size(&pt).unwrap() as u32);
                                mystream.write_all(&bincode::serialize(&ptmsg).unwrap()).unwrap();

                                mystream.write_all(&bincode::serialize(&pt).unwrap()).unwrap();


                                thread::sleep(Duration::from_millis(100));
                                
                                //ID header then ID as u64 pair
                                let idmsg = Message::new(
                                    MessageType::YourId,
                                    Vec3::ZERO,
                                    0.0,
                                    bincode::serialized_size(&client_id.as_u64_pair()).unwrap() as u32,
                                );
                                mystream.write_all(&bincode::serialize(&idmsg).unwrap()).unwrap();
                                mystream.write_all(&bincode::serialize(&client_id.as_u64_pair()).unwrap()).unwrap();

                                thread::sleep(Duration::from_millis(100));

                                shutupmobmsgs.store(false, std::sync::atomic::Ordering::Relaxed);
                            }
                            _ => {}
                        }

                        // Redistribute the message to all clients
                        let clients = clients.lock().unwrap();
                        let writelock = wl.lock().unwrap();
                        match message.message_type {
                            MessageType::BlockSet => {
                                println!("REDISTRIB  A BLOCKSET  RIGHT NOW");
                            }
                            _ => {
                                
                            }
                        }
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
                        // let mut clients = clients.lock().unwrap();
                        // clients.get_mut(&client_id).unwrap().errorstrikes += 1;

                        // if clients.get_mut(&client_id).unwrap().errorstrikes > 4 {
                        //     should_break = true;
                        // }
                    }
                }
            }
        }

        if should_break {
            let mut locked_clients = clients.lock().unwrap();
            locked_clients.remove(&client_id);
            break;
        }

        //thread::sleep(Duration::from_millis(50));
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

    let mut gameh = Game::new(&Arc::new(RwLock::new(window)), false, true);

    while !gameh.is_finished() {
        thread::sleep(Duration::from_millis(100));
    }
    let mut game: Game;

    match gameh.join() {
        Ok(g) => {
            game = g;
        }
        Err(e) => {
            panic!("Jumbotron Shit Broken");
        }
    }

    let gamearc = Arc::new(RwLock::new(game));

    let gamewrite = gamearc.write().unwrap();

    let mut csys = gamewrite.chunksys.write().unwrap();



    csys.load_world_from_file(format!("world/{}", initialseed));

    csys.save_current_world_to_file(format!("world/{}", initialseed));

    drop(csys);

    let mut knowncams = &gamewrite.known_cameras.clone();

    let mut chunksys = &gamewrite.chunksys.clone();

    let nsme = &gamewrite.non_static_model_entities.clone();

    let mut nsme_bare = nsme.iter().map(|e| (e.id, e.position, e.rot.y, e.model_index, e.scale)).collect::<Vec<_>>();

    let mut mobspawnqueued = Arc::new(AtomicBool::new(true));


    

    let mut nsme_bare_arc: Arc<Mutex<Vec<Nsme>>> = Arc::new(Mutex::new(nsme_bare));



    let mut shutupmobmsgs = Arc::new(AtomicBool::new(false));


    let mut todclone = gamewrite.timeofday.clone();

    drop(gamewrite);

    listener.set_nonblocking(true);


    let writelock: Arc<Mutex<u8>> = Arc::new(Mutex::new(0u8));


    let queued_sql: Arc<Queue<(u32, IVec3, u32)>> = Arc::new(Queue::new());


    let qs = queued_sql.clone();
    let qs2 = qs.clone();

    fn handlesql(sql: &(u32, IVec3, u32)) {
        let mut retry = true;
                        let mut retries = 0;

                        while retry {
                            match {

                                let seed = sql.0;
                                let spot = sql.1;
                                let block = sql.2;
                                
                                let table_name = format!("userdatamap_{}", seed);
        
        
                                let conn = Connection::open("db").unwrap();
        
                                // Insert userdatamap entries
                                let mut stmt = conn.prepare(&format!(
                                    "INSERT OR REPLACE INTO {} (x, y, z, value) VALUES (?, ?, ?, ?)",
                                    table_name
                                )).unwrap();
        
                                stmt.execute(params![spot.x, spot.y, spot.z, block])
                                
                            } {
                                Ok(_) => {
                                    retry = false;
                                }
                                Err(e) => {
                                    println!("Sqlite failure, retrying..");
                                    retry = true;
                                    retries += 1;
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                            if retries > 30 {
                                panic!("Retried an operation more than 30 times. Aborting.");
                                
                            }
                        }
    }


    let _sqlthread = thread::spawn(move || {
        let queued_sql = qs.clone();
        loop {
            match queued_sql.pop() {
                Some(sql) => {
                    
                        handlesql(&sql);
                        let mut morestuff = true;
                        while morestuff {
                            match queued_sql.pop() {
                                Some(sql) => {
                                    handlesql(&sql);
                                }
                                None => {
                                    morestuff = false;
                                }
                            }
                        }
                        
                    
                    
                }
                None => {
                    thread::sleep(Duration::from_secs(1));
                }
                
            }
        }
    });

    loop {


        
            match listener.accept() {
                Ok((stream, _)) => {

                    println!("New connection: {}", stream.peer_addr().unwrap());
                    let client_id = Uuid::new_v4();
                    let stream = Arc::new(Mutex::new(stream));
                    stream.lock().unwrap().set_nonblocking(false);

                    let mut locked_clients = clients.lock().unwrap();
                    locked_clients.insert(
                        client_id,
                        Client {
                            stream: Arc::clone(&stream),
                            errorstrikes: 0,
                        },
                    );
                    drop(locked_clients);

                    let clients_ref_clone = Arc::clone(&clients);
                    let csysarc_clone = Arc::clone(&chunksys);
                    let knowncams_clone = Arc::clone(&knowncams);
                    //let nsme_clone = Arc::clone(&nsme);

                    let msq_clone = Arc::clone(&mobspawnqueued);
                    let su_clone = Arc::clone(&shutupmobmsgs);
                    let nsme_clone = Arc::clone(&nsme_bare_arc);
                    let wl_clone = Arc::clone(&writelock);

                    let todclone = todclone.clone();

                    let queued_sql = qs2.clone();

                    thread::spawn(move || {
                        handle_client(client_id, clients_ref_clone, &csysarc_clone, &knowncams_clone, &msq_clone, &su_clone, &nsme_clone, &wl_clone, &todclone, &queued_sql);
                    });
                    
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Ignore this specific error

                }
                Err(e) => {

                    println!("Connection failed: {}", e);
                }
            }



        //println!("Running this");
        glfw.poll_events();
        gamearc.write().unwrap().update();
        let mut nblock = nsme_bare_arc.lock().unwrap();
        
        
        *nblock = nsme.iter().map(|e| (*e.key(), e.position, e.rot.y, e.model_index, e.scale)).collect::<Vec<_>>();

        drop(nblock);

        thread::sleep(Duration::from_millis(50));
            // if !shutupmobmsgs.load(std::sync::atomic::Ordering::Relaxed) {

            //     for nsme in nsme_bare.iter() {
                    

            //         let id = nsme.0;
            //         let pos = nsme.1;
            //         let rot = nsme.2;
            //         let modind = nsme.3;
    
            //         for (uuid, client) in clients.lock().unwrap().iter() {
            //             let mut stream = client.stream.lock().unwrap();
            //             let mut mobmsg = Message::new(MessageType::MobUpdate, pos, rot, id);
            //             mobmsg.info2 = modind as u32;
    
    
            //             stream.write_all(&bincode::serialize(&mobmsg).unwrap());
            //         }
            //     }
            // }
            
        
            if mobspawnqueued.load(std::sync::atomic::Ordering::Relaxed) {

                if true {//chunksys.read().unwrap().planet_type == 1 {
                    let mut rng = StdRng::from_entropy();
                    let mut gamewrite = gamearc.write().unwrap();
                    gamewrite.create_non_static_model_entity(0, Vec3::new(-100.0, 100.0, 350.0), 5.0, Vec3::new(0.0, 0.0, 0.0), 7.0);
                    
                    gamewrite.create_non_static_model_entity(4, Vec3::new(-100.0, 100.0, -450.0), 30.0, Vec3::new(0.0, 0.0, 0.0), 7.0);
                    
                    for i in 0..6 {
                        if rng.gen_range(0..=3) <= 1 {
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-20.0..20.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-20.0..20.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-20.0..20.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-20.0..20.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            
                            gamewrite.create_non_static_model_entity(3, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-20.0..20.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 3.0);
                            gamewrite.create_non_static_model_entity(3, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-20.0..20.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 3.0);
                        }
                    }
                    
                }
                mobspawnqueued.store(false, std::sync::atomic::Ordering::Relaxed);
            }
    }
}