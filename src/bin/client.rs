use futures_util::SinkExt;
use futures_util::stream::StreamExt;

use http::Uri;

use tokio::io::{
    AsyncBufReadExt,
    BufReader,
};

use tokio_websockets::{
    ClientBuilder,
    Message,
};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    // MODIFIKASI: Mengubah port pada URI dari 2000 ke 8080
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(
            Uri::from_static("ws://127.0.0.1:8080")
        )
        .connect()
        .await?;

    let stdin = tokio::io::stdin();

    let mut stdin =
        BufReader::new(stdin).lines();

    println!("Connected to chat server!");
    println!("Type messages below:");

    loop {

        tokio::select! {

            // Read user input
            line = stdin.next_line() => {

                match line {
                    Ok(Some(line)) => {

                        ws_stream
                            .send(Message::text(line))
                            .await?;
                    }

                    Ok(None) => {
                        break;
                    }

                    Err(e) => {
                        eprintln!("stdin error: {e}");
                        break;
                    }
                }
            }

            // Receive messages from server
            msg = ws_stream.next() => {

                match msg {

                    Some(Ok(msg)) => {

                        if let Some(text) = msg.as_text() {

                            println!("{text}");
                        }
                    }

                    Some(Err(e)) => {
                        eprintln!("websocket error: {e}");
                        break;
                    }

                    None => {
                        println!("server disconnected");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}