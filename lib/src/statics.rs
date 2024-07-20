use std::{fs::{self, File}, io::Write, path::Path, str::FromStr};

use once_cell::sync::Lazy;
use uuid::Uuid;


use tracing::info;


pub static mut MY_MULTIPLAYER_UUID: Lazy<Uuid> = Lazy::new(|| Uuid::new_v4());

pub static mut LAST_ENTERED_SERVERADDRESS: Lazy<String> = Lazy::new(|| String::from(""));

pub fn SAVE_LESA() {
    let mmu = "lesa";

    // Always create or overwrite the file
    let mut file = File::create(mmu).unwrap();
    unsafe {
        file.write_all(LAST_ENTERED_SERVERADDRESS.to_string().as_bytes()).unwrap();
    }
    info!("UUID file created or overwritten.");
}

pub fn LOAD_OR_INITIALIZE_STATICS() {
    
    let mmu = "mmu";

    if Path::new(mmu).exists() {
        // If the file exists, read its contents
        let contents = fs::read_to_string(mmu).unwrap();
        let uuid = Uuid::from_str(&contents).unwrap();
        unsafe {
            (*MY_MULTIPLAYER_UUID) = uuid;
        }
        info!("UUID file loaded, its {uuid}.");
    } else {
        // If the file doesn't exist, create and write to it
        let mut file = File::create(mmu).unwrap();
        unsafe {
            file.write_all(MY_MULTIPLAYER_UUID.to_string().as_bytes()).unwrap();
        }
        unsafe {
            info!("UUID file created and written to, with {}.", *MY_MULTIPLAYER_UUID);
        }
        
    }

    let mmu = "lesa";

    if Path::new(mmu).exists() {
        // If the file exists, read its contents
        let contents = fs::read_to_string(mmu).unwrap();
        unsafe {
            (*LAST_ENTERED_SERVERADDRESS) = contents;
        }
        info!("UUID file loaded.");
    } else {
        // If the file doesn't exist, create and write to it
        let mut file = File::create(mmu).unwrap();
        unsafe {
            file.write_all(LAST_ENTERED_SERVERADDRESS.to_string().as_bytes()).unwrap();
        }
        info!("UUID file created and written to.");
    }



}