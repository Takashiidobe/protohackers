use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    loop {
        let mut read = [0; 1028];
        match stream.read(&mut read) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                stream.write_all(&read[0..n]).unwrap();
            }
            Err(err) => {
                panic!("{err}");
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:48000").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(err) => {
                eprintln!("Error: {err}");
            }
        }
    }
}
