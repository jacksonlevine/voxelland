use lockfree::queue::{self, Queue};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::{params, Connection};
use serde::{Serialize, Deserialize};
use voxelland::hud::SlotIndexType;
use voxelland::inventory::{self, ChestInventory, Inventory};
use std::collections::HashMap;
use std::fs::{self, File};

use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use glam::Vec3;
use voxelland::chunk::ChunkSystem;
use voxelland::game::{self, Game};
use voxelland::vec::{self, IVec3};
use voxelland::server_types::{self, *};
use dashmap::DashMap;
use crossbeam::queue::SegQueue;



static mut PACKET_SIZE: usize = 0;

type Nsme = (u32, Vec3, f32, usize, f32, bool);

pub enum QueuedSqlType {
    UserDataMap(u32, IVec3, u32),
    ChestInventoryUpdate(IVec3, [(u32, u32); 20], u32),
    None
}

pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
    inv: Inventory,
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
    queued_sql: &Arc<SegQueue<QueuedSqlType>>,
    chest_reg: &Arc<DashMap<vec::IVec3, ChestInventory>>,
) {
    let mut buffer;
    unsafe {
        buffer = vec![0; PACKET_SIZE];
    }

    println!("Inside thread");

    loop {
        let mut should_break = false;

        let stream = {
            let clients = clients.lock().unwrap();
            clients[&client_id].stream.clone()
        };

        let mut numbytes2 = 0;

        let mut message = {
            let mut mystream = stream.lock().unwrap();

            match mystream.read(&mut buffer) {
                Ok(numbytes) => {
                    numbytes2 = numbytes;
                    if numbytes > 0 {
                        let mut message: Message = match bincode::deserialize(&buffer[..numbytes]) {
                            Ok(m) => m,
                            Err(_) => {
                                println!("Erroneous message received!");
                                Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                            }
                        };
                        let pair = client_id.as_u64_pair();
                        message.goose = pair;

                        message
                    } else {
                        should_break = true;
                        Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                    } else {
                        should_break = true;
                    }

                    Message::new(MessageType::None, Vec3::ZERO, 0.0, 0)
                }
            }
        };

        match message.message_type {
            MessageType::ShutUpMobMsgs => {
                shutupmobmsgs.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            MessageType::RequestUdm => {
                println!("Recvd req world");

                let buffer = {
                    let mut file = File::open("db").unwrap();
                    println!("Opened the db file");
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).unwrap();
                    println!("Read the file to end");
                    buffer
                };

                let udmmsg = Message::new(MessageType::Udm, Vec3::ZERO, 0.0, buffer.len() as u32);

                {
                    let mut mystream = stream.lock().unwrap();
                    mystream.write_all(&bincode::serialize(&udmmsg).unwrap()).unwrap();
                    println!("Wrote the header");
                    mystream.write_all(&buffer).unwrap();
                    println!("Wrote the file buffer");
                }
            }
            MessageType::ReqChestReg => {
                println!("Recvd req chest reg");

                let buffer = {
                    let mut buffer = Vec::new();
                    match File::open("chestdb") {
                        Ok(mut file) => {
                            println!("Opened the db file");
                            file.read_to_end(&mut buffer).unwrap();
                        }
                        Err(_) => {}
                    };
                    println!("Read the file to end");
                    buffer
                };

                let chestmsg = Message::new(MessageType::ChestReg, Vec3::ZERO, 0.0, buffer.len() as u32);

                {
                    {
                        let mut mystream = stream.lock().unwrap();
                        mystream.write_all(&bincode::serialize(&chestmsg).unwrap());
                    }
                    println!("Wrote the chest header");

                    if buffer.len() > 0 {
                        let mut mystream = stream.lock().unwrap();
                        mystream.write_all(&buffer);
                        println!("Wrote the chest file buffer");
                    }
                }
            }
            MessageType::RequestSeed => {
                println!("Recvd req seed");

                let currseed = {
                    let csys = csys.read().unwrap();
                    let s = csys.currentseed.read().unwrap().clone();
                    s
                };

                let seedmsg = Message::new(MessageType::Seed, Vec3::ZERO, 0.0, currseed);

                thread::sleep(Duration::from_millis(100));

                {
                    let mut mystream = stream.lock().unwrap();
                    mystream.write_all(&bincode::serialize(&seedmsg).unwrap()).unwrap();
                }
            }
            MessageType::ChestInvUpdate => {
                let currchest = message.otherpos;
                let destslot = message.info;

                let slotindextype = match message.info2 {
                    0 => SlotIndexType::ChestSlot(destslot as i32),
                    1 => SlotIndexType::InvSlot(destslot as i32),
                    _ => SlotIndexType::None,
                };

                match slotindextype {
                    SlotIndexType::ChestSlot(e) => {
                        let mut chestinv = chest_reg.entry(currchest).or_insert(ChestInventory {
                            dirty: false,
                            inv: [(0, 0); 20],
                        });

                        let slot = &mut chestinv.inv[e as usize];
                        let wasthere = slot.clone();

                        slot.0 = message.rot as u32;
                        slot.1 = message.infof as u32;

                        message.x = wasthere.0 as f32;
                        message.y = wasthere.1 as f32;

                        queued_sql.push(QueuedSqlType::ChestInventoryUpdate(currchest, chestinv.inv.clone(), 0));
                    }
                    SlotIndexType::InvSlot(e) => {
                        let mut clientlock = clients.lock().unwrap();
                        if let Some(cli) = clientlock.get_mut(&client_id) {
                            let slot = &mut cli.inv.inv[e as usize];
                            let wasthere = slot.clone();

                            slot.0 = message.rot as u32;
                            slot.1 = message.infof as u32;

                            message.x = wasthere.0 as f32;
                            message.y = wasthere.1 as f32;
                        }
                    }
                    SlotIndexType::None => {}
                }
            }
            MessageType::PlayerUpdate => {
                let mobmsgs = {
                    knowncams.insert(client_id, Vec3::new(message.x, message.y, message.z));

                    let mut timeupdate = Message::new(MessageType::TimeUpdate, Vec3::ZERO, 0.0, 0);
                    let t = *tod.lock().unwrap();
                    timeupdate.infof = t;

                    {
                        let mut mystream = stream.lock().unwrap();
                        mystream.write_all(&bincode::serialize(&timeupdate).unwrap());
                    }

                    let nlock = nsmes.lock().unwrap();
                    let mobmsgs: Vec<Message> = nlock.iter().map(|nsme| {
                        let mut mobmsg = Message::new(MessageType::MobUpdate, nsme.1, nsme.2, nsme.0);
                        mobmsg.info2 = nsme.3 as u32;
                        mobmsg.infof = nsme.4;
                        mobmsg.bo = nsme.5;

                        
                        mobmsg
                    }).collect();

                    drop(nlock);
                    mobmsgs
                };

                for chunk in mobmsgs.chunks(server_types::MOB_BATCH_SIZE) {
                    let mobmsgbatch = MobUpdateBatch::new(chunk.len(), chunk);
                    let mobmsg = Message::new(MessageType::MobUpdateBatch, Vec3::ZERO, 0.0, bincode::serialized_size(&mobmsgbatch).unwrap() as u32);

                    {
                        let mut mystream = stream.lock().unwrap();
                        match mystream.write_all(&bincode::serialize(&mobmsg).unwrap()) {
                            Ok(_) => {
                                //println!("Sent mob header");
                            },
                            Err(e) => {
                                println!("Mob err {e}");
                            },
                        };
                    }

                    thread::sleep(Duration::from_millis(10));

                    {
                        let mut mystream = stream.lock().unwrap();
                        match mystream.write_all(&bincode::serialize(&mobmsgbatch).unwrap()) {
                            Ok(_) => {
                                //println!("Sent mob payload");
                            },
                            Err(e) => {
                                println!("Mob err {e}");
                            },
                        };
                    }

                    thread::sleep(Duration::from_millis(10));
                }
            }
            MessageType::BlockSet => {
                println!("Recvd block set");
                let spot = IVec3::new(message.x as i32, message.y as i32, message.z as i32);
                let block = message.info;

                let mut csys = csys.write().unwrap();
                csys.set_block(spot, block, true);
                queued_sql.push(QueuedSqlType::UserDataMap(0, spot, block));
            }
            MessageType::MultiBlockSet => {
                println!("Recvd multi block set");

                let spot = IVec3::new(message.x as i32, message.y as i32, message.z as i32);
                let spot2 = message.otherpos;

                let block = message.info;
                let block2 = message.info2;

                let mut csys = csys.write().unwrap();
                csys.set_block(spot, block, true);
                csys.set_block(spot2, block2, true);

                queued_sql.push(QueuedSqlType::UserDataMap(0, spot, block));
                queued_sql.push(QueuedSqlType::UserDataMap(0, spot2, block2));
            }
            MessageType::RequestTakeoff => {
                println!("Recvd req takeoff");
                let mut rng = StdRng::from_entropy();
                let newseed: u32 = rng.gen();
                let mut csys = csys.write().unwrap();

                let pt = csys.planet_type.clone();
                csys.reset(0, newseed, (pt + 1) as usize % 2);
                csys.save_current_world_to_file(format!("world/{}", newseed));
                mobspawnqueued.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            MessageType::RequestMyID => {
                println!("Telling someone their id is: {client_id}");

                let mut idmsg = Message::new(MessageType::YourId, Vec3::ZERO, 0.0, bincode::serialized_size(&client_id.as_u64_pair()).unwrap() as u32);
                idmsg.goose = client_id.as_u64_pair();

                {
                    let mut mystream = stream.lock().unwrap();
                    mystream.write_all(&bincode::serialize(&idmsg).unwrap()).unwrap();
                }
            }
            MessageType::RequestPt => {
                let currpt = {
                    let csys = csys.read().unwrap();
                    csys.planet_type
                };

                thread::sleep(Duration::from_millis(100));

                {
                    let ptmsg = Message::new(MessageType::Pt, Vec3::ZERO, 0.0, currpt as u32);
                    let mut mystream = stream.lock().unwrap();
                    mystream.write_all(&bincode::serialize(&ptmsg).unwrap());
                }

                thread::sleep(Duration::from_millis(100));

                {
                    println!("Telling someone their id is: {client_id}");
                    let mut idmsg = Message::new(MessageType::YourId, Vec3::ZERO, 0.0, bincode::serialized_size(&client_id.as_u64_pair()).unwrap() as u32);
                    idmsg.goose = client_id.as_u64_pair();

                    let mut mystream = stream.lock().unwrap();
                    mystream.write_all(&bincode::serialize(&idmsg).unwrap());
                }

                thread::sleep(Duration::from_millis(100));

                shutupmobmsgs.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {}
        }

        {   
            let clients = clients.lock().unwrap();
            let newmessageserial = bincode::serialize(&message).unwrap();
            for (id, client) in clients.iter() {
                if *id != client_id {
                    let mut stream = client.stream.lock().unwrap();
                    let _ = stream.write_all(&newmessageserial);
                } else if message.message_type != MessageType::PlayerUpdate {
                    let mut mystream = stream.lock().unwrap();
                    let _ = mystream.write_all(&newmessageserial[..numbytes2]);
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

    let mut gameh = Game::new(&Arc::new(RwLock::new(window)), false, true, &Arc::new(AtomicBool::new(false)), &Arc::new(Mutex::new(None)));

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

    let chestreg = csys.chest_registry.clone();

    csys.load_world_from_file(format!("world/{}", initialseed));

    csys.save_current_world_to_file(format!("world/{}", initialseed));

    drop(csys);

    let mut knowncams = &gamewrite.known_cameras.clone();

    let mut chunksys = &gamewrite.chunksys.clone();

    let nsme = &gamewrite.non_static_model_entities.clone();

    let mut nsme_bare = nsme.iter().map(|e| (e.id, e.position, e.rot.y, e.model_index, e.scale, e.sounding)).collect::<Vec<_>>();

    let mut mobspawnqueued = Arc::new(AtomicBool::new(true));


    

    let mut nsme_bare_arc: Arc<Mutex<Vec<Nsme>>> = Arc::new(Mutex::new(nsme_bare));



    let mut shutupmobmsgs = Arc::new(AtomicBool::new(false));


    let mut todclone = gamewrite.timeofday.clone();

    drop(gamewrite);

    listener.set_nonblocking(true);


    let writelock: Arc<Mutex<u8>> = Arc::new(Mutex::new(0u8));


    let queued_sql: Arc<SegQueue<QueuedSqlType>> = Arc::new(SegQueue::new());


    let qs = queued_sql.clone();
    let qs2 = qs.clone();

    fn handlesql(sql: &QueuedSqlType) {

        println!("Calling handlesql");
        let mut retry = true;
        let mut retries = 0;

        while retry {
            match {

                match sql {
                    QueuedSqlType::UserDataMap(seed, spot, block) => {

                        let table_name = format!("userdatamap_{}", seed);


                        let conn = Connection::open("db").unwrap();

                        // Insert userdatamap entries
                        let mut stmt = conn.prepare(&format!(
                            "INSERT OR REPLACE INTO {} (x, y, z, value) VALUES (?, ?, ?, ?)",
                            table_name
                        )).unwrap();

                        stmt.execute(params![spot.x, spot.y, spot.z, block])
                    },
                    QueuedSqlType::ChestInventoryUpdate(key, inv, seed) => {

                        let table_name = format!("chest_registry_{}", seed);
                
                        let conn = Connection::open("chestdb").unwrap();

                        // Ensure the table exists
                        conn.execute(
                            &format!(
                                "CREATE TABLE IF NOT EXISTS {} (
                                    x INTEGER,
                                    y INTEGER,
                                    z INTEGER,
                                    dirty BOOLEAN,
                                    inventory BLOB,
                                    PRIMARY KEY (x, y, z)
                                )",
                                table_name
                            ),
                            (),
                        )
                        .unwrap();
                    
                        let inv_bin = bincode::serialize(&inv).unwrap();
                            
                            // Update the specific entry in the database
                            let mut stmt = conn.prepare(&format!(
                                "INSERT OR REPLACE INTO {} (x, y, z, dirty, inventory) VALUES (?, ?, ?, ?, ?)",
                                table_name
                            )).unwrap();
                    
                            stmt.execute(params![
                                key.x,
                                key.y,
                                key.z,
                                false,
                                inv_bin
                            ])

                        


                    },
                    QueuedSqlType::None => {
                        Ok(0)
                    },
                }

                
                
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
                    stream.lock().unwrap().set_nonblocking(true);
                    println!("About to lock clients");
                    let mut gotlock = false;

                    while !gotlock {
                        match clients.try_lock() {
                            Ok(mut e) => {
                                
                                e.insert(
                                    client_id,
                                    Client {
                                        stream: Arc::clone(&stream),
                                        errorstrikes: 0,
                                        inv: inventory::Inventory{
                                            dirty: false, inv: game::STARTINGITEMS
                                        } 
                                    },
                                );
                                gotlock = true;
                            }
                            Err(e) => {
                            }
                        };
                    }
                    
                    
                    println!("Locked clients");


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
                    let chestreg = chestreg.clone();
                    println!("About to spawn thread");
                    thread::spawn(move || {
                        handle_client(client_id, clients_ref_clone, &csysarc_clone, &knowncams_clone, &msq_clone, &su_clone, &nsme_clone, &wl_clone, &todclone, &queued_sql, &chestreg);
                    });
                    println!("Spawned thread");
                    
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
        
        
        *nblock = nsme.iter().map(|e| (*e.key(), e.position, e.rot.y, e.model_index, e.scale, e.sounding)).collect::<Vec<_>>();

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
                println!("Spawning mobs");

                if true {//chunksys.read().unwrap().planet_type == 1 {
                    let mut rng = StdRng::from_entropy();
                    let mut gamewrite = gamearc.write().unwrap();
                    gamewrite.create_non_static_model_entity(0, Vec3::new(-100.0, 100.0, 350.0), 5.0, Vec3::new(0.0, 0.0, 0.0), 7.0);
                    
                    gamewrite.create_non_static_model_entity(4, Vec3::new(-100.0, 100.0, -450.0), 30.0, Vec3::new(0.0, 0.0, 0.0), 7.0);


                    
                    for i in 0..10 {
                        if rng.gen_range(0..=3) <= 2 {
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            gamewrite.create_non_static_model_entity(4, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 1.1);
                            
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);
                            gamewrite.create_non_static_model_entity(6, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 0.3, Vec3::new(0.0, 0.0, 0.0), 1.5);

                            
                            gamewrite.create_non_static_model_entity(3, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 3.0);
                            gamewrite.create_non_static_model_entity(3, Vec3::new(rng.gen_range(-200.0..200.0),600.0,rng.gen_range(-200.0..200.0)), 1.0, Vec3::new(0.0, 0.0, 0.0), 3.0);
                        
                        }
                    }
                    
                }
                mobspawnqueued.store(false, std::sync::atomic::Ordering::Relaxed);
            }
    }
}