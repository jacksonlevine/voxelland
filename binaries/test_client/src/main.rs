use voxelland::server_types::*;
use std::net::TcpStream;
use std::io::{self, Write, Read};
use std::thread;
use bincode;
use glam::Vec3;

fn main() -> io::Result<()> {


    let mut stream = TcpStream::connect("127.0.0.1:4848")?;
    println!("Connected to the server!");



    let message = Message::new(MessageType::RequestSeed, Vec3::ZERO, 0.0, 0);



    let serialized_message = bincode::serialize(&message).unwrap();
    stream.write_all(&serialized_message)?;
    println!("Sent message to the server!");




    let mut receive_stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut buffer = [0; 4096];
        loop {
            match receive_stream.read(&mut buffer) {
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
    });




    loop {

    }
}