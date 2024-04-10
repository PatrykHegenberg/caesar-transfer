use reqwest::blocking::Client;

mod args;
pub mod receiver;
pub mod sender;

fn main() {
    let client = Client::new();
    let args = args::Args::new();
    if let Err(e) = args.run(client) {
        eprintln!("Error {e}");
    }
}
