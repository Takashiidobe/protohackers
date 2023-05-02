use anyhow::Result;
use primes::is_prime;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Request {
    method: String,
    number: u64,
}

impl Request {
    fn from_str(data: &str) -> Result<Self> {
        let request: Self = serde_json::from_str(data)?;
        Ok(request)
    }
}

#[derive(Debug, Default, Serialize)]
struct Response {
    method: String,
    prime: bool,
}

impl Response {
    fn new(prime: bool) -> Self {
        Self {
            method: "isPrime".into(),
            prime,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

async fn handle_client(socket: &mut TcpStream) -> Result<()> {
    let (mut read_stream, mut write_stream) = tokio::io::split(socket);
    loop {
        let mut data = String::new();
        let mut read_stream = BufReader::new(&mut read_stream);

        loop {
            let read = read_stream.read_line(&mut data).await?;
            if read == 0 {
                break;
            }
            let (response, close) = match Request::from_str(&data) {
                Ok(request) => (Response::new(is_prime(request.number)), false),
                Err(e) => {
                    eprintln!("{}", e);
                    (Response::default(), true)
                }
            };
            let res_bytes = response.to_bytes();
            write_stream.write_all(&res_bytes).await?;
            write_stream.write_all(b"\n").await?;
            if close {
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:48000").await?;

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket).await {
                eprintln!("Error = {:?}", e);
            }
        });
    }
}
