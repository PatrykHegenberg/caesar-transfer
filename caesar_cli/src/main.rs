mod cli;
mod command;
use reqwest::blocking::Client;

pub use crate::cli::*;

fn main() {
    let client = Client::new();
    let cli = cli::Cli::new();

    if let Err(e) = cli.handle_cli_args(client) {
        eprintln!("Error: {e}");
    }
}
