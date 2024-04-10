use clap::{Parser, Subcommand};
use reqwest::blocking::Client;

use crate::command;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Name of Transfer to download files
    #[arg(short, long, value_name = "Transfer_Name")]
    pub name: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Files to Send
    Send {
        /// Path to file(s)
        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,
    },
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn handle_cli_args(&self, client: Client) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = self.config.as_deref() {
            println!("Value for config: {}", config_path);
        }
        match self.debug {
            0 => println!("Debug mode is off"),
            1 => println!("Debug mode is kind of on"),
            2 => println!("Debug mode is on"),
            _ => println!("Don't be crazy"),
        }

        match &self.command {
            Some(Commands::Send { file }) => {
                command::send_info(client, file.as_deref().unwrap_or("test.txt"))?;
            }
            None => {
                let filename = self.name.as_deref().unwrap_or("None");
                command::download_info(client, filename)?
            }
        }
        Ok(())
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}
