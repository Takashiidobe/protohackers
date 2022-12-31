use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

fn main() {
    match TcpStream::connect("localhost:8080") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 8080");

            let msg = b"Hello!";

            let _ = stream.write_all(msg);
            println!("Sent Hello, awaiting reply...");

            let mut data = [0_u8; 6];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        eprintln!("Unexpected reply: {text}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {e}");
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {e}");
        }
    }
    println!("Terminated.");
}
