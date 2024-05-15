use voxelland::server_types::*;
use std::net::TcpStream;
use std::io::{self, Write, Read};
use std::thread;
use bincode;

fn main() -> io::Result<()> {


    let mut stream = TcpStream::connect("127.0.0.1:6969")?;
    println!("Connected to the server!");



    let message = Message {
        message_type: MessageType::RequestSeed,
        x: 1.0,
        y: 2.0,
        z: 3.0,
        rot: 4.0,
        info: 42,
    };



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