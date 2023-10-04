use std::env;
use std::io::prelude::*;
use std::net::*;

fn run_client(host: &str){
    println!("Initializing client...");
    println!("Connecting to host: {:?}", host);
    let mut sock_stream : TcpStream = match TcpStream::connect(host) {
        Ok(sock_stream) => sock_stream,
        Err(err) => panic!("Error connecting to server {0}\n{1}", host, err)
    };
    println!("Successfully connected to host: {:?}", host);
    println!("Sending message 'Test'");
    sock_stream.write_all("Test".as_bytes()).unwrap();

 }

fn run_server(host: &str){
    println!("Initializing server...");
    let listener :TcpListener  = match TcpListener::bind(host) {
        Ok(listener) => listener,
        Err(err) => panic!("Error binding server: {:?}", err)
    };
    let (stream, client_addr): (TcpStream, SocketAddr) = match listener.accept() {
        Ok((stream, client_addr)) => (stream, client_addr),
        Err(err) => panic!("Error accepting client: {:?}", err)
    };
    println!("Client: {:?} successfully connected", client_addr);
}

fn check_args(args: &Vec<String>) -> bool {
    if args.len() > 2 {
        return true;
    }
    false
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if check_args(&args) != true {
        return;
    }
    let user_type: String = args[1].trim().to_lowercase();
    let host: &str = args[2].trim();
    if user_type == "server" {
        run_server(host);
    } else if user_type == "client" {
        run_client(host);
    }
}
