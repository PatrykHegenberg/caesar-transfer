use caesar_core::relay;
use caesar_core::sender;
use caesar_core::{receiver, sender::util::generate_random_name};
use clap::{Parser, Subcommand};
use std::{env, sync::Arc};
use tracing::debug;

use crate::config::GLOBAL_CONFIG;

/// Struct representing the command line arguments parsed by clap.
///
/// It uses the clap library to define the command line arguments and their
/// attributes. The version of the application is obtained from the cargo.toml
/// file.
///
/// The `command` field is an optional subcommand. It is represented by the
/// `Commands` enum which defines the different subcommands that can be used.
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_PKG_VERSION"), about = "Send and receive files securely")]
#[command(long_about = None)]
pub struct Args {
    /// The subcommand to run.
    ///
    /// It is an optional field. If it is not provided, the program will run without
    /// any specific subcommand.
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


/// Default implementation of the `Args` struct.
///
/// This implementation uses the `new` method to create a new instance of `Args`.
impl Default for Args {
    /// Creates a new instance of `Args` by calling the `new` method.
    ///
    /// # Returns
    ///
    /// A new instance of `Args`.
    fn default() -> Self {
        Self::new()
    }
}


/// Struct representing the parsed command line arguments.
///
/// This struct implements the `Default` trait to create a new instance of `Args` by calling the
/// `new` method.
///
/// The `run` method is used to execute the corresponding command based on the parsed arguments.
impl Args {
    /// Creates a new instance of `Args` by calling the `parse` method.
    pub fn new() -> Self {
        Self::parse()
    }

    /// Executes the corresponding command based on the parsed arguments.
    ///
    /// This method takes no parameters.
    ///
    /// # Returns
    ///
    /// A `Result` that either returns `Ok(())` indicating successful execution or an `Err`
    /// indicating an error.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Retrieve the global configuration
        let cfg = &GLOBAL_CONFIG;
        debug!("args: {:#?}", self);

        // Match on the `command` field of `Args` to execute the corresponding command
        match &self.command {
            // Command to send files to the receiver or relay server
            Some(Commands::Send { relay, files }) => {
                // Create a string representation of the relay address
                let relay_string: String = relay.as_deref().unwrap_or(&cfg.app_origin).to_string();
                // Create Arc wrappers for the relay address and file paths
                let relay_arc = Arc::new(relay_string);
                let files_arc = Arc::new(files.to_vec());
                // Generate a random name
                let rand_name = generate_random_name();
                // Start the sender with the generated name, relay address, and file paths
                sender::start_sender(rand_name, relay_arc, files_arc).await;
            }
            // Command to receive files from the sender with the matching password
            Some(Commands::Receive {
                relay,
                name,
            }) => {
                // Print the received transfer name
                println!("Receive for {name:?}");
                // Start the receiver with the current directory, relay address, and transfer name
                let _ = receiver::start_receiver(
                    ".".to_string(),
                    relay.as_deref().unwrap_or(&cfg.app_origin),
                    name,
                )
                .await;
            }
            // Command to start a relay server
            Some(Commands::Serve {
                port,
                listen_address,
            }) => {
                // Create a string representation of the listen address
                let address: String = listen_address
                    .as_deref()
                    .unwrap_or(&cfg.app_host)
                    .to_string();
                // Create an integer representation of the port
                let port_value = port.unwrap_or(cfg.app_port.parse::<i32>().unwrap_or(0));
                let port: i32 = port_value;
                // Start the relay server with the port and listen address
                relay::server::start_ws(&port, &address).await;
            }
            // No command provided
            None => {}
        }
        Ok(())
    }
}
