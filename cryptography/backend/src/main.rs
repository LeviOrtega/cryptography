use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

/** TODO
 * Setup with DB 
 * 
 * 
 */

#[derive(serde::Deserialize)]
struct Payload {
    plaintext: i32,
    key: i32,
    cipher: i32,

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

    println!("WebSocket connection established with: {}", socet_addr);
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(msg) => {
                // If it's text or binary, echo it right back
                if msg.is_text() || msg.is_binary(){
                    println!("Received from {}: {}", socet_addr, msg);

                        let text = msg.to_text().unwrap_or("");
                        let payload: Payload = match serde_json::from_str(text) {
                            Ok(p) => p,
                            Err(e) => {
                                eprintln!("Failed to parse JSON: {}", e);
                                continue; 
                            }
                        };

                        let result = calculation(payload.plaintext, payload.key, payload.cipher).to_string();
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

fn calculation(plain: i32, key: i32, cipher: i32) -> char {
    let result: i32 = (25 - (cipher + key)) % 26;

    ALPHABET[result as usize]
}


#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind TCP listener");
    println!("WebSocket server listening on: {}", addr);


    // Accept incoming TCP streams in a loop
    while let Ok((stream, socket_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, socket_addr));
    }
}
