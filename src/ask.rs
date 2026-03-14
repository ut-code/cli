use anyhow::{Context, Result};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn run() -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    // Step 1: POST /rooms to get a roomid
    let client = Client::new();
    let room_response = client.post(format!("{}/rooms", server_url)).send().await?;
    let room_data: serde_json::Value = room_response.json().await?;
    let room_id = room_data["roomId"]
        .as_str()
        .context("Failed to get roomId from response")?;

    println!("Room ID: {}", room_id);

    // Step 2: Connect to the room via WebSocket
    let ws_scheme = if server_url.starts_with("https://") {
        "wss"
    } else {
        "ws"
    };
    let server_host = server_url
        .strip_prefix("https://")
        .or_else(|| server_url.strip_prefix("http://"))
        .unwrap_or(&server_url);

    let ws_url = format!("{}://{}/rooms/{}/user", ws_scheme, server_host, room_id);

    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Step 3: Q&A loop
    loop {
        // Prompt for next question
        print!("\nQuestion: ");
        io::stdout().flush()?;
        let mut question = String::new();
        if io::stdin().read_line(&mut question)? == 0 {
            // EOF (Ctrl+D) — exit
            break;
        }
        let question = question.trim().to_string();

        if question == "/quit" {
            break;
        }

        // Send the question
        write.send(Message::text(question)).await?;

        // Show "thinking" spinner
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Thinking");

        // Wait for response chunks until [DONE]
        let mut first_chunk = true;
        loop {
            match read.next().await {
                Some(Ok(Message::Text(msg))) => {
                    if first_chunk {
                        spinner.finish_and_clear();
                        first_chunk = false;
                    }
                    if msg == "[DONE]" {
                        // Programmer finished — back to Question prompt
                        break;
                    }
                    print!("{}", msg);
                    io::stdout().flush()?;
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                    spinner.finish_and_clear();
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    Ok(())
}
