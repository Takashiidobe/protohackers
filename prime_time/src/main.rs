use anyhow::{anyhow, Result};
use primes::is_prime;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Request {
    method: String,
    number: f64,
}

impl TryFrom<String> for Request {
    type Error = Box<dyn Error>;

    fn try_from(value: String) -> Result<Request, Self::Error> {
        let request: Self = serde_json::from_str(&value)?;
        if request.method != "isPrime" {
            Err(anyhow!("incorrect method: {}", request.method).into())
        } else {
            Ok(request)
        }
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
        let mut read_stream = BufReader::new(&mut read_stream);

        loop {
            let mut data = String::new();
            let read = read_stream.read_line(&mut data).await?;
            if read == 0 {
                break;
            }
            log::info!("{:?}", data);
            let (response, close) = match Request::try_from(data.clone()) {
                Ok(request) => {
                    log::info!("{:?}", request);
                    if request.number < 0.0
                        || request.number.is_nan()
                        || request.number.fract() != 0.0
                    {
                        (Response::new(false), false)
                    } else {
                        (Response::new(is_prime(request.number as u64)), false)
                    }
                }
                Err(e) => {
                    log::error!("{}", e);
                    (Response::default(), true)
                }
            };
            if close {
                write_stream.write_all("mal}".as_bytes()).await?;
                write_stream.write_all(b"\n").await?;
                return Ok(());
            }

            let res_bytes = response.to_bytes();
            write_stream.write_all(&res_bytes).await?;
            write_stream.write_all(b"\n").await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:48000").await?;
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace");

    env_logger::init_from_env(env);

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket).await {
                eprintln!("Error = {:?}", e);
            }
        });
    }
}
