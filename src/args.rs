use crate::http_server;
use crate::receiver;
use crate::sender;
use clap::{Parser, Subcommand};
use log::debug;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Send files to the receiver or relay server
    Send {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,
        /// Path to file(s)
        #[arg(value_name = "FILE")]
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
    /// Start a relay server
    Serve {
        /// Port to run the relay server on
        #[arg(short, long)]
        port: Option<i32>,
        /// The Listen address to run the relay server on
        #[arg(short, long)]
        listen_address: Option<String>,
    },
    /// Work with your configuration
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
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        env_logger::init();
        debug!("args: {:?}", self);
        match &self.command {
            Some(Commands::Send { relay, file }) => {
                sender::send_info(
                    relay.as_deref().unwrap_or("http://192.168.178.43:1323"),
                    file.as_deref().unwrap_or("test.txt"),
                )
                .await?;
            }
            Some(Commands::Receive {
                relay: _,
                overwrite: _,
                name,
            }) => {
                let transfer_name = name.as_deref().unwrap_or("None");
                receiver::download_info(transfer_name).await?
            }
            Some(Commands::Serve {
                port,
                listen_address,
            }) => {
                http_server::start_server(port.as_ref(), listen_address.as_ref()).await;
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
