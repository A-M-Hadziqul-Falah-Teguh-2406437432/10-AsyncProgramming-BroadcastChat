// Bonus: Rust WebSocket server adapted to speak the YewChat JSON protocol.
//
// The original `server.rs` is a plain broadcast echo that prefixes each text
// frame with the sender's `[addr]:`. The YewChat client (Tutorial 3) instead
// exchanges JSON envelopes:
//
//   client -> server : {"messageType":"register","data":"<nick>"}
//   client -> server : {"messageType":"message","data":"<text>"}
//   server -> client : {"messageType":"users","dataArray":["alice","bob"]}
//   server -> client : {"messageType":"message",
//                       "data":"{\"from\":\"alice\",\"message\":\"hi\",\"time\":...}"}


use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::broadcast::{Sender, channel};

use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct WsMessage {
    message_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data_array: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ChatPayload {
    from: String,
    message: String,
    time: u128,
}

type Users = Arc<Mutex<HashMap<SocketAddr, String>>>;

async fn broadcast_users(users: &Users, bcast_tx: &Sender<String>) {
    let nicks: Vec<String> = users.lock().await.values().cloned().collect();
    let envelope = WsMessage {
        message_type: "users".into(),
        data: None,
        data_array: Some(nicks),
    };
    if let Ok(text) = serde_json::to_string(&envelope) {
        let _ = bcast_tx.send(text);
    }
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    users: Users,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            match serde_json::from_str::<WsMessage>(text) {
                                Ok(parsed) => match parsed.message_type.as_str() {
                                    "register" => {
                                        if let Some(nick) = parsed.data {
                                            users.lock().await.insert(addr, nick.clone());
                                            println!("registered {addr} as {nick}");
                                            broadcast_users(&users, &bcast_tx).await;
                                        }
                                    }
                                    "message" => {
                                        let nick = users.lock().await.get(&addr).cloned();
                                        if let (Some(nick), Some(body)) = (nick, parsed.data) {
                                            let payload = ChatPayload {
                                                from: nick,
                                                message: body,
                                                time: SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .map(|d| d.as_millis())
                                                    .unwrap_or(0),
                                            };
                                            if let Ok(inner) = serde_json::to_string(&payload) {
                                                let envelope = WsMessage {
                                                    message_type: "message".into(),
                                                    data: Some(inner),
                                                    data_array: None,
                                                };
                                                if let Ok(out) = serde_json::to_string(&envelope) {
                                                    let _ = bcast_tx.send(out);
                                                }
                                            }
                                        }
                                    }
                                    other => {
                                        eprintln!("ignoring unknown messageType: {other}");
                                    }
                                },
                                Err(e) => {
                                    eprintln!("failed to parse JSON from {addr}: {e}");
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("websocket error: {e}");
                        break;
                    }
                    None => {
                        println!("client disconnected: {addr}");
                        break;
                    }
                }
            }

            result = bcast_rx.recv() => {
                match result {
                    Ok(msg) => {
                        ws_stream.send(Message::text(msg)).await?;
                    }
                    Err(e) => {
                        eprintln!("broadcast receive error: {e}");
                    }
                }
            }
        }
    }

    let was_registered = users.lock().await.remove(&addr).is_some();
    if was_registered {
        broadcast_users(&users, &bcast_tx).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel::<String>(64);
    let users: Users = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Rust WebSocket server (YewChat protocol) listening on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("new connection from {addr:?}");

        let bcast_tx = bcast_tx.clone();
        let users = users.clone();

        tokio::spawn(async move {
            let (_req, ws_stream) = match ServerBuilder::new().accept(socket).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("handshake failed: {e}");
                    return Ok::<(), Box<dyn Error + Send + Sync>>(());
                }
            };
            handle_connection(addr, ws_stream, bcast_tx, users).await
        });
    }
}
