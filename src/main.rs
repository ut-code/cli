mod ui;

use clap::Parser;

/// ut.code() CLI
#[derive(Parser)]
#[command(name = "utcode", version)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    ui::run();
}
