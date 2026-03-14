use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::util::build_ws_url;

pub async fn run(room_id: String) -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let ws_url = build_ws_url(&server_url, &format!("/rooms/{}/programmer", room_id));

    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());

    println!("(Tip: type your answer line by line, then press Enter on a blank line to send)\n");

    loop {
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

        println!("\nUser question: {}\n", question);
        println!("Answer (blank line to finish):\n");

        loop {
            let mut line = String::new();
            if stdin.read_line(&mut line).await? == 0 {
                // stdin closed — exit cleanly
                return Ok(());
            }
            let trimmed = line.trim_end_matches(|c| c == '\n' || c == '\r');
            if trimmed.is_empty() {
                // Blank line — end of this response
                write.send(Message::Text("[DONE]".to_string())).await?;
                println!("[Response sent. Waiting for next question...]\n");
                break;
            }
            write.send(Message::text(trimmed)).await?;
        }
    }
}
