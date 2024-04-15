use crate::http_server;
use crate::receiver;
use crate::sender;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Send {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,
        /// Path to file(s)
        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,
    },
    /// Receives Files from the sender with the matching password
    Receive {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,

        /// Overwrite existing Files
        #[arg(short, long)]
        overwrite: bool,

        /// Name of Transfer to download files
        #[arg(short, long, value_name = "Transfer_Name")]
        name: Option<String>,
    },
    Serve {
        /// Port to run the relay server on
        #[arg(short, long)]
        port: i32,
    },
    Config {
        /// Show path to config file
        #[arg(short, long)]
        path: bool,

        /// View configured Options
        #[arg(short, long)]
        show: bool,

        /// Edit the config file
        #[arg(short, long)]
        edit: bool,

        /// Reset changed config
        #[arg(short, long)]
        reset: bool,
    },
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl Args {
    pub fn new() -> Self {
        Self::parse()
    }
    pub async fn run(
        &self,
        // client: Client,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &self.command {
            Some(Commands::Send { relay: _, file }) => {
                sender::send_info(file.as_deref().unwrap_or("test.txt")).await?;
            }
            Some(Commands::Receive {
                relay: _,
                overwrite: _,
                name,
            }) => {
                let transfer_name = name.as_deref().unwrap_or("None");
                receiver::download_info(transfer_name).await?
            }
            Some(Commands::Serve { port: _ }) => {
                http_server::start_server().await;
            }
            Some(Commands::Config {
                path: _,
                show: _,
                edit: _,
                reset: _,
            }) => {}
            None => {}
        }
        Ok(())
    }
}
