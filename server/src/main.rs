use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();

    for stream in listener.incoming() {
        let stream: TcpStream = stream.unwrap();
        let mut buf_reader = BufReader::new(stream);

        thread::spawn(move || loop {
            let mut buffer = vec![0u8; 64];
            
            buf_reader.read(&mut buffer).ok().unwrap();
            let message = String::from_utf8(buffer).unwrap();
    
            println!("{}", message);
        });
    }
}
