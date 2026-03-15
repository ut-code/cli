use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use reqwest::Client;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::protocol::WsMessage;

pub async fn run(label: String) -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let client = Client::new();

    // Step 1: Register in the queue and get a room ID
    let res = client
        .post(format!("{}/queue", server_url))
        .json(&serde_json::json!({ "label": label }))
        .send()
        .await?;
    let data: serde_json::Value = res.json().await?;
    let room_id = data["roomId"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get roomId"))?;

    println!("Registered as \"{}\" (room: {})", label, room_id);
    println!("Waiting for a client to connect...");

    // Step 2: Connect to the room via WebSocket as programmer
    let ws_url = ws_url(&server_url, &format!("/rooms/{}/programmer", room_id));
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut async_stdin = tokio::io::BufReader::new(tokio::io::stdin());

    // Step 3: Q&A loop — wait for questions, type answers
    loop {
        println!("\nWaiting for a question...");
        let question = loop {
            match read.next().await {
                Some(Ok(Message::Text(msg))) => break msg,
                Some(Ok(Message::Close(_))) => {
                    println!("Client disconnected.");
                    // Remove from queue on clean disconnect
                    let _ = client
                        .delete(format!("{}/queue/{}", server_url, room_id))
                        .send()
                        .await;
                    return Ok(());
                }
                Some(Err(e)) => return Err(e.into()),
                None => return Err(anyhow::anyhow!("Connection lost")),
                _ => {}
            }
        };

        println!("\nQuestion: {}\n", question);
        println!("Answer (Ctrl+D to finish, prefix a line with $ to run a command on client):\n");

        loop {
            let mut line = String::new();
            tokio::select! {
                n = async_stdin.read_line(&mut line) => {
                    if n? == 0 {
                        write.send(Message::Text("[DONE]".to_string())).await?;
                        println!();
                        break;
                    }
                    let trimmed = line.trim_end_matches('\n');
                    if let Some(cmd) = trimmed.strip_prefix('$') {
                        let command = cmd.trim().to_string();
                        let msg = serde_json::to_string(&WsMessage::Cmd { command })?;
                        write.send(Message::text(msg)).await?;
                    } else {
                        write.send(Message::text(trimmed)).await?;
                    }
                }
                ws_msg = read.next() => {
                    match ws_msg {
                        Some(Ok(Message::Text(msg))) => {
                            if let Ok(WsMessage::CmdResult { command, output }) =
                                serde_json::from_str::<WsMessage>(&msg)
                            {
                                println!("\n[cmd result] $ {}\n{}", command, output);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("Client disconnected.");
                            let _ = client
                                .delete(format!("{}/queue/{}", server_url, room_id))
                                .send()
                                .await;
                            return Ok(());
                        }
                        Some(Err(e)) => return Err(e.into()),
                        None => return Err(anyhow::anyhow!("Connection lost")),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ws_url(server_url: &str, path: &str) -> String {
    let scheme = if server_url.starts_with("https://") {
        "wss"
    } else {
        "ws"
    };
    let host = server_url
        .strip_prefix("https://")
        .or_else(|| server_url.strip_prefix("http://"))
        .unwrap_or(server_url);
    format!("{}://{}{}", scheme, host, path)
}
