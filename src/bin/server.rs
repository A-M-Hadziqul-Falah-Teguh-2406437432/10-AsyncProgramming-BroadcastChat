use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use std::error::Error;
use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};

use tokio::sync::broadcast::{channel, Sender};

use tokio_websockets::{
    Message,
    ServerBuilder,
    WebSocketStream,
};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {

    // Subscribe this client to the broadcast channel
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {

            // Receive message from websocket client
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {

                        // Only process text messages
                        if let Some(text) = msg.as_text() {

                            // MODIFIKASI 2.3: Sertakan informasi pengirim (IP:Port)
                            // ke dalam pesan broadcast
                            let formatted =
                                format!("[{addr}]: {text}");

                            println!("{formatted}");

                            // Broadcast to all clients
                            let _ = bcast_tx.send(formatted);
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

            // Receive broadcast messages
            result = bcast_rx.recv() => {
                match result {
                    Ok(msg) => {

                        // Send broadcast message to websocket client
                        ws_stream
                            .send(Message::text(msg))
                            .await?;
                    }

                    Err(e) => {
                        eprintln!("broadcast receive error: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // MODIFIKASI: Mengubah port dari 2000 ke 8080
    // Broadcast channel
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("listening on port 8080");

    loop {

        let (socket, addr) = listener.accept().await?;

        println!("New connection from {addr:?}");

        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {

            // Convert TCP stream into websocket stream
            let (_req, ws_stream) =
                ServerBuilder::new()
                    .accept(socket)
                    .await?;

            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}