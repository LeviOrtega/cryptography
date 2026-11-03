use axum::{
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use std::net::SocketAddr;
use tokio::sync::broadcast::error::RecvError;

use crate::protocol::{ClientMsg, ServerMsg};
use crate::state::{AttemptOutcome, SharedState};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn send(sink: &mut SplitSink<WebSocket, Message>, msg: &ServerMsg) -> bool {
    match serde_json::to_string(msg) {
        Ok(json) => sink.send(Message::Text(json.into())).await.is_ok(),
        Err(e) => {
            tracing::error!(error = %e, "failed to serialise server message");
            true
        }
    }
}

async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: SharedState) {
    tracing::info!(%addr, "client connected");

    let (mut sink, mut stream) = socket.split();
    let mut rx = state.tx.subscribe();

    let snapshot = state.game.read().await.snapshot();
    if !send(&mut sink, &snapshot).await {
        return;
    }

    loop {
        tokio::select! {
            broadcast = rx.recv() => {
                match broadcast {
                    Ok(msg) => {
                        if !send(&mut sink, &msg).await { break; }
                    }
                    Err(RecvError::Lagged(n)) => {
                        tracing::warn!(%addr, skipped = n, "client lagged, resending snapshot");
                        let snap = state.game.read().await.snapshot();
                        if !send(&mut sink, &snap).await { break; }
                    }
                    Err(RecvError::Closed) => break,
                }
            }

            inbound = stream.next() => {
                let Some(Ok(msg)) = inbound else { break };
                match msg {
                    Message::Text(text) => {
                        if let Some(reply) = handle_client_msg(&text, &state).await {
                            if !send(&mut sink, &reply).await { break; }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    }

    let _ = sink.close().await;
    tracing::info!(%addr, "client disconnected");
}

async fn handle_client_msg(text: &str, state: &SharedState) -> Option<ServerMsg> {
    let msg: ClientMsg = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            return Some(ServerMsg::Error {
                message: format!("bad message: {e}"),
            });
        }
    };

    match msg {
        ClientMsg::Rotate { ring, index } => {
            let mut game = state.game.write().await;
            match game.rotate(ring, index) {
                Ok(true) => {
                    let snap = game.snapshot();
                    drop(game);
                    state.broadcast(snap);
                    None
                }
                Ok(false) => None,
                Err(message) => Some(ServerMsg::Error { message }),
            }
        }

        ClientMsg::Attempt => {
            let mut game = state.game.write().await;
            let outcome = game.attempt();
            let snap = game.snapshot();
            drop(game);
            match outcome {
                AttemptOutcome::Solved(values) => {
                    state.broadcast(snap);
                    state.broadcast(ServerMsg::Solved { values });
                }
                AttemptOutcome::Shifted => state.broadcast(snap),
                AttemptOutcome::AlreadySolved => {}
            }
            None
        }

        ClientMsg::Reset { token } => {
            let authorised = matches!(&state.admin_token, Some(t) if *t == token);
            if !authorised {
                return Some(ServerMsg::Error {
                    message: "reset not authorised".into(),
                });
            }
            let mut game = state.game.write().await;
            game.reset();
            let snap = game.snapshot();
            drop(game);
            state.broadcast(snap);
            None
        }
    }
}
