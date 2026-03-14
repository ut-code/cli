mod ask;
mod respond;
mod ui;

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
    /// Respond to user questions as a programmer
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
        Some(Commands::Respond { roomid }) => {
            if let Err(e) = respond::run(roomid).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            ui::run().unwrap();
        }
    }
}
