mod client;
mod coder;
mod protocol;
mod tui;

use clap::{Parser, Subcommand};

/// Coding Human CLI
#[derive(Parser)]
#[command(name = "coding-human", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to an available coder and ask questions
    Client {
        /// Your display name shown to the coder
        name: String,
        /// Automatically execute commands sent by the coder without prompting
        #[arg(long)]
        yes: bool,
    },
    /// Register as a coder and wait for questions
    Coder {
        /// Your display name shown to clients
        label: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Client { name, yes } => {
            if let Err(e) = client::run(name, yes).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Coder { label } => {
            if let Err(e) = coder::run(label).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
