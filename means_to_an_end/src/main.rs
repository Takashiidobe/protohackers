use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use std::sync::Mutex;

type Timestamp = i32;
type Price = i32;
type MinTime = i32;
type MaxTime = i32;

type SyncTreeMap = Arc<Mutex<BTreeMap<i32, i32>>>;

#[derive(Debug, Clone)]
enum Request {
    Insert(Timestamp, Price),
    Query(MinTime, MaxTime),
}

impl Request {
    fn from_bytes(data: &[u8; 9]) -> Result<Self> {
        let left = i32::from_be_bytes(data[1..5].try_into().unwrap());
        let right = i32::from_be_bytes(data[5..9].try_into().unwrap());
        match data[0] {
            b'I' => Ok(Request::Insert(left, right)),
            b'Q' => Ok(Request::Query(left, right)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct Response {
    price: i32,
}

impl Response {
    fn new(price: i32) -> Self {
        Self { price }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

async fn handle_client(socket: &mut TcpStream, tree_map: &mut SyncTreeMap) -> Result<()> {
    let (mut read_stream, mut write_stream) = tokio::io::split(socket);
    loop {
        let mut data = [0u8; 9];
        let mut read_stream = BufReader::new(&mut read_stream);

        loop {
            let _read = read_stream.read_exact(&mut data).await?;
            let response = match Request::from_bytes(&data) {
                Ok(request) => {
                    dbg!(&request);
                    let tree_map = tree_map.clone();
                    match request {
                        Request::Query(start, end) => {
                            let tree_map = tree_map.lock().unwrap();
                            let mut total = 0;
                            let mut count = 0;
                            for (_key, val) in tree_map.range(start..=end) {
                                total += val;
                                count += 1;
                            }
                            Response::new(total / count)
                        }
                        Request::Insert(timestamp, price) => {
                            let mut tree_map = tree_map.lock().unwrap();
                            tree_map.insert(timestamp, price);
                            Response::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    Response::default()
                }
            };
            let res_bytes = response.to_bytes();
            write_stream.write_all(&res_bytes).await?;
            write_stream.write_all(b"\n").await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("localhost:48000").await?;
    let tree_map = Arc::new(Mutex::new(BTreeMap::default()));

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        let mut tree_map = tree_map.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket, &mut tree_map).await {
                eprintln!("Error = {:?}", e);
            }
        });
    }
}
