use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;

use anyhow::Result;

use std::sync::Mutex;

type SyncHashMap<K, V> = Arc<Mutex<HashMap<K, V>>>;

async fn handle_client(
    buf: [u8; 1024],
    hash_map: &mut SyncHashMap<String, String>,
) -> Result<String> {
    let string_buf = String::from_utf8_lossy(&buf);
    let string_buf: Vec<String> = string_buf.split('\n').map(|s| s.to_string()).collect();
    let string_buf = string_buf.first().unwrap();

    let mut response = String::default();
    let mut hash_map = hash_map.lock().unwrap();

    dbg!(&hash_map);

    if string_buf == "version" {
        response.push_str("version=UDP DB version 1");
    } else if string_buf.contains('=') {
        let (key, val) = string_buf.split_once('=').unwrap();
        hash_map.insert(key.to_string(), val.to_string());
    } else if hash_map.contains_key(string_buf) {
        dbg!(&hash_map[string_buf]);
        response.push_str(&format!("{}={}", string_buf, hash_map[string_buf]));
    } else {
        response.push_str("Error");
    }
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let socket = UdpSocket::bind("localhost:48000").await?;
    let hash_map = Arc::new(Mutex::new(HashMap::default()));

    let arced_socket = Arc::new(socket);

    loop {
        let mut hash_map = hash_map.clone();
        let mut buf = [0; 1024];
        let socket = arced_socket.clone();
        let (_amt, src) = socket.recv_from(&mut buf).await?;
        tokio::spawn(async move {
            if let Ok(response) = handle_client(buf, &mut hash_map).await {
                socket.send_to(response.as_bytes(), src).await.unwrap();
                socket.send_to(b"\n", src).await.unwrap();
            }
        });
    }
}
