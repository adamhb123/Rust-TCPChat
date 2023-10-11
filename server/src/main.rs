use queues::*;
use std::collections::HashMap;
use std::{env, u8};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

struct Client {
    name: String,
    write_stream: OwnedWriteHalf,
}

impl Client {
    fn new(name: String, write_stream: OwnedWriteHalf) -> Client {
        Client {
            name: name,
            write_stream: write_stream,
        }
    }
}

type ClientStore = Arc<Mutex<HashMap<SocketAddr, Client>>>;

async fn read_message(buffer_reader: &mut BufReader<OwnedReadHalf>, buf: &mut Vec<u8>) -> Option<String> {
    match buffer_reader.read_until(b'\n', buf).await {
        Ok(n) => {
            if n == 0 {
                println!("EOF received");
                return None;
            }
            let buf_string: String = String::from_utf8_lossy(&buf).trim_end().to_string();
            buf.clear();
            return Some(buf_string);
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:9000".to_string());
    let clients: ClientStore = Arc::new(Mutex::new(HashMap::new()));
    let clients_ncc: ClientStore = Arc::clone(&clients);
    let message_queue: Queue<String> = queue![];
    let message_queue = Arc::new(Mutex::new(message_queue));
    let message_queue_ncc = Arc::clone(&message_queue);
    let mut socket = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on: {}", addr);
    // Writer handler
    let writer_handle = tokio::spawn(async move {
        loop {
            // println!("Message queue size: {}", message_queue.lock().await.size());
            while message_queue.lock().await.size() > 0 {
                let message: String = message_queue.lock().await.remove().unwrap();
                for mut client in clients.lock().await.iter_mut() {
                    println!("Sending: {}", message);
                    client.1.write_stream.write_all(message.as_bytes()).await;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });
    // NCC (new client connections)
    while let Ok((mut stream, peer)) = socket.accept().await {
        println!("Incoming connection from: {}", peer.to_string());
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut buf: Vec<u8> = vec![];
        writer.write_all("Please enter your name:\n".as_bytes()).await.unwrap();
        let client_name = read_message(&mut buf_reader, &mut buf).await.unwrap();
        // Writer handle is stored in client hashmap
        clients_ncc
            .lock()
            .await
            .insert(peer, Client::new(client_name, writer));
        let welcome_message = String::from(format!(
            "Welcome user {}({})!\n",
            clients_ncc.lock().await.get(&peer).unwrap().name,
            peer
        ));
        message_queue_ncc.lock().await.add(welcome_message).unwrap();

        let clients_reader: ClientStore = Arc::clone(&clients_ncc);
        let message_queue_reader: Arc<Mutex<_>> = Arc::clone(&message_queue_ncc);
        // Reader handle, (one created per client)
        let reader_handle = tokio::spawn(async move {
            let client_name = clients_reader.lock().await.get(&peer).unwrap().name.clone();
            loop {
                let message = read_message(&mut buf_reader, &mut buf).await.unwrap();
                message_queue_reader.lock().await.add(format!("{}: {}\n", client_name, message)).unwrap();
            }
        });
    }
    Ok(())
}
