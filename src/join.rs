use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use reqwest::Client;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::util::build_ws_url;

async fn deregister(client: &Client, server_url: &str, room_id: &str) {
    let _ = client
        .delete(format!("{}/queue/{}", server_url, room_id))
        .send()
        .await;
}

pub async fn run(name: String) -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let client = Client::new();

    // Step 1: POST /rooms to create a room
    let room_response = client
        .post(format!("{}/rooms", server_url))
        .send()
        .await?
        .error_for_status()?;
    let room_data: serde_json::Value = room_response.json().await?;
    let room_id = room_data["roomId"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Server did not return roomId"))?
        .to_string();

    // Step 2: POST /queue/register { roomId, name }
    // If this fails, the room is orphaned — but rooms are ephemeral DO instances so this is acceptable.
    let queue_response = client
        .post(format!("{}/queue/register", server_url))
        .json(&serde_json::json!({
            "roomId": room_id,
            "name": name
        }))
        .send()
        .await?;

    if !queue_response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to register in queue"));
    }

    println!("Joined as '{}' - Room ID: {}", name, room_id);
    println!("Waiting for user's question...\n");
    println!("(Tip: type your answer line by line, then press Enter on a blank line to send)\n");

    // Step 3: Connect to the room via WebSocket as programmer
    let ws_url = build_ws_url(&server_url, &format!("/rooms/{}/programmer", room_id));

    let (ws_stream, _) = match connect_async(&ws_url).await {
        Ok(r) => r,
        Err(e) => {
            deregister(&client, &server_url, &room_id).await;
            return Err(e.into());
        }
    };
    let (mut write, mut read) = ws_stream.split();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());

    // Step 4: Q&A loop
    let result: Result<()> = 'qa: loop {
        // Wait for the user's question
        let question: String = 'recv: loop {
            match read.next().await {
                Some(Ok(Message::Text(msg))) => break 'recv msg,
                Some(Ok(Message::Close(_))) => {
                    println!("User disconnected");
                    break 'qa Ok(());
                }
                Some(Err(e)) => break 'qa Err(anyhow::anyhow!("WebSocket error: {}", e)),
                None => break 'qa Err(anyhow::anyhow!("Connection lost")),
                _ => continue 'recv,
            }
        };

        println!("User question: {}\n", question);
        println!("Answer (blank line to finish):\n");

        // Read lines and send each immediately. A blank line ends the response.
        'answer: loop {
            let mut line = String::new();
            match stdin.read_line(&mut line).await {
                Ok(0) => {
                    // stdin closed (process exit) — deregister and quit cleanly
                    break 'qa Ok(());
                }
                Ok(_) => {
                    let trimmed = line.trim_end_matches(|c| c == '\n' || c == '\r');
                    if trimmed.is_empty() {
                        // Blank line — end of this response
                        if let Err(e) = write.send(Message::Text("[DONE]".to_string())).await {
                            break 'qa Err(e.into());
                        }
                        println!("[Response sent. Waiting for next question...]\n");
                        break 'answer;
                    }
                    if let Err(e) = write.send(Message::text(trimmed)).await {
                        break 'qa Err(e.into());
                    }
                }
                Err(e) => break 'qa Err(e.into()),
            }
        }
    };

    deregister(&client, &server_url, &room_id).await;
    result
}
