mod ask;
mod protocol;
mod serve;

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
    /// Ask a question to an available programmer
    Ask {
        /// Your display name shown to the programmer
        name: String,
        /// Automatically execute commands sent by the programmer without prompting
        #[arg(long)]
        yes: bool,
    },
    /// Register as a programmer and wait for questions
    Serve {
        /// Your display name shown to clients
        label: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ask { name, yes } => {
            if let Err(e) = ask::run(name, yes).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Serve { label } => {
            if let Err(e) = serve::run(label).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
