use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();

    for stream in listener.incoming() {
        let stream: TcpStream = stream.unwrap();
        let mut buf_reader = BufReader::new(stream);
        
        let mut buffer = [0u8; 1024];
        buf_reader.read(&mut buffer).ok().unwrap();
        let message = String::from_utf8(buffer.to_vec()).unwrap();

        println!("{}", message);
    }

    println!("Hello, world!");
}
