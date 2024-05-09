use caesar_core::receiver;
use caesar_core::relay;
use caesar_core::sender;
use clap::{Parser, Subcommand};
use std::{env, sync::Arc};
use tracing::debug;

use crate::config::GLOBAL_CONFIG;

#[derive(Parser, Debug)]
#[command(version = env!("CARGO_PKG_VERSION"), about = "Send and receive files securely")]
#[command(long_about = None)]
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
        #[arg(value_name = "FILES")]
        files: Vec<String>,
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
        #[arg(value_name = "Transfer_Name")]
        name: String,
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
        let cfg = &GLOBAL_CONFIG;
        debug!("args: {:#?}", self);
        match &self.command {
            Some(Commands::Send { relay, files }) => {
                let relay_string: String = relay.as_deref().unwrap_or(&cfg.app_origin).to_string();
                let relay_arc = Arc::new(relay_string);
                let files_arc = Arc::new(files.to_vec());
                sender::start_sender(relay_arc, files_arc).await;
            }
            Some(Commands::Receive {
                relay,
                overwrite: _,
                name,
            }) => {
                println!("Receive for {name:?}");
                receiver::start_receiver(relay.as_deref().unwrap_or(&cfg.app_origin), name).await;
            }
            Some(Commands::Serve {
                port,
                listen_address,
            }) => {
                println!("Serve with address '{listen_address:?}' and '{port:?}'");
                let address: String = listen_address
                    .as_deref()
                    .unwrap_or(&cfg.app_host)
                    .to_string();
                let port_value = port.unwrap_or(cfg.app_port.parse::<i32>().unwrap_or(0));
                let port: i32 = port_value;
                relay::server::start_ws(&port, &address).await;
            }
            None => {}
        }
        Ok(())
    }
}
