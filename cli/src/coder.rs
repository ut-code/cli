use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::protocol::{RegisterRequest, RegisterResponse, WsMessage};
use crate::tui::{self, ChatEvent, ChatMsg};

pub async fn run(label: String) -> Result<()> {
    tui::print_banner(&label);
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let client = Client::new();

    // ── Step 1: Register in the queue ─────────────────────────────────────────
    let res = client
        .post(format!("{}/queue", server_url))
        .json(&RegisterRequest {
            label: label.clone(),
        })
        .send()
        .await?;
    let data: RegisterResponse = res.json().await?;
    let room_id = data.room_id.clone();

    // ── Step 2: Connect via WebSocket ─────────────────────────────────────────
    let ws_url = ws_url(&server_url, &format!("/rooms/{}/coder", room_id));
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // ── Step 3: Chat TUI ──────────────────────────────────────────────────────
    let (log_tx, log_rx) = mpsc::unbounded_channel::<ChatMsg>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<ChatEvent>();

    log_tx
        .send(ChatMsg::sys(format!(
            "Registered as \"{}\" (room: {})\nWaiting for a client to connect…",
            label, room_id
        )))
        .ok();
    // Locked while waiting for the client to connect
    log_tx.send(ChatMsg::SetWaiting(true)).ok();

    let label_clone = label.clone();

    let net_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        // ── Wait for Matched ──────────────────────────────────────────────────
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
                    log_tx
                        .send(ChatMsg::sys("Client disconnected before matching."))
                        .ok();
                    return Ok(());
                }
                Some(Err(e)) => return Err(e.into()),
                None => return Err(anyhow::anyhow!("Connection lost")),
                _ => {}
            }
        };

        log_tx
            .send(ChatMsg::sys("Matched with {}!".replace("{}", &client_name)))
            .ok();

        // ── Q&A loop ──────────────────────────────────────────────────────────
        loop {
            // Still locked: waiting for the next question from the client
            log_tx.send(ChatMsg::SetWaiting(true)).ok();
            log_tx.send(ChatMsg::sys("Waiting for a question…")).ok();

            // Wait for a Question (or File) message
            let question = loop {
                tokio::select! {
                    ws_msg = read.next() => {
                        match ws_msg {
                            Some(Ok(Message::Text(msg))) => {
                                if let Ok(WsMessage::File { path, content }) =
                                    serde_json::from_str::<WsMessage>(&msg)
                                {
                                    match safe_tmp_path(&path) {
                                        None => {
                                            log_tx.send(ChatMsg::sys(format!("Rejected unsafe path: {}", path))).ok();
                                        }
                                        Some(dest) => {
                                            if let Some(parent) = dest.parent() {
                                                tokio::fs::create_dir_all(parent).await?;
                                            }
                                            tokio::fs::write(&dest, &content).await?;
                                            let orig_snap = orig_snap_path(&path);
                                            let _ = tokio::fs::write(&orig_snap, &content).await;
                                            log_tx.send(ChatMsg::sys(format!(
                                                "Received file: {} → {}", path, dest.display()
                                            ))).ok();
                                        }
                                    }
                                    continue;
                                }
                                if let Ok(WsMessage::Question { from: _, text }) =
                                    serde_json::from_str::<WsMessage>(&msg)
                                {
                                    break text;
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                log_tx.send(ChatMsg::sys("Client disconnected.")).ok();
                                return Ok(());
                            }
                            Some(Err(e)) => return Err(e.into()),
                            None => return Err(anyhow::anyhow!("Connection lost")),
                            _ => {}
                        }
                    }
                    // Keep draining keyboard events so Quit still works while idle
                    ev = event_rx.recv() => {
                        match ev {
                            Some(ChatEvent::Quit) | None => return Ok(()),
                            _ => {}
                        }
                    }
                }
            };

            // Question received — unlock the input so the coder can type an answer
            log_tx.send(ChatMsg::SetWaiting(false)).ok();
            log_tx.send(ChatMsg::peer(question)).ok();
            log_tx
                .send(ChatMsg::sys(format!(
                    "Type your answer below.\n\
                 Prefix a line with $ to run a command on {}.\n\
                 Ctrl+D to finish answering.",
                    client_name
                )))
                .ok();

            // ── Answer loop ───────────────────────────────────────────────────
            loop {
                tokio::select! {
                    ev = event_rx.recv() => {
                        match ev {
                            Some(ChatEvent::Line(line)) => {
                                let trimmed = line.trim_end_matches('\n').to_string();

                                if let Some(file_path) = trimmed.strip_prefix('@') {
                                    // Open the received file in $EDITOR via the TUI
                                    let file_path = file_path.trim();
                                    match safe_tmp_path(file_path) {
                                        None => {
                                            log_tx.send(ChatMsg::sys(format!(
                                                "Rejected unsafe path: {}", file_path
                                            ))).ok();
                                        }
                                        Some(dest) => {
                                            // Ensure an .orig snapshot exists before editing
                                            let orig_snap = orig_snap_path(file_path);
                                            if !orig_snap.exists() {
                                                if let Ok(bytes) = tokio::fs::read(&dest).await {
                                                    let _ = tokio::fs::write(&orig_snap, &bytes).await;
                                                }
                                            }
                                            // Ask the TUI to suspend itself, open the editor,
                                            // and resume — fixes the broken-layout issue.
                                            let path_str = dest.to_string_lossy().into_owned();
                                            log_tx.send(ChatMsg::OpenEditor(path_str)).ok();
                                            // Wait for the TUI to signal that the editor exited
                                            wait_for_editor_closed(&mut event_rx).await;
                                        }
                                    }

                                } else if let Some(rest) = trimmed.strip_prefix('$') {
                                    if let Some(diff_path) = rest.strip_prefix("send-diff ") {
                                        // Generate and send a unified diff to the client
                                        let diff_path = diff_path.trim();
                                        match safe_tmp_path(diff_path) {
                                            None => {
                                                log_tx.send(ChatMsg::sys(format!(
                                                    "Rejected unsafe path: {}", diff_path
                                                ))).ok();
                                            }
                                            Some(dest) => {
                                                let orig_snap = orig_snap_path(diff_path);
                                                if let (Some(orig_str), Some(dest_str)) =
                                                    (orig_snap.to_str(), dest.to_str())
                                                {
                                                    match tokio::process::Command::new("diff")
                                                        .args(["-u", orig_str, dest_str])
                                                        .output()
                                                        .await
                                                    {
                                                        Ok(out) => {
                                                            let diff_text = String::from_utf8_lossy(&out.stdout).to_string();
                                                            if diff_text.is_empty() {
                                                                log_tx.send(ChatMsg::sys("No changes detected.")).ok();
                                                            } else if let Ok(m) = serde_json::to_string(&WsMessage::Diff {
                                                                path: diff_path.to_string(),
                                                                diff: diff_text,
                                                            }) {
                                                                let _ = write.send(Message::text(m)).await;
                                                                log_tx.send(ChatMsg::sys("Diff sent, waiting for client response…")).ok();
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log_tx.send(ChatMsg::sys(format!("diff error: {}", e))).ok();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // $ command — send to client for execution
                                        let command = rest.trim().to_string();
                                        log_tx.send(ChatMsg::you(format!("$ {}", command))).ok();
                                        if let Ok(m) = serde_json::to_string(&WsMessage::Cmd { command }) {
                                            let _ = write.send(Message::text(m)).await;
                                        }
                                    }

                                } else {
                                    // Plain text line — send directly as answer chunk
                                    log_tx.send(ChatMsg::you(trimmed.clone())).ok();
                                    let _ = write.send(Message::text(trimmed)).await;
                                }
                            }

                            Some(ChatEvent::Eof) => {
                                // Ctrl+D: finish the current answer
                                if let Ok(m) = serde_json::to_string(&WsMessage::Done) {
                                    let _ = write.send(Message::text(m)).await;
                                }
                                log_tx.send(ChatMsg::sys("─── Answer sent ───")).ok();
                                // Lock input again: need to wait for the next question
                                log_tx.send(ChatMsg::SetWaiting(true)).ok();
                                break; // back to waiting for the next question
                            }

                            Some(ChatEvent::Quit) | None => return Ok(()),

                            // Sent by the TUI after OpenEditor completes (handled in wait_for_editor_closed)
                            Some(ChatEvent::EditorClosed) => {}
                        }
                    }

                    // Incoming WS messages while answering (CmdResult, DiffResponse)
                    ws_msg = read.next() => {
                        match ws_msg {
                            Some(Ok(Message::Text(msg))) => {
                                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&msg) {
                                    match ws_msg {
                                        WsMessage::CmdResult { command, output } => {
                                            log_tx.send(ChatMsg::sys(format!(
                                                "[cmd result] $ {}\n{}", command, output
                                            ))).ok();
                                        }
                                        WsMessage::DiffResponse { accepted } => {
                                            log_tx.send(ChatMsg::sys(if accepted {
                                                "[diff] Client accepted and applied the changes.".to_string()
                                            } else {
                                                "[diff] Client rejected the changes.".to_string()
                                            })).ok();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                log_tx.send(ChatMsg::sys("Client disconnected.")).ok();
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
    });

    tui::run_chat(
        format!("Coding Human — {} (coder)", label_clone),
        "Answer".to_string(),
        log_rx,
        event_tx,
    )
    .await?;

    // Deregister from the queue on exit
    let _ = client
        .delete(format!("{}/queue/{}", server_url, room_id))
        .send()
        .await;

    net_task.abort();
    Ok(())
}

/// Drain event_rx until ChatEvent::EditorClosed arrives.
/// This blocks the net task while the TUI is showing the editor.
async fn wait_for_editor_closed(event_rx: &mut mpsc::UnboundedReceiver<ChatEvent>) {
    loop {
        match event_rx.recv().await {
            Some(ChatEvent::EditorClosed) | None => break,
            Some(ChatEvent::Quit) => break,
            _ => {} // ignore Line / Eof while editor is open
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

fn orig_snap_path(relative_path: &str) -> std::path::PathBuf {
    std::path::Path::new("/tmp/coding-human")
        .join(format!(".orig.{}", relative_path.replace('/', "_")))
}
