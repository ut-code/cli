use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use reqwest::Client;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::protocol::{RegisterRequest, RegisterResponse, WsMessage};

pub async fn run(label: String) -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let client = Client::new();

    // Step 1: Register in the queue and get a room ID
    let res = client
        .post(format!("{}/queue", server_url))
        .json(&RegisterRequest {
            label: label.clone(),
        })
        .send()
        .await?;
    let data: RegisterResponse = res.json().await?;
    let room_id = &data.room_id;

    println!("Registered as \"{}\" (room: {})", label, room_id);
    println!("Waiting for a client to connect...");

    // Step 2: Run the session; always deregister from the queue when done.
    let result = session(&server_url, room_id).await;
    let _ = client
        .delete(format!("{}/queue/{}", server_url, room_id))
        .send()
        .await;
    result
}

async fn session(server_url: &str, room_id: &str) -> Result<()> {
    // Connect to the room via WebSocket as coder
    let ws_url = ws_url(server_url, &format!("/rooms/{}/coder", room_id));
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut async_stdin = tokio::io::BufReader::new(tokio::io::stdin());

    // Wait for the Matched message to know who connected
    let client_name = loop {
        match read.next().await {
            Some(Ok(Message::Text(msg))) => {
                if let Ok(WsMessage::Matched { client_name }) =
                    serde_json::from_str::<WsMessage>(&msg)
                {
                    break client_name;
                }
            }
            Some(Ok(Message::Close(_))) => {
                println!("Client disconnected before sending match.");
                return Ok(());
            }
            Some(Err(e)) => return Err(e.into()),
            None => return Err(anyhow::anyhow!("Connection lost")),
            _ => {}
        }
    };
    println!("Matched with {}", client_name);

    // Q&A loop — wait for questions, type answers
    loop {
        println!("\nWaiting for a question...");
        let question = loop {
            match read.next().await {
                Some(Ok(Message::Text(msg))) => {
                    if let Ok(WsMessage::File { path, content }) =
                        serde_json::from_str::<WsMessage>(&msg)
                    {
                        match safe_tmp_path(&path) {
                            None => eprintln!("Rejected unsafe file path: {}", path),
                            Some(dest) => {
                                if let Some(parent) = dest.parent() {
                                    tokio::fs::create_dir_all(parent).await?;
                                }
                                tokio::fs::write(&dest, &content).await?;
                                // Save an original snapshot for later diff generation
                                let orig_snap = orig_snap_path(&path);
                                if let Err(e) = tokio::fs::write(&orig_snap, &content).await {
                                    eprintln!("Warning: could not save original snapshot: {}", e);
                                }
                                println!("Received file: {} -> {}", path, dest.display());
                            }
                        }
                        continue;
                    }
                    if let Ok(WsMessage::Question { from: _, text }) =
                        serde_json::from_str::<WsMessage>(&msg)
                    {
                        break text;
                    }
                    // Ignore unrecognised messages while waiting for a question
                }
                Some(Ok(Message::Close(_))) => {
                    println!("Client disconnected.");
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
                        write.send(Message::text(serde_json::to_string(&WsMessage::Done)?)).await?;
                        println!();
                        break;
                    }
                    let trimmed = line.trim_end_matches('\n');
                    if let Some(file_path) = trimmed.strip_prefix('@') {
                        // Open the file from /tmp/coding-human/ in $EDITOR
                        let file_path = file_path.trim();
                        match safe_tmp_path(file_path) {
                            None => eprintln!("Rejected unsafe file path: {}", file_path),
                            Some(dest) => {
                                // Ensure an .orig. snapshot exists before editing
                                let orig_snap = orig_snap_path(file_path);
                                if !orig_snap.exists() {
                                    match tokio::fs::read(&dest).await {
                                        Ok(bytes) => {
                                            if let Err(e) =
                                                tokio::fs::write(&orig_snap, &bytes).await
                                            {
                                                eprintln!(
                                                    "Warning: could not save original snapshot: {}",
                                                    e
                                                );
                                            }
                                        }
                                        Err(e) => eprintln!(
                                            "Warning: could not read '{}' for snapshot: {}",
                                            dest.display(),
                                            e
                                        ),
                                    }
                                }
                                let editor =
                                    std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
                                if let Err(e) = tokio::process::Command::new(&editor)
                                    .arg(&dest)
                                    .status()
                                    .await
                                {
                                    eprintln!("Failed to open editor '{}': {}", editor, e);
                                }
                            }
                        }
                    } else if let Some(rest) = trimmed.strip_prefix('$') {
                        // $diff<space>path — local diff (no space between $ and diff)
                        // $ cmd      — shell command sent to the client (space after $)
                        if let Some(diff_path) = rest.strip_prefix("diff ") {
                            // Generate a unified diff and send it to the client
                            let diff_path = diff_path.trim();
                            match safe_tmp_path(diff_path) {
                                None => eprintln!("Rejected unsafe file path: {}", diff_path),
                                Some(dest) => {
                                    let orig_snap = orig_snap_path(diff_path);
                                    if let (Some(orig_str), Some(dest_str)) =
                                        (orig_snap.to_str(), dest.to_str())
                                    {
                                        let diff_out = tokio::process::Command::new("diff")
                                            .args(["-u", orig_str, dest_str])
                                            .output()
                                            .await;
                                        match diff_out {
                                            Ok(out) => {
                                                let diff_text =
                                                    String::from_utf8_lossy(&out.stdout)
                                                        .to_string();
                                                if diff_text.is_empty() {
                                                    println!("No changes detected.");
                                                } else {
                                                    match serde_json::to_string(&WsMessage::Diff {
                                                        path: diff_path.to_string(),
                                                        diff: diff_text,
                                                    }) {
                                                        Ok(diff_msg) => {
                                                            if let Err(e) = write
                                                                .send(Message::text(diff_msg))
                                                                .await
                                                            {
                                                                eprintln!(
                                                                    "Failed to send diff: {}",
                                                                    e
                                                                );
                                                            } else {
                                                                println!("Diff sent, waiting for client response...");
                                                            }
                                                        }
                                                        Err(e) => eprintln!(
                                                            "Failed to serialize diff: {}",
                                                            e
                                                        ),
                                                    }
                                                }
                                            }
                                            Err(e) => eprintln!("Failed to run diff: {}", e),
                                        }
                                    }
                                }
                            }
                        } else {
                            let command = rest.trim().to_string();
                            let msg = serde_json::to_string(&WsMessage::Cmd { command })?;
                            write.send(Message::text(msg)).await?;
                        }
                    } else {
                        write.send(Message::text(trimmed)).await?;
                    }
                }
                ws_msg = read.next() => {
                    match ws_msg {
                        Some(Ok(Message::Text(msg))) => {
                            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&msg) {
                                match ws_msg {
                                    WsMessage::CmdResult { command, output } => {
                                        println!("\n[cmd result] $ {}\n{}", command, output);
                                    }
                                    WsMessage::DiffResponse { accepted } => {
                                        if accepted {
                                            println!("\n[diff] Client accepted and applied the changes.");
                                        } else {
                                            println!("\n[diff] Client rejected the changes.");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("Client disconnected.");
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

/// Returns the validated destination path inside `/tmp/coding-human/`, or
/// `None` if the path would escape the base directory.
fn safe_tmp_path(relative_path: &str) -> Option<std::path::PathBuf> {
    let base = std::path::Path::new("/tmp/coding-human");
    let dest = base.join(relative_path);
    if dest.starts_with(base)
        && !std::path::Path::new(relative_path)
            .components()
            .any(|c| c == std::path::Component::ParentDir)
    {
        Some(dest)
    } else {
        None
    }
}

/// Returns the path of the `.orig.` snapshot for the given relative file path.
fn orig_snap_path(relative_path: &str) -> std::path::PathBuf {
    std::path::Path::new("/tmp/coding-human")
        .join(format!(".orig.{}", relative_path.replace('/', "_")))
}
