use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::protocol::{QueueResponse, WsMessage};
use crate::tui::{self, ChatEvent, ChatMsg, ChatRole};

pub async fn run(name: String, yes: bool) -> Result<()> {
    tui::print_banner("CODING|HUMAN");
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let client = Client::new();

    // ── Step 1: Fetch the queue ───────────────────────────────────────────────
    let queue: QueueResponse = client
        .get(format!("{}/queue", server_url))
        .send()
        .await?
        .json()
        .await?;

    if queue.is_empty() {
        println!("No coders are currently available. Try again later.");
        return Ok(());
    }

    // ── Step 2: TUI coder picker ──────────────────────────────────────────────
    let entries: Vec<(String, String)> = queue.into_iter().collect(); // (room_id, label)
    let labels: Vec<String> = entries.iter().map(|(_, l)| l.clone()).collect();

    let selection = tui::pick_from_list("Select a Coder", &labels)?;
    let room_id = match selection {
        Some(i) => entries[i].0.clone(),
        None => return Ok(()),
    };

    // ── Step 3: Connect via WebSocket ─────────────────────────────────────────
    let ws_url = ws_url(&server_url, &format!("/rooms/{}/client", room_id));
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Notify the coder that a client has matched
    write
        .send(Message::text(serde_json::to_string(&WsMessage::Matched {
            client_name: name.clone(),
        })?))
        .await?;

    // ── Step 4: Chat TUI ──────────────────────────────────────────────────────
    let (log_tx, log_rx) = mpsc::unbounded_channel::<ChatMsg>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<ChatEvent>();

    log_tx
        .send(ChatMsg {
            role: ChatRole::System,
            text: format!("Connected to coder. Welcome, {}!", name),
        })
        .ok();

    let name_clone = name.clone();

    // Spawn the async networking task
    let net_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Incoming WebSocket message
                ws_msg = read.next() => {
                    match ws_msg {
                        Some(Ok(Message::Text(msg))) => {
                            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&msg) {
                                match ws_msg {
                                    WsMessage::Done => {
                                        log_tx.send(ChatMsg {
                                            role: ChatRole::System,
                                            text: "─── Answer finished ───".into(),
                                        }).ok();
                                    }

                                    WsMessage::Cmd { command } => {
                                        // Show the command and ask y/n — original logic preserved
                                        log_tx.send(ChatMsg {
                                            role: ChatRole::System,
                                            text: format!("Run: {}?", command),
                                        }).ok();

                                        let execute = if yes {
                                            true
                                        } else {
                                            // Prompt the user inside the TUI input box
                                            log_tx.send(ChatMsg {
                                                role: ChatRole::System,
                                                text: "[y/n]: type y or n and press Enter".into(),
                                            }).ok();
                                            wait_for_yn(&mut event_rx).await
                                        };

                                        let output = if execute {
                                            let result = tokio::process::Command::new("sh")
                                                .arg("-c")
                                                .arg(&command)
                                                .output()
                                                .await;
                                            match result {
                                                Ok(out) => {
                                                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                                                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                                                    match (stdout.is_empty(), stderr.is_empty()) {
                                                        (false, false) => format!("{}\n[stderr]\n{}", stdout, stderr),
                                                        (false, true) => stdout,
                                                        (true, false) => stderr,
                                                        (true, true) => String::new(),
                                                    }
                                                }
                                                Err(e) => format!("error: {}", e),
                                            }
                                        } else {
                                            "(skipped)".to_string()
                                        };

                                        log_tx.send(ChatMsg {
                                            role: ChatRole::System,
                                            text: format!("$ {}\n{}", command, output),
                                        }).ok();

                                        if let Ok(result_msg) = serde_json::to_string(&WsMessage::CmdResult { command, output }) {
                                            let _ = write.send(Message::text(result_msg)).await;
                                        }

                                        // Coder will continue streaming the answer — show waiting indicator
                                        log_tx.send(ChatMsg {
                                            role: ChatRole::System,
                                            text: "Waiting for answer…".into(),
                                        }).ok();
                                    }

                                    WsMessage::Diff { path, diff } => {
                                        log_tx.send(ChatMsg {
                                            role: ChatRole::System,
                                            text: format!("Diff for {}:\n{}", path, diff),
                                        }).ok();

                                        let apply = if yes {
                                            true
                                        } else {
                                            log_tx.send(ChatMsg {
                                                role: ChatRole::System,
                                                text: "apply? [y/n]: type y or n and press Enter".into(),
                                            }).ok();
                                            wait_for_yn(&mut event_rx).await
                                        };

                                        if apply {
                                            let patch_tmp = std::env::temp_dir().join(format!(
                                                "coding-human-patch-{}.patch",
                                                std::process::id()
                                            ));
                                            if tokio::fs::write(&patch_tmp, &diff).await.is_ok() {
                                                let result = tokio::process::Command::new("patch")
                                                    .args(["-i", patch_tmp.to_str().unwrap_or(""), &path])
                                                    .output()
                                                    .await;
                                                let _ = tokio::fs::remove_file(&patch_tmp).await;
                                                match result {
                                                    Ok(out) if out.status.success() => {
                                                        log_tx.send(ChatMsg {
                                                            role: ChatRole::System,
                                                            text: "Patch applied successfully.".into(),
                                                        }).ok();
                                                    }
                                                    Ok(out) => {
                                                        let err = String::from_utf8_lossy(&out.stderr).to_string();
                                                        log_tx.send(ChatMsg {
                                                            role: ChatRole::System,
                                                            text: format!("Patch failed: {}", err),
                                                        }).ok();
                                                    }
                                                    Err(e) => {
                                                        log_tx.send(ChatMsg {
                                                            role: ChatRole::System,
                                                            text: format!("Patch error: {}", e),
                                                        }).ok();
                                                    }
                                                }
                                            }
                                        } else {
                                            log_tx.send(ChatMsg {
                                                role: ChatRole::System,
                                                text: "Diff not applied.".into(),
                                            }).ok();
                                        }

                                        if let Ok(response) = serde_json::to_string(&WsMessage::DiffResponse { accepted: apply }) {
                                            let _ = write.send(Message::text(response)).await;
                                        }

                                        // Coder will continue streaming the answer — show waiting indicator
                                        log_tx.send(ChatMsg {
                                            role: ChatRole::System,
                                            text: "Waiting for answer…".into(),
                                        }).ok();
                                    }

                                    _ => {
                                        // Raw text answer chunks
                                        log_tx.send(ChatMsg { role: ChatRole::Peer, text: msg }).ok();
                                    }
                                }
                            } else {
                                // Not a protocol message — raw answer text
                                log_tx.send(ChatMsg { role: ChatRole::Peer, text: msg }).ok();
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            log_tx.send(ChatMsg {
                                role: ChatRole::System,
                                text: "Coder disconnected.".into(),
                            }).ok();
                            break;
                        }
                        Some(Err(e)) => {
                            log_tx.send(ChatMsg {
                                role: ChatRole::System,
                                text: format!("WS error: {}", e),
                            }).ok();
                            break;
                        }
                        _ => {}
                    }
                }

                // User event from the TUI (questions typed by the client)
                ev = event_rx.recv() => {
                    match ev {
                        Some(ChatEvent::Line(line)) => {
                            if line == "/quit" {
                                break;
                            }
                            // Handle @filepath mentions
                            let mut tokens_out: Vec<&str> = Vec::new();
                            for token in line.split_whitespace() {
                                if let Some(path) = token.strip_prefix('@') {
                                    if !path.is_empty() {
                                        match tokio::fs::read_to_string(path).await {
                                            Ok(content) => {
                                                if let Ok(file_msg) = serde_json::to_string(&WsMessage::File {
                                                    path: path.to_string(),
                                                    content,
                                                }) {
                                                    let _ = write.send(Message::text(file_msg)).await;
                                                }
                                            }
                                            Err(e) => {
                                                log_tx.send(ChatMsg {
                                                    role: ChatRole::System,
                                                    text: format!("Warning: could not read '{}': {}", path, e),
                                                }).ok();
                                            }
                                        }
                                    }
                                }
                                tokens_out.push(token);
                            }
                            let clean = tokens_out.join(" ");
                            if !clean.is_empty() {
                                log_tx.send(ChatMsg {
                                    role: ChatRole::You,
                                    text: clean.clone(),
                                }).ok();
                                if let Ok(q) = serde_json::to_string(&WsMessage::Question {
                                    from: name_clone.clone(),
                                    text: clean,
                                }) {
                                    let _ = write.send(Message::text(q)).await;
                                }
                                // Show waiting indicator while the coder composes an answer
                                log_tx.send(ChatMsg {
                                    role: ChatRole::System,
                                    text: "Waiting for answer…".into(),
                                }).ok();
                            }
                        }
                        Some(ChatEvent::Eof) | Some(ChatEvent::Quit) | None => {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Run the TUI chat on the current thread (blocks until Ctrl+C / Quit)
    tui::run_chat(
        format!("Coding Human — {}", name),
        "Question".to_string(),
        log_rx,
        event_tx,
    )
    .await?;

    net_task.abort();
    Ok(())
}

/// Wait for the user to type "y" or "n" in the TUI input, ignoring other lines.
/// Returns `true` for "y", `false` for "n" or if the channel closes.
async fn wait_for_yn(event_rx: &mut mpsc::UnboundedReceiver<ChatEvent>) -> bool {
    loop {
        match event_rx.recv().await {
            Some(ChatEvent::Line(line)) => {
                let answer = line.trim().to_ascii_lowercase();
                if answer == "y" {
                    return true;
                } else if answer == "n" {
                    return false;
                }
                // Any other input: ignore and keep waiting
            }
            // Ctrl+D or Ctrl+C: treat as "no"
            Some(ChatEvent::Eof) | Some(ChatEvent::Quit) | None => return false,
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
