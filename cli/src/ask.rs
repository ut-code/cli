use anyhow::Result;
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

        write.send(Message::text(question)).await?;

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
