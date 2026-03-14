use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn run(room_id: String) -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    // Step 1: Connect to the room via WebSocket as programmer
    let ws_scheme = if server_url.starts_with("https://") {
        "wss"
    } else {
        "ws"
    };
    let server_host = server_url
        .strip_prefix("https://")
        .or_else(|| server_url.strip_prefix("http://"))
        .unwrap_or(&server_url);

    let ws_url = format!(
        "{}://{}/rooms/{}/programmer",
        ws_scheme, server_host, room_id
    );

    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut async_stdin = tokio::io::BufReader::new(tokio::io::stdin());

    // Step 2: Q&A loop
    loop {
        // Wait for the user's question
        println!("Waiting for user's question...");
        let question = loop {
            match read.next().await {
                Some(Ok(Message::Text(msg))) => break msg,
                Some(Ok(Message::Close(_))) => {
                    println!("Connection closed");
                    return Ok(());
                }
                Some(Err(e)) => return Err(e.into()),
                None => return Err(anyhow::anyhow!("Connection lost")),
                _ => {}
            }
        };

        // Display the question
        println!("\nUser question: {}\n", question);
        println!("Answer (Ctrl+D to finish):\n");

        // Read stdin line by line and send over WebSocket
        // Ctrl+D (EOF) sends [DONE] and loops back to wait for next question
        loop {
            let mut line = String::new();
            if async_stdin.read_line(&mut line).await? == 0 {
                // EOF — signal end of response
                write.send(Message::Text("[DONE]".to_string())).await?;
                println!();
                break;
            }
            write
                .send(Message::text(line.trim_end_matches('\n')))
                .await?;
        }
    }
}
