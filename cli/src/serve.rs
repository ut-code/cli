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
                Some(Ok(Message::Text(msg))) => {
                    if let Ok(WsMessage::File { path, content }) =
                        serde_json::from_str::<WsMessage>(&msg)
                    {
                        let base = std::path::Path::new("/tmp/coding-human");
                        let dest = base.join(&path);
                        // Reject paths that escape the base directory
                        if !dest.starts_with(base)
                            || std::path::Path::new(&path)
                                .components()
                                .any(|c| c == std::path::Component::ParentDir)
                        {
                            eprintln!("Rejected unsafe file path: {}", path);
                            continue;
                        }
                        if let Some(parent) = dest.parent() {
                            tokio::fs::create_dir_all(parent).await?;
                        }
                        tokio::fs::write(&dest, &content).await?;
                        println!("Received file for context: {} -> {}", path, dest.display());
                        continue;
                    }
                    break msg;
                }
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
                    // Check for @filepath token to trigger an edit request
                    let edit_path = trimmed
                        .split_whitespace()
                        .find(|t| t.starts_with('@') && t.len() > 1)
                        .and_then(|t| t.strip_prefix('@'))
                        .map(|p| p.to_string());
                    if let Some(path) = edit_path {
                        let edit_req =
                            serde_json::to_string(&WsMessage::EditRequest { path: path.clone() })?;
                        write.send(Message::text(edit_req)).await?;
                        // Wait for the File response and then run the edit workflow
                        'wait_file: loop {
                            match read.next().await {
                                Some(Ok(Message::Text(msg))) => {
                                    if let Ok(WsMessage::File { path: fp, content }) =
                                        serde_json::from_str::<WsMessage>(&msg)
                                    {
                                        let base = std::path::Path::new("/tmp/coding-human");
                                        let dest = base.join(&fp);
                                        if !dest.starts_with(base)
                                            || std::path::Path::new(&fp)
                                                .components()
                                                .any(|c| c == std::path::Component::ParentDir)
                                        {
                                            eprintln!("Rejected unsafe file path: {}", fp);
                                            break 'wait_file;
                                        }
                                        if let Some(parent) = dest.parent() {
                                            tokio::fs::create_dir_all(parent).await?;
                                        }
                                        tokio::fs::write(&dest, &content).await?;
                                        println!("Received file: {} -> {}", fp, dest.display());
                                        let editor = std::env::var("EDITOR")
                                            .unwrap_or_else(|_| "nvim".to_string());
                                        match tokio::process::Command::new(&editor)
                                            .arg(&dest)
                                            .status()
                                            .await
                                        {
                                            Err(e) => eprintln!(
                                                "Failed to open editor '{}': {}",
                                                editor, e
                                            ),
                                            Ok(_) => {
                                                let orig_tmp = base.join(format!(
                                                    ".orig.{}",
                                                    fp.replace('/', "_")
                                                ));
                                                if tokio::fs::write(&orig_tmp, &content)
                                                    .await
                                                    .is_ok()
                                                {
                                                    if let (Some(orig_str), Some(dest_str)) =
                                                        (orig_tmp.to_str(), dest.to_str())
                                                    {
                                                        let diff_out =
                                                            tokio::process::Command::new("diff")
                                                                .args(["-u", orig_str, dest_str])
                                                                .output()
                                                                .await;
                                                        let _ =
                                                            tokio::fs::remove_file(&orig_tmp).await;
                                                        if let Ok(out) = diff_out {
                                                            let diff_text =
                                                                String::from_utf8_lossy(&out.stdout)
                                                                    .to_string();
                                                            if !diff_text.is_empty() {
                                                                println!("Waiting for your confirmation to send diff...");
                                                                let mut buf = String::new();
                                                                let _ = async_stdin
                                                                    .read_line(&mut buf)
                                                                    .await;
                                                                match serde_json::to_string(
                                                                    &WsMessage::Diff {
                                                                        path: fp.clone(),
                                                                        diff: diff_text,
                                                                    },
                                                                ) {
                                                                    Ok(diff_msg) => {
                                                                        if let Err(e) = write
                                                                            .send(Message::text(
                                                                                diff_msg,
                                                                            ))
                                                                            .await
                                                                        {
                                                                            eprintln!(
                                                                                "Failed to send diff: {}",
                                                                                e
                                                                            );
                                                                        } else {
                                                                            println!("Waiting for client response...");
                                                                            loop {
                                                                                match read
                                                                                    .next()
                                                                                    .await
                                                                                {
                                                                                    Some(Ok(Message::Text(msg))) => {
                                                                                        if let Ok(WsMessage::DiffResponse { accepted }) =
                                                                                            serde_json::from_str::<WsMessage>(&msg)
                                                                                        {
                                                                                            if accepted {
                                                                                                println!("Client accepted the changes.");
                                                                                            } else {
                                                                                                println!("Client rejected the changes.");
                                                                                            }
                                                                                            break;
                                                                                        } else {
                                                                                            eprintln!("Unexpected message while waiting for diff response: {}", msg);
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
                                                                                    Some(Err(e)) => {
                                                                                        return Err(e.into());
                                                                                    }
                                                                                    None => {
                                                                                        return Err(anyhow::anyhow!("Connection lost while waiting for diff response"));
                                                                                    }
                                                                                    _ => {}
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => eprintln!(
                                                                        "Failed to serialize diff: {}",
                                                                        e
                                                                    ),
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        break 'wait_file;
                                    }
                                    // Not a File message; keep waiting
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
                    } else if let Some(cmd) = trimmed.strip_prefix('$') {
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
