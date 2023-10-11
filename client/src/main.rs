use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();
    let (reader, writer) = stream.into_split();
    let mut input_buffer = String::new();
    let mut buf_reader = BufReader::new(reader);
    let mut buf = vec![];

    let reader_handle = tokio::spawn(async move {
        loop {
            match buf_reader.read_until(b'\n', &mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        println!("EOF received");
                        break;
                    }
                    let buf_string = String::from_utf8_lossy(&buf);
                    println!("{}", buf_string);
                    buf.clear();
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    });
    let writer_handle = tokio::spawn(async move {
        loop {
            io::stdin().read_line(&mut input_buffer).unwrap();
            writer.writable().await.unwrap();
            writer
                .try_write(format!("{}", input_buffer.as_str()).as_bytes())
                .unwrap();
            input_buffer.clear();
        }
    });
    loop {}
}
