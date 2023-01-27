use std::{
    io::{prelude::*},
    net::{TcpListener, TcpStream}
};

fn handle_player(player: &mut TcpStream, opponent: &mut TcpStream) {
    // Fetch move made
    let mut buffer = [0u8; 5];
    player.read(&mut buffer).unwrap();
    
    // Relay move to opponent
    opponent.write_all(&buffer).unwrap();
    opponent.flush().unwrap();
    
    let message = String::from_utf8(buffer.to_vec()).unwrap();
    println!("Relayed move: {}", message);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();

    let mut player1 = listener.accept().unwrap();
    player1.0.write_all(b"white").unwrap(); // Assign color
    player1.0.flush().unwrap();
    println!("Player 1 connected");

    let mut player2 = listener.accept().unwrap();
    println!("Player 2 connected");
    player2.0.write_all(b"black").unwrap(); // Assign color
    player2.0.flush().unwrap();

    println!("Connection established!");
    println!("Player 1: {:?}\nPlayer 2: {:?}", player1.1, player2.1);

    loop {
        handle_player(&mut player1.0, &mut player2.0);
        handle_player(&mut player2.0, &mut player1.0);
    }
}
