mod ask;
mod join;
mod queue_select_tui;
mod respond;
mod util;

use clap::{Parser, Subcommand};

/// Coding Human CLI
#[derive(Parser)]
#[command(name = "coding-human", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Ask a question and get a response
    Ask,
    /// Join as a programmer to answer questions
    Join {
        /// Your name
        name: String,
    },
    /// Respond to user questions as a programmer (legacy)
    Respond {
        /// The room ID to connect to
        roomid: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ask) => {
            if let Err(e) = ask::run().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Join { name }) => {
            if let Err(e) = join::run(name).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Respond { roomid }) => {
            if let Err(e) = respond::run(roomid).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            if let Err(e) = queue_select_tui::run().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
