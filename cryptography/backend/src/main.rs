use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

/** TODO
 * Setup with DB 
 * 
 * 
 */

const KEY: [i32; 8] = [0; 8];

#[derive(serde::Deserialize)]
struct Payload {
    attempt: [i32; 8]
}

async fn handle_connection(raw_stream: TcpStream, socet_addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", socet_addr);

    let ws_stream = match accept_async(raw_stream).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let mut password: [i32; 8] = [0; 8];
    let mut coded: [i32; 8] = derive_coded(password);

    println!("WebSocket connection established with: {}", socet_addr);
    let (mut write, mut read) = ws_stream.split();

    let setup_msg = Message::text(build_coded_string(coded));

    if let Err(e) = write.send(setup_msg).await {
        eprintln!("Error sending message to {}: {}", socet_addr, e);
    }

    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(msg) => {
                if msg.is_text(){
                    println!("Received from {}: {}", socet_addr, msg);

                        let text = msg.to_text().unwrap_or("");
                        let payload: Payload = match serde_json::from_str(text) {
                            Ok(p) => p,
                            Err(e) => {
                                eprintln!("Failed to parse JSON: {}", e);
                                continue; 
                            }
                        };

                    let result = if password == payload.attempt {
                    String::from("success")
                    } else {
                    password = generate_password();                    
                    coded = derive_coded(password);

                    println!("Password {} Code {}", format!("{:?}", password), format!("{:?}", coded));

                    build_coded_string(coded)
                    };                        
                    let new_msg = Message::text(result);

                    if let Err(e) = write.send(new_msg).await {
                        eprintln!("Error sending message to {}: {}", socet_addr, e);
                        break;
                    }
                } else if msg.is_close() {
                    println!("Client {} disconnected.", socet_addr);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading message from {}: {}", socet_addr, e);
                break;
            }
        }
    }
}

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

fn build_coded_string(coded: [i32; 8]) -> String {
    let string_result: String = coded.iter().map(|&num| ALPHABET[num as usize]).collect();
    string_result
}

fn derive_coded(password: [i32;8]) -> [i32; 8] {
    let mut coded: [i32; 8] = [0; 8];
    for (index, value) in coded.iter_mut().enumerate() {
        *value = (25 - (password[index] + KEY[index])) % 26;
    }
    coded
}

fn generate_password() -> [i32; 8] {
    let mut password: [i32; 8] = [0; 8];
    for value in &mut password {
        *value = rand::random_range(0..26);
    }
    password
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind TCP listener");
    println!("WebSocket server listening on: {}", addr);


    while let Ok((stream, socket_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, socket_addr));
    }
}
