use std::env;
use std::io::prelude::*;
use std::net::TcpStream;

fn run_client(host: &str){
    println!("Initializing client...");
    println!("Connecting to host: {0}", host);
}

fn run_server(host: &str){
    println!("Initializing server...");
}

fn check_args(args: &Vec<String>) -> bool {
    if 
    return true;
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
