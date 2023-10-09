use std::borrow::BorrowMut;
use std::env;
use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr};
use std::ops::Deref;
use tokio::net::tcp::{WriteHalf, ReadHalf, OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use queues::*;

struct Client {
    name: String,
    write_stream: OwnedWriteHalf
}

impl Client {
    fn new(name: String, write_stream: OwnedWriteHalf) -> Client {
        Client { name:name, write_stream: write_stream }
    }
}

type ClientStore = Arc<Mutex<HashMap<SocketAddr, Client>>>;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:9000".to_string());
    let clients: ClientStore = Arc::new(Mutex::new(HashMap::new()));
    let clients_ncc: ClientStore = Arc::clone(&clients);
    let message_queue: Queue<String> = queue![];
    let message_queue = Arc::new(Mutex::new(message_queue));
    let message_queue_reader = Arc::clone(&message_queue);
    let message_queue_ncc = Arc::clone(&message_queue);
    let mut socket = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on: {}", addr);
    // Writer handler
    let writer_handle = tokio::spawn(async move {
        loop {
            println!("Message queue size: {}", message_queue.lock().await.size());
            while message_queue.lock().await.size() > 0 {
                let message: String = message_queue.lock().await.remove().unwrap();
                for mut client in clients.lock().await.iter_mut() {
                    println!("Writing: {}", message);
                    client.1.write_stream.write_all(message.as_bytes()).await;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });
    // Reader handler
    
    
    // NCC (new client connections)
    while let Ok((mut stream, peer)) = socket.accept().await {
        println!("Incoming connection from: {}", peer.to_string());
        let (mut reader, writer) = stream.into_split();
        clients_ncc.lock().await.insert(peer,Client::new(String::from("Jonesy"), writer));
        let welcome_message = String::from(format!("Welcome user {}!\n", peer));
        message_queue_ncc.lock().await.add(welcome_message);

        let clients_reader: ClientStore = Arc::clone(&clients_ncc);
        let reader_handle = tokio::spawn(async move {
        loop {
            let mut buf_reader = BufReader::new(reader.borrow_mut());
            let mut buf = vec![];
            println!("LOOP");
            match buf_reader.read_until(b'\n', &mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        println!("EOF received");
                        break;
                    }
                    let buf_string = String::from_utf8_lossy(&buf);
                    println!(
                        "Received message: {}",
                        buf_string
                    );
                    buf.clear();
                },
                Err(e) => panic!("{:?}", e)
            }
        }
    });
    }

    Ok(())
}