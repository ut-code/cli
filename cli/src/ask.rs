use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::protocol::WsMessage;

pub async fn run(yes: bool) -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    let client = Client::new();

    // Step 1: Fetch the queue of waiting programmers
    let queue: std::collections::HashMap<String, String> = client
        .get(format!("{}/queue", server_url))
        .send()
        .await?
        .json()
        .await?;

    if queue.is_empty() {
        println!("No programmers are currently available. Try again later.");
        return Ok(());
    }

    // Step 2: Display the list and let the client choose
    let entries: Vec<(&String, &String)> = queue.iter().collect();
    println!("Available programmers:");
    for (i, (_, label)) in entries.iter().enumerate() {
        println!("  [{}] {}", i + 1, label);
    }

    let room_id = loop {
        print!("Select a programmer (1-{}): ", entries.len());
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(());
        }
        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= entries.len() => break entries[n - 1].0.clone(),
            _ => println!("Invalid selection."),
        }
    };

    // Step 3: Connect to the chosen room as client
    let ws_url = ws_url(&server_url, &format!("/rooms/{}/client", room_id));
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    println!("Connected. Type your questions below (Ctrl+D or /quit to exit).\n");

    // Step 4: Q&A loop
    loop {
        print!("Question: ");
        io::stdout().flush()?;
        let mut question = String::new();
        if io::stdin().read_line(&mut question)? == 0 {
            break;
        }
        let question = question.trim().to_string();
        if question == "/quit" {
            break;
        }
        if question.is_empty() {
            continue;
        }

        // Extract @filepath mentions and send each file before the question
        let mut cleaned_tokens: Vec<&str> = Vec::new();
        for token in question.split_whitespace() {
            if let Some(path) = token.strip_prefix('@') {
                if !path.is_empty() {
                    match tokio::fs::read_to_string(path).await {
                        Ok(content) => {
                            let file_msg = serde_json::to_string(&WsMessage::File {
                                path: path.to_string(),
                                content,
                            })?;
                            write.send(Message::text(file_msg)).await?;
                        }
                        Err(e) => eprintln!("Warning: could not read '{}': {}", path, e),
                    }
                    continue;
                }
            }
            cleaned_tokens.push(token);
        }
        let cleaned_question = cleaned_tokens.join(" ");
        if cleaned_question.is_empty() {
            continue;
        }

        write.send(Message::text(cleaned_question)).await?;

        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Waiting for answer");

        let mut first_chunk = true;
        loop {
            match read.next().await {
                Some(Ok(Message::Text(msg))) => {
                    if first_chunk {
                        spinner.finish_and_clear();
                        first_chunk = false;
                    }
                    if msg == "[DONE]" {
                        println!();
                        break;
                    }
                    // Check if this is a typed protocol message
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&msg) {
                        match ws_msg {
                            WsMessage::Cmd { command } => {
                                println!("\nRun: {}?", command);

                                let execute = if yes {
                                    true
                                } else {
                                    print!("[y/n]: ");
                                    io::stdout().flush()?;
                                    let mut answer = String::new();
                                    io::stdin().read_line(&mut answer)?;
                                    answer.trim().eq_ignore_ascii_case("y")
                                };

                                let output = if execute {
                                    let result = tokio::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(&command)
                                        .output()
                                        .await?;
                                    let stdout =
                                        String::from_utf8_lossy(&result.stdout).to_string();
                                    let stderr =
                                        String::from_utf8_lossy(&result.stderr).to_string();
                                    match (stdout.is_empty(), stderr.is_empty()) {
                                        (false, false) => {
                                            format!("{}\n[stderr]\n{}", stdout, stderr)
                                        }
                                        (false, true) => stdout,
                                        (true, false) => stderr,
                                        (true, true) => String::new(),
                                    }
                                } else {
                                    "(skipped)".to_string()
                                };

                                let result_msg = serde_json::to_string(&WsMessage::CmdResult {
                                    command,
                                    output,
                                })?;
                                write.send(Message::text(result_msg)).await?;
                                // Reset spinner to wait for the rest of the answer
                                first_chunk = true;
                                spinner.reset();
                                spinner.set_message("Waiting for answer");
                                continue;
                            }
                            WsMessage::Diff { path, diff } => {
                                if first_chunk {
                                    spinner.finish_and_clear();
                                }
                                println!("\nDiff for {}:\n{}", path, diff);

                                let apply = if yes {
                                    true
                                } else {
                                    print!("apply? [y/n]: ");
                                    io::stdout().flush()?;
                                    let mut answer = String::new();
                                    io::stdin().read_line(&mut answer)?;
                                    answer.trim().eq_ignore_ascii_case("y")
                                };

                                if apply {
                                    let patch_tmp = std::env::temp_dir().join(format!(
                                        "coding-human-patch-{}.patch",
                                        std::process::id()
                                    ));
                                    tokio::fs::write(&patch_tmp, &diff).await?;
                                    let result = tokio::process::Command::new("patch")
                                        .args([
                                            "-i",
                                            patch_tmp.to_str().unwrap_or(""),
                                            &path,
                                        ])
                                        .output()
                                        .await?;
                                    let _ = tokio::fs::remove_file(&patch_tmp).await;
                                    let stdout =
                                        String::from_utf8_lossy(&result.stdout).to_string();
                                    let stderr =
                                        String::from_utf8_lossy(&result.stderr).to_string();
                                    if result.status.success() {
                                        println!("Patch applied successfully.");
                                        if !stdout.is_empty() {
                                            print!("{}", stdout);
                                        }
                                    } else {
                                        eprintln!("Patch failed: {}{}", stdout, stderr);
                                    }
                                } else {
                                    println!("Diff not applied.");
                                }

                                let response_msg = serde_json::to_string(&WsMessage::DiffResponse {
                                    accepted: apply,
                                })?;
                                write.send(Message::text(response_msg)).await?;

                                // Reset spinner to wait for the rest of the answer
                                first_chunk = true;
                                spinner.reset();
                                spinner.set_message("Waiting for answer");
                                continue;
                            }
                            _ => {}
                        }
                    }
                    print!("{}", msg);
                    io::stdout().flush()?;
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                    spinner.finish_and_clear();
                    println!("\nProgrammer disconnected.");
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    Ok(())
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
